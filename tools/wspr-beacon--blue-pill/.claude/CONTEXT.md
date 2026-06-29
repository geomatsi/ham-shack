# Project context

A WSPR beacon firmware project in Rust targeting the STM32F103C8T6 "BluePill"
board. Lives under `ham-shack/tools/wspr-beacon--blue-pill/`, part of a broader
ham-radio tooling repository.

## Hardware

- **MCU**: STM32F103C8T6 (Cortex-M3, 72 MHz, 64 KB Flash, 20 KB RAM)
- **Board**: BluePill (generic STM32F103 dev board)
- **Display**: SSD1306 OLED (I2C), supported via the `ssd1306` crate
- **Linker target**: `thumbv7m-none-eabi`

## Layout

```
.cargo/config.toml   — target + linker flags (link-arg=-Tlink.x)
build.rs             — copies memory.x into OUT_DIR for the linker
Cargo.toml           — workspace-free, single [[bin]]
memory.x             — 64K Flash / 20K RAM memory map
src/
  beacon.rs          — main binary (#![no_std], #![no_main], nop loop)
```

## Crate versions

| Crate | Version | Notes |
|-------|---------|-------|
| `stm32f1xx-hal` | 0.11 | features: stm32f103, medium (rt feature removed in 0.11) |
| `cortex-m` | 0.7 | features: critical-section-single-core |
| `cortex-m-rt` | 0.7 | |
| `ssd1306` | 0.10 | embedded-hal 1.x compatible |
| `embedded-graphics` | 0.8 | |
| `panic-halt` | 0.2 | |

All crates use **embedded-hal 1.x** (not the legacy 0.2 API).

## Key linker constraint

`use stm32f1xx_hal as _;` must appear in every binary, even if the HAL is not
otherwise used. Without it the linker drops the PAC and the `__INTERRUPTS`
symbol is missing, causing a link error from `cortex-m-rt`.

## Build

```sh
cargo build --bin beacon
cargo build --bin beacon --release
```

`rustup target add thumbv7m-none-eabi` is required once.

## Flashing

Typical workflow (adapt to your probe):
```sh
cargo flash --chip STM32F103C8 --bin beacon
# or with probe-rs:
probe-rs run --chip STM32F103C8Tx target/thumbv7m-none-eabi/release/beacon
```

## Rust edition

2024

## User

Sergey (geomatsi@gmail.com) — ham radio operator, comfortable with Rust and
embedded work. Prefers terse responses.
