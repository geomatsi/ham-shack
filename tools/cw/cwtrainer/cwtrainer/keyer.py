"""Iambic keyer: pure state machine plus a thin threaded runner."""

from __future__ import annotations

import queue
import threading
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from enum import Enum, auto

from cwtrainer.clock import Clock
from cwtrainer.config import KeyerMode
from cwtrainer.morse import Element, unit
from cwtrainer.paddle import PaddleEvent, Side


@dataclass(frozen=True)
class KeyEvent:
    down: bool
    t: float
    element: Element | None  # set on key-down


KeySink = Callable[[KeyEvent], None]

_ELEMENT = {Side.DIT: Element.DIT, Side.DAH: Element.DAH}
_SIDE = {v: k for k, v in _ELEMENT.items()}


class Phase(Enum):
    IDLE = auto()
    ON = auto()   # element sounding
    GAP = auto()  # inter-element space after it


class IambicKeyer:
    """Feed paddle events in time order via on_paddle(); call advance(now) at next_deadline()."""

    def __init__(self, wpm: float, mode: KeyerMode, sink: KeySink) -> None:
        self.mode = mode
        self.sink = sink
        self.set_wpm(wpm)
        self.down: dict[Side, bool] = {Side.DIT: False, Side.DAH: False}
        self.phase = Phase.IDLE
        self.phase_end = 0.0
        self.current: Side | None = None   # element on air or just sent
        self.memory: Side | None = None
        self.squeezed = False              # both paddles down during current element (mode B)

    def set_wpm(self, wpm: float) -> None:
        self.unit = unit(wpm)

    def next_deadline(self) -> float | None:
        return None if self.phase is Phase.IDLE else self.phase_end

    def on_paddle(self, ev: PaddleEvent) -> None:
        self.advance(ev.t)
        self.down[ev.side] = ev.pressed
        if self.phase is Phase.IDLE:
            if ev.pressed:
                self._start(ev.side, ev.t)
            return
        if ev.pressed and ev.side is not self.current:
            self.memory = ev.side
        if self.phase is Phase.ON and all(self.down.values()):
            self.squeezed = True

    def advance(self, now: float) -> None:
        while self.phase is not Phase.IDLE and self.phase_end <= now:
            t = self.phase_end
            if self.phase is Phase.ON:
                self.sink(KeyEvent(False, t, None))
                self.phase = Phase.GAP
                self.phase_end = t + self.unit
            else:
                nxt = self._choose()
                if nxt is None:
                    self.phase = Phase.IDLE
                else:
                    self._start(nxt, t)

    def _choose(self) -> Side | None:
        assert self.current is not None
        if self.memory is not None:
            return self.memory
        if all(self.down.values()):
            return self.current.other
        for side in Side:
            if self.down[side]:
                return side
        if self.mode is KeyerMode.B and self.squeezed:
            return self.current.other
        return None

    def _start(self, side: Side, t: float) -> None:
        self.current = side
        self.memory = None
        self.squeezed = all(self.down.values())
        self.phase = Phase.ON
        self.phase_end = t + (self.unit if side is Side.DIT else 3 * self.unit)
        self.sink(KeyEvent(True, t, _ELEMENT[side]))


def simulate(events: Iterable[PaddleEvent], wpm: float, mode: KeyerMode, until: float) -> list[KeyEvent]:
    """Run the keyer offline over a paddle script."""
    out: list[KeyEvent] = []
    k = IambicKeyer(wpm, mode, out.append)
    for ev in sorted(events, key=lambda e: e.t):
        k.on_paddle(ev)
    k.advance(until)
    return out


class KeyerRunner:
    """Runs a keyer in its own thread; blocks on input until the next element boundary."""

    def __init__(self, keyer: IambicKeyer, clock: Clock) -> None:
        self.keyer = keyer
        self.clock = clock
        self.q: queue.Queue[PaddleEvent | None] = queue.Queue()
        self._thread = threading.Thread(target=self._run, daemon=True, name="keyer")
        self.wakeups = 0

    def start(self) -> None:
        self._thread.start()

    def put(self, ev: PaddleEvent) -> None:
        self.q.put(ev)

    def stop(self) -> None:
        self.q.put(None)
        self._thread.join()

    def _run(self) -> None:
        while True:
            deadline = self.keyer.next_deadline()
            timeout = None if deadline is None else max(deadline - self.clock.now(), 0.0)
            try:
                ev = self.q.get(timeout=timeout)
            except queue.Empty:
                self.wakeups += 1
                self.keyer.advance(self.clock.now())
                continue
            self.wakeups += 1
            if ev is None:
                return
            self.keyer.on_paddle(ev)
