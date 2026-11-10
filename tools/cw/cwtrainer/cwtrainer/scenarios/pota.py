"""POTA activator: call CQ, work simulated hunters."""

from __future__ import annotations

import argparse
import random
import re
from dataclasses import dataclass, field
from enum import Enum, auto

from cwtrainer.callgen import CallsignGenerator
from cwtrainer.clock import Clock, MonotonicClock
from cwtrainer.config import Config
from cwtrainer.grading import Grade, apply_hh, grade, normalise
from cwtrainer.morse import text_to_elements
from cwtrainer.scenarios import register
from cwtrainer.scenarios.base import Prompt, Result, Voice
from cwtrainer.scenarios.callsigns import probability

RST = re.compile(r"[1-5][1-9N][1-9N]")
SIGNOFF = {"TU", "73", "QRZ", "QRZ?", "EE", "GL"}
REPEAT_REQUESTS = {"?", "AGN", "AGN?", "PSE", "RPT", "QRZ?"}
HUNTER_REPLIES = ["R 5NN TU", "TU 73", "R TU 5NN", "RR 5NN TU 73", "TU 5NN 73"]
PARK_PREFIXES = ["K", "VE", "G", "DL", "OH", "SM", "F", "I", "JA", "VK", "SP", "EA", "ON", "PA"]
TONE_SPREAD = 100.0   # hunter tone offset, +/- Hz
PILEUP_DELAY = (0.1, 0.6)


class Step(Enum):
    CQ = auto()        # user calls CQ
    PARTIAL = auto()   # part of a call heard; user asks "<PART>?"
    EXCHANGE = auto()  # user sends "<HCALL> 5NN"
    FINAL = auto()     # hunter acknowledged; user signs off


@dataclass
class Hunter:
    call: str
    wpm: float
    tone_offset: float
    park: str | None = None  # set for park-to-park

    def voice(self, text: str, delay: float = 0.0) -> Voice:
        return Voice(text, self.wpm, self.tone_offset, delay)


@dataclass
class Realism:
    agn: float = 0.0        # hunter missed your exchange, sends AGN?
    partial: float = 0.0    # only part of the call heard
    pileup: float = 0.0     # two hunters at once
    p2p: float = 0.0        # hunter is a park activator too
    no_answer: float = 0.1  # nobody answers a CQ


@dataclass
class Qso:
    call: str
    seconds: float
    scores: list[float] = field(default_factory=list)


def words_of(sent: str) -> list[str]:
    text = normalise(apply_hh(normalise(sent)))
    return re.sub(r"\s+\?", "?", text).split()


def best_match(token: str, words: list[str]) -> Grade:
    if not words:
        return grade(token, "")
    return min((grade(token, w) for w in words), key=lambda g: g.distance)


def combine(expected: str, sent: str, grades: list[Grade], ok: bool, notes: list[str],
            reveal: bool = True) -> Result:
    score = sum(g.score for g in grades) / len(grades) if grades else float(ok)
    mistakes = [m for g in grades for m in g.mistakes]
    attempted = [c for g in grades for c in g.attempted]
    if reveal:
        notes = [str(m) for m in mistakes] + notes
    return Result(ok and all(g.distance == 0 for g in grades), expected, sent, score, notes,
                  mistakes, attempted, reveal)


