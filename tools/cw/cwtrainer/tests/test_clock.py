from cwtrainer.clock import FakeClock, MonotonicClock


def test_fake_clock():
    c = FakeClock(1.0)
    c.advance(0.5)
    assert c.now() == 1.5
    c.sleep_until(1.0)
    assert c.now() == 1.5
    c.sleep_until(2.0)
    assert c.now() == 2.0


def test_monotonic_clock():
    c = MonotonicClock()
    t = c.now()
    c.sleep_until(t + 0.01)
    assert c.now() >= t + 0.01
