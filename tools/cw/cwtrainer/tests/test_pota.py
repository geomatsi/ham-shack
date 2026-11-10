import random

import pytest

from cwtrainer.clock import FakeClock
from cwtrainer.main import build_parser
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.pota import Pota, Realism, Step, partial_call, words_of

MY = "R3XYZ"


def pota(seed=1, **realism):
    r = Realism(**{"no_answer": 0.0, **realism})
    return Pota(MY, "OH-0001", count=0, realism=r, rng=random.Random(seed), clock=FakeClock())


def heard(p):
    return " ".join(v.text for v in p.next_prompt().voices)


def test_registered_and_cli():
    assert SCENARIOS["pota"] is Pota
    a = build_parser().parse_args(["pota", "--mycall", "r3xyz", "--park", "oh-0001", "--agn",
                                   "--pileup", "0.5", "--hunter-wpm", "15-20"])
    assert (a.mycall, a.park, a.agn, a.pileup, a.partial, a.hunter_wpm) == \
        (MY, "OH-0001", 0.2, 0.5, 0.0, (15, 20))


def test_words_of():
    assert words_of("oh2 ?  tnx <HH> TU") == ["OH2?", "TU"]


def test_cq_needs_cq_and_mycall():
    p = pota()
    pr = p.next_prompt()
    assert pr.instruction == "Call CQ" and not pr.play_audio
    r = p.check("CQ POTA DE R3XY K")
    assert not r.ok and r.notes == ["missing Z"] and p.step is Step.CQ
    assert p.check("cq cq pota de r3xyz r3xyz k").ok
    assert p.step is Step.EXCHANGE and 1 <= len(p.queue) <= 3


def test_no_answer():
    p = Pota(MY, realism=Realism(no_answer=1.0), rng=random.Random(1))
    p.check(f"CQ POTA DE {MY} K")
    assert p.step is Step.CQ and p.next_prompt().instruction == "No answer, call CQ again"


def test_basic_qso_flow():
    p = pota()
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    pr = p.next_prompt()
    assert pr.play_audio and not pr.show_text and pr.text == h.call
    assert pr.voices[0].wpm == h.wpm and abs(pr.voices[0].tone_offset) <= 100
    p.clock.advance(12)
    r = p.check(f"{h.call} 5NN")
    assert r.ok and p.step is Step.FINAL
    assert heard(p) in {"R 5NN TU", "TU 73", "R TU 5NN", "RR 5NN TU 73", "TU 5NN 73"}
    n = len(p.queue)
    assert p.check("TU 73").ok
    assert p.log[0].call == h.call and p.log[0].seconds == 12
    assert p.step is (Step.EXCHANGE if n else Step.CQ)


def test_wrong_call_hunter_repeats_without_revealing():
    p = pota()
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    r = p.check("XX1XX 5NN")
    assert not r.ok and not r.reveal and r.notes == ["call not copied"]
    assert p.step is Step.EXCHANGE and heard(p) == f"{h.call} {h.call}"


def test_missing_rst():
    p = pota()
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    r = p.check(h.call)
    assert not r.ok and "no RST" in r.notes
    assert heard(p) in {"AGN?", "?", "RST?"} and p.step is Step.EXCHANGE
    assert p.check(f"{h.call} 599").ok and p.step is Step.FINAL


def test_repeat_request_replays():
    p = pota()
    p.check(f"CQ POTA DE {MY} K")
    before = heard(p)
    r = p.check("?")
    assert r.ok and r.notes == ["repeat"] and heard(p) == before and p.step is Step.EXCHANGE


def test_agn():
    p = pota(agn=1.0)
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    assert p.check(f"{h.call} 5NN").ok
    assert heard(p) in {"AGN?", "?"} and p.step is Step.EXCHANGE
    assert p.check(f"{h.call} 5NN").ok and p.step is Step.FINAL  # only once per QSO


def test_partial():
    p = pota(partial=1.0)
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    frag = p.fragment
    assert p.step is Step.PARTIAL and heard(p) == frag and frag in h.call and frag != h.call
    r = p.check("QRL?")
    assert not r.ok and not r.reveal and heard(p) == frag
    assert p.check(f"{frag} ?").ok
    assert p.step is Step.EXCHANGE and heard(p) == f"{h.call} {h.call}"


@pytest.mark.parametrize("call,frags", [("OH2ABC", {"OH2", "ABC"}), ("9A1XY/P", {"9A1", "XY"}),
                                        ("S51A", {"S51", "A"})])
def test_partial_call(call, frags):
    rng = random.Random(0)
    assert {partial_call(call, rng) for _ in range(50)} == frags


def test_pileup_work_either_then_the_other():
    for seed in range(20):
        p = pota(seed, pileup=1.0)
        p.check(f"CQ POTA DE {MY} K")
        if len(p.queue) >= 2:
            break
    a, b = p.calling
    pr = p.next_prompt()
    assert len(pr.voices) == 2 and 0.1 <= pr.voices[1].delay <= 0.6
    assert p.check(f"{b.call} 5NN").ok and p.calling == [b]
    p.check("TU")
    assert a in p.calling and a.call in heard(p)


def test_p2p_requires_park():
    p = pota(p2p=1.0)
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    p.check(f"{h.call} 5NN")
    assert heard(p) == f"R 5NN {h.park} TU"
    r = p.check("TU 73")
    assert not r.ok and r.expected == f"R {h.park} TU"
    assert len(p.log) == 1  # QSO still logged


def test_p2p_ok():
    p = pota(p2p=1.0)
    p.check(f"CQ POTA DE {MY} K")
    h = p.calling[0]
    p.check(f"{h.call} 5NN")
    assert p.check(f"R {h.park} TU 73").ok


def test_count_and_summary():
    p = Pota(MY, count=2, realism=Realism(no_answer=0.0), rng=random.Random(4), clock=FakeClock())
    while not p.done():
        p.next_prompt()
        if p.step is Step.CQ:
            p.check(f"CQ POTA DE {MY} K")
        elif p.step is Step.EXCHANGE:
            p.clock.advance(10)
            p.check(f"{p.calling[0].call} 5NN")
        else:
            p.check("TU")
    assert len(p.log) == 2
    s = p.summary()
    assert s[0] == "2 QSOs, accuracy 100%, 10 s per QSO"