@register
class Pota:
    name = "pota"
    help = "activate a park: call CQ and work hunters"

    def __init__(self, mycall: str, park: str | None = None, count: int = 10, show: bool = False,
                 hunter_wpm: tuple[float, float] = (18, 24), realism: Realism | None = None,
                 rng: random.Random | None = None, clock: Clock | None = None) -> None:
        self.mycall = mycall.upper()
        self.park = park
        self.count = count
        self.show = show
        self.hunter_wpm = hunter_wpm
        self.realism = realism or Realism()
        self.rng = rng or random.Random()
        self.clock = clock or MonotonicClock()
        self.gen = CallsignGenerator(self.rng)

        self.step = Step.CQ
        self.queue: list[Hunter] = []        # waiting to be worked
        self.calling: list[Hunter] = []      # heard in the current over
        self.voices: list[Voice] = []        # what the next prompt plays
        self.instruction: str | None = "Call CQ"
        self.fragment = ""
        self.agn_done = False
        self.qso_start = 0.0
        self.qso_scores: list[float] = []
        self.log: list[Qso] = []
        self.scores: list[float] = []

    # --- CLI

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        p.add_argument("--mycall", required=True, type=str.upper, help="your callsign")
        p.add_argument("--park", type=str.upper, help="your park reference, e.g. OH-0001")
        p.add_argument("--count", type=int, default=10, help="QSOs per session, 0 = endless")
        p.add_argument("--show", action="store_true", help="show what hunters send")
        p.add_argument("--hunter-wpm", type=wpm_range, metavar="LO-HI",
                       help="hunter speed range (default WPM-2 to WPM+4)")
        for name, default, what in [("agn", 0.2, "hunter asks AGN? after your exchange"),
                                    ("partial", 0.3, "only part of the call is heard"),
                                    ("pileup", 0.3, "two hunters call at once"),
                                    ("p2p", 0.2, "hunter is park-to-park, sends its park")]:
            p.add_argument(f"--{name}", type=probability, nargs="?", const=default, default=0.0,
                           metavar="P", help=f"{what} (P defaults to {default})")
        p.add_argument("--seed", type=int, help="random seed")

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Pota:
        for s in (args.mycall, args.park or ""):
            text_to_elements(s)  # KeyError on unsendable characters
        hw = args.hunter_wpm or (max(cfg.wpm - 2, 5), cfg.wpm + 4)
        realism = Realism(args.agn, args.partial, args.pileup, args.p2p)
        return cls(args.mycall, args.park, args.count, args.show, hw, realism,
                   random.Random(args.seed))

    # --- Scenario

    def next_prompt(self) -> Prompt:
        text = " ".join(v.text for v in self.voices)
        return Prompt(text, show_text=self.show, play_audio=bool(self.voices),
                      instruction=self.instruction, voices=list(self.voices))

    def done(self) -> bool:
        return self.count > 0 and len(self.log) >= self.count

    def check(self, sent: str) -> Result:
        words = words_of(sent)
        if self.step in (Step.PARTIAL, Step.EXCHANGE) and words and set(words) <= REPEAT_REQUESTS:
            # asked for a repeat: same voices again, nothing graded
            self.instruction = None
            return Result(True, sent, sent, 1.0, ["repeat"], reveal=False)
        r = {Step.CQ: self._cq, Step.PARTIAL: self._partial,
             Step.EXCHANGE: self._exchange, Step.FINAL: self._final}[self.step](sent, words)
        self.scores.append(r.score)
        self.qso_scores.append(r.score)
        return r

    def summary(self) -> list[str]:
        if not self.log:
            return ["no QSOs"]
        avg = sum(q.seconds for q in self.log) / len(self.log)
        acc = sum(self.scores) / len(self.scores)
        return [f"{len(self.log)} QSOs, accuracy {acc:.0%}, {avg:.0f} s per QSO",
                "log: " + " ".join(q.call for q in self.log)]

    # --- steps

    def _cq(self, sent: str, words: list[str]) -> Result:
        r = combine(f"CQ POTA DE {self.mycall} K", sent,
                    [best_match("CQ", words), best_match(self.mycall, words)], True, [])
        if not r.ok:
            self.instruction = "Call CQ"
            self.voices = []
            return r
        self.queue = [self._hunter() for _ in range(self._callers())]
        if self.queue:
            self._call_next()
        else:
            self.instruction = "No answer, call CQ again"
            self.voices = []
        return r

    def _partial(self, sent: str, words: list[str]) -> Result:
        h = self.calling[0]
        r = combine(f"{self.fragment}?", sent, [best_match(f"{self.fragment}?", words)], True, [],
                    reveal=False)
        if r.ok:
            self.step = Step.EXCHANGE
            self.voices = [h.voice(f"{h.call} {h.call}")]
        else:
            r.notes = ["send the part you heard with ?"]
            self.voices = [h.voice(self.fragment)]
        self.instruction = None
        return r

    def _exchange(self, sent: str, words: list[str]) -> Result:
        matches = [(best_match(h.call, words), h) for h in self.calling]
        g, h = min(matches, key=lambda m: m[0].distance)
        has_rst = any(RST.fullmatch(w) for w in words)
        notes = [] if has_rst else ["no RST"]
        call_ok = g.distance == 0
        r = combine(f"{h.call} 5NN", sent, [g], has_rst, notes, reveal=call_ok)
        self.instruction = None
        if not call_ok:
            r.notes = ["call not copied"] + notes
            self.voices = [x.voice(f"{x.call} {x.call}", i * 0.3) for i, x in enumerate(self.calling)]
        elif not has_rst or (not self.agn_done and self.rng.random() < self.realism.agn):
            self.agn_done = True
            self.calling = [h]
            self.voices = [h.voice(self.rng.choice(["AGN?", "?", "RST?"]) if not has_rst
                                   else self.rng.choice(["AGN?", "?"]))]
        else:
            self.step = Step.FINAL
            self.calling = [h]
            self.queue.remove(h)
            reply = f"R 5NN {h.park} TU" if h.park else self.rng.choice(HUNTER_REPLIES)
            self.voices = [h.voice(reply)]
        return r

    def _final(self, sent: str, words: list[str]) -> Result:
        h = self.calling[0]
        signed = any(w in SIGNOFF for w in words)
        notes = [] if signed else ["no TU/73/QRZ"]
        grades = [best_match(h.park, words)] if h.park else []
        expected = f"R {h.park} TU" if h.park else "TU 73"
        r = combine(expected, sent, grades, signed, notes)
        self.log.append(Qso(h.call, self.clock.now() - self.qso_start, self.qso_scores))
        if self.queue:
            self._call_next()
        else:
            self.step = Step.CQ
            self.instruction = "No more callers, call CQ"
            self.voices = []
        return r

    # --- hunters

    def _callers(self) -> int:
        if self.rng.random() < self.realism.no_answer:
            return 0
        return self.rng.randint(1, 3)

    def _hunter(self) -> Hunter:
        lo, hi = self.hunter_wpm
        h = Hunter(self.gen.generate(), round(self.rng.uniform(lo, hi)),
                   round(self.rng.uniform(-TONE_SPREAD, TONE_SPREAD)))
        if self.rng.random() < self.realism.p2p:
            h.park = f"{self.rng.choice(PARK_PREFIXES)}-{self.rng.randint(1, 9999):04d}"
        return h

    def _call_next(self) -> None:
        self.qso_start = self.clock.now()
        self.qso_scores = []
        self.agn_done = False
        self.instruction = None
        self.step = Step.EXCHANGE
        if len(self.queue) >= 2 and self.rng.random() < self.realism.pileup:
            self.calling = self.queue[:2]
            delay = self.rng.uniform(*PILEUP_DELAY)
            self.voices = [self.calling[0].voice(self.calling[0].call),
                           self.calling[1].voice(self.calling[1].call, delay)]
            return
        h = self.queue[0]
        self.calling = [h]
        if self.rng.random() < self.realism.partial:
            self.step = Step.PARTIAL
            self.fragment = partial_call(h.call, self.rng)
            self.voices = [h.voice(self.fragment)]
        else:
            self.voices = [h.voice(h.call)]


def partial_call(call: str, rng: random.Random) -> str:
    """Prefix with area digit, or the suffix."""
    base = call.split("/")[0]
    m = re.fullmatch(r"(.*?\d)([A-Z]+)", base)
    if not m:
        return base[:3]
    return rng.choice(m.groups())


def wpm_range(s: str) -> tuple[float, float]:
    try:
        lo, hi = (float(x) for x in s.split("-"))
    except ValueError:
        raise argparse.ArgumentTypeError("expected LO-HI, e.g. 18-25") from None
    if not 5 <= lo <= hi:
        raise argparse.ArgumentTypeError("need 5 <= LO <= HI")
    return lo, hi
