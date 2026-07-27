#!/usr/bin/env node
/*
 * Regenerates contest.populatedFields in data.js.
 *
 * A Maidenhead field is 20 degrees of longitude by 10 of latitude.  Contest
 * QTHs are drawn at a random bearing and distance from your own square, and
 * without a filter most of them land in an ocean, so qso.js retries until it
 * hits a field on this list.  The list is derived from the coarse continent
 * boxes below rather than hand-written, so it can be re-derived after tweaking
 * them -- and the boxes are readable, which a bare list of 172 field names is
 * not.
 *
 *   node tools/make-land-fields.js            print the list
 *   node tools/make-land-fields.js --map      draw it, to eyeball the result
 *
 * Paste the printed block over the populatedFields array in data.js.
 */
'use strict';

/* [west, east, south, north] in degrees.  Deliberately generous: a field only
   has to hold some inhabited land, not be covered in it. */
const LAND = [
  // Europe
  [-11, 3, 36, 44], [-10, 30, 36, 55], [5, 31, 55, 71], [-11, 2, 50, 61],
  [20, 45, 44, 62], [28, 60, 44, 68], [-25, -13, 63, 67], [-32, -25, 36, 40],
  // Asia
  [60, 180, 50, 72], [46, 88, 35, 55], [73, 135, 20, 53], [68, 90, 8, 35],
  [92, 110, 5, 28], [95, 141, -11, 6], [129, 146, 30, 46], [125, 131, 34, 43],
  [34, 63, 12, 40], [26, 45, 36, 42], [60, 75, 24, 38],
  // Africa
  [-17, 35, 20, 37], [-17, 42, 5, 20], [8, 52, -12, 12], [11, 41, -35, -12],
  [43, 51, -26, -12], [-19, -13, 27, 30],
  // The Americas
  [-170, -130, 54, 72], [-141, -53, 42, 72], [-125, -67, 25, 49],
  [-118, -83, 7, 32], [-85, -59, 10, 27], [-73, -12, 60, 84], [-82, -34, -56, 13],
  // Oceania
  [113, 154, -44, -10], [166, 179, -47, -34], [140, 155, -11, -1],
  [-161, -154, 18, 23], [115, 127, 4, 7]
];

/* A field counts as land if this much of it is covered.  Low enough to keep
   coastal and island squares, high enough to drop the slivers. */
const THRESHOLD = 0.08;

const overlap = (a0, a1, b0, b1) => Math.max(0, Math.min(a1, b1) - Math.max(a0, b0));

const fields = [];
const grid = [];
for (let lonIndex = 0; lonIndex < 18; lonIndex++) {
  const west = -180 + 20 * lonIndex;
  for (let latIndex = 0; latIndex < 18; latIndex++) {
    const south = -90 + 10 * latIndex;
    const covered = LAND.reduce((sum, box) =>
      sum + overlap(west, west + 20, box[0], box[1]) *
            overlap(south, south + 10, box[2], box[3]), 0);
    if (covered / 200 >= THRESHOLD) {
      const name = String.fromCharCode(97 + lonIndex) + String.fromCharCode(97 + latIndex);
      fields.push(name);
      (grid[latIndex] || (grid[latIndex] = []))[lonIndex] = true;
    }
  }
}

if (process.argv.includes('--map')) {
  console.log('\n  ' + '-'.repeat(18) + '   180W ... 180E');
  for (let latIndex = 17; latIndex >= 0; latIndex--) {
    const row = [];
    for (let lonIndex = 0; lonIndex < 18; lonIndex++) {
      row.push((grid[latIndex] || [])[lonIndex] ? '#' : '.');
    }
    console.log(' |' + row.join('') + `|  ${String(-90 + 10 * latIndex).padStart(4)} deg`);
  }
  console.log('  ' + '-'.repeat(18) + `\n\n  ${fields.length} fields\n`);
  process.exit(0);
}

const lines = [];
let line = '';
for (const name of fields) {
  const piece = `"${name}", `;
  if (line.length + piece.length > 90) { lines.push('      ' + line.trimEnd()); line = ''; }
  line += piece;
}
if (line) lines.push('      ' + line.trimEnd().replace(/,$/, ''));

console.log(`    populatedFields: [\n${lines.join('\n')}\n    ]`);
console.error(`\n${fields.length} fields — paste the block above over the one in data.js\n`);
