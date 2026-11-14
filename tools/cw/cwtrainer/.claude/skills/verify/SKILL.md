---
name: verify
description: Run the cwtrainer checks - unit tests plus scripted dry runs of the scenarios through the real session UI. Use after changing anything under cwtrainer/, and before telling the user a change works.
---

# Verify cwtrainer

From the `cwtrainer` directory:

```sh
uv run pytest -q
```

Then dry-run every scenario the change touches (all of them after changes to
`session.py`, `grading.py`, `player.py` or `scenarios/base.py`):

```sh
uv run python tools/dry_run.py echo --both --count 4 --seed 1
uv run python tools/dry_run.py callsigns --ramp --count 8 --seed 1
uv run python tools/dry_run.py groups --charset mix --size 1-6 --count 4 --seed 1
uv run python tools/dry_run.py sentences --level 300 --max-words 8 --count 4 --seed 1
uv run python tools/dry_run.py pota --mycall R3XYZ --agn --partial --pileup --p2p --count 4 --seed 5
uv run python tools/dry_run.py pota-hunter --mycall R3XYZ --agn --partial --busted --pileup --log --count 4 --seed 8
```

`--error P` (default 0.15) corrupts that share of answers; `--error 0` for a
clean run. Read the transcript, not just the exit code: check prompts look
right, wrong answers get the expected reaction (repeat, AGN?, busted call),
nothing that should stay hidden is revealed, and the summary adds up.

## What this does not cover

Audio (clicks, latency, timing) and the real paddle. Dry runs use
`NullSidetone` and skip the keyer and decoder entirely. Say so when reporting;
those need the owner's ears and hands (see "Human check" in `docs/PLAN.md`).
