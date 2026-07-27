#!/usr/bin/env node
/*
 * The random source.
 *
 * Checks the three things that would make the trainer feel repetitive or
 * lopsided: biased integer ranges, clumping within a session, and a broken
 * fallback when Web Crypto is missing.
 *
 *   node tools/random-test.js            run the checks
 *   node tools/random-test.js --sweep    re-derive the anti-repeat window size
 */
'use strict';

const vm = require('vm');
const fs = require('fs');
const path = require('path');
const { APP_DIR, loadApp, checker } = require('./lib');

/* --------------------------------------------------------------------- *
 *  --sweep: what does the anti-repeat window cost in weighting?
 * --------------------------------------------------------------------- */

if (process.argv.includes('--sweep')) {
  const { Random } = loadApp();
  // A list shaped like a country's city list, with one entry listed twice.
  const list = ['dup', 'dup', ...Array.from({ length: 34 }, (_, i) => 'x' + i)];
  console.log('\n36 entries, 35 distinct, one of them listed twice\n');
  console.log('  window   duplicate/average ratio   smallest gap between repeats');
  for (const limit of [0, 1, 2, 3, 4, 6, 8]) {
    const recent = [], tally = {}, lastSeen = {};
    let minGap = Infinity;
    for (let i = 0; i < 400000; i++) {
      let v = list[Random.below(list.length)];
      for (let n = 0; limit && n < 12 && recent.includes(v); n++) {
        v = list[Random.below(list.length)];
      }
      if (limit) { recent.push(v); if (recent.length > limit) recent.shift(); }
      tally[v] = (tally[v] || 0) + 1;
      if (lastSeen[v] !== undefined) minGap = Math.min(minGap, i - lastSeen[v]);
      lastSeen[v] = i;
    }
    const others = Object.keys(tally).filter((k) => k !== 'dup').map((k) => tally[k]);
    const average = others.reduce((a, b) => a + b, 0) / others.length;
    console.log(`  ${String(limit).padStart(5)}   ${(tally.dup / average).toFixed(3).padStart(21)}` +
                `   ${String(minGap).padStart(28)}`);
  }
  console.log('\nqso.js uses min(4, distinct / 8): most of the weighting, no repeat' +
              ' inside five draws.\n');
  process.exit(0);
}

/* --------------------------------------------------------------------- *
 *  The checks
 * --------------------------------------------------------------------- */

const { D, QSO, Random } = loadApp();
const t = checker();

console.log('\n  source:');
t.ok(Random.usingCrypto === true, 'draws come from the platform CSPRNG');

/* Uniformity.  The bound is a generous approximation of the 99.9th percentile
   of chi-square with n-1 degrees of freedom, so a healthy generator does not
   trip it, but a modulo bias (which skews the low values hard) does. */
console.log('\n  uniformity of below(n):');
for (const n of [2, 3, 7, 9, 79, 255, 1000]) {
  const rounds = 200000;
  const counts = new Array(n).fill(0);
  let outOfRange = false;
  for (let i = 0; i < rounds; i++) {
    const v = Random.below(n);
    if (!Number.isInteger(v) || v < 0 || v >= n) { outOfRange = true; break; }
    counts[v]++;
  }
  const expected = rounds / n;
  const chi2 = counts.reduce((a, c) => a + (c - expected) ** 2 / expected, 0);
  const bound = (n - 1) + 3.3 * Math.sqrt(2 * (n - 1)) + 5;
  t.ok(!outOfRange && chi2 < bound,
       `below(${n})`.padEnd(12) + `chi2 ${chi2.toFixed(1)} < ${bound.toFixed(1)}`,
       outOfRange ? 'VALUE OUT OF RANGE' : '');
}

let lowest = 1, highest = 0;
for (let i = 0; i < 200000; i++) {
  const f = Random.fraction();
  if (f < lowest) lowest = f;
  if (f > highest) highest = f;
}
t.ok(lowest >= 0 && highest < 1, `fraction() stays in [0, 1)`,
     `saw ${lowest.toFixed(6)} .. ${highest.toFixed(6)}`);

