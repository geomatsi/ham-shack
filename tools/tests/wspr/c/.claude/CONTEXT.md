# Project context

A small WSPR signal-processing library in C, used as a scratchpad for
WSPR-related experiments. Lives under `ham-shack/tools/tests/wspr/c/`, which is part of a
broader ham-radio tooling repository.

## Scope

- Pure C11 library — no platform-specific code so far.
- Intentionally minimal: this is a test harness skeleton, not a production lib.
- Functions added incrementally; each gets its own Unity test runner.

## Layout

- `wspr.h` / `wspr.c` — library sources.
- `main.c` — minimal driver, currently just a smoke-test entry point.
- `tests/test_<fn>.c` — one Unity test runner per function under test.
- `CMakeLists.txt` — single top-level config; tests are wired in here.

## Tooling

- Build: CMake (>= 3.14).
- Test framework: **Unity** (ThrowTheSwitch), pinned to **v2.6.0**, pulled in via
  CMake `FetchContent` — no system install or submodule. First configure needs
  network access; subsequent builds are offline.
- Test driver: `ctest` via `enable_testing()`; one `add_test` per runner.
- Compiler flags: `-Wall -Wextra -Wpedantic`, C11, `CMAKE_C_EXTENSIONS OFF`.

Unity was chosen specifically because it has good float-tolerance assertions
(`TEST_ASSERT_FLOAT_WITHIN`, `TEST_ASSERT_EQUAL_FLOAT_ARRAY`) — relevant once
real DSP code starts comparing fp results.

## Conventions

- Indentation: **tabs** (Linux-kernel style braces/spacing).
- Each library function gets its own test file: `tests/test_<name>.c`.
- Each test file is its own runner with `int main(void)` + `UNITY_BEGIN/END`.
- Validate inputs at API boundaries (NULL pointer, negative length) and return
  a non-zero error code rather than asserting.

## Build & test

```sh
cmake -S . -B build
cmake --build build -j
ctest --test-dir build --output-on-failure
```

`build/` is gitignored.

## User

- Sergey (geomatsi@gmail.com) — ham radio operator, comfortable with C and
  embedded work. Prefers terse responses and explicit questions when a spec is
  ambiguous (e.g. confirm `uint64_t a[]` vs `uint64_t *a[]` before coding).
