"""Session loop: prompt, collect the sent answer, grade, repeat."""

from __future__ import annotations

import os
import select
import sys
import termios
import threading
import tty
from collections.abc import Callable
from dataclasses import dataclass

from cwtrainer.decoder import Decoder
from cwtrainer.keyer import KeyEvent
from cwtrainer.paddle import ButtonEvent, InputClosed
from cwtrainer.player import MorsePlayer
from cwtrainer.rig import Rig
from cwtrainer.scenarios.base import Prompt, Result, Scenario
from cwtrainer.stats import Stats

POLL = 0.05


@dataclass(frozen=True)
class KeyPress:
    ch: str


class Quit(Exception):
    pass


class Keyboard:
    """Single-key commands from a tty, posted to the rig's event queue."""

    def __init__(self, post: Callable[[KeyPress], None]) -> None:
        self.post = post
        self.fd = sys.stdin.fileno() if sys.stdin.isatty() else None
        self._saved = None
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True, name="keyboard")

    def __enter__(self) -> Keyboard:
        if self.fd is not None:
            self._saved = termios.tcgetattr(self.fd)
            tty.setcbreak(self.fd)
            self._thread.start()
        return self

    def __exit__(self, *exc: object) -> None:
        self._stop.set()
        if self._thread.is_alive():
            self._thread.join()
        if self._saved is not None:
            termios.tcsetattr(self.fd, termios.TCSADRAIN, self._saved)

    def _run(self) -> None:
        assert self.fd is not None
        while not self._stop.is_set():
            r, _, _ = select.select([self.fd], [], [], 0.2)
            if r:
                ch = os.read(self.fd, 1).decode(errors="ignore")
                if ch:
                    self.post(KeyPress(ch))


class SessionUI:
    def __init__(self, rig: Rig, scenario: Scenario, player: MorsePlayer, stats: Stats,
                 decoder: Decoder) -> None:
        self.rig = rig
        self.cfg = rig.cfg
        self.scenario = scenario
        self.player = player
        self.stats = stats
        self.dec = decoder
        self.results: list[Result] = []

    @staticmethod
    def label(p: Prompt) -> str:
        text = p.text if p.show_text else "(listen)" if p.play_audio and p.audio() else ""
        return ": ".join(s for s in (p.instruction, text) if s)

    def present(self, n: int, p: Prompt) -> None:
        print(f"\n#{n}  {self.label(p)}")
        if p.play_audio:
            self.play(p)
        print("  > ", end="", flush=True)

    def play(self, p: Prompt) -> None:
        self.player.play_voices(p.audio(), p.wpm or self.cfg.wpm, self.cfg.tone)

    def replay(self, p: Prompt) -> None:
        self.dec.reset()
        print(f"  (replay) {self.label(p)}")
        self.play(p)
        print("  > ", end="", flush=True)

    def collect(self, p: Prompt) -> str:
        """Decode until answer_complete; button or Enter replays."""
        self.dec.reset()
        while True:
            ev = self.rig.get(timeout=POLL)
            if isinstance(ev, InputClosed):
                raise Quit(f"input closed: {ev.reason}")
            if isinstance(ev, KeyPress):
                if ev.ch in "qQ":
                    raise Quit()
                if ev.ch in "\n\r ":
                    self.replay(p)
                continue
            if isinstance(ev, ButtonEvent):
                self.replay(p)
                continue
            if isinstance(ev, KeyEvent):
                if ev.down and self.rig.audio.playing():
                    self.rig.audio.stop_playback()
                print(self.dec.feed(ev), end="", flush=True)
                continue
            now = self.rig.clock.now()
            print(self.dec.poll(now), end="", flush=True)
            if self.dec.answer_complete(now):
                print()
                return self.dec.finish()

    def show(self, r: Result, p: Prompt) -> None:
        mark = "✓" if r.ok else "✗"
        print(f"  {mark} {r.score:4.0%}", end="")
        if not r.ok and r.reveal and not p.show_text:
            print(f"  expected: {r.expected}", end="")
        if r.notes:
            print(f"  ({', '.join(r.notes)})", end="")
        print()

    def run(self) -> None:
        self.stats.start_session(self.scenario.name, self.cfg.wpm)
        n = 0
        try:
            while not self.scenario.done():
                n += 1
                p = self.scenario.next_prompt()
                self.present(n, p)
                sent = self.collect(p)
                r = self.scenario.check(sent)
                self.results.append(r)
                self.stats.record(r)
                self.stats.save()
                self.show(r, p)
        except Quit as e:
            if str(e):
                print(f"\n{e}")
        except KeyboardInterrupt:
            print()
        finally:
            self.rig.audio.stop_playback()
            self.summary()

    def summary(self) -> None:
        s = self.stats.end_session()
        self.stats.save()
        if s is None or not s.prompts:
            return
        print(f"\n{s.prompts} prompts, {s.perfect} perfect, average {s.score:.0%} at {s.wpm} WPM")
        for line in getattr(self.scenario, "summary", list)():
            print(line)
        weak = self.stats.weak_characters(5, min_attempts=1, session_only=True)
        if weak:
            print("weak this session: " + " ".join(weak))
