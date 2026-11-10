---
name: add-scenario
description: Checklist for adding a new cwtrainer practice scenario - module, registration, CLI options, tests, dry-run oracle, docs. Use when the user asks for a new scenario or drill.
---

# Add a cwtrainer scenario

1. **Pick a base.** Prompt-then-send-it-back drills subclass `EchoBase`
   (`scenarios/echo.py`) and implement only `_pick()`; see `groups.py`,
   `callsigns.py`. Multi-step exchanges implement the `Scenario` protocol
   directly as a state machine; see `pota.py`, `hunter.py` and reuse their
   helpers (`words_of`, `best_match`, `combine`, `Station`, `overs`).
2. **Module** `cwtrainer/scenarios/<name>.py` with `@register`, class attrs
   `name` (the CLI subcommand) and `help`, `add_arguments(p)` and
   `from_args(args, cfg)`. Validate options with argparse `type=` functions so
   errors come out as usage errors.
3. **Register the import** in `scenarios/__init__.py` (the `from
   cwtrainer.scenarios import ...` line) — without it the subcommand is missing.
4. **Prompt/Result extras** if needed: `Prompt.instruction`, `Prompt.voices`
   (mixed stations with wpm/tone/delay), `Result.reveal=False` to keep an
   answer the user must still copy hidden, optional `summary()`. Extend these
   with defaults rather than changing existing fields.
5. **Grading.** Don't feed stats characters the user never attempted: grade a
   field only when the sent word is close to it (see hunter `_call`).
6. **Tests** `tests/test_<name>.py`: CLI parsing and defaults, registration,
   generator shapes over many seeds, every state transition, `done()`.
7. **Dry-run oracle**: add a branch to `oracle()` in `tools/dry_run.py` unless
   the scenario is an `EchoBase`.
8. **Docs**: a section in `README.md`; any choice the user didn't specify goes
   in `docs/DECISIONS.md`.
9. Run the `verify` skill, then commit as `cwtrainer: add <name> scenario`.
