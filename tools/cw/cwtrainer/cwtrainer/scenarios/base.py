"""Scenario interface."""

from __future__ import annotations

import argparse
from dataclasses import dataclass, field
from typing import Protocol

from cwtrainer.config import Config
from cwtrainer.grading import Grade, Mistake


@dataclass
class Prompt:
    text: str
    show_text: bool
    play_audio: bool
    wpm: int | None = None


@dataclass
class Result:
    ok: bool
    expected: str
    sent: str
    score: float
    notes: list[str] = field(default_factory=list)
    mistakes: list[Mistake] = field(default_factory=list)
    attempted: list[str] = field(default_factory=list)

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


class ScenarioFactory(Protocol):
    name: str
    help: str

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None: ...

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Scenario: ...
