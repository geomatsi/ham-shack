#!/usr/bin/env node
/*
 * The real UI, in a real browser.
 *
 * Loads index.html from file:// in headless chromium -- which is how the app is
 * actually used -- and clicks through it: start, answer right, answer wrong,
 * ask for a repeat.  Then checks the score, the automatic speed adjustment, the
 * colour of each log row and that settings survive.
 *
 * Needs chromium (or chrome) on PATH.  Skips with a notice if there is none, so
 * it can sit in a check script without becoming a nuisance.
 *
 *   node tools/browser-test.js                  run against index.html
 *   node tools/browser-test.js --bundle         run against the standalone
 *                                               build, alone in a temp dir
 *   node tools/browser-test.js --shot out.png   also save a screenshot
 */
'use strict';

const fs = require('fs');
const os = require('os');
const path = require('path');
const { execFileSync } = require('child_process');
const { APP_DIR, checker } = require('./lib');

const CANDIDATES = ['chromium', 'chromium-browser', 'google-chrome',
                    'google-chrome-stable', 'chrome'];

function findBrowser() {
  for (const name of CANDIDATES) {
    try {
      execFileSync('which', [name], { stdio: 'pipe' });
      return name;
    } catch (e) { /* keep looking */ }
  }
  return null;
}

const browser = findBrowser();
if (!browser) {
  console.log('\nbrowser-test: no chromium on PATH, skipping\n');
  process.exit(0);
}

/* The driver runs inside the page after everything has loaded.  It wraps
   QSO.build so it can know the expected answer and type it in, which is the
   only way to test the marking through the UI rather than around it. */
const DRIVER = `
<script>
window.addEventListener('load', function () {
  const result = {};
  try {
    const play = document.getElementById('play');
    const answer = document.getElementById('itext');
    const categories = [];
    const build = QSO.build;
    let expected = null;
    QSO.build = function (category, settings) {
      categories.push(category);
      const exchange = build(category, settings);
      expected = exchange.answer;
      return exchange;
    };

    document.querySelectorAll('.cat').forEach((c) => { c.checked = true; });
    document.getElementById('autospeed').checked = true;
    const startingSpeed = Number(document.getElementById('speed').value);

    play.click();                                   // Start
    for (let i = 0; i < 6; i++) {                   // 4 right, 2 wrong
      answer.value = (i % 3 === 2) ? 'definitely wrong' : expected;
      play.click();
    }
    document.getElementById('again').click();       // repeat, then answer right
    answer.value = expected;
    play.click();

    result.usingCrypto = Random.usingCrypto;
    result.keyOptions = document.getElementById('key').options.length;
    result.score = document.getElementById('score').textContent;
    result.speed = Number(document.getElementById('speed').value);
    result.startingSpeed = startingSpeed;
    result.rows = document.querySelectorAll('#logcontainer tbody tr').length;
    result.verdicts = [].map.call(
      document.querySelectorAll('#logcontainer td:nth-child(2)'), (c) => c.className);
    result.backToBack = categories.filter((c, i) => i && c === categories[i - 1]).length;
    result.categories = categories;
    result.stored = !!localStorage.getItem('seiuchy-ng');
    result.buttonLabel = play.textContent;
    // Catches a bundle that lost its CSS or icon: both would still "work".
    result.pageBg = getComputedStyle(document.getElementById('page')).backgroundColor;
    result.styled = result.pageBg === 'rgb(255, 255, 224)';
    const logo = document.getElementById('logo');
    result.logoLoaded = logo.complete && logo.naturalWidth > 0;

    // The controls must not throw either.
    document.getElementById('test').click();
    document.getElementById('stop').click();
    document.getElementById('uncheck-all').click();
    play.click();                                   // no category ticked
    result.rowsAfterUncheck = document.querySelectorAll('#logcontainer tbody tr').length;
    document.getElementById('reset-score').click();
    result.scoreAfterReset = document.getElementById('score').textContent;
    document.getElementById('legend-settings').click();
    result.folded = document.getElementById('settings-body').hasAttribute('hidden');
    document.getElementById('legend-settings').click();
    document.getElementById('check-all').click();
    document.getElementById('instructions').open = true;
  } catch (error) {
    result.error = error.message + ' @ ' + (error.stack || '').split('\\n')[1];
  }
  const box = document.createElement('pre');
  box.id = 'result';
  box.textContent = 'RESULT' + JSON.stringify(result) + 'ENDRESULT';
  box.style.cssText = 'background:#000;color:#0f0;padding:.4rem;white-space:pre-wrap;font-size:10px';
  document.getElementById('page').prepend(box);
});
</script>
`;

