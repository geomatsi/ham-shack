# STM32 Blue Pill SI5351 Generator

Simple SI5351-based signal generator firmware for the STM32F103 Blue Pill with:

- HD44780 16x2 LCD
- rotary encoder with push button
- bit-banged I2C to the SI5351 module

## Blue Pill Pin Connections

| Function | Blue Pill Pin | Notes |
| --- | --- | --- |
| LCD `RS` | `PB9` | HD44780 control |
| LCD `RW` | `PB8` | HD44780 control |
| LCD `EN` | `PB7` | HD44780 control |
| LCD low data pins | `PB3`, `PB4`, `PB5`, `PB6` | kept low in 4-bit mode |
| LCD data `D4` | `PA9` | HD44780 data |
| LCD data `D5` | `PA10` | HD44780 data |
| LCD data `D6` | `PA11` | HD44780 data |
| LCD data `D7` | `PA12` | HD44780 data |
| Encoder `CLK` | `PB10` | rotary encoder |
| Encoder `DT` | `PB11` | rotary encoder |
| Encoder button | `PB1` | active low |
| SI5351 `SCL` | `PB13` | bit-banged I2C |
| SI5351 `SDA` | `PB12` | bit-banged I2C |

## UI HOWTO

- The application has two main screens:
  - `CLK0`
  - `CLK1`
- Long press of the encoder button switches between `CLK0` and `CLK1`.
- The first LCD line shows the active clock name.
- If there are unapplied changes, `*` appears right after the clock name.
- The second LCD line contains three controls:
  - frequency
  - frequency step
  - output state
- Rotate the encoder to change the currently selected control.
- Single short press moves to the next control.
- Double short press applies the current control value.

Power-on defaults:

- all outputs disabled
- `CLK0` selected frequency = `7 MHz`
- `CLK1` selected frequency = `14 MHz`

Available frequency steps:

- `10 Hz`
- `100 Hz`
- `500 Hz`
- `1 kHz`
- `10 kHz`
- `100 kHz`
- `500 kHz`
- `1 MHz`
- `10 MHz`

## Build And Flash

Start tmux debug environment with ST-Link or Segger:

```bash
cargo make debug
```

Flash release image:

```bash
cargo make flash_release <binary name>
```

Flash debug image:

```bash
cargo make flash_debug <binary name>
```

## Debug

Semihosting debug:

```bash
sudo openocd -f tools/openocd.cfg -c 'attach ()'
cargo build --bin si5351-gen
cargo run --bin si5351-gen
```
