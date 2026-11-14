"""Echo drills: show and/or play a prompt, grade what is sent back."""

from __future__ import annotations

import argparse
import random
from collections.abc import Mapping
from enum import Enum
from importlib.resources import files

from cwtrainer.grading import grade
from cwtrainer.scenarios.base import Prompt, Result


class PromptMode(Enum):
    SHOW = "show"
    LISTEN = "listen"
    BOTH = "both"


def load_lines(name: str) -> list[str]:
    text = (files("cwtrainer") / "data" / name).read_text()
    return [ln.strip().upper() for ln in text.splitlines() if ln.strip() and not ln.startswith("#")]


class EchoBase:
    """Prompt, send it back, grade by edit distance. Subclasses supply _pick()."""

    name = ""
    equivalents: Mapping[str, str] | None = None

    def __init__(self, mode: PromptMode = PromptMode.SHOW, count: int = 10,
                 rng: random.Random | None = None) -> None:
        self.mode = mode
        self.count = count
        self.rng = rng or random.Random()
        self.n = 0
        self.current = ""

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        g = p.add_mutually_exclusive_group()
        for m in PromptMode:
            g.add_argument(f"--{m.value}", dest="prompt_mode", action="store_const", const=m,
                           help={"show": "show text only (default)", "listen": "audio only",
                                 "both": "show text and play audio"}[m.value])
        p.set_defaults(prompt_mode=PromptMode.SHOW)
        p.add_argument("--count", type=int, default=10, help="prompts per session, 0 = endless")
        p.add_argument("--seed", type=int, help="random seed")

    def _pick(self) -> str:
        raise NotImplementedError

    def next_prompt(self) -> Prompt:
        text = self._pick()
        for _ in range(10):  # avoid an immediate repeat
            if text != self.current:
                break
            text = self._pick()
        self.current = text
        self.n += 1
        return Prompt(text, show_text=self.mode is not PromptMode.LISTEN,
                      play_audio=self.mode is not PromptMode.SHOW)

    def check(self, sent: str) -> Result:
        return Result.from_grade(grade(self.current, sent, self.equivalents))

    def done(self) -> bool:
        return self.count > 0 and self.n >= self.count
