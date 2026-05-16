# Worklog

Date: 2026-05-05
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

## Task

- Refresh the project dependencies.
- Update the crate edition to the latest stable Rust edition.
- Modernize local cargo tooling, including replacing `.cargo/config` with `.cargo/config.toml`.

## Sources Checked

- Rust 2024 edition guide: Rust 2024 is stable starting with Rust `1.85.0`.
- Cargo Book config reference: `.cargo/config.toml` is the preferred config filename; Cargo still reads `.cargo/config`, but it is legacy.
- docs.rs crate pages used to confirm current published versions:
  - `cortex-m 0.7.7`
  - `cortex-m-rt 0.7.5`
  - `cortex-m-semihosting 0.5.0`
  - `panic-semihosting 0.6.0`
  - `nb 1.1.0`
  - `stm32f1xx-hal 0.11.0`
  - `hd44780-driver 0.4.0` (already latest)

## Commands

```text
rg --files -uu
sed -n '1,220p' Cargo.toml
sed -n '1,220p' .cargo/config
git status --short
sed -n '1,260p' src/bin/lcd-hd44780-test1.rs
sed -n '1,240p' Makefile.toml
sed -n '1,220p' README.md
sed -n '1,220p' Cargo.lock
rustc --version
cargo --version
ls -la .cargo
rg --files ~/.cargo/registry/src | rg 'stm32f1xx-hal-0.11.0|cortex-m-rt-0.7.5|panic-semihosting-0.6.0|cortex-m-semihosting-0.5.0|nb-1.1.0'
rg -n "disable_jtag|afio|into_push_pull_output|Delay::new|Peripherals::take|SysTick" ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0 -g '*.rs'
sed -n '1,220p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/examples/blinky.rs
cargo check --bin lcd-hd44780-test1
cargo info stm32f1xx-hal
cargo info cortex-m-rt
find ~/.cargo/registry/index -path '*stm32f1xx-hal*' -o -path '*cortex-m-rt*' -o -path '*panic-semihosting*' -o -path '*cortex-m-semihosting*' -o -path '*hd44780-driver*' -o -path '*nb'
sed -n '1,260p' ~/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/st/m3/stm32f1xx-hal
sed -n '1,220p' ~/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/co/rt/cortex-m-rt
sed -n '1,220p' ~/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/pa/ni/panic-semihosting
sed -n '1,220p' ~/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/hd/44/hd44780-driver
sed -n '1,220p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/examples/delay.rs
sed -n '1,220p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/examples/gpio_input.rs
sed -n '1,220p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src/afio.rs
rg -n "struct Rcc|impl Rcc|fn cfgr|fn freeze\\(|Config::hse|use_hse\\(" ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src/rcc.rs
sed -n '1,260p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src/rcc.rs
cargo fmt
git status --short
nl -ba Cargo.toml | sed -n '1,160p'
nl -ba src/bin/lcd-hd44780-test1.rs | sed -n '1,260p'
```

## Plan

1. Update `Cargo.toml` to Rust 2024 and latest crate versions.
2. Rename cargo config to `.cargo/config.toml`.
3. Download and resolve the new dependency graph.
4. Compile and patch source code for `stm32f1xx-hal 0.11`.
5. Record the final outcome and any compatibility notes.

## Reasoning

- `edition = "2024"` is the current stable Rust edition, and Rust 2024 was released with Rust `1.85.0`, so setting `rust-version = "1.85"` is a consistent floor.
- `hd44780-driver` has not released a newer version, so leaving it at `0.4.0` is part of a full refresh, not an omission.
- The MCU HAL jump from `0.7` to `0.11` is the high-risk part because its GPIO and peripheral APIs changed substantially.
- The project still uses an LCD driver built on `embedded-hal 0.2`, so a direct `embedded-hal = "0.2.7"` dependency remains justified even though the ecosystem has also moved to `1.0`.
- Replacing `.cargo/config` with `.cargo/config.toml` removes the current Cargo deprecation warning without changing behavior.
- `stm32f1xx-hal 0.11.0` removed the old `rt` feature, so the manifest must stop enabling it explicitly.
- The new HAL exposes a newer clock configuration API based around `rcc::Config` and uses `RateExtU32` methods like `8.MHz()` instead of the older lowercase frequency helpers.

## Progress

- Manifest updated.
- `.cargo/config` replaced with `.cargo/config.toml`.
- Source migrated to `stm32f1xx-hal 0.11`.
- Build verification completed.

