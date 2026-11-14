"""Sentences: send back plain English sentences, graded by vocabulary level."""

from __future__ import annotations

import argparse
import random
import re
from importlib.resources import files

from cwtrainer.config import Config
from cwtrainer.scenarios import register
from cwtrainer.scenarios.echo import EchoBase, PromptMode

LEVELS = range(100, 1700, 100)
_SECTION = re.compile(r"#\s*Top (\d+) words")
_PUNCT = re.compile(r"[.,?]")


def load_sentences(name: str = "sentences.txt") -> list[tuple[int, str]]:
    """(level, sentence) pairs; level comes from the preceding '# Top N words' header."""
    out: list[tuple[int, str]] = []
    level = 0
    for ln in (files("cwtrainer") / "data" / name).read_text().splitlines():
        ln = ln.strip()
        if m := _SECTION.fullmatch(ln):
            level = int(m.group(1))
        elif ln and not ln.startswith("#"):
            out.append((level, ln.upper()))
    return out


def strip_punct(text: str) -> str:
    return " ".join(_PUNCT.sub("", text.replace("-", " ")).split())


def vocab_level(s: str) -> int:
    try:
        v = int(s)
    except ValueError:
        v = 0
    if v not in LEVELS:
        raise argparse.ArgumentTypeError(f"one of {LEVELS.start}, {LEVELS.start + LEVELS.step}, ..., "
                                         f"{LEVELS[-1]}")
    return v


def word_limit(s: str) -> int:
    v = int(s)
    if v < 3:
        raise argparse.ArgumentTypeError("must be >= 3")
    return v


def select(pairs: list[tuple[int, str]], lvl: int, max_words: int | None = None,
           punct: bool = False) -> list[str]:
    """Sentences up to vocabulary level `lvl`, at most `max_words` long, deduped."""
    out = []
    for lv, s in pairs:
        if lv > lvl:
            continue
        if not punct:
            s = strip_punct(s)
        if max_words and len(s.split()) > max_words:
            continue
        out.append(s)
    return list(dict.fromkeys(out))


@register
class Sentences(EchoBase):
    name = "sentences"
    help = "send back plain English sentences"

    def __init__(self, sentences: list[str], mode: PromptMode = PromptMode.SHOW, count: int = 10,
                 rng: random.Random | None = None) -> None:
        if not sentences:
            raise ValueError("no sentences")
        super().__init__(mode, count, rng)
        self.sentences = sentences

    @staticmethod
    def add_arguments(p: argparse.ArgumentParser) -> None:
        EchoBase.add_arguments(p)
        p.add_argument("--level", type=vocab_level, default=LEVELS[-1], metavar="N",
                       help=f"vocabulary: top N words, {LEVELS.start}-{LEVELS[-1]} in steps of "
                            f"{LEVELS.step} (default {LEVELS[-1]})")
        p.add_argument("--max-words", type=word_limit, metavar="N",
                       help="skip sentences longer than N words (min 3)")
        p.add_argument("--punct", action="store_true",
                       help="keep punctuation (. , ?); stripped by default")

    @classmethod
    def from_args(cls, args: argparse.Namespace, cfg: Config) -> Sentences:
        return cls(select(load_sentences(), args.level, args.max_words, args.punct),
                   args.prompt_mode, args.count, random.Random(args.seed))

    def _pick(self) -> str:
        return self.rng.choice(self.sentences)

