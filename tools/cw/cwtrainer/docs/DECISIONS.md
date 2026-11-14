# Decisions

Choices made where `PLAN.md` was open or ambiguous.

- `<BT>` and `=` share a code; the decoder emits `=`. Prosigns that collide with
  punctuation not in the table (`<AR>`/`+`, `<KN>`/`(`) decode as the prosign.
  `<BK>` added later for the MorseNinja phrases; it decodes as `<BK>`, not `BK`.
- Element memory latches only the opposite paddle, during the element or the
  gap after it. Latching the same paddle would turn contact bounce into extra
  elements.
- Mode B "squeezed" means both paddles were down at some point while the
  element was sounding.
- The keyer timestamps elements from the paddle event time, so the decoder sees
  exact timing even if the keyer thread wakes late.
- Prompt audio is mixed into the sidetone stream rather than played with
  `sd.play()`, so only one output stream is ever open.
- Adapter button: exits in `--keyer-only`; replays the prompt in scenarios
  (exit there with Ctrl+C or `q`). The adapter reports it as relative motion,
  so there is no press duration to distinguish long/short presses; repeats
  within 0.3 s are dropped.
- Decoder thresholds are in units of the keyer WPM, so a user who sends with
  Farnsworth-wide gaps gets spurious word breaks. Fine for now; an adaptive
  decoder can track the user's actual spacing later.
- Echo prompts are 60% phrases, 40% one or two random words; no immediate
  repeats. Answers are exact-match for "ok" with a 0..1 score for partials.
- `phrases.txt` adds MorseNinja's Common QSO Phrases (+Encore), Practice QSO
  (Complete One-Sided), Top 180 QSO Elements and Abbreviations: trailing ` =`
  stripped, deduped, templated lines capped (2 per call/RST template, 3 per
  practice-QSO template), `VVV` and single-word job fragments dropped. Single
  elements are ~30% of lines. `words.txt` adds Top 500 Words.
- Keying during prompt playback cuts the playback; the keying counts as the
  answer.
- Replay always plays audio, also in `--show` mode, and discards what was sent
  so far.
- Stats are saved after every prompt; the in-progress session is included so
  a crash loses nothing.
- Callsign difficulty levels: 1 = 1x2/2x1, 2 = +1x3/2x2, 3 = +2x3,
  4 = +digit prefixes, 5 = +portable suffixes. Without `--ramp` everything is
  level 5. Area digit is any 0-9 regardless of country.
- `-` added to the Morse table for park references (`OH-0001`).
- POTA grades loosely: the required tokens (CQ, calls, park) are matched
  against the closest sent word; anything else in the over is ignored.
  RST is any `[1-5][1-9N][1-9N]`.
- A miscopied hunter call is never revealed; the hunter repeats it. `AGN?`
  happens at most once per QSO.
- A wrong sign-off (or missing P2P park) is marked wrong but the QSO is still
  logged: the hunter has gone and won't repeat.
- Repeat requests (`?`, `AGN`, `RPT`, `QRZ?`) are not graded.
- A CQ gets 1-3 callers, none with probability 0.1.
- `--park` is not checked anywhere yet; it's accepted in the exchange like any
  other extra text.
- `Prompt` gained `instruction` and `voices`, `Result` gained `reveal`; all
  default to the old behaviour. Scenarios may define `summary()`.
- POTA hunter: a call within edit distance 2 of yours is "answered as sent",
  so a miskeyed call comes back busted and must be corrected; anything further
  off gets `?`. Further-off answers aren't graded per character.
- Each hunter realism event happens at most once per QSO. Activators send
  their park in some CQ forms; with `--log` they always do. `--log` is off by
  default.
- The activator's sign-off and the next CQ are mixed into one prompt rather
  than asking the user to acknowledge.

## Tooling: uv (2026-11-15)

- Managed with uv, Python pinned to 3.12 in `.python-version`;
  `requires-python >= 3.12`. `uv.lock` is committed.
- pytest moved from the `test` extra to the `dev` dependency group, so plain
  `uv sync` / `uv run` installs it.
- Build backend is `uv_build` (flat layout via `module-root = ""`; `data/`
  ships automatically). setuptools left `cwtrainer.egg-info` in the tree on
  every editable install.

## Sentences scenario (2026-11-15)

- `data/sentences.txt` has one `# Top N words` section per MorseNinja file;
  a sentence sits in the first (lowest) level it appears in, and `--level N`
  takes every section up to N. Default is all 1600.
- Punctuation is stripped by default: every sentence ends in `.` or `?`, which
  would dominate the error stats. `--punct` keeps `. , ?`; `-` becomes a space.
- `--max-words` is at least 3, so no level/limit combination leaves an empty
  pool (the shortest top-100 sentences have 3 words).

## Open: sidetone latency (postponed 2026-11-11)

Extra dots while sending. Measured without the paddle: keyer thread lag
< 1 ms, so not CPU or priority. Output through ALSA `default` -> PulseAudio
is ~50 ms to the DAC (PortAudio's reported 10.7 ms is only its own buffer);
`hw:0,0` direct is ~12 ms. Late sidetone makes the operator release late, so
the keyer sees the paddle held at gap end and adds a dit. Callbacks also come
in ~9 ms bursts, jittering element edges. Wireless headphones add more.

Possible fixes: `--audio-device` for direct ALSA hw; key edges scheduled by
timestamp in the audio callback; a `--debug-timing` log to confirm the cause.
