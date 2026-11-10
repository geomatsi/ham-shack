import pytest

from cwtrainer.morse import (
    CODE, Element, char_to_elements, elements_to_char, text_to_elements, timeline, tokenize, unit,
)

D, A = Element.DIT, Element.DAH


def test_unit():
    assert unit(20) == pytest.approx(0.06)


def test_tokenize_prosigns_and_spaces():
    assert tokenize("  cq  de <ar>x ") == ["C", "Q", " ", "D", "E", " ", "<AR>", "X"]
    assert tokenize("a <BT> b") == ["A", " ", "=", " ", "B"]


def test_roundtrip_every_code():
    for ch in CODE:
        assert elements_to_char(char_to_elements(ch)) == ch


def test_text_to_elements():
    assert text_to_elements("AN T") == [(D, A), (A, D), None, (A,)]


def test_unknown():
    assert elements_to_char([A, A, A, A, A, A]) is None
    with pytest.raises(KeyError):
        text_to_elements("#")


def test_timeline_paris_is_50_units():
    # PARIS plus the trailing word gap = 50 units
    marks, length = timeline("PARIS", 20)
    assert length + 7 * unit(20) == pytest.approx(50 * unit(20))
    assert marks[0].start == 0 and marks[0].element is D


def test_timeline_gaps():
    u = unit(20)
    marks, length = timeline("E E", 20)
    assert [(m.start, m.end) for m in marks] == [(0, pytest.approx(u)), (pytest.approx(8 * u), pytest.approx(9 * u))]
    assert length == pytest.approx(9 * u)


def test_farnsworth_stretches_only_gaps():
    m20, _ = timeline("EE", 20)
    mf, _ = timeline("EE", 20, effective_wpm=10)
    assert mf[0] == m20[0]
    assert mf[1].end - mf[1].start == pytest.approx(m20[1].end - m20[1].start)
    assert mf[1].start > m20[1].start
