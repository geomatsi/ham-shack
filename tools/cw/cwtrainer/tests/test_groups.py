import random
import re

import pytest

from cwtrainer.main import build_parser
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.echo import PromptMode
from cwtrainer.scenarios.groups import Charset, Groups


def args(*a):
    return build_parser().parse_args(["groups", *a])


def test_registered_and_defaults():
    assert SCENARIOS["groups"] is Groups
    a = args()
    assert (a.charset, a.size, a.per_prompt, a.count, a.prompt_mode) == \
        (Charset.LETTERS, (5, 5), 1, 20, PromptMode.SHOW)


def test_cli_options():
    a = args("--charset", "mix", "--size", "1-6", "--per-prompt", "3", "--listen")
    assert (a.charset, a.size, a.per_prompt, a.prompt_mode) == (Charset.MIX, (1, 6), 3, PromptMode.LISTEN)
    assert args("--size", "3").size == (3, 3)


@pytest.mark.parametrize("bad", ["0", "7", "4-2", "1-7", "x"])
def test_bad_size(bad):
    with pytest.raises(SystemExit):
        args("--size", bad)


def test_bad_per_prompt():
    with pytest.raises(SystemExit):
        args("--per-prompt", "0")


@pytest.mark.parametrize("charset,rx", [(Charset.LETTERS, "[A-Z]"), (Charset.DIGITS, "[0-9]"),
                                        (Charset.MIX, "[A-Z0-9]")])
def test_charset(charset, rx):
    g = Groups(charset, (5, 5), rng=random.Random(1))
    for _ in range(50):
        assert re.fullmatch(f"{rx}{{5}}", g.next_prompt().text)


def test_mix_uses_both():
    g = Groups(Charset.MIX, (6, 6), count=0, rng=random.Random(2))
    text = "".join(g.next_prompt().text for _ in range(50))
    assert re.search("[A-Z]", text) and re.search("[0-9]", text)


def test_variable_size_and_per_prompt():
    g = Groups(Charset.LETTERS, (1, 6), per_prompt=3, count=0, rng=random.Random(3))
    sizes = set()
    for _ in range(100):
        parts = g.next_prompt().text.split()
        assert len(parts) == 3
        sizes |= {len(p) for p in parts}
    assert sizes == set(range(1, 7))


def test_check_and_count():
    g = Groups(Charset.DIGITS, (5, 5), count=2, rng=random.Random(4))
    t = g.next_prompt().text
    assert g.check(t).ok
    r = g.check(t[:-1])
    assert not r.ok and r.wrong == [t[-1]]
    g.next_prompt()
    assert g.done()
