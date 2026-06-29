# wspr-beacon (BluePill)

WSPR beacon firmware for STM32F103C8T6 (BluePill).

## Commands

```sh
# Build (debug)
cargo build --bin beacon

# Build (release)
cargo build --bin beacon --release

# Flash via probe-rs
probe-rs run --chip STM32F103C8Tx target/thumbv7m-none-eabi/release/beacon

# Clean
cargo clean
```

## Prerequisites

```sh
rustup target add thumbv7m-none-eabi
```