/* --bundle proves the standalone build really is standalone: it is generated
   into an empty temp directory, so any leftover reference to a sibling file
   fails the way it would on a phone. */
const bundleMode = process.argv.includes('--bundle');
let sourceHtml, pagePath, cleanUp;

if (bundleMode) {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'seiuchy-bundle-'));
  const bundle = path.join(temp, 'seiuchy.html');
  execFileSync(process.execPath, [path.join(__dirname, 'make-single-file.js'), bundle],
               { stdio: 'pipe' });
  sourceHtml = fs.readFileSync(bundle, 'utf8');
  pagePath = path.join(temp, '_driven.html');
  cleanUp = () => fs.rmSync(temp, { recursive: true, force: true });
} else {
  sourceHtml = fs.readFileSync(path.join(APP_DIR, 'index.html'), 'utf8');
  pagePath = path.join(APP_DIR, '_browser-test.html');
  cleanUp = () => fs.unlinkSync(pagePath);
}

fs.writeFileSync(pagePath, sourceHtml.replace('</body>', DRIVER + '</body>'));

function run(extraArgs) {
  return execFileSync(browser, [
    '--headless', '--disable-gpu', '--no-sandbox',
    '--autoplay-policy=no-user-gesture-required',
    '--virtual-time-budget=5000', ...extraArgs, pagePath
  ], { encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'], maxBuffer: 64 * 1024 * 1024 });
}

const t = checker();
try {
  console.log(`\n  driving ${browser} against ` +
              (bundleMode ? 'the standalone build, alone in a temp directory'
                          : 'file://' + path.join(APP_DIR, 'index.html')) + '\n');

  const dom = run(['--dump-dom']);
  const match = /RESULT(\{.*?\})ENDRESULT/s.exec(dom);
  if (!match) {
    console.log('  FAIL  the page did not report a result (did it fail to load?)');
    process.exit(1);
  }
  const r = JSON.parse(match[1]);

  t.ok(!r.error, 'no exception while driving the page', r.error);
  t.ok(r.usingCrypto === true, 'the page draws from the CSPRNG over file://');
  t.ok(r.keyOptions === Object.keys(require('./lib').loadApp().D.keys).length + 1,
       'the key list is built from the data plus "random"', r.keyOptions);
  t.ok(r.buttonLabel === 'Answer', 'the button becomes "Answer" after the first exchange');
  // 4 clean hits, 2 misses, 1 hit after a repeat: seven attempts, four scored.
  t.ok(r.score === '4/7', 'a hit after a repeat counts as an attempt but does not score',
       r.score);
  t.ok(r.rows === 7, 'every marked answer lands in the log', r.rows);
  t.ok(String(r.verdicts) === 'repeat,wrong,correct,correct,wrong,correct,correct',
       'log rows are coloured right, newest first', String(r.verdicts));
  // 4 clean hits (+4), 2 misses (-2), 1 hit after a repeat (no change).
  t.ok(r.speed === r.startingSpeed + 2, 'automatic speed adjustment follows the answers',
       `${r.startingSpeed} -> ${r.speed}`);
  t.ok(r.backToBack === 0, 'no category is sent twice in a row', r.categories.join(','));
  t.ok(r.stored === true, 'settings are written to localStorage');
  t.ok(r.rowsAfterUncheck === 8, 'unticking every category still produces an exchange');
  t.ok(r.scoreAfterReset === '0/0', 'reset clears the score');
  t.ok(r.folded === true, 'tapping the legend folds the settings away');
  t.ok(r.styled === true, 'the stylesheet is in effect', r.pageBg);
  t.ok(r.logoLoaded === true, 'the masthead icon resolves');

  const shotIndex = process.argv.indexOf('--shot');
  if (shotIndex !== -1 && process.argv[shotIndex + 1]) {
    const out = path.resolve(process.argv[shotIndex + 1]);
    run(['--window-size=412,2600', '--screenshot=' + out]);
    t.note('screenshot: ' + out);
  }
} finally {
  cleanUp();
}

t.finish('browser-test');
