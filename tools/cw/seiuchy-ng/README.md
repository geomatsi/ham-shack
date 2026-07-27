# Seiuchy NG

A Morse **head-copy trainer**. A fictional operator sends you one thing worth
logging — a name, a callsign, a QTH, a report, a contest exchange — wrapped in
the filler that real overs are made of. You type the part that belongs in the
log, and the trainer tells you whether you got it.

Inspired by *Seiuchy*, the CW trainer HB9FXW ran until his site went off the
air. This is an independent implementation: none of his code or material is in
here.

## Running it

Open `index.html`. That is the whole procedure — no server, no build step, no
network access, no dependencies. It runs equally well from a local file, a USB
stick or a webserver.

On a desktop browser that is the end of it. Android needs a little more care;
see below.

## Putting it on an Android device

Two things about Android shape all of this, and both bite silently:

* **A file manager hands the browser a `content://` URI**, which addresses one
  document and has no sibling directory. A page that pulls in `seiuchy.css` and
  `data.js` gets neither — you see unstyled text and nothing works.
* **Service workers only run in a "secure context"**, which means `https://` or
  `http://localhost`. A LAN address such as `http://192.168.1.5:8000` is *not*
  one. Chrome will happily add a shortcut and register nothing, so it looks
  installed right up until the machine serving it goes away.

So pick one of these.

### 1. One file, copied across (simplest, always works)

Build a standalone copy with everything inlined:

```sh
node tools/make-single-file.js          # writes seiuchy.html, ~134 KB
```

Copy that single file to the phone and open it from the file manager. Because
there is nothing beside it to find, the `content://` problem cannot arise: the
CSS, all five scripts and the icon travel inside the file. Sidetone, scoring and
saved settings all work.

What you give up is the home-screen icon and the standalone window — you reopen
it through the file manager, or bookmark it. Rebuild the file after editing
`data.js`.

### 2. Installed as an app, with an icon (needs a secure context)

Serve the folder and install from the browser. The server only has to exist
once: after the service worker has cached everything, the app runs with no
network and no server at all. Either of these gives you the secure context that
requires:

* **Termux, on the phone itself.** `pkg install python`, `cd` to the folder,
  `python3 -m http.server 8000`, then open `http://localhost:8000/` in Chrome
  and choose *Install app* from the ⋮ menu. `localhost` counts as secure, so the
  worker registers. Afterwards you can stop Termux entirely.
* **Any https host** — GitHub Pages, Netlify, your own web space. Open the site,
  *Install app*, done. This is also the easiest way to share it with someone
  else.

**Check it before you rely on it**: turn on flight mode and launch the installed
icon. If it opens, the cache is real. If you get an error page, the worker never
registered — you were almost certainly on `http://` with an IP address rather
than `localhost`.

To update an installed copy, bump `CACHE` in `sw.js` before republishing;
otherwise the phone keeps serving what it already has.

### 3. Wrapped in an APK

