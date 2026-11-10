"""Callsigns: echo generated callsigns."""

from __future__ import annotations

import argparse
import random
from collections import deque

from cwtrainer.callgen import MAX_LEVEL, CallsignGenerator
from cwtrainer.config import Config
from cwtrainer.scenarios import register
from cwtrainer.scenarios.base import Result
from cwtrainer.scenarios.echo import EchoBase, PromptMode


def probability(s: str) -> float:
    v = float(s)
    if not 0.0 <= v <= 1.0:
        raise argparse.ArgumentTypeError("probability must be in 0..1")
    return v


@register
class Callsigns(EchoBase):
    name = "callsigns"
    help = "send back generated callsigns"

    WINDOW = 5         # prompts averaged for the ramp
    PROMOTE = 0.95     # mean score to step up
    DEMOTE = 0.6       # mean score to step down

    def __init__(self, gen: CallsignGenerator, mode: PromptMode = PromptMode.SHOW,
                 count: int = 20, rng: random.Random | None = None, ramp: bool = False) -> None:
        super().__init__(mode, count, rng)
        self.gen = gen
        self.ramp = ramp
        self.level = 1 if ramp else MAX_LEVEL
        self.recent: deque[float] = deque(maxlen=self.WINDOW)

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        EchoBase.add_arguments(p)
        p.set_defaults(count=20)
        p.add_argument("--portable", type=probability, default=0.1, metavar="P",
                       help="chance of a /P, /M or /QRP suffix (default 0.1)")
        p.add_argument("--digit-prefix", type=probability, default=0.1, metavar="P",
                       help="chance of a prefix like 9A or S5 (default 0.1)")
        p.add_argument("--ramp", action="store_true",
                       help="start with short calls, get harder as accuracy improves")

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Callsigns:
        rng = random.Random(args.seed)
        gen = CallsignGenerator(rng, args.portable, args.digit_prefix)
        return cls(gen, args.prompt_mode, args.count, rng, args.ramp)

    def _pick(self) -> str:
        return self.gen.generate(self.level)

    def check(self, sent: str) -> Result:
        r = super().check(sent)
        if self.ramp:
            self.recent.append(r.score)
            if len(self.recent) == self.WINDOW:
                mean = sum(self.recent) / self.WINDOW
                step = (mean >= self.PROMOTE) - (mean < self.DEMOTE)
                new = min(max(self.level + step, 1), MAX_LEVEL)
                if new != self.level:
                    r.notes.append(f"level {new}")
                    self.level = new
                    self.recent.clear()
        return r
