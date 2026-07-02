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

Attach RTT log monitor to the running binary:

```bash
$ probe-rs attach --chip STM32F103C8 target/thumbv7m-none-eabi/debug/examples/led-test1
```

# Clean
cargo clean
```

## Prerequisites

```sh
rustup target add thumbv7m-none-eabi
```
