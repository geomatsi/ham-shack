"""Per-character and per-session statistics, persisted as JSON."""

from __future__ import annotations

import json
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path

from cwtrainer.scenarios.base import Result


@dataclass
class CharStat:
    attempts: int = 0
    errors: int = 0

    @property
    def error_rate(self) -> float:
        return self.errors / self.attempts if self.attempts else 0.0


@dataclass
class Session:
    scenario: str
    wpm: int
    started: float = field(default_factory=time.time)
    prompts: int = 0
    perfect: int = 0
    score_sum: float = 0.0

    @property
    def score(self) -> float:
        return self.score_sum / self.prompts if self.prompts else 0.0


class Stats:
    def __init__(self, path: Path | None = None) -> None:
        self.path = path
        self.chars: dict[str, CharStat] = {}
        self.sessions: list[dict] = []
        self.session: Session | None = None
        self.session_chars: dict[str, CharStat] = {}
        if path is not None and path.exists():
            self.load()

    def load(self) -> None:
        assert self.path is not None
        data = json.loads(self.path.read_text())
        self.chars = {k: CharStat(**v) for k, v in data.get("chars", {}).items()}
        self.sessions = data.get("sessions", [])

    def save(self) -> None:
        if self.path is None:
            return
        sessions = list(self.sessions)
        if self.session is not None and self.session.prompts:
            sessions.append(self._session_dict())
        data = {"chars": {k: asdict(v) for k, v in sorted(self.chars.items())}, "sessions": sessions}
        self.path.parent.mkdir(parents=True, exist_ok=True)
        tmp = self.path.with_suffix(".tmp")
        tmp.write_text(json.dumps(data, indent=1))
        tmp.replace(self.path)

    def start_session(self, scenario: str, wpm: int) -> None:
        self.session = Session(scenario, wpm)
        self.session_chars = {}

    def end_session(self) -> Session | None:
        s = self.session
        if s is not None and s.prompts:
            self.sessions.append(self._session_dict())
        self.session = None
        return s

    def _session_dict(self) -> dict:
        assert self.session is not None
        d = asdict(self.session)
        d["score"] = round(self.session.score, 4)
        del d["score_sum"]
        return d

    def record(self, result: Result) -> None:
        wrong = list(result.wrong)
        for ch in result.attempted:
            err = ch in wrong
            if err:
                wrong.remove(ch)
            for table in (self.chars, self.session_chars):
                st = table.setdefault(ch, CharStat())
                st.attempts += 1
                st.errors += err
        if self.session is not None:
            self.session.prompts += 1
            self.session.perfect += result.ok
            self.session.score_sum += result.score

    def weak_characters(self, n: int, min_attempts: int = 5, session_only: bool = False) -> list[str]:
        table = self.session_chars if session_only else self.chars
        cands = [(st.error_rate, ch) for ch, st in table.items()
                 if st.attempts >= min_attempts and st.errors]
        return [ch for _, ch in sorted(cands, key=lambda c: (-c[0], c[1]))[:n]]
