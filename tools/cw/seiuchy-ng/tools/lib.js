/*
 * Shared plumbing for the test scripts.
 *
 * The app has no module system on purpose (it must load from file://), so the
 * source files declare globals.  Here we eval them in order and hoist what the
 * tests need onto globalThis, which is the same thing the browser does with the
 * <script> tags in index.html.
 */
'use strict';

const fs = require('fs');
const path = require('path');

const APP_DIR = path.resolve(__dirname, '..');

function loadApp() {
  if (!globalThis.window) globalThis.window = {};
  const read = (file) => fs.readFileSync(path.join(APP_DIR, file), 'utf8');
  // Same order as index.html; each file's top-level const is eval-scoped, so it
  // has to be published explicitly.
  eval(read('data.js') + ';globalThis.SEIUCHY_DATA = SEIUCHY_DATA;');
  eval(read('random.js') + ';globalThis.Random = Random;');
  eval(read('morse.js') + ';globalThis.MORSE_TABLE = MORSE_TABLE;' +
       'globalThis.MorsePlayer = MorsePlayer;');
  eval(read('qso.js') + ';globalThis.QSO = QSO;');
  return { D: globalThis.SEIUCHY_DATA, QSO: globalThis.QSO, Random: globalThis.Random,
           MORSE_TABLE: globalThis.MORSE_TABLE };
}

/* Minimal check helper: counts failures instead of throwing, so one broken
   expectation does not hide the rest of the run. */
function checker() {
  let failures = 0;
  return {
    ok(condition, label, detail) {
      if (condition) {
        console.log('  ok    ' + label);
      } else {
        failures++;
        console.log('  FAIL  ' + label + (detail === undefined ? '' : '  ' + detail));
      }
    },
    note(label) { console.log('        ' + label); },
    get failures() { return failures; },
    finish(name) {
      console.log(failures ? `\n${name}: ${failures} FAILED\n` : `\n${name}: all good\n`);
      process.exit(failures ? 1 : 0);
    }
  };
}

const SETTINGS = {
  realQth: false, myName: 'om', contestType: 'serial', myGrid: '', localOnly: false
};

module.exports = { APP_DIR, loadApp, checker, SETTINGS };
