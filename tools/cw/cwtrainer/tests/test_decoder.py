import pytest

from cwtrainer.decoder import Decoder
from cwtrainer.keyer import KeyEvent
from cwtrainer.morse import Element, timeline, unit


def events(text, wpm, effective=None, t0=0.0, keyer_elements=True):
    marks, length = timeline(text, wpm, effective)
    out = []
    for m in marks:
        out += [KeyEvent(True, t0 + m.start, m.element if keyer_elements else None),
                KeyEvent(False, t0 + m.end, None)]
    return out, t0 + length


def decode(evs, end, d):
    live = "".join(d.feed(e) for e in evs) + d.poll(end + 10)
    assert d.answer_complete(end + d.answer_timeout + 1e-9)
    return live, d.finish()


@pytest.mark.parametrize("wpm", [12, 20, 28, 35])
@pytest.mark.parametrize("text", ["CQ CQ DE R3XYZ K", "UR RST 5NN", "TNX FER QSO <SK>", "73 ES GL <AR>"])
def test_roundtrip(text, wpm):
    evs, end = events(text, wpm)
    live, final = decode(evs, end, Decoder(wpm))
    assert final == text
    assert live == text


def two_dits(gap_units):
    u = unit(20)
    return [KeyEvent(True, 0, Element.DIT), KeyEvent(False, u, None),
            KeyEvent(True, u + gap_units * u, Element.DIT),
            KeyEvent(False, 2 * u + gap_units * u, None)], 2 * u + gap_units * u


@pytest.mark.parametrize("gap,text", [(1, "I"), (1.9, "I"), (2.0, "EE"), (3.5, "EE"), (5.0, "E E"), (8, "E E")])
def test_gap_thresholds(gap, text):
    evs, end = two_dits(gap)
    assert decode(evs, end, Decoder(20))[1] == text


def test_bt_decodes_as_equals():
    evs, end = events("<BT>", 20)
    assert decode(evs, end, Decoder(20))[1] == "="


def test_unknown_is_star():
    u = unit(20)
    evs, t = [], 0.0
    for _ in range(6):
        evs += [KeyEvent(True, t, Element.DAH), KeyEvent(False, t + 3 * u, None)]
        t += 4 * u
    assert decode(evs, t, Decoder(20))[1] == "*"


@pytest.mark.parametrize("dits", [6, 8, 10])
def test_hh(dits):
    u = unit(20)
    evs, t = [], 0.0
    for _ in range(dits):
        evs += [KeyEvent(True, t, Element.DIT), KeyEvent(False, t + u, None)]
        t += 2 * u
    assert decode(evs, t, Decoder(20))[1] == "<HH>"


def test_hh_inside_text():
    evs, end = events("TNX <HH> TU", 20)
    assert decode(evs, end, Decoder(20))[1] == "TNX <HH> TU"


def test_five_dits_is_five_not_hh():
    evs, end = events("5NN", 20)
    assert decode(evs, end, Decoder(20))[1] == "5NN"


def test_straight_key_classified_by_length():
    evs, end = events("K", 20, keyer_elements=False)
    assert decode(evs, end, Decoder(20))[1] == "K"


def test_poll_flushes_char_and_answer_timeout():
    u = unit(20)
    d = Decoder(20, answer_timeout=1.5)
    assert d.idle
    d.feed(KeyEvent(True, 0, Element.DIT))
    d.feed(KeyEvent(False, u, None))
    assert d.poll(u + u) == ""        # still within the character
    assert d.poll(u + 2 * u) == "E"
    assert not d.answer_complete(1.0)
    assert d.answer_complete(u + 1.5)
    d.reset()
    assert d.idle and d.text == ""


def test_no_answer_complete_while_key_down():
    d = Decoder(20)
    d.feed(KeyEvent(True, 0, Element.DAH))
    assert not d.answer_complete(100)


@pytest.mark.parametrize("wpm", [15, 25])
def test_keyer_to_decoder(wpm):
    from cwtrainer.config import KeyerMode
    from cwtrainer.keyer import simulate
    from cwtrainer.paddle import ReplayPaddle, Side
    from scripts import paddle_script, squeeze_script

    script, end = paddle_script("CQ DE OH2XX", wpm)
    u = unit(wpm)
    # then a squeezed C (dah first, released during the 4th element)
    script += squeeze_script(Side.DAH, 10.5 * u, t0=end + 7 * u)
    kev = simulate(ReplayPaddle(script).events(), wpm, KeyerMode.A, end + 30 * u)
    d = Decoder(wpm)
    for e in kev:
        d.feed(e)
    assert d.finish() == "CQ DE OH2XX C"
