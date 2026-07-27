#!/usr/bin/env node
/*
 * Generators and marking rules.
 *
 * Hammers every category, checks that what comes out is sendable, is not
 * obviously broken, and that the expected answer is actually accepted when
 * typed back verbatim -- the last one has caught more real bugs than the rest
 * put together, because it exercises generator and marker against each other.
 *
 *   node tools/smoke-test.js [rounds-per-category]
 */
'use strict';

const { loadApp, checker, SETTINGS } = require('./lib');
const { D, QSO, MORSE_TABLE } = loadApp();
const t = checker();

const ROUNDS = Number(process.argv[2]) || 4000;
const CONTESTS = ['serial', 'grid', 'vhf', 'fd', 'loc'];

console.log(`\nGenerating ${ROUNDS} exchanges per category\n`);

const answers = {};
const problems = { empty: [], junk: [], unsendable: [], selfCheck: [] };

for (const category of QSO.CATEGORIES) {
  answers[category] = new Set();
  for (let i = 0; i < ROUNDS; i++) {
    // Vary the settings that change what generators do.
    const settings = Object.assign({}, SETTINGS, {
      realQth: i % 3 === 0,
      contestType: CONTESTS[i % CONTESTS.length],
      myGrid: i % 7 === 0 ? 'JN47' : '',
      localOnly: i % 14 === 0
    });

    const q = QSO.build(category, settings);

    if (!q.sent || !q.answer) problems.empty.push(category);
    if (/undefined|NaN|[{}]/.test(q.sent + q.answer)) problems.junk.push(category + ': ' + q.sent);
    for (const ch of q.sent.toLowerCase()) {
      if (ch !== ' ' && !MORSE_TABLE[ch]) {
        problems.unsendable.push(`${JSON.stringify(ch)} in ${category}: ${q.sent}`);
      }
    }
    if (!QSO.check(category, q.answer, q.answer)) {
      problems.selfCheck.push(category + ': ' + JSON.stringify(q.answer));
    }
    answers[category].add(q.answer);
  }
}

t.ok(!problems.empty.length, 'every exchange has text and an answer', problems.empty[0]);
t.ok(!problems.junk.length, 'no unsubstituted placeholders or undefined/NaN', problems.junk[0]);
t.ok(!problems.unsendable.length, 'every character is in the Morse table', problems.unsendable[0]);
t.ok(!problems.selfCheck.length, 'the expected answer is always accepted', problems.selfCheck[0]);

console.log('\n  distinct answers seen per category:');
for (const [category, set] of Object.entries(answers)) {
  console.log(`        ${category.padEnd(9)} ${set.size}`);
}
console.log();

/* --- marking rules ------------------------------------------------------- */

const MARKING = [
  ['age',     '43',              '43',              true,  'exact'],
  ['age',     'nr 43',           '43',              true,  'leading "nr" ignored'],
  ['age',     '44',              '43',              false, 'wrong number rejected'],
  ['name',    '',                'leo',             false, 'empty answer is wrong'],
  ['rst',     '5nn',             '599',             true,  'cut numbers in a report'],
  ['rst',     '599',             '599',             true,  'plain report'],
  ['rig',     'ic7000',          'ic 7000',         true,  'rig without the space'],
  ['rig',     'ic-7000',         'ic 7000',         true,  'rig with a hyphen'],
  ['rig',     'ic 7300',         'ic 7000',         false, 'wrong model rejected'],
  ['club',    'skcc 1234',       'skcc 1234',       true,  'club and number'],
  ['club',    '1234',            'skcc 1234',       true,  'number alone is enough'],
  ['club',    'fists 1234',      'skcc 1234',       false, 'wrong club rejected'],
  ['club',    'cw 88',           'cwops 88',        true,  'club abbreviated'],
  ['club',    'qrp arci 51929',  'qrp arci 51929',  true,  'two-word club'],
  ['contest', '599 049',         '049',             true,  'exchange with the report'],
  ['contest', '049',             '049',             true,  'exchange without it'],
  ['qth',     'berlin',          'berlin',          true,  'town'],
  ['call',    'dl3abc',          'dl3abc',          true,  'callsign'],
  ['call',    'de dl3abc k',     'dl3abc',          true,  'callsign with prosigns typed'],
];

console.log('  marking:');
for (const [category, typed, expected, want, label] of MARKING) {
  const got = QSO.check(category, typed, expected);
  t.ok(got === want, label, `(${category} "${typed}" vs "${expected}" -> ${got})`);
}

/* --- locators ------------------------------------------------------------ */

console.log('\n  locators:');
for (const [input, want] of [['jn47', 'jn47ll'], ['JN', 'jn55ll'],
                             ['fn31pr', 'fn31pr'], ['zz', null], ['', null]]) {
  const got = QSO.normaliseGrid(input);
  t.ok(got === want, `normalise ${JSON.stringify(input)} -> ${want}`, `got ${got}`);
}

// A locator generated near a home square should actually be near it.
const near = new Set();
for (let i = 0; i < 500; i++) {
  near.add(QSO.randomGrid(4, 'vhf', Object.assign({}, SETTINGS,
    { myGrid: 'JN47', localOnly: true })).slice(0, 2));
}
t.ok(near.size <= 6 && near.has('jn'),
     'local-only VHF grids stay in the neighbourhood', [...near].join(','));

/* --- data sanity --------------------------------------------------------- */

console.log('\n  data:');
const countries = Object.keys(D.countries);
const missingStyle = countries.filter((c) => !D.placeStyles[D.countries[c].style]);
t.ok(!missingStyle.length, 'every country names a style that exists', missingStyle.join(','));

const unusedStyle = Object.keys(D.placeStyles)
  .filter((s) => !countries.some((c) => D.countries[c].style === s));
t.ok(!unusedStyle.length, 'every style is used by some country', unusedStyle.join(','));

const badCountry = countries.filter((c) => {
  const k = D.countries[c];
  return !(k.weight > 0) || !k.prefixes.length || !k.cities.length ||
         (k.digits !== undefined && !/^\d+$/.test(k.digits));
});
t.ok(!badCountry.length, 'every country has a weight, prefixes, cities, sane digits',
     badCountry.join(','));

const badStyle = Object.entries(D.placeStyles).filter(([, s]) =>
  !(s.syllables ? s.syllables.length : (s.prefixes || []).length && (s.suffixes || []).length));
t.ok(!badStyle.length, 'every style can actually build a name',
     badStyle.map(([n]) => n).join(','));

t.note(`${countries.length} countries, ` +
       `${countries.reduce((a, c) => a + D.countries[c].cities.length, 0)} cities, ` +
       `${Object.keys(D.placeStyles).length} place styles, ` +
       `${Object.keys(D.keys).length} keying styles`);

const badKey = Object.entries(D.keys).filter(([, k]) =>
  ['dit', 'dah', 'gap', 'letter', 'word'].some((f) =>
    !Array.isArray(k[f]) || k[f].length !== 2 || !(k[f][0] > 0)));
t.ok(!badKey.length, 'every keying style has five [base, spread] pairs',
     badKey.map(([n]) => n).join(','));

t.finish('smoke-test');