## Source Migration Notes

- `hal::stm32::Peripherals` changed to `stm32f1xx_hal::pac::Peripherals`.
- The old `Delay::new(cp.SYST, clocks)` path was replaced with `cp.SYST.delay(&rcc.clocks)`.
- RCC configuration was updated from the old builder usage on `rcc.cfgr` to:
  - `dp.RCC.freeze(rcc::Config::hse(8.MHz()).sysclk(32.MHz()).pclk1(16.MHz()), &mut flash.acr)`
- Direct GPIO output helpers in the current HAL are infallible in this usage, so the old `.unwrap()` calls on `set_low()` / `set_high()` were removed.

## Verification

- `cargo check --bin lcd-hd44780-test1` succeeds after the refresh.
- `cargo fmt` succeeds.
- Cargo updated `Cargo.lock` to the latest Rust `1.85` compatible set for the new manifest.

## Final Dependency State

- `cortex-m = "0.7.7"`
- `cortex-m-rt = "0.7.5"`
- `cortex-m-semihosting = "0.5.0"`
- `panic-semihosting = "0.6.0"`
- `nb = "1.1.0"`
- `stm32f1xx-hal = "0.11.0"`
- `hd44780-driver = "0.4.0"` (already latest published version)
- `embedded-hal = "0.2.7"` kept intentionally because `hd44780-driver 0.4.0` still uses the `0.2.x` trait generation

---

## Encoder Task

### Task

- Add `rotary-encoder-embedded`.
- Use encoder pins:
  - `PB10` = `CLK`
  - `PB11` = `DT`
  - `PB1` = button
- Show encoder state on the LCD:
  - first line at `0x00`: rotary counter starting at `0`
  - second line at `0x40`: button press counter starting at `0`

### Commands

```text
git status --short
sed -n '1,220p' Cargo.toml
sed -n '1,260p' src/bin/lcd-hd44780-test1.rs
sed -n '1,260p' worklog.md
rg --files ~/.cargo/registry/src ~/.cargo/registry/index | rg 'rotary-encoder-embedded|rotary_encoder_embedded'
sed -n '1,260p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rotary-encoder-embedded-0.5.0/src/lib.rs
sed -n '1,220p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rotary-encoder-embedded-0.5.0/src/standard.rs
rg -n "impl .*embedded_hal::digital::InputPin|impl .*embedded_hal_02::digital::v2::InputPin|is_low\\(&self\\)|is_low\\(&mut self\\)" ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src/gpio.rs
sed -n '500,700p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src/gpio.rs
sed -n '1,220p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rotary-encoder-embedded-0.5.0/README.md
rg -n "impl .*InputPin for .*Pin|impl .*ErrorType for .*Pin|embedded_hal_02::digital::v2::InputPin for .*Pin|embedded_hal::digital::InputPin for .*Pin" ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src -g '*.rs'
sed -n '1,280p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src/gpio/hal_1.rs
cargo check --bin lcd-hd44780-test1
cargo fmt
```

### Plan

1. Add the encoder crate to `Cargo.toml`.
2. Replace the LCD demo loop with encoder polling and button counting.
3. Keep the current refreshed HAL setup and reuse the existing LCD wiring.
4. Debounce the button in software and update only the changed LCD line.
5. Verify with `cargo check`.

### Reasoning

- `rotary-encoder-embedded 0.5.0` is already available in the local cargo registry cache and matches the crate used in the earlier encoder task.
- `stm32f1xx-hal 0.11.0` already implements `embedded-hal 1.0` digital `InputPin` for typed GPIO input pins, so the encoder pins can be passed directly to `RotaryEncoder::new(...)` without the compatibility wrapper that was needed on the older HAL.
- The encoder crate’s standard mode recommends polling around `900 Hz`, so a `1 ms` loop delay is appropriate here.
- Raw button sampling would overcount due to bounce, so the button press counter uses stable-state debounce and increments only on a confirmed pressed transition.
- The LCD is updated only when a value changes to avoid unnecessary rewrites and flicker.

### Progress

- Dependency updated.
- Source replaced with encoder/button polling logic.
- Verification completed.

### Verification

- `cargo check --bin lcd-hd44780-test1` succeeds after adding the encoder support.
- `cargo fmt` succeeds on the updated source.
- `Cargo.lock` was updated to add `rotary-encoder-embedded 0.5.0`.

---

## LCD Label Tweak

### Task

