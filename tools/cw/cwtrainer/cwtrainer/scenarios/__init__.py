"""Scenario registry."""

from __future__ import annotations

from cwtrainer.scenarios.base import Prompt, Result, Scenario, ScenarioFactory, Voice

SCENARIOS: dict[str, ScenarioFactory] = {}


def register(cls):
    SCENARIOS[cls.name] = cls
    return cls


from cwtrainer.scenarios import callsigns, groups, hunter, pota, qso, sentences  # noqa: E402,F401  (register themselves)

__all__ = ["SCENARIOS", "Prompt", "Result", "Scenario", "Voice", "register"]
