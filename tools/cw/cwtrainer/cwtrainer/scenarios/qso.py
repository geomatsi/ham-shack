"""QSO: send back QSO phrases."""

from __future__ import annotations

import argparse
import random

from cwtrainer.config import Config
from cwtrainer.grading import RST_EQUIVALENTS
from cwtrainer.scenarios import register
from cwtrainer.scenarios.echo import EchoBase, PromptMode, load_lines


@register
class Qso(EchoBase):
    name = "qso"
    help = "send back QSO phrases"
    equivalents = RST_EQUIVALENTS

    def __init__(self, phrases: list[str], mode: PromptMode = PromptMode.SHOW,
                 count: int = 10, rng: random.Random | None = None) -> None:
        super().__init__(mode, count, rng)
        self.phrases = phrases

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Qso:
        return cls(load_lines("phrases.txt"), args.prompt_mode, args.count, random.Random(args.seed))

    def _pick(self) -> str:
        return self.rng.choice(self.phrases)
