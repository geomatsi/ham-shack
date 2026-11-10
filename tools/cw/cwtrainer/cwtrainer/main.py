"""Command line entry point."""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

from cwtrainer import __version__
from cwtrainer.config import Config, KeyerMode
from cwtrainer.keyer import KeyEvent
from cwtrainer.morse import Element
from cwtrainer.paddle import ButtonEvent, InputClosed
from cwtrainer.rig import Rig


def volume(s: str) -> float:
    v = float(s)
    if not 0.0 <= v <= 1.0:
        raise argparse.ArgumentTypeError("volume must be in 0..1")
    return v


def build_parser() -> argparse.ArgumentParser:
    d = Config()
    p = argparse.ArgumentParser(prog="cwtrainer", description="CW sending trainer")
    p.add_argument("--version", action="version", version=__version__)
    p.add_argument("-d", "--device", help="paddle input device, /dev/input/eventX")
    p.add_argument("-w", "--wpm", type=int, default=d.wpm, help=f"keyer speed (default {d.wpm})")
    p.add_argument("-f", "--tone", type=float, default=d.tone, help=f"sidetone Hz (default {d.tone:g})")
    p.add_argument("-v", "--volume", type=volume, default=d.volume, help=f"0..1 (default {d.volume})")
    p.add_argument("--mode", type=KeyerMode, choices=list(KeyerMode), default=d.mode,
                   metavar="{A,B}", help="iambic mode (default A)")
    p.add_argument("--swap", action="store_true", help="swap dit and dah paddles")
    p.add_argument("--keyer-only", action="store_true", help="practice keyer with live decoded text")
    p.add_argument("--rate", type=int, default=d.samplerate, dest="samplerate", help="audio sample rate")
    p.add_argument("--rise", type=float, default=d.rise_time * 1000, help="envelope rise/fall in ms")
    p.add_argument("--answer-timeout", type=float, default=d.answer_timeout,
                   help=f"silence that ends an answer, seconds (default {d.answer_timeout})")
    p.add_subparsers(dest="scenario", metavar="SCENARIO")
    return p


def config_from_args(args: argparse.Namespace) -> Config:
    return Config(wpm=args.wpm, tone=args.tone, volume=args.volume, mode=args.mode, swap=args.swap,
                  device=args.device, samplerate=args.samplerate, rise_time=args.rise / 1000,
                  answer_timeout=args.answer_timeout)


def open_rig(cfg: Config) -> Rig:
    from cwtrainer.paddle import EvdevPaddle
    from cwtrainer.sidetone import Sidetone

    paddle = EvdevPaddle(cfg.device, swap=cfg.swap)
    return Rig(cfg, paddle, Sidetone(cfg))


def run_keyer_only(rig: Rig) -> None:
    print("Keyer ready. Adapter button or Ctrl+C exits.")
    last_up: float | None = None
    while True:
        ev = rig.get(timeout=0.1)
        if isinstance(ev, (ButtonEvent, InputClosed)):
            if isinstance(ev, InputClosed):
                print(f"\ninput closed: {ev.reason}")
            break
        if isinstance(ev, KeyEvent):
            if ev.down:
                last_up = None
                print("." if ev.element is Element.DIT else "-", end="", flush=True)
            else:
                last_up = ev.t
        elif last_up is not None and rig.clock.now() - last_up > rig.cfg.answer_timeout:
            print()
            last_up = None
    print()


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    cfg = config_from_args(args)
    if not args.keyer_only and args.scenario is None:
        parser.print_help()
        return 0
    if cfg.device is None or not Path(cfg.device).exists():
        parser.error(f"paddle device not found: {cfg.device}; pass --device /dev/input/eventX")
    try:
        with open_rig(cfg) as rig:
            run_keyer_only(rig)
    except KeyboardInterrupt:
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
