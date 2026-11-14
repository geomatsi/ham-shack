"""Morse code table, tokenising and timing."""

from __future__ import annotations

import re
from collections.abc import Sequence
from dataclasses import dataclass
from enum import Enum


class Element(Enum):
    DIT = 1
    DAH = 2


CODE: dict[str, str] = {
    "A": ".-", "B": "-...", "C": "-.-.", "D": "-..", "E": ".", "F": "..-.",
    "G": "--.", "H": "....", "I": "..", "J": ".---", "K": "-.-", "L": ".-..",
    "M": "--", "N": "-.", "O": "---", "P": ".--.", "Q": "--.-", "R": ".-.",
    "S": "...", "T": "-", "U": "..-", "V": "...-", "W": ".--", "X": "-..-",
    "Y": "-.--", "Z": "--..",
    "0": "-----", "1": ".----", "2": "..---", "3": "...--", "4": "....-",
    "5": ".....", "6": "-....", "7": "--...", "8": "---..", "9": "----.",
    "/": "-..-.", "-": "-....-", "?": "..--..", ".": ".-.-.-", ",": "--..--", "=": "-...-",
    "<AR>": ".-.-.", "<SK>": "...-.-", "<KN>": "-.--.", "<HH>": "........",
    "<BK>": "-...-.-",
}
# <BT> is the same code as "="; decoding yields "="
ALIASES: dict[str, str] = {"<BT>": "="}

_SYM = {".": Element.DIT, "-": Element.DAH}
_REVERSE: dict[tuple[Element, ...], str] = {
    tuple(_SYM[s] for s in code): ch for ch, code in CODE.items()
}

_TOKEN = re.compile(r"<[A-Z]+>|\S|\s+")


def tokenize(text: str) -> list[str]:
    """Split text into characters, prosigns and single " " word breaks."""
    out: list[str] = []
    for tok in _TOKEN.findall(text.upper()):
        if tok.isspace():
            if out and out[-1] != " ":
                out.append(" ")
        else:
            out.append(ALIASES.get(tok, tok))
    if out and out[-1] == " ":
        out.pop()
    return out


def char_to_elements(ch: str) -> tuple[Element, ...]:
    ch = ALIASES.get(ch, ch)
    return tuple(_SYM[s] for s in CODE[ch])


def text_to_elements(text: str) -> list[tuple[Element, ...] | None]:
    """One entry per character; None marks a word break. Unknown chars raise KeyError."""
    return [None if t == " " else char_to_elements(t) for t in tokenize(text)]


def elements_to_char(seq: Sequence[Element]) -> str | None:
    return _REVERSE.get(tuple(seq))


def unit(wpm: float) -> float:
    """Dit length in seconds (PARIS)."""
    return 1.2 / wpm


@dataclass(frozen=True)
class Mark:
    start: float
    end: float
    element: Element


def timeline(text: str, wpm: float, effective_wpm: float | None = None) -> tuple[list[Mark], float]:
    """Key-down intervals for text, and total length. effective_wpm < wpm stretches gaps (Farnsworth)."""
    u = unit(wpm)
    char_gap, word_gap = 3 * u, 7 * u
    if effective_wpm is not None and effective_wpm < wpm:
        ta = (60 * wpm - 37.2 * effective_wpm) / (effective_wpm * wpm)
        char_gap, word_gap = 3 * ta / 19, 7 * ta / 19

    marks: list[Mark] = []
    t = 0.0
    prev_word_break = False
    for i, entry in enumerate(text_to_elements(text)):
        if entry is None:
            prev_word_break = True
            continue
        if i:
            t += (word_gap if prev_word_break else char_gap) - u
        prev_word_break = False
        for el in entry:
            d = u if el is Element.DIT else 3 * u
            marks.append(Mark(t, t + d, el))
            t += d + u
    return marks, max(t - u, 0.0)
