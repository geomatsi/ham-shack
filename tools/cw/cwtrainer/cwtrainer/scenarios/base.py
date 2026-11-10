"""Scenario interface."""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from typing import Protocol

from cwtrainer.config import Config
from cwtrainer.grading import Grade, Mistake


@dataclass(frozen=True)
class Voice:
    """One station sending; several voices are mixed."""
    text: str
    wpm: float | None = None   # None: configured speed
    tone_offset: float = 0.0   # Hz from configured tone
    delay: float = 0.0         # seconds from prompt start


@dataclass
class Prompt:
    text: str
    show_text: bool
    play_audio: bool
    wpm: int | None = None
    instruction: str | None = None                    # shown even when text is not
    voices: list[Voice] = field(default_factory=list)  # empty: text at wpm

    def audio(self) -> list[Voice]:
        if self.voices:
            return self.voices
        return [Voice(self.text, self.wpm)] if self.text else []


@dataclass
class Result:
    ok: bool
    expected: str
    sent: str
    score: float
    notes: list[str] = field(default_factory=list)
    mistakes: list[Mistake] = field(default_factory=list)
    attempted: list[str] = field(default_factory=list)
    reveal: bool = True  # False: don't show expected text (it's still to be copied)

    @property
    def wrong(self) -> list[str]:
        return [m.expected for m in self.mistakes if m.expected not in (None, " ")]

    @classmethod
    def from_grade(cls, g: Grade) -> Result:
        return cls(g.distance == 0, g.expected, g.sent, g.score, [str(m) for m in g.mistakes],
                   g.mistakes, g.attempted)


class Scenario(Protocol):
    name: str

    def next_prompt(self) -> Prompt: ...
    def check(self, sent: str) -> Result: ...
    def done(self) -> bool: ...
    # optional: def summary(self) -> list[str]


class ScenarioFactory(Protocol):
    name: str
    help: str

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None: ...

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Scenario: ...
