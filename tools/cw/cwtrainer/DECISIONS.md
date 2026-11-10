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
