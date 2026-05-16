# Project Notes

## Structure

- `src/lib.rs`: local library root
- `src/support/bitbang_i2c_compat.rs`: adapts `bitbang-hal` I2C to `embedded-hal 1.0`
- `src/support/si5351.rs`: vendored SI5351 driver adapted for this project
- `src/bin/si5351-gen.rs`: main firmware app

## SI5351 Notes

- Based on upstream `ilya-epifanov/si5351`
- Kept intentionally close to upstream shape
- Current app strategy is simple:
  - call `set_frequency(...)`
  - explicitly enable/disable outputs afterward as needed

## Button Timing

- Debounce: `20 ms`
- Long press threshold: `600 ms`
- Double-click window: `400 ms`

## Frequency Step Set

- `10 Hz`
- `100 Hz`
- `500 Hz`
- `1 kHz`
- `10 kHz`
- `100 kHz`
- `500 kHz`
- `1 MHz`
- `10 MHz`

## Known UX Tradeoff

- Single-click navigation is deferred until the double-click window expires.
- This is intentional so double-click apply is reliable.

## Future Session Prompt

Suggested prompt:

```text
Read codex/AGENTS.md, codex/notes.md, and codex/worklog.md first.
```
