# wspr-beacon (BluePill)

WSPR beacon firmware for STM32F103C8T6 (BluePill).

## Commands

```sh
# Build (debug)
cargo build --bin beacon

# Build (release)
cargo build --bin beacon --release
```

The project uses `.cargo/config.toml` with a `probe-rs run` target runner for
`thumbv7m-none-eabi`, so the default happy path is:

```sh
$ cargo run --bin <binary name>
```

Flash-only with probe-rs tools:

```sh
$ cargo flash --release --chip STM32F103C8 --bin <binary name>
```

Attach RTT log monitor to the running binary:

```sh
$ probe-rs attach --chip STM32F103C8 target/thumbv7m-none-eabi/debug/examples/led-test1
```

## Test (host)
`cargo test` cannot be used: it builds every declared dependency to compile
the lib, and rtic does not compile for a host target. Platform-agnostic
modules are tested one file at a time instead.

```sh
rustc --test --edition 2024 -o /tmp/qth src/beacon/qth.rs && /tmp/qth
```

## Clean

```
cargo clean
```

## Prerequisites

```sh
rustup target add thumbv7m-none-eabi
```
