/*
 * Seiuchy-NG -- the fictional operators.
 *
 * Every generator returns { sent, answer }: `sent` is what you hear, `answer` is
 * the bit of it you are supposed to log.  Nothing in here touches the DOM, so
 * the whole file can be exercised from a console.  All the words come from
 * SEIUCHY_DATA (data.js).
 */
'use strict';

const QSO = (function (D) {

  /* ------------------------------------------------------------------ *
   *  Small helpers
   * ------------------------------------------------------------------ */

  const chance = (p) => Random.fraction() < p;
  const randInt = (lo, hi) => lo + Random.below(hi - lo + 1);

  /* Drawing independently every time is correct and feels wrong: over a short
     session it hands you the same name, the same town or the same wording twice
     in a row often enough to be irritating.  So each list remembers what it has
     just given out and will not repeat while that is still fresh.

     The window is an eighth of the list, at most 4.  Refusing repeats does bend
     the odds -- an entry listed twice comes up about 1.8x as often rather than
     exactly 2x, because being popular is what puts it in the window -- and a
     wider window bends them further, so it stays narrow.  Short lists get no
     memory at all: with three choices, refusing repeats would only make them
     alternate predictably, which is worse than the clumping it cures. */
  const memories = new WeakMap();

  function memoryOf(list) {
    let memory = memories.get(list);
    if (!memory) {
      const distinct = new Set(list).size;
      memory = { limit: Math.min(4, Math.floor(distinct / 8)), recent: [] };
      memories.set(list, memory);
    }
    return memory;
  }

  function pick(list) {
    if (list.length < 2) return list[0];
    const memory = memoryOf(list);
    let value = list[Random.below(list.length)];
    // Rejection, with a bound so a list of near-duplicates cannot spin here.
    for (let tries = 0; memory.limit && tries < 12 && memory.recent.includes(value); tries++) {
      value = list[Random.below(list.length)];
    }
    if (memory.limit) {
      memory.recent.push(value);
      if (memory.recent.length > memory.limit) memory.recent.shift();
    }
    return value;
  }

  /* Replace {placeholders}, fix up "a" in front of a vowel (u is left alone:
     "a ubitx" is right, "an ubitx" is not) and tidy the whitespace. */
  const fill = (template, values) =>
    template.replace(/\{(\w+)\}/g, (_, key) => values[key] ?? '')
            .replace(/\s+/g, ' ')
            .replace(/\ba (?=[aeio])/g, 'an ')
            .trim();

  const sep = () => pick(D.phrases.separators);

  /* Countries come up in proportion to their weight. */
  const countryPool = [];
  for (const [cc, country] of Object.entries(D.countries)) {
    for (let i = 0; i < country.weight; i++) countryPool.push(cc);
  }
  const pickCountry = () => pick(countryPool);

  /* Cut numbers: contest operators send t for 0 and n for 9. */
  const cut = (n) => String(n).replace(/0/g, 't').replace(/9/g, 'n');
  const cutOrNot = (n) => (chance(0.8) ? cut(n) : String(n));

  /* ------------------------------------------------------------------ *
   *  Maidenhead locators
   * ------------------------------------------------------------------ */

  const EARTH_RADIUS = 6371;  // km
  const toNum = (c) => c.toLowerCase().charCodeAt(0) - 97;
  const toLetter = (n) => String.fromCharCode(n + 97);

  function gridToLatLon(grid) {
    const lon = toNum(grid[0]) * 20 + parseInt(grid[2], 10) * 2 + toNum(grid[4]) * 0.08333;
    const lat = toNum(grid[1]) * 10 + parseInt(grid[3], 10) + toNum(grid[5]) * 0.04167;
    return { lat: lat - 90, lon: lon - 180 };
  }

  function latLonToGrid(lat, lon) {
    let latRest = (lat + 90 + 180) % 180;
    let lonRest = (lon + 180 + 360) % 360;
    const lonField = Math.trunc(lonRest / 20);
    const latField = Math.trunc(latRest / 10);
    lonRest -= lonField * 20;
    latRest -= latField * 10;
    const lonSquare = Math.trunc(lonRest / 2);
    const latSquare = Math.trunc(latRest);
    lonRest -= lonSquare * 2;
    latRest -= latSquare;
    return toLetter(lonField) + toLetter(latField) + lonSquare + latSquare +
           toLetter(Math.trunc(lonRest * 12)) + toLetter(Math.trunc(latRest * 24));
  }

  /* Point `km` away from a position, along the given bearing. */
  function destination(lat, lon, bearingDeg, km) {
    const rad = Math.PI / 180;
    const lat1 = lat * rad, lon1 = lon * rad, brg = bearingDeg * rad;
    const d = km / EARTH_RADIUS;
    const lat2 = Math.asin(Math.sin(lat1) * Math.cos(d) +
                           Math.cos(lat1) * Math.sin(d) * Math.cos(brg));
    const lon2 = lon1 + Math.atan2(Math.sin(brg) * Math.sin(d) * Math.cos(lat1),
                                   Math.cos(d) - Math.sin(lat1) * Math.sin(lat2));
    return { lat: lat2 / rad, lon: lon2 / rad };
  }

  /* Accepts a 2, 4 or 6 character locator and pads it out to 6. */
  function normaliseGrid(text) {
    const m = /^([a-r]{2})(\d{2})?([a-x]{2})?/i.exec((text || '').trim());
    if (!m) return null;
    return (m[1] + (m[2] || '55') + (m[3] || 'll')).toLowerCase();
  }

  const isOnLand = (grid) => D.contest.populatedFields.includes(grid.slice(0, 2));

  /* Pick a locator at a plausible distance.  `band` is 'hf' or 'vhf'. */
  function randomGrid(chars, band, settings) {
    const home = normaliseGrid(settings.myGrid);
    const vhf = band === 'vhf';
    let origin = gridToLatLon(home || (settings.localOnly ? 'en00cd' : 'aa00aa'));
    let min = 1, max = 19000;

    if (home && settings.localOnly)  { max = vhf ? 500 : 1500; }
    else if (home) {
      const roll = Random.fraction();
      if (vhf) {
        if (roll < 0.65)      { max = 550; }
        else if (roll < 0.97) { min = 200; max = 1000; }
        else                  { min = 600; max = 4170; }   // rare big dx
      } else {
        if (roll < 0.5)      { max = 2000; }
        else if (roll < 0.8) { min = 1500; max = 9000; }
        else                 { min = 7000; max = 19000; }
      }
    } else if (settings.localOnly) {  // no grid entered: sit in the middle of the USA
      max = vhf ? 1000 : 2400;
    }

    let grid;
    for (let tries = 0; tries < 5; tries++) {
      const there = destination(origin.lat, origin.lon,
                                Random.fraction() * 360,
                                min + Random.fraction() * (max - min));
      grid = latLonToGrid(there.lat, there.lon);
      if (isOnLand(grid)) break;
    }
    return chars === 4 ? grid.slice(0, 4) : grid;
  }

  /* ------------------------------------------------------------------ *
   *  Invented place names
   * ------------------------------------------------------------------ */

  function inventPlace(styleName) {
    const style = D.placeStyles[styleName] || D.placeStyles.generic;
    let name;
    if (style.syllables) {
      const count = randInt(style.minSyllables || 2, style.maxSyllables || 3);
      name = '';
      for (let i = 0; i < count; i++) name += pick(style.syllables);
    } else {
      name = pick(style.prefixes) + pick(style.suffixes);
    }
    if (style.postWords && chance(style.postWordChance || 0)) name += pick(style.postWords);
    if (style.preWords && chance(style.preWordChance || 0)) name = pick(style.preWords) + name;
    return name;
  }

  /* ------------------------------------------------------------------ *
   *  The QSO elements
   * ------------------------------------------------------------------ */

  function name() {
    const value = pick(D.names);
    const spoken = chance(0.8) ? value + ' ' + value : value;   // doubling is common
    return { sent: fill(pick(D.phrases.name), { name: spoken }), answer: value };
  }

  function rst() {
    let report;
    if (chance(0.5))       { report = chance(0.5) ? '599' : '5nn'; }
    else if (chance(0.5))  { report = pick(D.reports); }
    else if (chance(0.2))  { report = `${randInt(1, 5)}${randInt(1, 9)}9`; }
    else                   { report = `${randInt(1, 5)}${randInt(1, 9)}${randInt(1, 9)}`; }

    const answer = report === '5nn' ? '599' : report;
    const spoken = chance(0.5) ? report + ' ' + report.replace(/9/g, 'n')
                               : report + ' ' + report;
    return { sent: fill(pick(D.phrases.rst), { rst: spoken }), answer };
  }

  function age() {
    const years = randInt(10, 119);
    return { sent: fill(pick(D.phrases.age), { age: years }), answer: String(years) };
  }

  function qth(settings, cc) {
    const country = D.countries[cc || pickCountry()];
    let town = pick(country.cities);
    if (!settings.realQth && chance(0.25)) town = inventPlace(country.style);
    // Doubling the town is common; "nr" and the like live in the templates.
    const spoken = chance(0.8) ? town + ' ' + town : town;
    return { sent: fill(pick(D.phrases.qth), { qth: spoken }), answer: town };
  }

  /* Power and antenna are padding: some templates mention them, and they are
     never the answer. */
  function rig() {
    const value = pick(D.rigs);
    const sent = fill(pick(D.phrases.rig),
                      { rig: value, pwr: pick(D.powers), ant: pick(D.antennas) });
    return { sent, answer: value };
  }

  /* `years` is optional and only decides the phrasing -- the answer is the job
     either way.  It must come second: build() calls every generator with the
     settings object first. */
  function job(settings, years) {
    const value = fill(pick(D.jobs), { animal: pick(D.animals) });
    const old = years || randInt(19, 105);
    let templates = D.phrases.jobNow;
    if (old > 59 && chance(0.8))  templates = D.phrases.jobPast;
    else if (old < 25)            templates = D.phrases.jobFuture;
    return { sent: fill(pick(templates), { job: value }) + sep(), answer: value };
  }

  function club() {
    const which = pick(D.clubs);
    const nr = randInt(100, 100098);
    let sent = fill(pick(D.phrases.club), { club: which, nr });
    if (chance(0.7)) sent += ' ' + nr;
    return { sent, answer: which + ' ' + nr };
  }

  function call() {
    const country = D.countries[pickCountry()];
    // Most countries use the whole 0-9 range; some only ever issue a couple of
    // digits (HB9, VK1-8), and those name them in the data.
    const digits = country.digits || '0123456789';
    let sign = pick(country.prefixes) + digits[Random.below(digits.length)];
    const letters = chance(0.05) ? 1 : chance(0.2) ? 2 : 3;
    for (let i = 0; i < letters; i++) sign += toLetter(randInt(0, 25));
    return { sent: fill(pick(D.phrases.call), { call: sign + ' ' + sign }), answer: sign };
  }

  /* Contest serial numbers cluster low, the way they do early in a contest. */
  function serial() {
    const roll = Random.fraction();
    if (roll > 0.5) {
      let nr = randInt(100, 6600);
      if (nr > 2500) nr = 100 + (nr % 400);
      return nr === 599 ? '001' : String(nr);
    }
    if (roll > 0.2) return '0' + randInt(10, 99);
    return '00' + randInt(1, 9);
  }

  function fieldDayExchange() {
    const category = pick(D.contest.fieldDayCategories);
    let transmitters;
    if (category === 'a' || category === 'f') {
      transmitters = randInt(1, 11);
      if (transmitters > 10) transmitters = randInt(1, 30);
    } else if (category === 'b' || category === 'c') {
      transmitters = randInt(1, 4);
    } else {
      transmitters = randInt(1, 2);
    }
    const klass = transmitters + category;
    const section = pick(D.contest.fieldDaySections);
    return {
      sent: `${cutOrNot(599)} ${klass} ${klass} ${section} ${section}`,
      answer: `${klass} ${section}`
    };
  }

  function contest(settings) {
    switch (settings.contestType) {
      case 'grid': {
        const grid = randomGrid(4, 'hf', settings);
        return { sent: '5nn ' + grid, answer: grid };
      }
      case 'loc': {
        const grid = randomGrid(6, 'hf', settings);
        return { sent: '5nn ' + grid, answer: grid };
      }
      case 'vhf': {
        const nr = serial();
        const grid = randomGrid(6, 'vhf', settings);
        return { sent: `5nn ${cutOrNot(nr)} ${grid}`, answer: `${nr} ${grid}` };
      }
      case 'fd':
        return fieldDayExchange();
      default: {
        const nr = serial();
        return { sent: '5nn ' + cutOrNot(nr), answer: nr };
      }
    }
  }

  /* ------------------------------------------------------------------ *
   *  Putting an over together
   * ------------------------------------------------------------------ */

  /* Each is called as fn(settings); any further parameter is optional and only
     used when poking at them from the console. */
  const GENERATORS = { name, rst, age, qth, rig, job, club, call, contest };

  const CATEGORIES = Object.keys(GENERATORS);

  /* Wrap the answer in the padding you are supposed to learn to ignore. */
  function padded(element, settings) {
    const address = fill(pick(D.phrases.address), { name: settings.myName });
    let sent = element.sent;
    if (chance(0.3))      sent = fill(pick(D.phrases.fluffBefore), { om: address }) + ' ' + sent;
    else if (chance(0.3)) sent = sent + ' ' + fill(pick(D.phrases.fluffAfter), { om: address });
    return '= ' + sent;
  }

  /* Build one exchange for the given category. */
  function build(category, settings) {
    const element = GENERATORS[category](settings);
    const sent = (category === 'call' || category === 'contest')
      ? element.sent
      : padded(element, settings);
    return { category, answer: element.answer, sent: sent.replace(/\s+/g, ' ').trim() };
  }

  /* ------------------------------------------------------------------ *
   *  Marking
   * ------------------------------------------------------------------ */

  /* What you typed is accepted if it contains the expected answer, so that
     "nr 43" passes for "43" and stray filler words are forgiven. */
  function check(category, typed, expected) {
    let given = (typed || 'no copy').toLowerCase().trim()
      .replace(/^n[ea]*r /, '')   // a leading "nr" belongs to the padding
      .replace(/-/g, ' ')
      .replace(/\s+/g, ' ');

    if (given.replace(/ /g, '') === '5nn') given = '599';
    if (category === 'contest') given = given.replace(/^5(99|nn) ?/, '');
    if (category === 'club') {
      // The number is what matters.  If you also typed the club, the first two
      // letters have to agree -- "cwops 1234", "cw 1234" and "1234" all pass,
      // "fists 1234" against a CWops number does not.
      const nr = expected.match(/\d+$/)[0];          // club names may be two words
      const club = expected.slice(0, -nr.length).trim();
      const typedNr = (given.match(/\d+/) || [''])[0];
      const typedClub = given.replace(/[^a-z]/g, '');
      return typedNr === nr &&
             (typedClub === '' || typedClub.slice(0, 2) === club.slice(0, 2));
    }
    if (category === 'rig') {
      // "ic7000" and "ic-7000" should both match "ic 7000".
      const spaced = (s) => s.replace(/(\w+?)\W*?(\d+)/, '$1 $2');
      return spaced(given).includes(spaced(expected));
    }
    return given.includes(expected);
  }

  return { build, check, CATEGORIES, normaliseGrid,
           // exposed for tinkering from the console
           elements: GENERATORS, randomGrid, inventPlace };

})(SEIUCHY_DATA);
