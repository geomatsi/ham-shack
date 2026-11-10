# Decisions

Choices made where `PLAN.md` was open or ambiguous.

- `<BT>` and `=` share a code; the decoder emits `=`. Prosigns that collide with
  punctuation not in the table (`<AR>`/`+`, `<KN>`/`(`) decode as the prosign.
