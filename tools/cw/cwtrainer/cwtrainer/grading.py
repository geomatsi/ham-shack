"""Normalise, apply <HH> corrections, score by edit distance."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field

from cwtrainer.morse import tokenize

HH = "<HH>"
# canonical form for RST cut numbers
RST_EQUIVALENTS: Mapping[str, str] = {"5NN": "599", "5NNN": "599", "55N": "559", "57N": "579"}


def normalise(text: str, equivalents: Mapping[str, str] | None = None) -> str:
    words = text.upper().split()
    if equivalents:
        words = [equivalents.get(w, w) for w in words]
    return " ".join(words)


def apply_hh(text: str) -> str:
    """<HH> erases the word fragment it is glued to, or else the previous word."""
    out: list[str] = []
    for word in text.split():
        segs = word.split(HH)
        cur = segs[0]
        for seg in segs[1:]:
            if cur:
                cur = ""
            elif out:
                out.pop()
            cur = seg
        if cur:
            out.append(cur)
    return " ".join(out)


@dataclass(frozen=True)
class Mistake:
    expected: str | None  # None: extra token sent
    sent: str | None      # None: token missing

    def __str__(self) -> str:
        def show(t: str) -> str:
            return "space" if t == " " else t
        if self.expected is None:
            return f"extra {show(self.sent or '')}"
        if self.sent is None:
            return f"missing {show(self.expected)}"
        return f"{show(self.sent)} for {show(self.expected)}"


@dataclass
class Grade:
    expected: str
    sent: str
    distance: int
    score: float
    mistakes: list[Mistake] = field(default_factory=list)
    attempted: list[str] = field(default_factory=list)  # expected chars, spaces excluded


def align(a: list[str], b: list[str]) -> tuple[int, list[Mistake]]:
    n, m = len(a), len(b)
    d = [[0] * (m + 1) for _ in range(n + 1)]
    for i in range(n + 1):
        d[i][0] = i
    for j in range(m + 1):
        d[0][j] = j
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            d[i][j] = min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] != b[j - 1]))
    mistakes: list[Mistake] = []
    i, j = n, m
    # on ties prefer match, then missing/extra, then substitution: reads more naturally
    while i or j:
        if i and j and a[i - 1] == b[j - 1] and d[i][j] == d[i - 1][j - 1]:
            i, j = i - 1, j - 1
        elif i and d[i][j] == d[i - 1][j] + 1:
            mistakes.append(Mistake(a[i - 1], None))
            i -= 1
        elif j and d[i][j] == d[i][j - 1] + 1:
            mistakes.append(Mistake(None, b[j - 1]))
            j -= 1
        else:
            mistakes.append(Mistake(a[i - 1], b[j - 1]))
            i, j = i - 1, j - 1
    mistakes.reverse()
    return d[n][m], mistakes


def grade(expected: str, sent: str, equivalents: Mapping[str, str] | None = None) -> Grade:
    exp = normalise(expected, equivalents)
    got = normalise(apply_hh(normalise(sent)), equivalents)
    a, b = tokenize(exp), tokenize(got)
    dist, mistakes = align(a, b)
    score = max(0.0, 1.0 - dist / max(len(a), 1))
    return Grade(exp, got, dist, score, mistakes, [t for t in a if t != " "])
