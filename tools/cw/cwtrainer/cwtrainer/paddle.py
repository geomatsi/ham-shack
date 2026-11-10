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


@dataclass(frozen=True)
class InputClosed:
    """Source went away (e.g. adapter unplugged)."""
    t: float
    reason: str


InputEvent = PaddleEvent | ButtonEvent | InputClosed
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


BTN_LEFT, BTN_RIGHT = 272, 273
EV_KEY, EV_REL, REL_Y = 1, 2, 1


class EvdevPaddle:
    """vBand-style adapter: paddles are mouse buttons, the adapter button sends EV_REL REL_Y."""

    BUTTON_DEBOUNCE = 0.3

    def __init__(self, path: str, clock: Clock | None = None, dit_code: int = BTN_LEFT,
                 dah_code: int = BTN_RIGHT, button: tuple[int, int] = (EV_REL, REL_Y),
                 swap: bool = False) -> None:
        from evdev import InputDevice

        self.clock = clock or MonotonicClock()
        self.dev = InputDevice(path)
        self.sides = {dit_code: Side.DIT, dah_code: Side.DAH}
        if swap:
            self.sides = {dit_code: Side.DAH, dah_code: Side.DIT}
        self.button = button
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None

    def start(self, sink: Sink) -> None:
        self.dev.grab()
        self._thread = threading.Thread(target=self._run, args=(sink,), daemon=True, name="evdev")
        self._thread.start()

    def _run(self, sink: Sink) -> None:
        import select

        last_button = -1e9
        while not self._stop.is_set():
            r, _, _ = select.select([self.dev.fd], [], [], 0.2)
            if not r:
                continue
            try:
                events = list(self.dev.read())
            except BlockingIOError:
                continue
            except OSError as e:  # unplugged
                sink(InputClosed(self.clock.now(), str(e)))
                return
            now = self.clock.now()
            for ev in events:
                if ev.type == EV_KEY and ev.code in self.sides and ev.value in (0, 1):
                    sink(PaddleEvent(self.sides[ev.code], bool(ev.value), now))
                elif (ev.type, ev.code) == self.button and now - last_button > self.BUTTON_DEBOUNCE:
                    last_button = now
                    sink(ButtonEvent(now))

    def close(self) -> None:
        self._stop.set()
        if self._thread is not None:
            self._thread.join()
        try:
            self.dev.ungrab()
        except OSError:
            pass
        self.dev.close()
