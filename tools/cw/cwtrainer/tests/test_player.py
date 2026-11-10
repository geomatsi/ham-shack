import numpy as np
import pytest

from cwtrainer.morse import timeline
from cwtrainer.player import MorsePlayer, render
from cwtrainer.sidetone import NullSidetone

SR = 8000


def test_render_length_and_marks():
    buf = render("TE", 20, 600, 0.5, SR, 0.005)
    marks, length = timeline("TE", 20)
    assert len(buf) == round((length + 0.005) * SR)
    for m in marks:
        mid = round((m.start + m.end) / 2 * SR)
        assert np.abs(buf[mid - 20:mid + 20]).max() > 0.4
    gap = round((marks[0].end + 0.01) * SR), round((marks[1].start - 0.001) * SR)
    assert not buf[gap[0]:gap[1]].any()


def test_player_queues_audio():
    audio = NullSidetone()
    d = MorsePlayer(audio, SR, 0.5, 0.005).play("E", 20, 600)
    assert len(audio.played) == 1
    assert d == pytest.approx(0.06 + 0.005, abs=1 / SR)


def test_play_voices_mixes_with_delay_and_tone():
    from cwtrainer.scenarios.base import Voice

    audio = NullSidetone()
    pl = MorsePlayer(audio, SR, 0.5, 0.005)
    d = pl.play_voices([Voice("E"), Voice("E", wpm=10, tone_offset=-100, delay=0.5)], 20, 600)
    buf = audio.played[0]
    assert d == pytest.approx(0.5 + 0.12 + 0.005, abs=1 / SR)
    assert np.abs(buf[round(0.03 * SR)]) > 0 or np.abs(buf[round(0.03 * SR) + 1]) > 0
    assert not buf[round(0.1 * SR):round(0.49 * SR)].any()
    assert np.abs(buf[round(0.55 * SR):round(0.6 * SR)]).max() > 0.4
    assert pl.play_voices([], 20, 600) == 0.0
