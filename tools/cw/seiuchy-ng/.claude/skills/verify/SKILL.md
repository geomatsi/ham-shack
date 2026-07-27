---
name: verify
description: Run the Seiuchy NG checks - generator smoke test, randomness validation and a headless-browser pass through the real UI. Use after changing data.js, qso.js, morse.js, random.js, app.js or index.html, and before telling the user a change works.
---

# Verify Seiuchy NG

Three scripts, run from the app directory. Each exits non-zero on failure.

```sh
node tools/smoke-test.js && node tools/random-test.js && node tools/browser-test.js
```

Run all three even when a change looks confined to one file — the data and the
code constrain each other, and the failures worth catching are the ones that
cross that line (a template referring to a placeholder no generator fills, a
generator whose parameters no longer match how `build()` calls it, a country
naming a place style that does not exist).

## Reading the output

- **smoke-test** — every category generated thousands of times. Watch the
  distinct-answer counts: a category that collapses to a handful of answers means
  a list stopped being reached.
- **random-test** — chi-square bounds are generous, so a failure there is a real
  bias, not noise. The weighting check is expected to land near 1.8, not 2.0;
  that is the documented cost of the anti-repeat window.
- **browser-test** — skips with a notice if chromium is absent. That is a skip,
  **not a pass**: say so rather than reporting the UI as verified.

## When something fails

Decide which side is wrong before changing anything. Twice now the *test* has
held the mistaken expectation while the app was right — a correct answer after a
repeat scores nothing, and a locator normalises to six characters. Check the
intended behaviour in `README.md` and `CLAUDE.md` first, and only then fix
whichever of the two is actually wrong.

## Beyond the scripts

The scripts do not cover audio or layout. When a change touches either, also:

- `node tools/browser-test.js --shot /tmp/ui.png` and look at the image, at a
  narrow window size, checking that nothing overflows sideways;
- for timing or keying changes, reason about `morse.js` against the
  `[base, spread]` factors in `data.js` — nothing here can hear the sidetone, so
  do not claim it sounds right. Say what was and was not checked.

Report what actually ran. If a step was skipped, say which and why.
