# cwtrainer

CW sending trainer for an iambic paddle on a vBand USB adapter (Linux, evdev).
See `PLAN.md` for the design and `DECISIONS.md` for choices made along the way.

## Install

```
pip install -e '.[test]'
pytest
cwtrainer --help
```

## Practice keyer

```
cwtrainer --device /dev/input/eventX --keyer-only [--wpm 22] [--mode B] [--swap]
```

The vBand adapter shows up as two `HID 413d:2107` nodes; use the one with
`BTN_LEFT`/`BTN_RIGHT`. The adapter button or Ctrl+C exits.

What you send is decoded live; a new line starts after `--answer-timeout`
seconds of silence. Gaps of 2 units split characters, 5 units split words,
six or more dits decode as `<HH>`, an unknown pattern as `*`.
