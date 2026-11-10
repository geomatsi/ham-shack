import time

import numpy as np
import pytest

from cwtrainer.config import Config
from cwtrainer.keyer import KeyEvent
from cwtrainer.paddle import ReplayPaddle, Side
from cwtrainer.rig import Rig
from cwtrainer.sidetone import NullSidetone, ToneGenerator

SR = 48000


def keyed(gen, on, n):
    gen.keyed = on
    return gen.render(n)


def test_silent_when_idle():
    g = ToneGenerator(SR, 600, 0.5, 0.005)
    assert not keyed(g, False, 128).any()


def test_rise_reaches_full_volume_and_falls_to_zero():
    g = ToneGenerator(SR, 600, 0.5, 0.005)
    rise = int(0.005 * SR)
    x = np.concatenate([keyed(g, True, 2 * rise), keyed(g, False, 2 * rise)])
    peak = np.abs(x[rise:2 * rise]).max()
    assert peak == pytest.approx(0.5, abs=0.01)
    assert np.abs(x[:rise // 10]).max() < 0.02          # soft start
    assert not x[-rise // 2:].any()                       # fully off after the fall


def test_no_clicks_across_blocks_and_toggles():
    g = ToneGenerator(SR, 600, 1.0, 0.005)
    parts = [keyed(g, on, n) for on, n in [(True, 100), (True, 37), (False, 50), (True, 300), (False, 900)]]
    x = np.concatenate(parts)
    # a pure 600 Hz full-scale sine changes by at most 2*pi*600/SR per sample
    assert np.abs(np.diff(x)).max() <= 2 * np.pi * 600 / SR + 1e-6


def test_rig_keys_sidetone_and_reports_events():
    audio = NullSidetone()
    paddle = ReplayPaddle([(0.0, Side.DAH, True), (0.05, Side.DAH, False)])
    with Rig(Config(wpm=40), paddle, audio) as rig:
        time.sleep(0.25)
    kev = []
    while (ev := rig.get(0)) is not None:
        kev.append(ev)
    assert [e.down for e in kev] == [True, False]
    assert all(isinstance(e, KeyEvent) for e in kev)
    assert kev[1].t - kev[0].t == pytest.approx(3 * 1.2 / 40)
    assert audio.keyed == [True, False, False]  # last from shutdown
