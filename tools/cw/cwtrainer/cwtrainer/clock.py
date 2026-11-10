"""Time sources. Logic takes a Clock so tests can drive time by hand."""

from __future__ import annotations

import time
from typing import Protocol


class Clock(Protocol):
    def now(self) -> float: ...
    def sleep_until(self, t: float) -> None: ...


class MonotonicClock:
    def now(self) -> float:
        return time.monotonic()

    def sleep_until(self, t: float) -> None:
        dt = t - time.monotonic()
        if dt > 0:
            time.sleep(dt)


class FakeClock:
    def __init__(self, t: float = 0.0) -> None:
        self.t = t

    def now(self) -> float:
        return self.t

    def sleep_until(self, t: float) -> None:
        self.t = max(self.t, t)

    def advance(self, dt: float) -> None:
        self.t += dt
