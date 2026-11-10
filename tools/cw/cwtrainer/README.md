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

## Echo

```
cwtrainer --device /dev/input/eventX echo [--show | --listen | --both] [--count 10]
```

Send back each prompt, a QSO phrase or random words. The answer ends after
`--answer-timeout` seconds of silence; it is graded by character edit distance
(`5NN` counts as `599`, `<HH>` erases the word before it). Enter, space or the
adapter button replays the prompt and restarts the answer; `q` or Ctrl+C ends
the session. Per-character stats go to `~/.local/share/cwtrainer/stats.json`.

## Callsigns

```
cwtrainer --device /dev/input/eventX callsigns [--listen] [--count 20] [--ramp]
```

Echo with generated callsigns: 1x2 to 2x3 patterns on real prefixes, sometimes
a digit prefix (`9A`, `S5`, `--digit-prefix`) or a `/P` `/M` `/QRP` suffix
(`--portable`). `--ramp` starts with short calls and moves up a level when
the last 5 average 95%, down below 60%.