For a real installable package, point a WebView wrapper at the folder:
[Bubblewrap](https://github.com/GoogleChromeLabs/bubblewrap) turns an installed
web app into a Trusted Web Activity, and Cordova or Capacitor will bundle the
files straight into the APK's assets. Nothing in the app needs changing — no
build step, no absolute paths, no external requests, and
`manifest.webmanifest` already declares the name, icons, colours and
`display: standalone` a wrapper reads.

It is the most work for the least gain, though: option 1 needs no tooling and
option 2 already gives you an icon and an offline app.

### Sizing note

The app is about 164 KB across its files, or 134 KB as a single file, most of it
`data.js`. It is not worth minifying — keeping the data readable so it can be
edited on the device is the point.

## Files

| file | what it is |
| --- | --- |
| `index.html` | the page |
| `seiuchy.css` | styling |
| `data.js` | **all editable content** — word lists and phrase templates |
| `morse.js` | Morse table, keying models, Web Audio playback |
| `random.js` | the random source: platform CSPRNG, unbiased ranges |
| `qso.js` | the fictional operators: builds exchanges and marks answers |
| `app.js` | wiring: controls, scoring, log, persistence |
| `icon.svg`, `icon-*.png` | app icon (the letters CW in Morse) |
| `sw.js`, `manifest.webmanifest` | offline install when served over http(s) |
| `tools/` | development scripts — tests, the single-file builder, generators |
| `CLAUDE.md`, `.claude/` | notes and a `/verify` skill for Claude Code sessions |

## Development

There is nothing to install. The checks live in `tools/` and need only node,
plus chromium for the browser pass:

```sh
node tools/smoke-test.js && node tools/random-test.js && node tools/browser-test.js
```

See `tools/README.md`. The development files — `tools/`, `CLAUDE.md` and
`.claude/` — are not part of the app and can be deleted or left behind when
deploying it without touching anything that runs.

## What it sends

Nine categories, any combination of them:

**name** · **call** · **qth** · **rst** · **age** · **rig** · **job** ·
**club nr** · **contest** (serial, grid, VHF serial + grid, Field Day, full
locator)

The material behind them: 79 countries with real cities and correct callsign
prefixes, ~100 transceivers from an FT-101 to a QMX, and a made-up-place-name
generator per language region for when you want towns you cannot possibly guess.

## Editing the material

Everything you are likely to want to change lives in `data.js`. It is plain
JavaScript data — lists of strings between `[` and `]`, one comma between
entries — so any text editor will do. Reload the page to hear the result.

* **New name, rig or job**: add a string to `names`, `rigs` or `jobs`.
* **Make something come up more often**: list it twice. That is roughly, not
  exactly, double — see *Randomness* below.
* **New phrasing**: add a line to the matching list in `phrases`, keeping the
  `{placeholder}` — that is the part you are expected to copy. Placeholders that
  are only padding (`{pwr}`, `{ant}`, `{om}`) can be added or dropped freely.
* **Your own country**: add an entry to `countries` with a `weight` (how often it
  turns up), the callsign `prefixes`, the `cities`, and which invented-place
  `style` to use. Add `digits` if the real allocation is narrow — `"39"` for
  Switzerland, `"12345678"` for Australia.
* **Keying feel**: the `keys` entries are `[base, spread]` multipliers on the
  textbook duration of each element and gap; `1` is perfect timing. The factor
  used is `base + random() * spread`, drawn fresh for every element.

Keep the text lowercase and to characters Morse can send: `a-z`, `0-9`, space,
and `. , ? = + /`. Anything else is silently skipped when sent.

If you edit files while a copy is installed as an app, bump `CACHE` in `sw.js`,
otherwise the old cached version keeps being served.

## Randomness

Two things could make a trainer feel repetitive, and both are dealt with.

*Between sessions*: every draw comes from `crypto.getRandomValues` (via
`random.js`), seeded by the operating system, so a reload or a new day starts
somewhere genuinely unrelated. `Math.random` is only a fallback for an
environment that lacks Web Crypto. Integer ranges use rejection sampling rather
than a modulo, which would quietly favour the low values — with nine categories,
plain `% 9` would send you the first few noticeably more often.

*Within a session*: independent draws clump. Left alone they hand you three
names in a row, or the same town twice in ten overs, which reads as a broken
shuffle even though it is correct. So the category never repeats twice running
while another one is available, and every list refuses to hand back anything in
its last few draws — an eighth of the list, at most four. The cost is that
weighting by duplication is slightly compressed: an entry listed twice comes up
about 1.8 times as often rather than exactly 2. A wider window would clump less
and distort more; four is where that trade sits.

## Marking

Your answer is accepted if it *contains* the expected text, so filler you typed
by accident is forgiven. On top of that:

* a leading `nr` is stripped — "nr york" passes for "york";
* `5nn` counts as `599`;
* rigs ignore the punctuation around the model number — `ic7300`, `ic-7300` and
  `ic 7300` are the same answer;
* contest exchanges may keep or drop the leading `599`;
* for a club number, the number is what matters; if you name the club too, it has
  to be the right one.

## Licence

MIT — see `LICENSE`. Fork it, rehost it, bundle it into an app, teach with it,
sell it if you can find a buyer. Keep the copyright notice and that is the whole
obligation.

The choice is deliberate. This project exists because a good trainer disappeared
with its author's website and nobody was free to revive it. MIT is the strongest
guarantee that this one cannot die the same way: if this repository goes quiet,
anyone can pick it up without having to track anyone down.

## Credits

The idea — drilling head copy against randomly generated overs where the
filler matters as much as the answer — is **HB9FXW's**, from *Seiuchy*, which
ran at seiuchy.macache.com for years and taught a lot of operators to stop
writing everything down. Full credit to him for it.

Everything in this directory was written from scratch: no file, word list,
phrase, city list, keying table, image or line of code was copied from his app.
That was deliberate. Seiuchy carried a copyright notice and no licence, so it
was never anyone's to redistribute, and a clean reimplementation is the honest
way to keep a good idea alive. If you extend this, please keep it that way.

Much of the implementation was written with Claude (Anthropic), directed and
reviewed by the author.
