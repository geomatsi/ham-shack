"""Sidetone: one persistent output stream, keyed tone plus mixed-in playback."""

from __future__ import annotations

import threading
from typing import Protocol

import numpy as np

from cwtrainer.config import Config


class ToneGenerator:
    """Continuous-phase sine with a raised-cosine key envelope."""

    def __init__(self, samplerate: int, tone: float, volume: float, rise_time: float) -> None:
        self.samplerate = samplerate
        self.tone = tone
        self.volume = volume
        self.step = 1.0 / max(rise_time * samplerate, 1.0)  # ramp progress per sample
        self.keyed = False
        self._ramp = 0.0   # 0..1 position on the ramp
        self._phase = 0.0

    def render(self, n: int) -> np.ndarray:
        idle = not self.keyed and self._ramp == 0.0
        if idle:
            return np.zeros(n, dtype=np.float32)
        d = self.step if self.keyed else -self.step
        ramp = np.clip(self._ramp + d * np.arange(1, n + 1), 0.0, 1.0)
        self._ramp = float(ramp[-1])
        env = 0.5 - 0.5 * np.cos(np.pi * ramp)
        w = 2 * np.pi * self.tone / self.samplerate
        ph = self._phase + w * np.arange(n)
        self._phase = float((self._phase + w * n) % (2 * np.pi))
        return (self.volume * env * np.sin(ph)).astype(np.float32)


class AudioSink(Protocol):
    def key(self, down: bool) -> None: ...
    def play(self, samples: np.ndarray) -> None: ...
    def playing(self) -> bool: ...
    def wait(self, timeout: float | None = None) -> bool: ...
    def stop_playback(self) -> None: ...
    def close(self) -> None: ...


class Sidetone:
    def __init__(self, cfg: Config) -> None:
        import sounddevice as sd

        self.samplerate = cfg.samplerate
        self.gen = ToneGenerator(cfg.samplerate, cfg.tone, cfg.volume, cfg.rise_time)
        self._lock = threading.Lock()
        self._buf: np.ndarray | None = None
        self._pos = 0
        self._done = threading.Event()
        self._done.set()
        self.stream = sd.OutputStream(samplerate=cfg.samplerate, blocksize=cfg.blocksize,
                                      channels=1, dtype="float32", latency="low",
                                      callback=self._callback)
        self.stream.start()

    def _callback(self, outdata, frames, time, status) -> None:  # noqa: ARG002
        out = self.gen.render(frames)
        with self._lock:
            if self._buf is not None:
                chunk = self._buf[self._pos:self._pos + frames]
                out[:len(chunk)] += chunk
                self._pos += len(chunk)
                if self._pos >= len(self._buf):
                    self._buf = None
                    self._done.set()
        np.clip(out, -1.0, 1.0, out=outdata[:, 0])

    def key(self, down: bool) -> None:
        self.gen.keyed = down

    def play(self, samples: np.ndarray) -> None:
        """Mix samples into the output, replacing any playback in progress."""
        with self._lock:
            self._buf = samples.astype(np.float32)
            self._pos = 0
            self._done.clear()

    def playing(self) -> bool:
        return not self._done.is_set()

    def wait(self, timeout: float | None = None) -> bool:
        return self._done.wait(timeout)

    def stop_playback(self) -> None:
        with self._lock:
            self._buf = None
            self._done.set()

    def close(self) -> None:
        self.stream.stop()
        self.stream.close()


class NullSidetone:
    def __init__(self) -> None:
        self.keyed: list[bool] = []
        self.played: list[np.ndarray] = []

    def key(self, down: bool) -> None:
        self.keyed.append(down)

    def play(self, samples: np.ndarray) -> None:
        self.played.append(samples)

    def playing(self) -> bool:
        return False

    def wait(self, timeout: float | None = None) -> bool:
        return True

    def stop_playback(self) -> None:
        pass

    def close(self) -> None:
        pass
