"""Runtime configuration."""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path


class KeyerMode(Enum):
    A = "A"
    B = "B"


def default_stats_path() -> Path:
    base = os.environ.get("XDG_DATA_HOME") or Path.home() / ".local" / "share"
    return Path(base) / "cwtrainer" / "stats.json"


@dataclass
class Config:
    wpm: int = 20
    tone: float = 600.0
    volume: float = 0.5
    mode: KeyerMode = KeyerMode.A
    swap: bool = False
    device: str | None = None
    samplerate: int = 48000
    blocksize: int = 128
    rise_time: float = 0.005
    # decoder thresholds, in units of the user's WPM; answer_timeout in seconds
    char_gap: float = 2.0
    word_gap: float = 5.0
    answer_timeout: float = 1.5
    stats_path: Path = field(default_factory=default_stats_path)
