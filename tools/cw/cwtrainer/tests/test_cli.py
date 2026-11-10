import pytest

from cwtrainer.config import KeyerMode
from cwtrainer.main import build_parser, config_from_args


def test_help(capsys):
    with pytest.raises(SystemExit) as e:
        build_parser().parse_args(["--help"])
    assert e.value.code == 0
    assert "--keyer-only" in capsys.readouterr().out


def test_defaults():
    cfg = config_from_args(build_parser().parse_args([]))
    assert (cfg.wpm, cfg.tone, cfg.volume, cfg.mode) == (20, 600.0, 0.5, KeyerMode.A)


def test_options():
    cfg = config_from_args(build_parser().parse_args(["--mode", "B", "-v", "0.2", "--swap"]))
    assert cfg.mode is KeyerMode.B and cfg.volume == 0.2 and cfg.swap


def test_volume_range():
    with pytest.raises(SystemExit):
        build_parser().parse_args(["-v", "1.5"])
