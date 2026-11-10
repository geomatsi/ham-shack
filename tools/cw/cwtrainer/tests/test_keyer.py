import time

import pytest

from cwtrainer.clock import MonotonicClock
from cwtrainer.config import KeyerMode
from cwtrainer.keyer import IambicKeyer, KeyerRunner, simulate
from cwtrainer.morse import Element
from cwtrainer.paddle import PaddleEvent, ReplayPaddle, Side

U = 0.06  # 20 WPM
DIT, DAH = Side.DIT, Side.DAH


def run(script, mode=KeyerMode.A, until=5.0):
    events = ReplayPaddle([(t * U, s, p) for t, s, p in script]).events()
    return simulate(events, 20, mode, until)


def elements(kev):
    return "".join("." if e.element is Element.DIT else "-" for e in kev if e.down)


def marks(kev):
    """(start, end, element) in units."""
    out = []
    for ev in kev:
        if ev.down:
            start, el = ev.t, ev.element
        else:
            out.append((round(start / U, 6), round(ev.t / U, 6), el))
    return out


def test_single_dit_hold():
    kev = run([(0, DIT, True), (6.5, DIT, False)])
    assert elements(kev) == "...."  # starts at 0, 2, 4, 6
    assert marks(kev)[:2] == [(0, 1, Element.DIT), (2, 3, Element.DIT)]


def test_single_dah_hold():
    kev = run([(0, DAH, True), (5, DAH, False)])
    assert elements(kev) == "--"
    assert marks(kev) == [(0, 3, Element.DAH), (4, 7, Element.DAH)]


def test_short_tap_completes_element():
    assert elements(run([(0, DAH, True), (0.2, DAH, False)])) == "-"


def test_key_events_alternate_and_end_up():
    kev = run([(0, DIT, True), (3.5, DIT, False)])
    assert [e.down for e in kev] == [True, False, True, False]
    assert all(e.element is None for e in kev if not e.down)


def test_squeeze_starting_on_dit():
    kev = run([(0, DIT, True), (0.1, DAH, True), (7.5, DIT, False), (7.5, DAH, False)])
    assert elements(kev) == ".-."  # dit 0-1, dah 2-5, dit 6-7


def test_squeeze_starting_on_dah():
    kev = run([(0, DAH, True), (0.1, DIT, True), (7.5, DIT, False), (7.5, DAH, False)])
    assert elements(kev) == "-.-"  # dah 0-3, dit 4-5, dah 6-9


def test_squeeze_after_idle_starts_with_first_paddle():
    # a previous dit must not seed the next squeeze
    kev = run([(0, DIT, True), (0.5, DIT, False),
               (10, DAH, True), (10.1, DIT, True), (12.5, DIT, False), (12.5, DAH, False)])
    assert elements(kev) == ".-."


def test_memory_tap_during_element():
    # dah tapped and released inside a held dit's element
    kev = run([(0, DIT, True), (0.3, DAH, True), (0.6, DAH, False), (0.9, DIT, False)])
    assert elements(kev) == ".-"


def test_memory_tap_during_gap():
    kev = run([(0, DIT, True), (1.2, DAH, True), (1.6, DAH, False), (1.8, DIT, False)])
    assert elements(kev) == ".-"


def test_memory_same_paddle_not_latched():
    # a re-press of the paddle already sounding is not memorised
    kev = run([(0, DAH, True), (0.5, DAH, False), (1.0, DAH, True), (1.5, DAH, False)])
    assert elements(kev) == "-"


def test_alternation_after_single_paddle_element():
    # dit held, then dah added: next squeeze element is the opposite of the last one sent
    kev = run([(0, DIT, True), (2.5, DAH, True), (8.5, DIT, False), (8.5, DAH, False)])
    # dit 0-1, dit 2-3 (dah latched at 2.5), dah 4-7, dit 8-9
    assert elements(kev) == "..-."


def test_mode_a_release_finishes_current_element():
    # squeeze established before the dah starts, released during it
    script = [(0, DIT, True), (0.1, DAH, True), (2.5, DIT, False), (2.5, DAH, False)]
    assert elements(run(script, KeyerMode.A)) == ".-"


def test_mode_b_release_adds_opposite_element():
    script = [(0, DIT, True), (0.1, DAH, True), (2.5, DIT, False), (2.5, DAH, False)]
    assert elements(run(script, KeyerMode.B)) == ".-."


def test_mode_b_single_paddle_no_extra():
    assert elements(run([(0, DAH, True), (1, DAH, False)], KeyerMode.B)) == "-"


def test_mode_b_extra_is_only_one():
    script = [(0, DAH, True), (0.1, DIT, True), (1, DIT, False), (1, DAH, False)]
    # dit latched by memory, then mode B stops: the dit's element saw no squeeze
    assert elements(run(script, KeyerMode.B)) == "-."


def test_c_and_k():
    # C by squeeze starting on dah, released during the 4th element
    c = run([(0, DAH, True), (0.1, DIT, True), (10.5, DIT, False), (10.5, DAH, False)])
    assert elements(c) == "-.-."
    k = run([(0, DAH, True), (0.1, DIT, True), (6.5, DIT, False), (6.5, DAH, False)])
    assert elements(k) == "-.-"


def test_deadlines():
    out = []
    k = IambicKeyer(20, KeyerMode.A, out.append)
    assert k.next_deadline() is None
    k.on_paddle(PaddleEvent(DIT, True, 0.0))
    assert k.next_deadline() == pytest.approx(U)
    k.on_paddle(PaddleEvent(DIT, False, 0.01))
    k.advance(U)
    assert k.next_deadline() == pytest.approx(2 * U)
    k.advance(2 * U)
    assert k.next_deadline() is None


def test_runner_blocks_when_idle():
    k = IambicKeyer(20, KeyerMode.A, lambda e: None)
    r = KeyerRunner(k, MonotonicClock())
    r.start()
    time.sleep(0.2)
    wakeups = r.wakeups
    r.stop()
    assert wakeups == 0


def test_runner_wakes_per_boundary():
    clock = MonotonicClock()
    out = []
    k = IambicKeyer(60, KeyerMode.A, out.append)  # 20 ms unit
    r = KeyerRunner(k, clock)
    r.start()
    r.put(PaddleEvent(DIT, True, clock.now()))
    time.sleep(0.15)
    r.put(PaddleEvent(DIT, False, clock.now()))
    time.sleep(0.1)
    r.stop()
    n = sum(e.down for e in out)
    assert 3 <= n <= 5
    # two boundaries per element plus the two events and the stop
    assert r.wakeups <= 2 * n + 4
