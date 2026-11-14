import random

from cwtrainer.morse import text_to_elements
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.echo import PromptMode, load_lines
from cwtrainer.scenarios.qso import Qso


def test_registered():
    assert SCENARIOS["qso"] is Qso


def test_data_is_sendable():
    lines = load_lines("phrases.txt")
    assert len(lines) > 20
    for ln in lines:
        text_to_elements(ln)


def test_modes():
    for mode, show, play in [(PromptMode.SHOW, True, False), (PromptMode.LISTEN, False, True),
                             (PromptMode.BOTH, True, True)]:
        p = Qso(["CQ CQ"], mode, rng=random.Random(1)).next_prompt()
        assert (p.show_text, p.play_audio) == (show, play)


def test_count_and_check():
    e = Qso(["UR RST 599"], count=2, rng=random.Random(0))
    assert not e.done()
    p = e.next_prompt()
    assert p.text == "UR RST 599"
    assert e.check("ur rst 5nn").ok
    r = e.check("UR RST 589")
    assert not r.ok and r.notes == ["8 for 9"] and r.wrong == ["9"]
    e.next_prompt()
    assert e.done()


def test_endless():
    e = Qso(["A", "B"], count=0, rng=random.Random(0))
    for _ in range(50):
        e.next_prompt()
    assert not e.done()


def test_no_immediate_repeat():
    e = Qso(["A", "B"], count=0, rng=random.Random(3))
    seq = [e.next_prompt().text for _ in range(30)]
    assert all(a != b for a, b in zip(seq, seq[1:]))
