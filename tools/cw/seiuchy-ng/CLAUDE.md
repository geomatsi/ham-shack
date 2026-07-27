# Seiuchy NG — working notes

A Morse head-copy trainer. A simulated operator sends one loggable item (name,
callsign, QTH, RST, age, rig, job, club number, contest exchange) padded with
on-air filler; the user types the part that belongs in the log.

## Hard constraints

These are the point of the project, not preferences. Breaking one breaks the app
for its actual use case — a phone in a field with no signal.

- **No dependencies, no build step, no package manager.** Plain files a browser
  opens directly.
- **Must work from `file://`.** This rules out ES modules (`import`/`export`),
  `fetch()` of local files, and anything else the file scheme blocks. Scripts are
  classic `<script src>` tags with globals, loaded in dependency order:
  `data.js → random.js → morse.js → qso.js → app.js`.
- **No network access, ever.** No CDNs, fonts, analytics or telemetry. The
  original app pinged a PHP endpoint on every exchange; that is exactly what this
  one must not do.
- **Everything the user might want to edit lives in `data.js`** as plain data.
  Content does not belong in code.
- **Morse-safe text.** Sendable characters are `a-z`, `0-9`, space and
  `. , ? = + /` (see `MORSE_TABLE` in `morse.js`). Anything else is silently
  dropped when sent, so keep data lowercase and accent-free — `zurich`, not
  `Zürich`.
- **Mobile first.** Touch targets ≥ 2.75rem, base font 16px (smaller makes iOS
  zoom on focus), layout must not scroll horizontally at 360px.

## Provenance — read before importing anything

This app owes its idea to *Seiuchy*, the CW trainer HB9FXW ran at
seiuchy.macache.com until it went off the air. That app carried a copyright
notice and **no licence**, which means all rights reserved — its disappearance
does not change that.

This is a clean-room reimplementation: same concept, none of his code, data,
wording or artwork. Every word list, phrase template, city list, keying model
and pixel here was written for this project. It was done that way deliberately,
so the result can be published without asking anyone's permission.

**Keep it that way.** Do not import material from the original app — not word
lists, not phrasings, not its walrus logo, not instruction text — even if a copy
turns up in the Wayback Machine or someone pastes one in. Studying what a
feature *did* is fine; reproducing how it was *written* is not. If asked to
bring something across, explain the problem before acting.

## Files

| file | role |
| --- | --- |
| `index.html` | markup; script order matters |
| `seiuchy.css` | styling |
| `data.js` | all content: word lists, phrase templates, countries, keying styles |
| `random.js` | CSPRNG-backed draws, unbiased integer ranges |
| `morse.js` | Morse table, keying model, Web Audio playback |
| `qso.js` | exchange generation and answer marking; no DOM access |
| `app.js` | controls, scoring, log, localStorage |
| `sw.js` | offline cache; **bump `CACHE` when files change** |
| `tools/` | tests, the single-file builder, generators (see `tools/README.md`) |

`qso.js` deliberately touches no DOM so it can be exercised from node.

## Testing

```sh
node tools/smoke-test.js      # generators + marking rules
node tools/random-test.js     # distribution, bias, anti-repeat behaviour
node tools/browser-test.js    # drives the real UI in headless chromium
node tools/browser-test.js --bundle   # ...and the standalone single-file build
```

Two Android constraints are easy to break and invisible on the desktop: a page
opened from a file manager arrives as a `content://` URI and cannot load sibling
files (hence `tools/make-single-file.js`), and service workers need a secure
context, so `http://` on a LAN address never caches anything.

Run all three after touching `data.js`, `qso.js`, `morse.js`, `random.js` or
`app.js`. The `/verify` skill runs the set and summarises.

## Conventions worth keeping

- Two-space indent, single quotes in JS, comments that explain *why*.
- Randomness goes through `Random.below(n)` / `Random.fraction()`, never
  `Math.random()` directly (`random.js` is the one exception, as fallback).
- `pick()` in `qso.js` carries per-list anti-repeat memory; the window size is a
  measured trade-off against weighting-by-duplication — see the *Randomness*
  section of `README.md` before changing it.
- Adding a category means: a generator in `qso.js`, an entry in `GENERATORS`, a
  checkbox in `index.html`, and templates in `data.js`.
