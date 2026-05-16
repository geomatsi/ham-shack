# Project Agent Notes

Read this file first in new sessions for this repository.

## Project

- Board: STM32F103 Blue Pill
- App: SI5351 signal generator with HD44780 LCD and rotary encoder
- Main binary: `src/bin/si5351-gen.rs`

## Current Wiring

- LCD:
  - `PB9` = `RS`
  - `PB8` = `RW`
  - `PB7` = `EN`
  - `PB3/PB4/PB5/PB6` are forced low
  - `PA9/PA10/PA11/PA12` = LCD data lines in 4-bit mode
- Encoder:
  - `PB10` = `CLK`
  - `PB11` = `DT`
  - `PB1` = button
- SI5351 bit-banged I2C:
  - `PB13` = `SCL`
  - `PB12` = `SDA`

## Current UI Behavior

- Two main screens: `CLK0` and `CLK1`
- Long press switches screen
- Single short press moves to the next control after the double-click window expires
- Double short press applies the current control value
- Controls on line 2:
  - frequency
  - step
  - state
- Pending-change marker:
  - `*` appears immediately after `CLK0` / `CLK1` on line 1

## Current Defaults

- On power-up:
  - all outputs disabled
  - `CLK0` selected frequency = `7 MHz`
  - `CLK1` selected frequency = `14 MHz`
- Only `CLK0` and `CLK1` are used

## Build Notes

- Default target: `thumbv7m-none-eabi`
- `cargo build` is tuned to fit flash using size-oriented `profile.dev` settings
- Common verification commands:
  - `cargo fmt`
  - `cargo check --bin si5351-gen`
  - `cargo build`

## Repo Conventions

- Keep project context files in `codex/`
- Append important decisions and changes to `codex/worklog.md`
- Prefer keeping vendored support code under `src/support/`
