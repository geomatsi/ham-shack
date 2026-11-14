import random

from cwtrainer.config import Config, KeyerMode
from cwtrainer.decoder import Decoder
from cwtrainer.keyer import simulate
from cwtrainer.morse import unit
from cwtrainer.paddle import ReplayPaddle
from cwtrainer.player import MorsePlayer
from cwtrainer.rig import Rig
from cwtrainer.scenarios.echo import PromptMode
from cwtrainer.scenarios.qso import Qso
from cwtrainer.session import SessionUI
from cwtrainer.sidetone import NullSidetone
from cwtrainer.stats import Stats
from scripts import paddle_script


def send(text, wpm):
    script, end = paddle_script(text, wpm)
    d = Decoder(wpm)
    for ev in simulate(ReplayPaddle(script).events(), wpm, KeyerMode.A, end + 10 * unit(wpm)):
        d.feed(ev)
    return d.finish()


def test_paddle_to_grade():
    e = Qso(["UR RST 599"], ["X"], rng=random.Random(0))
    e.PHRASE_RATIO = 1.0
    e.next_prompt()
    assert send("UR RST 5NN", 22) == "UR RST 5NN"
    assert e.check(send("UR RST 5NN", 22)).ok
    r = e.check(send("UR RTS <HH> RST 599", 22))
    assert r.ok
    r = e.check(send("UR RST 589", 22))
    assert not r.ok and r.wrong == ["9"]


def test_session_live(tmp_path, capsys):
    cfg = Config(wpm=50, answer_timeout=0.2)
    script, end = paddle_script("TU 73", cfg.wpm, t0=0.1)
    second, _ = paddle_script("CQ", cfg.wpm, t0=end + 0.5)
    audio = NullSidetone()
    qso = Qso(["TU 73", "CQ"], ["X"], PromptMode.BOTH, count=2, rng=random.Random(0))
    qso.PHRASE_RATIO = 1.0
    # make prompt order deterministic
    order = iter(["TU 73", "CQ"])
    qso._pick = lambda: next(order)
    stats = Stats(tmp_path / "s.json")
    with Rig(cfg, ReplayPaddle(script + second), audio) as rig:
        ui = SessionUI(rig, qso, MorsePlayer(audio, 8000, 0.5, 0.005), stats, Decoder(cfg.wpm, answer_timeout=0.2))
        ui.run()
    assert [r.sent for r in ui.results] == ["TU 73", "CQ"]
    assert all(r.ok for r in ui.results)
    assert len(audio.played) == 2
    out = capsys.readouterr().out
    assert "2 prompts, 2 perfect" in out
    assert Stats(tmp_path / "s.json").sessions[0]["prompts"] == 2


def test_session_labels():
    from cwtrainer.scenarios.base import Prompt, Voice

    assert SessionUI.label(Prompt("CQ", True, False)) == "CQ"
    assert SessionUI.label(Prompt("CQ", False, True)) == "(listen)"
    assert SessionUI.label(Prompt("", False, False, instruction="Call CQ")) == "Call CQ"
    p = Prompt("OH2AB", True, True, instruction="Hunter", voices=[Voice("OH2AB")])
    assert SessionUI.label(p) == "Hunter: OH2AB"
