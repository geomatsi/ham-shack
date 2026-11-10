"""POTA hunter: answer simulated activators' CQs."""

from __future__ import annotations

import argparse
import random
import string
from dataclasses import dataclass
from enum import Enum, auto

from cwtrainer.callgen import CallsignGenerator
from cwtrainer.clock import Clock, MonotonicClock
from cwtrainer.config import Config
from cwtrainer.morse import text_to_elements, timeline
from cwtrainer.scenarios import register
from cwtrainer.scenarios.base import Prompt, Result, Voice
from cwtrainer.scenarios.callsigns import probability
from cwtrainer.scenarios.pota import (
    REPEAT_REQUESTS, RST, TONE_SPREAD, Qso, Station, best_match, combine, partial_call,
    random_park, words_of, wpm_range,
)

CQ_FORMS = ["CQ POTA DE {a} {a} K", "CQ CQ POTA DE {a} K", "CQ POTA {p} DE {a} K",
            "CQ POTA DE {a} {a} {p} K"]
REPORTS = ["{m} 5NN", "{m} GM 5NN", "{m} TU 5NN", "{m} UR 5NN 5NN", "{m} 599"]
SIGNOFFS = ["TU 73 QRZ", "TU QRZ", "73 TU EE", "R TU 73 QRZ"]
OTHER_REPLIES = ["R TU 5NN", "TU 5NN 73", "R 599 TU"]
OVER_GAP = 0.8  # seconds between overs mixed into one prompt


class Step(Enum):
    CALL = auto()      # user sends own call
    BUSTED = auto()    # activator copied the call wrong; user corrects it
    EXCHANGE = auto()  # user sends report
    LOG = auto()       # user sends activator's call and park


@dataclass
class Realism:
    agn: float = 0.0     # activator asks AGN? after your report
    partial: float = 0.0  # activator got part of your call: "R3?"
    busted: float = 0.0   # activator copies your call wrong
    pileup: float = 0.0   # activator works another hunter first


def overs(parts: list[tuple[Station, str]], start: float = 0.0) -> list[Voice]:
    """Consecutive overs, each starting after the previous one ends."""
    out: list[Voice] = []
    t = start
    for st, text in parts:
        out.append(st.voice(text, t))
        t += timeline(text, st.wpm)[1] + OVER_GAP
    return out


def bust(call: str, rng: random.Random) -> str:
    """Change one letter after the area digit."""
    base, sep, tail = call.partition("/")
    area = max((i for i, c in enumerate(base) if c.isdigit()), default=-1)
    idx = [i for i, c in enumerate(base) if i > area and c.isalpha()]
    i = rng.choice(idx) if idx else len(base) - 1
    c = rng.choice([x for x in string.ascii_uppercase if x != base[i]])
    return base[:i] + c + base[i + 1:] + sep + tail


