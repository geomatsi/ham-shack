"""Render text to audio with the sidetone's envelope."""

from __future__ import annotations

import numpy as np

from cwtrainer.morse import timeline
from cwtrainer.sidetone import AudioSink, ToneGenerator


def render(text: str, wpm: float, tone: float, volume: float, samplerate: int, rise_time: float,
           effective_wpm: float | None = None) -> np.ndarray:
    gen = ToneGenerator(samplerate, tone, volume, rise_time)
    marks, length = timeline(text, wpm, effective_wpm)
    parts: list[np.ndarray] = []
    pos = 0  # samples rendered
    for m in marks:
        for keyed, t in ((False, m.start), (True, m.end)):
            n = round(t * samplerate) - pos
            gen.keyed = keyed
            parts.append(gen.render(n))
            pos += n
    gen.keyed = False
    tail = round((length + rise_time) * samplerate) - pos
    parts.append(gen.render(max(tail, 0)))
    return np.concatenate(parts) if parts else np.zeros(0, dtype=np.float32)


class MorsePlayer:
    def __init__(self, audio: AudioSink, samplerate: int, volume: float, rise_time: float) -> None:
        self.audio = audio
        self.samplerate = samplerate
        self.volume = volume
        self.rise_time = rise_time

    def play(self, text: str, wpm: float, tone: float, effective_wpm: float | None = None) -> float:
        """Start playback; returns its length in seconds."""
        buf = render(text, wpm, tone, self.volume, self.samplerate, self.rise_time, effective_wpm)
        self.audio.play(buf)
        return len(buf) / self.samplerate