/* Anti-repeat behaviour. */
console.log('\n  clumping:');
const WINDOW = Math.min(4, Math.floor(new Set(D.names).size / 8));
const names = [];
for (let i = 0; i < 20000; i++) names.push(QSO.elements.name().answer);

let adjacent = 0;
for (let i = 1; i < names.length; i++) if (names[i] === names[i - 1]) adjacent++;
t.ok(adjacent === 0, 'a name never follows itself', adjacent + ' cases');

let inWindow = 0;
for (let i = WINDOW; i < names.length; i++) {
  if (names.slice(i - WINDOW, i).includes(names[i])) inWindow++;
}
t.ok(inWindow === 0, `no name repeats inside the last ${WINDOW} draws`, inWindow + ' cases');

t.ok(new Set(names).size === new Set(D.names).size,
     'every name is still reachable',
     `${new Set(names).size} of ${new Set(D.names).size}`);

/* Weighting by duplication survives, approximately.  Berlin is listed twice in
   the German city list, so it should come up near twice as often as the rest. */
const tally = {};
const settings = { realQth: true, myName: 'om', contestType: 'serial',
                   myGrid: '', localOnly: false };
for (let i = 0; i < 200000; i++) {
  const town = QSO.elements.qth(settings, 'dl').answer;
  tally[town] = (tally[town] || 0) + 1;
}
// Compare inside one country, or the country weights would swamp the effect.
const germanSingles = new Set(D.countries.dl.cities.filter((c) => c !== 'berlin'));
const counts = [...germanSingles].map((c) => tally[c] || 0);
const ratio = tally.berlin / (counts.reduce((a, b) => a + b, 0) / counts.length);
t.ok(ratio > 1.6 && ratio < 2.05,
     'an entry listed twice comes up about twice as often',
     `ratio ${ratio.toFixed(2)} (anti-repeat compresses it a little; expected ~1.8)`);

/* Every phrasing family must be reachable: a generator whose parameters do not
   line up with how build() calls it can silently kill a whole set of templates. */
const jobText = new Set();
for (let i = 0; i < 8000; i++) jobText.add(QSO.elements.job(settings).sent);
const allJobs = [...jobText].join(' | ');
for (const [family, probe] of [['jobNow', 'im a '], ['jobPast', 'used to be'],
                               ['jobFuture', 'studying to be']]) {
  t.ok(allJobs.includes(probe), `${family} phrasings are reachable`);
}

/* Independence between runs: two fresh loads must not agree. */
console.log('\n  independence:');
function sequence() {
  const context = { console };
  vm.createContext(context);
  const read = (f) => fs.readFileSync(path.join(APP_DIR, f), 'utf8');
  vm.runInContext(read('random.js'), context);
  return vm.runInContext(
    'Array.from({length: 40}, () => Random.below(1000)).join(",")', context);
}
const runs = [sequence(), sequence(), sequence()];
t.ok(new Set(runs).size === 3, 'three fresh contexts give three different sequences');

/* The fallback, in a context with no Web Crypto at all. */
console.log('\n  fallback when Web Crypto is missing:');
const bare = { console };
vm.createContext(bare);
vm.runInContext(fs.readFileSync(path.join(APP_DIR, 'random.js'), 'utf8'), bare);
t.ok(vm.runInContext('Random.usingCrypto', bare) === false, 'falls back to Math.random');
const fallbackOk = vm.runInContext(`
  (function () {
    for (const n of [2, 9, 79]) {
      const counts = new Array(n).fill(0);
      for (let i = 0; i < 60000; i++) {
        const v = Random.below(n);
        if (!Number.isInteger(v) || v < 0 || v >= n) return false;
        counts[v]++;
      }
      const expected = 60000 / n;
      const chi2 = counts.reduce((a, c) => a + (c - expected) ** 2 / expected, 0);
      if (chi2 >= (n - 1) + 3.3 * Math.sqrt(2 * (n - 1)) + 5) return false;
    }
    return true;
  })()
`, bare);
t.ok(fallbackOk === true, 'the fallback still produces uniform values in range');

t.finish('random-test');
