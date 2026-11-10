import pytest

from cwtrainer.grading import RST_EQUIVALENTS, Mistake, apply_hh, grade, normalise


def test_normalise():
    assert normalise("  ur  rst\t5nn ") == "UR RST 5NN"
    assert normalise("ur rst 5nn", RST_EQUIVALENTS) == "UR RST 599"


@pytest.mark.parametrize("sent,clean", [
    ("TNX <HH> TNX FER", "TNX FER"),
    ("TNX FER<HH> FER CALL", "TNX FER CALL"),
    ("TNX FR<HH>FER", "TNX FER"),
    ("<HH> CQ", "CQ"),
    ("CQ <HH> <HH>", ""),
    ("CQ DE", "CQ DE"),
])
def test_apply_hh(sent, clean):
    assert apply_hh(sent) == clean


def test_perfect():
    g = grade("TNX FER CALL", "tnx  fer call")
    assert g.distance == 0 and g.score == 1.0 and g.mistakes == []
    assert g.attempted == list("TNXFERCALL")


def test_rst_equivalence():
    assert grade("UR RST 599", "UR RST 5NN", RST_EQUIVALENTS).distance == 0
    assert grade("UR RST 599", "UR RST 5NN").distance == 2


def test_hh_correction_is_clean():
    assert grade("TNX FER CALL", "TNX FR <HH> FER CALL").distance == 0


@pytest.mark.parametrize("sent,mistakes", [
    ("TNK FR", ["K for X", "missing E"]),
    ("TNX FERR", ["extra R"]),
    ("TNXFER", ["missing space"]),
    ("TN X FER", ["extra space"]),
])
def test_mistakes(sent, mistakes):
    g = grade("TNX FER", sent)
    assert [str(m) for m in g.mistakes] == mistakes
    assert g.distance == len(mistakes)
    assert g.score == pytest.approx(1 - len(mistakes) / 7)


def test_wrong_excludes_extras_and_spaces():
    from cwtrainer.scenarios.base import Result

    g = grade("TNX FER", "TNKFRR")
    assert Result.from_grade(g).wrong == ["X", "E"]


def test_prosign_is_one_token():
    g = grade("TU <SK>", "TU <KN>")
    assert g.distance == 1 and g.mistakes == [Mistake("<SK>", "<KN>")]
    assert grade("<BT>", "=").distance == 0


def test_empty_and_garbage():
    assert grade("CQ", "").score == 0.0
    assert grade("CQ", "XXXXXXXX").score == 0.0
