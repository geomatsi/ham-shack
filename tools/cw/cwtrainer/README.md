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

## POTA activator

```
cwtrainer --device /dev/input/eventX pota --mycall R3XYZ [--park OH-0001] [--show]
          [--agn] [--partial] [--pileup] [--p2p] [--hunter-wpm 18-25]
```

1. Call CQ: must contain `CQ` and your call.
2. A hunter (or none) calls, at its own speed and tone (±100 Hz).
3. Send `<HCALL> 5NN`. A miscopied call makes the hunter repeat it; a missing
   RST gets `AGN?`. Sending `?` or `AGN` asks for a repeat.
4. Hunter acknowledges; sign off with `TU`, `73` or `QRZ`.
5. Next caller, or call CQ again.

Realism flags take an optional probability: `--agn` hunter missed your
exchange, `--partial` only part of the call is heard (send `<PART>?`),
`--pileup` two hunters at once (work either), `--p2p` hunter sends its park,
which you must send back. Hunter text stays hidden unless `--show`; the
summary gives QSO count, accuracy and time per QSO.

## POTA hunter

```
cwtrainer --device /dev/input/eventX pota-hunter --mycall R3XYZ [--show] [--log]
          [--agn] [--partial] [--busted] [--pileup] [--activator-wpm 18-25]
```

The other side of a POTA QSO: an activator calls CQ, at its own speed and tone.

1. Send your call.
2. Activator sends `<YOURCALL> 5NN`; send your report (`R 5NN TU`).
3. Activator signs off and the next one calls CQ.

If the activator sends your call wrong, send your call again; sending a report
instead logs a busted QSO. `?` or `AGN` asks for a repeat. Realism flags take
an optional probability: `--partial` activator sends `R3?`, `--busted` copies
your call wrong, `--pileup` works another hunter first (call again after
`QRZ`), `--agn` asks for your report again. `--log` adds a step after each
QSO: send the activator's call and park as you logged them.
