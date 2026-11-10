"""Scenario registry."""

from __future__ import annotations

from cwtrainer.scenarios.base import Prompt, Result, Scenario, ScenarioFactory

SCENARIOS: dict[str, ScenarioFactory] = {}


def register(cls):
    SCENARIOS[cls.name] = cls
    return cls


from cwtrainer.scenarios import callsigns, echo  # noqa: E402,F401  (register themselves)

__all__ = ["SCENARIOS", "Prompt", "Result", "Scenario", "register"]
