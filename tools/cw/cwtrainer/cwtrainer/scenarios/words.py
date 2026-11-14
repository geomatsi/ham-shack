"""Words: send back common English words."""

from __future__ import annotations

import argparse
import random

from cwtrainer.config import Config
from cwtrainer.scenarios import register
from cwtrainer.scenarios.echo import EchoBase, PromptMode, load_lines
from cwtrainer.scenarios.groups import positive


@register
class Words(EchoBase):
    name = "words"
    help = "send back common English words"

    def __init__(self, words: list[str], per_prompt: int = 1, mode: PromptMode = PromptMode.SHOW,
                 count: int = 10, rng: random.Random | None = None) -> None:
        super().__init__(mode, count, rng)
        self.words = words
        self.per_prompt = per_prompt

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        EchoBase.add_arguments(p)
        p.add_argument("--per-prompt", type=positive, default=1, metavar="N",
                       help="words per prompt (default 1)")

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Words:
        return cls(load_lines("words.txt"), args.per_prompt, args.prompt_mode, args.count,
                   random.Random(args.seed))

    def _pick(self) -> str:
        return " ".join(self.rng.sample(self.words, min(self.per_prompt, len(self.words))))
