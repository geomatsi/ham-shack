"""Wiring: paddle -> keyer -> sidetone, with everything for the UI on one queue."""

from __future__ import annotations

import queue
from typing import Union

from cwtrainer.clock import Clock, MonotonicClock
from cwtrainer.config import Config
from cwtrainer.keyer import IambicKeyer, KeyerRunner, KeyEvent
from cwtrainer.paddle import ButtonEvent, InputClosed, InputEvent, PaddleEvent, PaddleSource
from cwtrainer.sidetone import AudioSink

UiEvent = Union[KeyEvent, ButtonEvent, InputClosed]


class Rig:
    def __init__(self, cfg: Config, paddle: PaddleSource, audio: AudioSink,
                 clock: Clock | None = None) -> None:
        self.cfg = cfg
        self.clock = clock or MonotonicClock()
        self.paddle = paddle
        self.audio = audio
        self.events: queue.Queue[UiEvent] = queue.Queue()
        self.keyer = IambicKeyer(cfg.wpm, cfg.mode, self._on_key)
        self.runner = KeyerRunner(self.keyer, self.clock)

    def _on_key(self, ev: KeyEvent) -> None:
        self.audio.key(ev.down)
        self.events.put(ev)

    def _on_input(self, ev: InputEvent) -> None:
        if isinstance(ev, PaddleEvent):
            self.runner.put(ev)
        else:
            self.events.put(ev)

    def get(self, timeout: float | None) -> UiEvent | None:
        try:
            return self.events.get(timeout=timeout)
        except queue.Empty:
            return None

    def __enter__(self) -> Rig:
        self.runner.start()
        self.paddle.start(self._on_input)
        return self

    def __exit__(self, *exc: object) -> None:
        self.paddle.close()
        self.runner.stop()
        self.audio.key(False)
        self.audio.close()
