"""Run a scenario through the real session UI with scripted answers instead of a paddle.

    python tools/dry_run.py [--error P] [--answer-seed N] <cwtrainer args without --device>

e.g. python tools/dry_run.py pota --mycall R3XYZ --pileup --p2p --count 4 --seed 5
Prints the session as the user would see it, answers echoed after "> ".
"""

from __future__ import annotations

import argparse
import random
import sys

from cwtrainer.config import Config
from cwtrainer.decoder import Decoder
from cwtrainer.main import build_parser, config_from_args
from cwtrainer.paddle import ReplayPaddle
from cwtrainer.player import MorsePlayer
from cwtrainer.rig import Rig
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.base import Prompt
from cwtrainer.scenarios.echo import EchoBase
from cwtrainer.scenarios.hunter import PotaHunter
from cwtrainer.scenarios.hunter import Step as HStep
from cwtrainer.scenarios.pota import Pota
from cwtrainer.scenarios.pota import Step as PStep
from cwtrainer.session import SessionUI
from cwtrainer.sidetone import NullSidetone
from cwtrainer.stats import Stats


def oracle(s, p: Prompt) -> str:
    """The right answer for the scenario's current state."""
    if isinstance(s, EchoBase):
        return s.current
    if isinstance(s, Pota):
        if s.step is PStep.CQ:
            return f"CQ POTA DE {s.mycall} K"
        if s.step is PStep.PARTIAL:
            return f"{s.fragment}?"
        if s.step is PStep.EXCHANGE:
            return f"{s.calling[-1].call} 5NN"
        h = s.calling[0]
        return f"R {h.park} TU" if h.park else "TU 73"
    if isinstance(s, PotaHunter):
        if s.step in (HStep.CALL, HStep.BUSTED):
            return f"{s.mycall} {s.mycall}"
        if s.step is HStep.EXCHANGE:
            return "R 5NN TU"
        return f"{s.act.call} {s.act.park}"
    raise SystemExit(f"no oracle for {type(s).__name__}; add one to tools/dry_run.py")


def corrupt(text: str, rng: random.Random) -> str:
    """Drop or change one character."""
    if not text:
        return text
    i = rng.randrange(len(text))
    return text[:i] + rng.choice(["", "E", "T"]) + text[i + 1:]


def main() -> None:
    own = argparse.ArgumentParser(add_help=False)
    own.add_argument("--error", type=float, default=0.15, help="chance an answer is corrupted")
    own.add_argument("--answer-seed", type=int, default=1)
    opts, rest = own.parse_known_args()
    args = build_parser().parse_args(rest)
    if args.scenario is None:
        raise SystemExit(__doc__)
    cfg: Config = config_from_args(args)
    scenario = SCENARIOS[args.scenario].from_args(args, cfg)
    rng = random.Random(opts.answer_seed)

    class Scripted(SessionUI):
        def collect(self, p: Prompt) -> str:
            a = oracle(scenario, p)
            if rng.random() < opts.error:
                a = corrupt(a, rng)
            print(a)
            return a

    audio = NullSidetone()
    with Rig(cfg, ReplayPaddle([]), audio) as rig:
        player = MorsePlayer(audio, cfg.samplerate, cfg.volume, cfg.rise_time)
        Scripted(rig, scenario, player, Stats(None), Decoder(cfg.wpm)).run()


if __name__ == "__main__":
    sys.exit(main())
