# wspr-beacon (BluePill)

WSPR beacon firmware for STM32F103C8T6 (BluePill).

## Commands

```sh
# Build (debug)
cargo build --bin beacon

# Build (release)
cargo build --bin beacon --release

The project uses `.cargo/config.toml` with a `probe-rs run` target runner for
`thumbv7m-none-eabi`, so the default happy path is:

```bash
$ cargo run --bin <binary name>
```

Flash-only with probe-rs tools:

```bash
$ cargo flash --release --chip STM32F103C8 --bin <binary name>
```

# Clean
cargo clean
```

## Prerequisites

```sh
rustup target add thumbv7m-none-eabi
```
