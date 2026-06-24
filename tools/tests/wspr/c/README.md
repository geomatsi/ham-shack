# wspr

A simple WSPR signal-processing library in C with a Unity-based test harness.

## Layout

- `wspr.h` / `wspr.c` — library sources
- `main.c` — minimal executable linking the library
- `tests/` — Unity test files (one per function)

## Requirements

- CMake >= 3.14
- A C11 compiler (gcc, clang)
- Network access on first configure (Unity is pulled in via `FetchContent`)

## Build

```sh
cmake -S . -B build
cmake --build build -j
```

This produces:

- `build/libwspr.a` — static library
- `build/wspr_app` — sample executable
- `build/test_f1`, `build/test_f2` — test runners

## Run tests

```sh
ctest --test-dir build --output-on-failure
```

Or invoke a single runner directly:

```sh
./build/test_f1
./build/test_f2
```

## Clean

```sh
rm -rf build
```