- Add an information string to each LCD line so it is clear which counter is for rotation and which is for button presses.

### Reasoning

- Bare numeric counters are ambiguous on a 16x2 LCD.
- Adding fixed labels keeps the interaction unchanged while making the display self-describing.

### Changes

- First line now renders as `Rotary: <value>`.
- Second line now renders as `Button: <value>`.

### Verification

- Re-ran formatting and `cargo check --bin lcd-hd44780-test1` after the label change.

---

## Short/Long Press

### Task

- Distinguish short and long button presses.
- Show button counts as `BTN: S(N) L(M)` on the second LCD line.

### Reasoning

- The original long-form `Button: S(N) L(M)` does not fit cleanly on a 16-character HD44780 line.
- The user-approved shorter `BTN: S(N) L(M)` format fits while keeping both short and long counters visible.
- Long presses are detected with a simple duration threshold in the existing 1 kHz polling loop.

### Changes

- Added separate short-press and long-press counters.
- Added a long-press threshold of `750 ms`.
- Second line now renders as `BTN: S(N) L(M)`.

### Verification

- Re-ran formatting and `cargo check --bin lcd-hd44780-test1` after the short/long press change.

---

## Release-Only Button Update

### Task

- Make long-press timing shorter.
- Define long press as `3x` the normal press timing.
- Do not update the button status string until the button is released.

### Reasoning

- Updating the LCD while the button is still held makes the state feel provisional.
- Counting on release gives a single final classification per press: either short or long.
- I used a `200 ms` normal-press timing baseline and a `600 ms` long-press threshold (`3x`) so the long press is shorter than the previous `750 ms` threshold while remaining meaningfully distinct from a short tap.

### Changes

- Added `NORMAL_PRESS_TICKS = 200`.
- Changed long press threshold to `LONG_PRESS_TICKS = NORMAL_PRESS_TICKS * 3`.
- Removed in-hold LCD updates for long press detection.
- Button counters now increment only when the button is released and the total held duration is known.

### Verification

- Re-ran formatting and `cargo check --bin lcd-hd44780-test1` after the timing/release behavior change.

---

## Menu UI

### Task

- Replace the simple counter screen with a 5-level menu: `F1`, `F2`, `F3`, `F4`, `F5`.
- Display the current menu level on the first line.
- Use long press to jump to the next menu level.
- Each menu level owns one property value.
- Encoder rotation edits the value, but a new value is stored only after a short press.
- Display `*` at the end of the value when the currently shown value is the stored value.

### Plan

1. Replace the old button-counter UI with a small menu state machine.
2. Keep one stored value per menu level.
3. Keep one current draft value for the active menu.
4. Short press commits the draft into the active menu's stored value.
5. Long press advances to the next menu and loads that menu's stored value.
6. Re-verify formatting and build.

### Reasoning

- A stored-value marker naturally maps to a draft-vs-stored model: if the current displayed value matches the stored value, show `*`; otherwise omit it.
- To preserve the earlier requirement that rotating alone does not store the value, encoder movement only changes the active draft.
- On long-press menu switch, I reset the draft to the target menu's stored value. This discards uncommitted edits from the previous menu, which is consistent with "without press new value is not stored".
- The first line is kept minimal as `F1`..`F5` so the UI remains readable on a 16x2 LCD.

### Changes

- Added `MENU_COUNT = 5`.
- First LCD line now shows the active menu level.
- Second LCD line now shows `Value: <n>` and appends `*` when `<n>` is the stored value for the active menu.
- Long press advances cyclically through `F1 -> F2 -> F3 -> F4 -> F5 -> F1`.
- Short press stores the current draft value into the active menu.

### Verification

- Re-ran formatting and `cargo check --bin lcd-hd44780-test1` after the menu UI change.

---

## Menu Metadata Refactor

### Task

- Keep all menu names in a dedicated data structure instead of hardcoding `F1`..`F5` in rendering logic.
- Keep all feature names in a dedicated data structure instead of hardcoding `Value` in rendering logic.

### Reasoning

- Menu and feature labels are UI metadata, not logic.
- Moving them into arrays makes future menu renaming or per-menu feature renaming a data change instead of a code-path change.
- This keeps the current behavior identical while making the UI layer easier to extend.

### Changes

- Added `MENU_NAMES` as a fixed array of labels for the five menu levels.
- Added `FEATURE_NAMES` as a fixed array of per-menu feature labels.
- Updated menu-line rendering to read from `MENU_NAMES`.
- Updated value-line rendering to read from `FEATURE_NAMES`.

