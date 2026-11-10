# Decisions

Choices made where `PLAN.md` was open or ambiguous.

- `<BT>` and `=` share a code; the decoder emits `=`. Prosigns that collide with
  punctuation not in the table (`<AR>`/`+`, `<KN>`/`(`) decode as the prosign.
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
- Keying during prompt playback cuts the playback; the keying counts as the
  answer.
- Replay always plays audio, also in `--show` mode, and discards what was sent
  so far.
- Stats are saved after every prompt; the in-progress session is included so
  a crash loses nothing.
