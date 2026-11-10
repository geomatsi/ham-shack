"""Groups: random letter/digit groups, LCWO style."""

from __future__ import annotations

import argparse
import random
import string
from enum import Enum

from cwtrainer.config import Config
from cwtrainer.scenarios import register
from cwtrainer.scenarios.echo import EchoBase, PromptMode

MAX_SIZE = 6


class Charset(Enum):
    LETTERS = "letters"
    DIGITS = "digits"
    MIX = "mix"

    @property
    def chars(self) -> str:
        return {Charset.LETTERS: string.ascii_uppercase, Charset.DIGITS: string.digits,
                Charset.MIX: string.ascii_uppercase + string.digits}[self]


def size_range(s: str) -> tuple[int, int]:
    try:
        lo, _, hi = s.partition("-")
        r = (int(lo), int(hi or lo))
    except ValueError:
        raise argparse.ArgumentTypeError("expected N or LO-HI, e.g. 5 or 1-6") from None
    if not 1 <= r[0] <= r[1] <= MAX_SIZE:
        raise argparse.ArgumentTypeError(f"need 1 <= LO <= HI <= {MAX_SIZE}")
    return r


def positive(s: str) -> int:
    v = int(s)
    if v < 1:
        raise argparse.ArgumentTypeError("must be >= 1")
    return v


@register
class Groups(EchoBase):
    name = "groups"
    help = "send back random letter/digit groups"

    def __init__(self, charset: Charset = Charset.LETTERS, size: tuple[int, int] = (5, 5),
                 per_prompt: int = 1, mode: PromptMode = PromptMode.SHOW, count: int = 20,
                 rng: random.Random | None = None) -> None:
        super().__init__(mode, count, rng)
        self.charset = charset
        self.size = size
        self.per_prompt = per_prompt

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        EchoBase.add_arguments(p)
        p.set_defaults(count=20)
        p.add_argument("--charset", type=Charset, choices=list(Charset), default=Charset.LETTERS,
                       metavar="{letters,digits,mix}", help="characters to draw from (default letters)")
        p.add_argument("--size", type=size_range, default=(5, 5), metavar="N|LO-HI",
                       help=f"group length, fixed or random in a range, max {MAX_SIZE} (default 5)")
        p.add_argument("--per-prompt", type=positive, default=1, metavar="N",
                       help="groups per prompt (default 1)")

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Groups:
        return cls(args.charset, args.size, args.per_prompt, args.prompt_mode, args.count,
                   random.Random(args.seed))

    def _group(self) -> str:
        n = self.rng.randint(*self.size)
        return "".join(self.rng.choice(self.charset.chars) for _ in range(n))

    def _pick(self) -> str:
        return " ".join(self._group() for _ in range(self.per_prompt))