### Verification

- Re-ran formatting and `cargo check --bin lcd-hd44780-test1` after the metadata refactor.

---

## SI5351 Generator Tool

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Prompt

```text
Here we are in the directory with our stm32 blue-pill example with encoder and lcd.
Lets convert it into full-featured si5351 generator tool. Here are the inputs and the task:
# si5351 module i2c connected to pins PB13 (SCL) and PB12 (SDA)
# lets use bitbang-hal crate to use 2 gpios for i2c
  # bitbang-hal is a bit outdated, so you have adapter src/bitbang_i2c_compat.rs for new Rust embedded-hal API
# lets use crate si5351 (https://github.com/ilya-epifanov/si5351) to communicate with si5351 module
  # this module is abandoned and not supported, we will have to add features later, so lets pull it as a library into our project in the same way as src/bitbang_i2c_compat.rs
  # lets find a proper place for these two project built-in libs, maybe lib subdirectory ?
  # for now lets use only CLK0 and CLK1 output, leave CLK2 unused for now
  # example code for using si5351 for another MCU is in t/cw.rs
# after si5351 is initialized and ready to use in our rust application, lets rework interface in the following way:
  # we have two 'main' screens for CLK0 and CLK1 respectively
  # change between 'main' screens using long press of encoder button
  # first line on screen: CLK name
  # second line on screen contains the following controls: freq (initial 7 MHz), freq step (10 Hz, 100 Hz, 500 Hz, 1kHz, 10 kHz, 100 kHz, 500 kHz, 1MHz, 10 MHz), state (enabled/disabled)
  # rotation of encoder changes value of selected control, short press of encoder button applies selected new value, dual short press jumps to the next control
# common guidelines
  # create new chapter for your work in worklog.md
  # append this prompt to worklog.md as the first chapter in the new chapter
  # append all your worklogs (plans, choices, links, decisions) into worklog.md splitting to appropriate topic sections, e.g. si5351 work, ui work, and so on, with appropriate subsections

I forgot to mention initial state on power on. Since we don't have nonvolatile memory in this project at the moment, lets use the following initial condition:
- all clocks disabled
- CLK0 is 7MHz selection, CLK1 is 14MHz selection

No need to complicate: enable then disable is just fine for now. Try to avoid changes in vendored module for now as far as possible.
```

### Sources And Links

- `si5351` upstream repo: `https://github.com/ilya-epifanov/si5351`
- Local cached crate used as vendor base: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/si5351-0.2.0/src/lib.rs`
- Local cached `bitbang-hal` reference: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitbang-hal-0.3.3/src/i2c.rs`
- Local reference example reviewed: `t/cw.rs`

### Commands

```text
pwd
rg --files
git status --short
sed -n '1,220p' Cargo.toml
sed -n '1,260p' src/bin/lcd-hd44780-test1.rs
sed -n '1,260p' src/bitbang_i2c_compat.rs
sed -n '1,260p' t/cw.rs
sed -n '1,260p' worklog.md
rg --files ~/.cargo/registry/src ~/.cargo/registry/index | rg 'bitbang-hal|si5351'
rg -n "OpenDrain|into_open_drain_output|CountDown|Periodic|counter_hz" ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/stm32f1xx-hal-0.11.0/src -g '*.rs'
sed -n '1,260p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bitbang-hal-0.3.3/src/i2c.rs
sed -n '1,320p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/si5351-0.2.0/src/lib.rs
sed -n '320,640p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/si5351-0.2.0/src/lib.rs
tail -n 40 worklog.md
cargo check --bin lcd-hd44780-test1
cargo fmt
```

### Plan

1. Move the local compatibility helper into a dedicated in-tree support module area.
2. Vendor the `si5351` driver into the same support area, with only the minimum modernization needed to compile in this project.
3. Add `bitbang-hal`, `bitflags`, and `embedded-hal 1.0` alias dependencies.
4. Replace the old LCD menu demo with a two-screen `CLK0` / `CLK1` SI5351 UI.
5. Keep the initial power-on state disabled for both outputs, with stored selections `CLK0 = 7 MHz` and `CLK1 = 14 MHz`.
6. Verify with `cargo check` and `cargo fmt`.

### Module Layout

#### Choice

- I created `src/lib.rs` and placed the local built-in modules under `src/support/`.
- Files:
  - `src/support/bitbang_i2c_compat.rs`
  - `src/support/si5351.rs`

