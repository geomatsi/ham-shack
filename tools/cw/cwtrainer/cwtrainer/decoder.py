"""KeyEvents -> text, by gap timing."""

from __future__ import annotations

from cwtrainer.keyer import KeyEvent
from cwtrainer.morse import Element, elements_to_char, unit

HH_MIN_DITS = 6


class Decoder:
    """Feed KeyEvents in order; call poll(now) while idle to flush on silence."""

    def __init__(self, wpm: float, char_gap: float = 2.0, word_gap: float = 5.0,
                 answer_timeout: float = 1.5) -> None:
        self.answer_timeout = answer_timeout
        self.char_gap = char_gap
        self.word_gap = word_gap
        self.set_wpm(wpm)
        self.reset()

    def set_wpm(self, wpm: float) -> None:
        self.unit = unit(wpm)

    def reset(self) -> None:
        self.text = ""
        self._elements: list[Element] = []
        self._down_at: float | None = None
        self._down_el: Element | None = None
        self._last_up: float | None = None

    @property
    def idle(self) -> bool:
        """Nothing sent since reset()."""
        return not self.text and not self._elements and self._down_at is None

    def feed(self, ev: KeyEvent) -> str:
        """Returns text newly decoded."""
        if ev.down:
            out = ""
            if self._last_up is not None:
                gap = ev.t - self._last_up
                if gap >= self.char_gap * self.unit:
                    out += self._flush()
                if gap >= self.word_gap * self.unit and self.text and not self.text.endswith(" "):
                    out += self._emit(" ")
            self._down_at, self._down_el = ev.t, ev.element
            return out
        if self._down_at is None:
            return ""
        el = self._down_el
        if el is None:  # no keyer: classify by length
            el = Element.DAH if ev.t - self._down_at > 2 * self.unit else Element.DIT
        self._elements.append(el)
        self._down_at = None
        self._last_up = ev.t
        return ""

    def poll(self, now: float) -> str:
        if self._down_at is None and self._last_up is not None and \
                now - self._last_up >= self.char_gap * self.unit:
            return self._flush()
        return ""

    def answer_complete(self, now: float) -> bool:
        return (self._down_at is None and self._last_up is not None
                and now - self._last_up >= self.answer_timeout)

    def finish(self) -> str:
        """Flush and return the whole text."""
        self._flush()
        return self.text.strip()

    def _flush(self) -> str:
        if not self._elements:
            return ""
        els, self._elements = self._elements, []
        if len(els) >= HH_MIN_DITS and all(e is Element.DIT for e in els):
            return self._emit("<HH>")
        return self._emit(elements_to_char(els) or "*")

    def _emit(self, s: str) -> str:
        self.text += s
        return s
