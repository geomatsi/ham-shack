import random

import pytest

from cwtrainer.main import build_parser
from cwtrainer.morse import text_to_elements
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.echo import PromptMode, load_lines
from cwtrainer.scenarios.words import Words


def args(*a):
    return build_parser().parse_args(["words", *a])


def test_registered_and_defaults():
    assert SCENARIOS["words"] is Words
    a = args()
    assert (a.per_prompt, a.count, a.prompt_mode) == (1, 10, PromptMode.SHOW)
    a = args("--per-prompt", "3", "--listen")
    assert (a.per_prompt, a.prompt_mode) == (3, PromptMode.LISTEN)


def test_bad_per_prompt():
    with pytest.raises(SystemExit):
        args("--per-prompt", "0")


def test_data_is_sendable():
    words = load_lines("words.txt")
    assert len(words) > 400
    for w in words:
        assert " " not in w
        text_to_elements(w)


def test_pick():
    vocab = ["A", "B", "C", "D"]
    for seed in range(50):
        w = Words(vocab, per_prompt=3, rng=random.Random(seed))
        picked = w.next_prompt().text.split()
        assert len(picked) == 3 and len(set(picked)) == 3 and set(picked) <= set(vocab)
    assert len(Words(["A"], per_prompt=3).next_prompt().text.split()) == 1


def test_count():
    w = Words(["A", "B"], count=2, rng=random.Random(0))
    w.next_prompt()
    assert not w.done()
    w.next_prompt()
    assert w.done()
