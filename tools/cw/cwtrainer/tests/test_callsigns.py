import random
import re

import pytest

from cwtrainer.callgen import DIGIT_PREFIX, ONE_LETTER, PATTERNS, TWO_LETTER, CallsignGenerator, Pattern
from cwtrainer.morse import text_to_elements
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.callsigns import Callsigns

PORTABLE = r"(/P|/M|/QRP)?"


def gen(seed=1, **kw):
    return CallsignGenerator(random.Random(seed), **kw)


@pytest.mark.parametrize("pattern", PATTERNS, ids=str)
def test_pattern_shape(pattern):
    g = gen(portable=0.0, digit_prefix=0.0)
    rx = re.compile(rf"[A-Z]{{{pattern.prefix}}}[0-9][A-Z]{{{pattern.suffix}}}")
    prefixes = ONE_LETTER if pattern.prefix == 1 else TWO_LETTER
    for _ in range(200):
        c = g.generate(pattern=pattern)
        assert rx.fullmatch(c), c
        assert c[:pattern.prefix] in prefixes


def test_digit_prefix():
    g = gen(portable=0.0, digit_prefix=1.0)
    for _ in range(100):
        c = g.generate(pattern=Pattern(2, 2))
        assert c[:2] in DIGIT_PREFIX and re.fullmatch(r"..[0-9][A-Z]{2}", c), c


def test_portable_suffix():
    g = gen(portable=1.0)
    calls = [g.generate() for _ in range(100)]
    assert all(re.search(r"/(P|M|QRP)$", c) for c in calls)
    assert {c.rsplit("/", 1)[1] for c in calls} == {"P", "M", "QRP"}
    assert not any("/" in gen(portable=0.0).generate() for _ in range(100))


def test_levels_restrict_patterns_and_extras():
    g = gen(portable=1.0, digit_prefix=1.0)
    for _ in range(200):
        c = g.generate(level=1)
        assert re.fullmatch(r"[A-Z]{1}[0-9][A-Z]{2}|[A-Z]{2}[0-9][A-Z]{1}", c), c
    assert all(len(g.generate(level=2)) <= 5 for _ in range(200))
    assert not any("/" in g.generate(level=4) for _ in range(100))


def test_all_sendable_and_deterministic():
    g = gen(7, portable=0.5, digit_prefix=0.5)
    for _ in range(300):
        text_to_elements(g.generate())
    assert [gen(3).generate() for _ in range(5)] == [gen(3).generate() for _ in range(5)]


def test_mix_of_patterns():
    g = gen(portable=0.0, digit_prefix=0.0)
    shapes = {re.sub(r"[A-Z]", "A", re.sub(r"[0-9]", "9", g.generate())) for _ in range(500)}
    assert shapes == {"A9AA", "AA9A", "A9AAA", "AA9AA", "AA9AAA"}


def test_scenario():
    assert SCENARIOS["callsigns"] is Callsigns
    s = Callsigns(gen(), count=3)
    p = s.next_prompt()
    assert p.show_text and not p.play_audio
    assert s.check(p.text.lower()).ok
    assert not s.check("5NN").ok
    s.next_prompt(), s.next_prompt()
    assert s.done()


def test_ramp_up_and_down():
    s = Callsigns(gen(), count=0, ramp=True)
    assert s.level == 1
    for _ in range(Callsigns.WINDOW):
        r = s.check(s.next_prompt().text)
    assert s.level == 2 and r.notes == ["level 2"]
    for _ in range(Callsigns.WINDOW):
        s.check(s.next_prompt().text)
    assert s.level == 3
    for _ in range(Callsigns.WINDOW):
        s.next_prompt()
        s.check("")
    assert s.level == 2


def test_no_ramp_uses_all_levels():
    assert Callsigns(gen()).level == 5