#### Reasoning

- I did not use a top-level `lib/` directory because this crate now benefits from a normal Rust library root in `src/lib.rs`, and `src/support/` keeps the local vendor-like modules explicit without path hacks.
- This keeps the application binary in `src/bin/` small enough to read while still letting the project own the two embedded support modules directly.

### SI5351 Work

#### Driver Integration

- Added `bitbang-hal = "0.3.3"`.
- Added `bitflags = "2.11.1"` through Cargo resolution.
- Added `embedded-hal-1 = { package = "embedded-hal", version = "1.0.0" }`.
- Moved the existing bit-bang I2C adapter into `src/support/bitbang_i2c_compat.rs`.
- Vendored the `si5351` driver into `src/support/si5351.rs`.

#### Vendored Driver Decisions

- I kept the vendored SI5351 driver close to the cached upstream source.
- I did modernize the I2C trait usage from `embedded-hal 0.2` to `embedded-hal 1.0` so the existing compatibility adapter is actually used.
- I kept the application logic on the simpler path requested later by the user: call `set_frequency(...)`, then explicitly disable the output again when needed, instead of pushing more behavior into the vendored module.
- I left `CLK2` unused in the application layer, as requested.

#### Hardware Wiring

- Encoder:
  - `PB10` = `CLK`
  - `PB11` = `DT`
  - `PB1` = button
- SI5351 bit-banged I2C:
  - `PB13` = `SCL`
  - `PB12` = `SDA`
- I2C timing source:
  - `TIM2.counter_hz(...)`
  - started at `200 kHz`, matching the `bitbang-hal` expectation of `2x` the target `100 kHz` I2C clock

#### Output Strategy

- `CLK0` uses `PLL::A`.
- `CLK1` uses `PLL::B`.
- On startup:
  - `CLK0` stored/draft frequency = `7_000_000`
  - `CLK1` stored/draft frequency = `14_000_000`
  - both outputs stored/draft enabled state = `false`
- After SI5351 init, both channels are programmed once and then left disabled to match the requested initial condition.

### UI Work

#### Interaction Model

- Two main screens: `CLK0` and `CLK1`.
- Long press switches between the two screens.
- Short press applies the currently selected control.
- Double short press advances the selected control.
- Encoder rotation edits only the draft value of the selected control.

#### Control Set

- Frequency
- Step
- State

#### Step Values

- `10 Hz`
- `100 Hz`
- `500 Hz`
- `1 kHz`
- `10 kHz`
- `100 kHz`
- `500 kHz`
- `1 MHz`
- `10 MHz`

#### Display Packing Choice

- A strict literal rendering of full labels and full values does not fit on a `16x2` HD44780.
- I therefore packed all three controls onto the second line in fixed-width fields:
  - frequency field width `6`
  - step field width `4`
  - state field width `2`
- Selection marker:
  - `>` is placed in front of the currently selected field
- Pending-change marker:
  - `*` in the last LCD character when any draft value on the current screen differs from the applied value

#### Compact Labels

- Frequency uses compact engineering-style formatting:
  - examples: `7.000M`, `14.00M`, `150.0M`
- Step uses compact labels:
  - `10H`, `100H`, `500H`, `1K`, `10K`, `100K`, `500K`, `1M`, `10M`
- State uses:
  - `ON`
  - `OF`

#### Button Timing

- Debounce: `20 ms`
- Long press threshold: `600 ms`
- Double-short detection window: `400 ms`

### Code Changes

- Added library root: `src/lib.rs`
- Added support module tree: `src/support/mod.rs`
- Moved adapter:
  - from `src/bitbang_i2c_compat.rs`
  - to `src/support/bitbang_i2c_compat.rs`
- Added vendored SI5351 module:
  - `src/support/si5351.rs`
- Replaced the previous LCD demo application in `src/bin/lcd-hd44780-test1.rs`
- Updated manifest dependencies in `Cargo.toml`

### Verification

- `cargo check --bin lcd-hd44780-test1` succeeds.
- `cargo fmt` succeeds.

### Notes

- The vendored SI5351 module was modernized only enough to fit this project’s `embedded-hal 1.0` usage and the bit-banged I2C adapter path.
- The double-short behavior is implemented as:
  - first short press applies immediately
  - second short press inside the timeout both applies again and advances the selected control
- This interpretation matches the requested semantics without introducing a delayed single-press action.

---

## Debug Build Size Tuning

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Task