@register
class PotaHunter:
    name = "pota-hunter"
    help = "hunt parks: answer activators' CQs"

    def __init__(self, mycall: str, count: int = 10, show: bool = False,
                 activator_wpm: tuple[float, float] = (18, 24), realism: Realism | None = None,
                 log: bool = False, rng: random.Random | None = None,
                 clock: Clock | None = None) -> None:
        self.mycall = mycall.upper()
        self.count = count
        self.show = show
        self.activator_wpm = activator_wpm
        self.realism = realism or Realism()
        self.log_step = log
        self.rng = rng or random.Random()
        self.clock = clock or MonotonicClock()
        self.gen = CallsignGenerator(self.rng)

        self.step = Step.CALL
        self.act: Station
        self.voices: list[Voice] = []
        self.instruction: str | None = None
        self.copied = ""          # how the activator copied our call (BUSTED)
        self.used: set[str] = set()  # realism events already used this QSO
        self.busted_qso = False
        self.qso_start = 0.0
        self.log: list[Qso] = []
        self.busts = 0
        self.scores: list[float] = []
        self._new_activator([])

    # --- CLI

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        p.add_argument("--mycall", required=True, type=str.upper, help="your callsign")
        p.add_argument("--count", type=int, default=10, help="QSOs per session, 0 = endless")
        p.add_argument("--show", action="store_true", help="show what activators send")
        p.add_argument("--activator-wpm", type=wpm_range, metavar="LO-HI",
                       help="activator speed range (default WPM-2 to WPM+4)")
        p.add_argument("--log", action="store_true",
                       help="after each QSO, send the activator's call and park as you logged them")
        for name, default, what in [("agn", 0.2, "activator asks AGN? after your report"),
                                    ("partial", 0.3, "activator sends part of your call with ?"),
                                    ("busted", 0.2, "activator copies your call wrong"),
                                    ("pileup", 0.3, "activator works another hunter first")]:
            p.add_argument(f"--{name}", type=probability, nargs="?", const=default, default=0.0,
                           metavar="P", help=f"{what} (P defaults to {default})")
        p.add_argument("--seed", type=int, help="random seed")

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> PotaHunter:
        text_to_elements(args.mycall)  # KeyError on unsendable characters
        aw = args.activator_wpm or (max(cfg.wpm - 2, 5), cfg.wpm + 4)
        realism = Realism(args.agn, args.partial, args.busted, args.pileup)
        return cls(args.mycall, args.count, args.show, aw, realism, args.log,
                   random.Random(args.seed))

    # --- Scenario

    def next_prompt(self) -> Prompt:
        text = " / ".join(v.text for v in self.voices)
        return Prompt(text, show_text=self.show, play_audio=bool(self.voices),
                      instruction=self.instruction, voices=list(self.voices))

    def done(self) -> bool:
        return self.count > 0 and len(self.log) >= self.count

    def check(self, sent: str) -> Result:
        words = words_of(sent)
        if self.step is not Step.LOG and words and set(words) <= REPEAT_REQUESTS:
            self.instruction = None
            return Result(True, sent, sent, 1.0, ["repeat"], reveal=False)
        r = {Step.CALL: self._call, Step.BUSTED: self._busted,
             Step.EXCHANGE: self._exchange, Step.LOG: self._log}[self.step](sent, words)
        self.scores.append(r.score)
        return r

    def summary(self) -> list[str]:
        if not self.log:
            return ["no QSOs"]
        avg = sum(q.seconds for q in self.log) / len(self.log)
        acc = sum(self.scores) / len(self.scores)
        lines = [f"{len(self.log)} QSOs, accuracy {acc:.0%}, {avg:.0f} s per QSO"]
        if self.busts:
            lines.append(f"{self.busts} logged with a busted call")
        lines.append("log: " + " ".join(q.call for q in self.log))
        return lines

    # --- steps

    def _roll(self, event: str) -> bool:
        """Each realism event at most once per QSO."""
        if event in self.used or self.rng.random() >= getattr(self.realism, event):
            return False
        self.used.add(event)
        return True

    def _call(self, sent: str, words: list[str]) -> Result:
        g = best_match(self.mycall, words)
        attempted = g.distance <= 2
        r = combine(self.mycall, sent, [g] if attempted else [], attempted,
                    [] if attempted else ["send your call"])
        a, m = self.act, self.mycall
        self.instruction = None
        if 0 < g.distance <= 2:
            # activator answers what you actually sent
            self.copied = g.sent
            self.step = Step.BUSTED
            self.voices = [a.voice(self.rng.choice(REPORTS).format(m=self.copied))]
        elif g.distance:
            self.voices = [a.voice(self.rng.choice(["?", "QRZ?", "AGN?"]))]
        elif self._roll("pileup"):
            other = self.gen.generate()
            o = Station(other, round(self.rng.uniform(*self.activator_wpm)),
                        round(self.rng.uniform(-TONE_SPREAD, TONE_SPREAD)))
            self.voices = overs([(a, self.rng.choice(REPORTS).format(m=other)),
                                 (o, self.rng.choice(OTHER_REPLIES)),
                                 (a, "TU QRZ")])
        elif self._roll("partial"):
            self.voices = [a.voice(f"{partial_call(m, self.rng)}?")]
        elif self._roll("busted"):
            self.copied = bust(m, self.rng)
            self.step = Step.BUSTED
            self.voices = [a.voice(self.rng.choice(REPORTS).format(m=self.copied))]
        else:
            self.step = Step.EXCHANGE
            self.voices = [a.voice(self.rng.choice(REPORTS).format(m=m))]
        return r

    def _busted(self, sent: str, words: list[str]) -> Result:
        g = best_match(self.mycall, words)
        corrected = g.distance == 0
        accepted = not corrected and any(RST.fullmatch(w) for w in words)
        notes = [] if corrected else [f"copied as {self.copied}: resend your call"]
        # grade the call only if it was attempted, else stats blame its characters
        r = combine(self.mycall, sent, [g] if g.distance <= 2 else [], corrected, notes)
        if corrected:
            self.step = Step.EXCHANGE
            self.voices = [self.act.voice(f"R {self.mycall} 5NN")]
        elif accepted:
            # report sent to a wrong call: the QSO is in the log, busted
            self.busted_qso = True
            self._finish()
        else:
            self.voices = [self.act.voice(self.rng.choice(REPORTS).format(m=self.copied))]
        return r

    def _exchange(self, sent: str, words: list[str]) -> Result:
        has_rst = any(RST.fullmatch(w) for w in words)
        r = combine("R 5NN TU", sent, [], has_rst, [] if has_rst else ["no RST"])
        if not has_rst:
            self.voices = [self.act.voice(self.rng.choice(["AGN?", "RST?"]))]
        elif self._roll("agn"):
            self.voices = [self.act.voice(self.rng.choice(["AGN?", "?"]))]
        else:
            self._finish()
        return r

    def _log(self, sent: str, words: list[str]) -> Result:
        a = self.act
        r = combine(f"{a.call} {a.park}", sent, [best_match(a.call, words), best_match(a.park or "", words)],
                    True, [])
        self._new_activator([])
        return r

    # --- activators

    def _finish(self) -> None:
        a = self.act
        self.log.append(Qso(a.call, self.clock.now() - self.qso_start))
        self.busts += self.busted_qso
        signoff = [(a, self.rng.choice(SIGNOFFS))]
        if self.log_step:
            self.step = Step.LOG
            self.instruction = "Log: send the activator's call and park"
            self.voices = overs(signoff)
        else:
            self._new_activator(signoff)

    def _new_activator(self, before: list[tuple[Station, str]]) -> None:
        lo, hi = self.activator_wpm
        a = Station(self.gen.generate(), round(self.rng.uniform(lo, hi)),
                    round(self.rng.uniform(-TONE_SPREAD, TONE_SPREAD)), random_park(self.rng))
        self.act = a
        self.step = Step.CALL
        self.used = set()
        self.busted_qso = False
        self.instruction = None
        self.qso_start = self.clock.now()
        forms = [f for f in CQ_FORMS if "{p}" in f] if self.log_step else CQ_FORMS
        self.voices = overs(before + [(a, self.rng.choice(forms).format(a=a.call, p=a.park))])
