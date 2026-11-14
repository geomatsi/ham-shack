import random

import pytest

from cwtrainer.main import build_parser
from cwtrainer.morse import text_to_elements
from cwtrainer.scenarios import SCENARIOS
from cwtrainer.scenarios.echo import PromptMode
from cwtrainer.scenarios.sentences import LEVELS, Sentences, load_sentences, select, strip_punct


def args(*a):
    return build_parser().parse_args(["sentences", *a])


def test_registered_and_defaults():
    assert SCENARIOS["sentences"] is Sentences
    a = args()
    assert (a.level, a.max_words, a.punct, a.count, a.prompt_mode) == \
        (1600, None, False, 10, PromptMode.SHOW)


def test_cli_options():
    a = args("--level", "300", "--max-words", "5", "--punct", "--listen")
    assert (a.level, a.max_words, a.punct, a.prompt_mode) == (300, 5, True, PromptMode.LISTEN)


@pytest.mark.parametrize("opt,bad", [("--level", "0"), ("--level", "150"), ("--level", "1700"),
                                     ("--level", "x"), ("--max-words", "2"), ("--max-words", "x")])
def test_bad_options(opt, bad):
    with pytest.raises(SystemExit):
        args(opt, bad)


def test_data_levels_and_sendable():
    pairs = load_sentences()
    assert {lv for lv, _ in pairs} == set(LEVELS)
    assert len(pairs) > 10000
    assert len({s for _, s in pairs}) == len(pairs)
    for _, s in pairs:
        text_to_elements(s)
        assert not s.endswith("=")


def test_strip_punct():
    assert strip_punct("WELL, DID YOU DIG UP THE WATCH?") == "WELL DID YOU DIG UP THE WATCH"
    assert strip_punct("A WELL-KNOWN MAN.") == "A WELL KNOWN MAN"


def test_select():
    pairs = [(100, "SO IT IS."), (100, "SO IT IS?"), (200, "WHAT IS IT FOR, THEN?"),
             (300, "A B C D E F.")]
    assert select(pairs, 100) == ["SO IT IS"]
    assert select(pairs, 100, punct=True) == ["SO IT IS.", "SO IT IS?"]
    assert select(pairs, 300, max_words=5) == ["SO IT IS", "WHAT IS IT FOR THEN"]
    assert select(pairs, 300) == ["SO IT IS", "WHAT IS IT FOR THEN", "A B C D E F"]


def test_every_level_and_limit_has_sentences():
    pairs = load_sentences()
    for lv in LEVELS:
        assert select(pairs, lv, 3)


def test_from_args_filters():
    s = Sentences.from_args(args("--level", "100", "--max-words", "4", "--seed", "1"), None)
    for _ in range(50):
        t = s.next_prompt().text
        assert len(t.split()) <= 4 and not set(t) & set(".,?")


def test_check_and_count():
    s = Sentences(["SO IT IS", "SO BE IT"], count=2, rng=random.Random(0))
    t = s.next_prompt().text
    assert s.check(t.lower()).ok
    assert not s.check(t[:-1]).ok
    assert not s.done()
    s.next_prompt()
    assert s.done()


def test_empty_rejected():
    with pytest.raises(ValueError):
        Sentences([])
