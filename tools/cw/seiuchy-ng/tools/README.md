# tools

Development scripts. **None of this ships with the app** — the trainer itself is
the files in the parent directory and has no dependencies at all. These need
node (any recent version), and `browser-test.js` additionally wants chromium on
PATH, which it skips politely without.

Run them from the app directory:

```sh
node tools/smoke-test.js      # generators and marking rules
node tools/random-test.js     # distribution, bias, clumping, fallback
node tools/browser-test.js    # the real UI in headless chromium
```

All three exit non-zero on failure, so `&&` them together in a hook or a CI job.

| script | what it does |
| --- | --- |
| `lib.js` | loads the app's globals into node the way `index.html` does |
| `smoke-test.js` | hammers every category; checks nothing unsendable or malformed comes out, that a verbatim answer is always accepted, and that the data is internally consistent |
| `random-test.js` | chi-square on `Random.below()`, anti-repeat behaviour, weighting, independence between runs, and the no-Web-Crypto fallback |
| `browser-test.js` | drives `index.html` from `file://`: start, right, wrong, repeat; checks score, autospeed, log colours, persistence |
| `make-single-file.js` | inlines everything into a standalone `seiuchy.html` for copying to a phone |
| `make-land-fields.js` | regenerates `contest.populatedFields` |

## Useful invocations

```sh
node tools/smoke-test.js 20000            # more rounds per category
node tools/random-test.js --sweep         # re-derive the anti-repeat window size
node tools/browser-test.js --shot ui.png  # also save a screenshot
node tools/make-land-fields.js --map      # draw the land mask as ASCII
node tools/make-single-file.js           # build the standalone seiuchy.html
node tools/browser-test.js --bundle      # drive that build, alone in a temp dir
```

`--sweep` prints the trade-off table behind the window size chosen in `qso.js`:
a wider window clumps less but flattens weighting-by-duplication. Consult it
before changing that constant.

`make-single-file.js` exists because Android file managers hand the browser a
`content://` URI, under which a page cannot reach its sibling files. The build
is an artefact for carrying around, not part of the app, and is not committed —
`browser-test.js --bundle` generates it into an empty directory and drives it
there, so a reference that escapes the file fails the way it would on a phone.

`make-land-fields.js` prints a block to paste over `populatedFields` in
`data.js`. Edit the `LAND` boxes at the top, check the shape with `--map`, then
regenerate. Running it today reproduces the shipped list exactly.

## Adding a test

Both node scripts use the tiny helper in `lib.js`:

```js
const { loadApp, checker, SETTINGS } = require('./lib');
const { D, QSO } = loadApp();
const t = checker();

t.ok(condition, 'what should be true', 'detail shown on failure');
t.finish('my-test');       // prints a summary and sets the exit code
```

`checker()` counts failures rather than throwing, so one bad expectation does
not hide the rest of the run.
