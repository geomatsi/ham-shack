import random
import re

import pytest

from cwtrainer.clock import FakeClock
from cwtrainer.main import build_parser
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.hunter import CQ_FORMS, PotaHunter, Realism, Step, bust, overs
from cwtrainer.scenarios.pota import Station

MY = "R3XYZ"


def hunt(seed=1, log=False, **realism):
    return PotaHunter(MY, count=0, realism=Realism(**realism), log=log, rng=random.Random(seed),
                      clock=FakeClock())


def heard(h):
    return [v.text for v in h.next_prompt().voices]


def test_registered_and_cli():
    assert SCENARIOS["pota-hunter"] is PotaHunter
    a = build_parser().parse_args(["pota-hunter", "--mycall", "r3xyz", "--busted", "--log",
                                   "--activator-wpm", "20-30"])
    assert (a.mycall, a.busted, a.pileup, a.log, a.activator_wpm) == (MY, 0.2, 0.0, True, (20, 30))


def test_starts_with_cq():
    h = hunt()
    a = h.act
    (cq,) = heard(h)
    assert cq in {f.format(a=a.call, p=a.park) for f in CQ_FORMS}
    assert re.fullmatch(r"[A-Z]+-\d{4}", a.park)
    p = h.next_prompt()
    assert p.play_audio and not p.show_text and p.instruction is None


def test_basic_qso():
    h = hunt()
    first = h.act
    h.clock.advance(15)
    assert h.check("R3XYZ R3XYZ").ok and h.step is Step.EXCHANGE
    assert heard(h)[0].startswith(MY) and "5" in heard(h)[0]
    assert h.check("R 5NN TU").ok
    assert h.log[0].call == first.call and h.log[0].seconds == 15
    # signoff of the first, then a new activator's CQ, in one prompt
    v = h.next_prompt().voices
    assert h.act is not first and len(v) == 2 and v[1].delay > v[0].delay
    assert v[0].wpm == first.wpm and v[1].text.startswith("CQ")


def test_garbled_call_answered_as_sent():
    h = hunt()
    r = h.check("R3XYS")
    assert not r.ok and h.step is Step.BUSTED and heard(h)[0].startswith("R3XYS")
    r = h.check("R 5NN TU")
    assert not r.ok and r.notes == ["copied as R3XYS: resend your call"]
    assert len(h.log) == 1 and h.busts == 1 and h.step is Step.CALL


def test_busted_corrected():
    h = hunt(busted=1.0)
    assert h.check(MY).ok and h.step is Step.BUSTED
    wrong = h.copied
    assert wrong != MY and len(wrong) == len(MY) and wrong[:2] == "R3"
    assert h.check(f"{MY} {MY}").ok and heard(h) == [f"R {MY} 5NN"]
    assert h.check("TU 5NN").ok and h.busts == 0


def test_unreadable_call():
    h = hunt()
    r = h.check("TEST")
    assert not r.ok and r.notes == ["send your call"] and r.attempted == []
    assert heard(h)[0] in {"?", "QRZ?", "AGN?"} and h.step is Step.CALL


def test_repeat_request():
    h = hunt()
    before = heard(h)
    r = h.check("AGN?")
    assert r.notes == ["repeat"] and heard(h) == before


def test_partial_then_full_call():
    h = hunt(partial=1.0)
    h.check(MY)
    (q,) = heard(h)
    assert q in {"R3?", "XYZ?"} and h.step is Step.CALL
    assert h.check(MY).ok and h.step is Step.EXCHANGE  # partial only once per QSO


def test_pileup_other_worked_first():
    h = hunt(pileup=1.0)
    r = h.check(MY)
    assert r.ok and h.step is Step.CALL
    v = h.next_prompt().voices
    assert len(v) == 3 and not v[0].text.startswith(MY) and v[2].text == "TU QRZ"
    assert v[0].delay < v[1].delay < v[2].delay
    assert h.check(MY).ok and h.step is Step.EXCHANGE


def test_agn_and_missing_rst():
    h = hunt(agn=1.0)
    h.check(MY)
    r = h.check("TU")
    assert not r.ok and r.notes == ["no RST"] and heard(h)[0] in {"AGN?", "RST?"}
    assert h.check("R 599 TU").ok and heard(h)[0] in {"AGN?", "?"}
    assert h.check("5NN TU").ok and len(h.log) == 1


def test_log_step():
    h = hunt(log=True)
    a = h.act
    h.check(MY)
    h.check("5NN TU")
    p = h.next_prompt()
    assert h.step is Step.LOG and p.instruction.startswith("Log")
    r = h.check(f"{a.call} {a.park[:-1]}")
    assert not r.ok and r.wrong
    assert h.step is Step.CALL and h.act is not a


def test_log_cq_always_has_park():
    for seed in range(20):
        h = hunt(seed, log=True)
        assert h.act.park in heard(h)[-1]


def test_log_step_ok():
    h = hunt(log=True)
    a = h.act
    h.check(MY)
    h.check("5NN TU")
    assert h.check(f"{a.call} {a.park}").ok


@pytest.mark.parametrize("call", ["R3XYZ", "OH2AB/P", "9A1X"])
def test_bust(call):
    rng = random.Random(0)
    for _ in range(30):
        b = bust(call, rng)
        assert b != call and len(b) == len(call)
        diff = [i for i, (x, y) in enumerate(zip(call, b)) if x != y]
        assert len(diff) == 1 and call[diff[0]].isalpha() and diff[0] > 1


def test_overs_back_to_back():
    s = Station("K1AB", 20, 0)
    v = overs([(s, "E"), (s, "E")])
    assert v[0].delay == 0 and v[1].delay == pytest.approx(0.06 + 0.8)


def test_summary():
    h = hunt()
    for _ in range(3):
        h.clock.advance(10)
        h.check(MY)
        h.check("5NN TU")
    assert h.summary()[0] == "3 QSOs, accuracy 100%, 10 s per QSO"
