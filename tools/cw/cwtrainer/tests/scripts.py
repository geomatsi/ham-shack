"""Paddle scripts for tests: tap each element's paddle as the keyer would key it."""

from cwtrainer.morse import Element, timeline, unit
from cwtrainer.paddle import Side


def paddle_script(text, wpm, t0=0.0):
    marks, length = timeline(text, wpm)
    u = unit(wpm)
    script = []
    for m in marks:
        side = Side.DIT if m.element is Element.DIT else Side.DAH
        script += [(t0 + m.start, side, True), (t0 + m.start + 0.5 * u, side, False)]
    return script, t0 + length


def squeeze_script(first, t_release, t0=0.0):
    other = first.other
    return [(t0, first, True), (t0 + 0.001, other, True), (t0 + t_release, first, False),
            (t0 + t_release, other, False)]
