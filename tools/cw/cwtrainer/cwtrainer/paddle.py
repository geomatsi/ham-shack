"""Paddle input sources."""

from __future__ import annotations

import threading
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from enum import Enum
from typing import Protocol

from cwtrainer.clock import Clock, MonotonicClock


class Side(Enum):
    DIT = 1
    DAH = 2

    @property
    def other(self) -> Side:
        return Side.DAH if self is Side.DIT else Side.DIT


@dataclass(frozen=True)
class PaddleEvent:
    side: Side
    pressed: bool
    t: float


@dataclass(frozen=True)
class ButtonEvent:
    """Adapter's own button."""
    t: float


InputEvent = PaddleEvent | ButtonEvent
Sink = Callable[[InputEvent], None]


class PaddleSource(Protocol):
    def start(self, sink: Sink) -> None: ...
    def close(self) -> None: ...


class ReplayPaddle:
    """Plays back (t, side, pressed) tuples, t relative to start()."""

    def __init__(self, script: Iterable[tuple[float, Side, bool]], clock: Clock | None = None) -> None:
        self.script = sorted(script, key=lambda s: s[0])
        self.clock = clock or MonotonicClock()
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None

    def events(self, t0: float = 0.0) -> list[PaddleEvent]:
        return [PaddleEvent(side, pressed, t0 + t) for t, side, pressed in self.script]

    def start(self, sink: Sink) -> None:
        def run() -> None:
            for ev in self.events(self.clock.now()):
                self.clock.sleep_until(ev.t)
                if self._stop.is_set():
                    return
                sink(ev)

        self._thread = threading.Thread(target=run, daemon=True, name="replay-paddle")
        self._thread.start()

    def close(self) -> None:
        self._stop.set()
