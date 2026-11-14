# cwtrainer — working notes

CW sending trainer: iambic paddle on a vBand USB adapter (evdev) -> keyer ->
sidetone + decoder -> practice scenarios. Python 3.12 via uv, numpy,
sounddevice, evdev.

## Read first

- `docs/PLAN.md` — original spec and milestones (all done; status at top).
- `docs/DECISIONS.md` — every choice made where the plan was open, plus open
  issues (sidetone latency). Append to it when you make a new one.
- `README.md` — user-facing usage per scenario. Update with every new option.

## Layout

| path | role |
| --- | --- |
| `cwtrainer/keyer.py` | iambic state machine (pure) + threaded runner |
| `cwtrainer/decoder.py` | KeyEvents -> text by gap timing |
| `cwtrainer/sidetone.py`, `player.py` | one audio stream: keyed tone + mixed prompt playback |
| `cwtrainer/paddle.py`, `rig.py` | evdev input, wiring, UI event queue |
| `cwtrainer/session.py` | prompt / collect / grade loop, keyboard controls |
| `cwtrainer/grading.py`, `stats.py` | edit-distance grading, per-char stats |
| `cwtrainer/scenarios/` | one module per scenario, self-registering |
| `tools/dry_run.py` | run a scenario with scripted answers, no paddle |

## Constraints

- Logic never calls `time` directly: take a `Clock` (`FakeClock` in tests).
- Audio and evdev are imported lazily so tests run without PortAudio/devices.
- Agent can't hear audio or press paddles: real-hardware behaviour is the
  owner's call. Say what was and wasn't verified.
- Keep changes inside this directory.

## Testing

Run everything through uv: `uv run pytest`, `uv run python tools/dry_run.py ...`.
`uv sync` after editing dependencies; commit `uv.lock`. The `verify` skill
covers tests plus dry runs; the `add-scenario` skill is the checklist for new scenarios.

## Conventions

- Terse comments and commit messages; subject `cwtrainer: ...`.
- Type hints, dataclasses and enums over string constants.
- Commit per logical step; refactors needed by a feature go in their own commit.