- Fix the default `cargo build` failure in debug mode.
- Tune build options to shrink the firmware image.

### Problem

- `cargo build` failed at link time with:
  - `section '.rodata' will not fit in region 'FLASH'`
  - overflow by `7356 bytes`

### Reasoning

- The project is an embedded `no_std` firmware image for a `64K` flash target.
- Default Rust `dev` profile settings are intentionally not size-oriented.
- On MCU targets, a size-optimized debug profile is usually the simplest fix because debug info itself does not consume target flash, while unoptimized code size does.

### Changes

- Added to `Cargo.toml`:
  - `[profile.dev]`
  - `opt-level = "z"`
  - `lto = true`
  - `codegen-units = 1`
  - `panic = "abort"`
- Added to `Cargo.toml`:
  - `[profile.release]`
  - `panic = "abort"`

### Verification

- `cargo build` now succeeds.
- Measured linked debug firmware:

```text
text    data    bss    dec    hex
12472      0      4  12476   30bc
```

- Size command used:
  - `arm-none-eabi-size target/thumbv7m-none-eabi/debug/deps/lcd_hd44780_test1-...`

---

## UI Button Semantics Update

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Task

- Change main-screen button behavior:
  - single short press jumps to next control
  - double short press captures the current control value

### Reasoning

- The previous implementation applied on the first short press and advanced on the second.
- The requested behavior is the reverse, so the button state machine now needs to defer "single press" finalization until the double-press window expires.

### Changes

- Added a `pending_single_short` flag in the main loop button logic.
- First short press:
  - advances `selected_control`
  - opens the double-press timing window
- Second short press inside the window:
  - applies the currently selected control value
  - clears the pending single-press state
- Long press:
  - still switches between `CLK0` and `CLK1`
  - now also clears any pending single-short state

### Verification

- Re-ran `cargo fmt`.
- Re-ran `cargo check --bin lcd-hd44780-test1`.

---

## Binary Rename

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Task

- Rename the main binary from the old demo-oriented name to a proper application name.

### Changes

- Renamed:
  - `src/bin/lcd-hd44780-test1.rs`
  - to `src/bin/si5351-gen.rs`
- Updated project-context files in `codex/` to reference the new binary name.

### Reasoning

- The firmware is no longer an LCD test or demo.
- `si5351-gen.rs` describes the actual purpose and is the right name for future `cargo check --bin ...` and `cargo build --bin ...` usage.

### Verification

- Re-ran `cargo fmt`.
- Re-ran `cargo check --bin si5351-gen`.

---

## Title Marker Visibility And OFF Label

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Problem

- The pending-change `*` was not visible enough when placed at the far right of the first line.
- After removing the second-line `*`, the state field had room for `OFF` instead of the abbreviated `OF`.

### Changes

- Moved the first-line pending marker to appear immediately after the channel label:
  - `CLK0*`
  - `CLK1*`
- Expanded the disabled state label on line 2 from `OF` to `OFF`.
- Kept the enabled label as `ON ` so the state field still occupies a fixed width of 3 characters.

### Verification

- Re-ran `cargo fmt`.
- Re-ran `cargo check --bin lcd-hd44780-test1`.

---

## Double Click Reliability Fix

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Problem

- Double-clicks were sometimes behaving like two single clicks.
- Root cause: the first short press advanced `selected_control` immediately, so the second short press was racing the timeout window and the navigation side effect had already happened.

### Fix

- Changed short-press handling to deferred resolution:
  - first short press starts a pending window only
  - if no second short press arrives before timeout, it resolves as a single press and moves to the next control
  - if a second short press arrives inside the window, it resolves as a double press and applies the current control value
- Long press now cancels any pending short-press sequence before switching screens.

### Related UI Fix

- Since the pending-change `*` now lives on the first line, draft edits and apply actions now also mark the title line dirty so the indicator refreshes immediately.

### Verification

- Re-ran `cargo fmt`.
- Re-ran `cargo check --bin lcd-hd44780-test1`.

---

## Pending Marker Position

Date: 2026-05-16
Directory: `/home/matsi/src/ai-tests/mcu/stm32-gen`

### Task

- Move the pending-change `*` marker from the second line to the first line.

### Changes

- Updated title-line rendering so `*` appears at the last character of line 1 when the current channel has unapplied draft changes.
- Removed the same marker from the second line rendering.

### Verification

- Re-ran `cargo fmt`.
- Re-ran `cargo check --bin lcd-hd44780-test1`.
