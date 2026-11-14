"""QSO: send back QSO phrases and words."""

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
    help = "send back QSO phrases and words"
    equivalents = RST_EQUIVALENTS

    PHRASE_RATIO = 0.6

    def __init__(self, phrases: list[str], words: list[str], mode: PromptMode = PromptMode.SHOW,
                 count: int = 10, rng: random.Random | None = None) -> None:
        super().__init__(mode, count, rng)
        self.phrases = phrases
        self.words = words

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Qso:
        return cls(load_lines("phrases.txt"), load_lines("words.txt"), args.prompt_mode,
                   args.count, random.Random(args.seed))

    def _pick(self) -> str:
        if self.rng.random() < self.PHRASE_RATIO:
            return self.rng.choice(self.phrases)
        return " ".join(self.rng.sample(self.words, min(self.rng.randint(1, 2), len(self.words))))
