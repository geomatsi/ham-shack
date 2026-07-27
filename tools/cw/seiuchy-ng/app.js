/*
 * Seiuchy-NG -- user interface.
 *
 * Reads the controls, asks qso.js for an exchange, hands it to morse.js, marks
 * the answer and keeps the log.  Settings are remembered in localStorage.
 */
'use strict';

(function () {

  const $ = (id) => document.getElementById(id);

  const el = {
    freq: $('freq'), speed: $('speed'), volume: $('volume'), myName: $('myname'),
    key: $('key'), autospeed: $('autospeed'), realqth: $('realqth'),
    contestType: $('contest-type'), myGrid: $('mygrid'), localOnly: $('localonly'),
    contestOptions: $('contest-options'),
    answer: $('itext'), play: $('play'), again: $('again'),
    test: $('test'), stop: $('stop'), score: $('score'),
    log: document.querySelector('#logcontainer tbody'),
    settingsBody: $('settings-body'), legend: $('legend-settings')
  };

  const categories = Array.from(document.querySelectorAll('.cat'));

  const sender = new MorsePlayer();     // the fictional operator
  const feedback = new MorsePlayer();   // the C / ? after your answer

  const state = {
    started: false,
    sent: '',        // what is currently being sent, for "Again"
    answer: '',      // what you are supposed to type
    category: '',
    repeated: false,
    score: 0,
    count: 0
  };

  /* ------------------------------------------------------------------ *
   *  Settings
   * ------------------------------------------------------------------ */

  const STORE_KEY = 'seiuchy-ng';

  function keyStyle() {
    if (el.key.value === 'random') {
      const pool = Object.keys(SEIUCHY_DATA.keys).filter((k) => SEIUCHY_DATA.keys[k].inRandom);
      return SEIUCHY_DATA.keys[pool[Random.below(pool.length)]];
    }
    return SEIUCHY_DATA.keys[el.key.value] || SEIUCHY_DATA.keys.computer;
  }

  function clamp(input) {
    const value = Number(input.value);
    const min = Number(input.min), max = Number(input.max);
    if (!Number.isFinite(value)) return Number(input.defaultValue);
    return Math.min(max, Math.max(min, value));
  }

  /* Everything morse.js and qso.js need to know about the current settings. */
  function settings() {
    return {
      wpm: clamp(el.speed),
      freq: clamp(el.freq),
      volume: clamp(el.volume) / 100,
      key: keyStyle(),
      myName: el.myName.value.trim().toLowerCase() || 'om',
      realQth: el.realqth.checked,
      contestType: el.contestType.value,
      myGrid: el.myGrid.value,
      localOnly: el.localOnly.checked
    };
  }

  function saveSettings() {
    const data = {
      freq: el.freq.value, speed: el.speed.value, volume: el.volume.value,
      myName: el.myName.value, key: el.key.value,
      autospeed: el.autospeed.checked, realqth: el.realqth.checked,
      contestType: el.contestType.value, myGrid: el.myGrid.value,
      localOnly: el.localOnly.checked,
      categories: categories.filter((c) => c.checked).map((c) => c.dataset.cat)
    };
    try { localStorage.setItem(STORE_KEY, JSON.stringify(data)); } catch (e) { /* private mode */ }
  }

  function loadSettings() {
    let data;
    try { data = JSON.parse(localStorage.getItem(STORE_KEY)); } catch (e) { return; }
    if (!data) return;
    for (const name of ['freq', 'speed', 'volume', 'myName', 'key', 'contestType', 'myGrid']) {
      if (data[name] !== undefined) el[name].value = data[name];
    }
    for (const name of ['autospeed', 'realqth', 'localOnly']) {
      if (data[name] !== undefined) el[name].checked = data[name];
    }
    if (Array.isArray(data.categories)) {
      categories.forEach((c) => { c.checked = data.categories.includes(c.dataset.cat); });
    }
  }

  /* ------------------------------------------------------------------ *
   *  Sending
   * ------------------------------------------------------------------ */

  function send(text, notBefore) {
    unlockAudio();
    sender.play(text, settings(), notBefore);
  }

  function chosenCategories() {
    const picked = categories.filter((c) => c.checked).map((c) => c.dataset.cat);
    return picked.length ? picked : QSO.CATEGORIES;   // none ticked means all
  }

  /* Never twice in a row while there is something else to send: three names
     running is the fastest way to make a random trainer feel broken. */
  function nextCategory() {
    const picked = chosenCategories();
    if (picked.length < 2) return picked[0];
    let category;
    do { category = picked[Random.below(picked.length)]; }
    while (category === state.category);
    return category;
  }

  function nextExchange(notBefore) {
    const exchange = QSO.build(nextCategory(), settings());
    state.category = exchange.category;
    state.answer = exchange.answer;
    state.sent = exchange.sent;
    state.repeated = false;
    send(' ' + exchange.sent, notBefore);
  }

  /* ------------------------------------------------------------------ *
   *  Marking
   * ------------------------------------------------------------------ */

  function markAnswer() {
    const typed = el.answer.value.trim();
    const correct = QSO.check(state.category, typed, state.answer);
    const wpm = clamp(el.speed);

    if (correct) {
      if (!state.repeated) {
        state.score++;
        if (el.autospeed.checked && wpm < Number(el.speed.max)) el.speed.value = wpm + 1;
      }
    } else if (el.autospeed.checked && wpm > Number(el.speed.min)) {
      el.speed.value = wpm - 1;
    }
    state.count++;

    // Slightly faster and lower than the QSO, so it cannot be mistaken for it.
    const feedbackEnds = feedback.play(correct ? 'c' : '?', Object.assign(settings(), {
      wpm: Math.round(clamp(el.speed) * 1.2),
      freq: clamp(el.freq) - 30,
      key: SEIUCHY_DATA.keys.computer
    }));

    addLogRow(state.sent, typed || 'no copy',
              correct ? (state.repeated ? 'repeat' : 'correct') : 'wrong');
    showScore();
    return feedbackEnds;
  }

  function addLogRow(exchange, typed, verdict) {
    const row = el.log.insertRow(0);
    row.insertCell().textContent = exchange;
    const cell = row.insertCell();
    cell.textContent = typed;
    cell.className = verdict;
    while (el.log.rows.length > 50) el.log.deleteRow(el.log.rows.length - 1);
  }

  function showScore() {
    el.score.textContent = state.score + '/' + state.count;
  }

  /* ------------------------------------------------------------------ *
   *  Controls
   * ------------------------------------------------------------------ */

  el.play.addEventListener('click', function () {
    let notBefore = 0;
    if (state.started) notBefore = markAnswer() + 0.4;
    else { state.started = true; el.play.textContent = 'Answer'; }
    el.answer.value = '';
    nextExchange(notBefore);
    saveSettings();
  });

  el.again.addEventListener('click', function () {
    if (!state.sent) return;
    state.repeated = true;
    send(' ' + state.sent);
  });

  el.test.addEventListener('click', function () {
    send('vvv de ' + settings().myName);
  });

  el.stop.addEventListener('click', function () {
    sender.stop();
    feedback.stop();
  });

  el.answer.addEventListener('keydown', function (event) {
    if (event.key === 'Enter') { event.preventDefault(); el.play.click(); }
  });

  $('check-all').addEventListener('click', function () {
    categories.forEach((c) => { c.checked = true; });
    toggleContestOptions();
    saveSettings();
  });

  $('uncheck-all').addEventListener('click', function () {
    categories.forEach((c) => { c.checked = false; });
    toggleContestOptions();
    saveSettings();
  });

  $('reset-score').addEventListener('click', function () {
    state.score = 0;
    state.count = 0;
    showScore();
  });

  function toggleContestOptions() {
    const contest = categories.find((c) => c.dataset.cat === 'contest');
    el.contestOptions.hidden = !contest.checked;
  }

  categories.forEach((c) => c.addEventListener('change', function () {
    toggleContestOptions();
    saveSettings();
  }));

  [el.freq, el.speed, el.volume, el.myName, el.key, el.autospeed, el.realqth,
   el.contestType, el.myGrid, el.localOnly].forEach((input) =>
    input.addEventListener('change', saveSettings));

  /* Tapping the legend folds the settings away, which matters on a phone. */
  function toggleSettings() {
    const hidden = el.settingsBody.hasAttribute('hidden');
    if (hidden) el.settingsBody.removeAttribute('hidden');
    else el.settingsBody.setAttribute('hidden', '');
    el.legend.setAttribute('aria-expanded', String(hidden));
  }
  el.legend.addEventListener('click', toggleSettings);
  el.legend.addEventListener('keydown', function (event) {
    if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); toggleSettings(); }
  });

  /* ------------------------------------------------------------------ *
   *  Start up
   * ------------------------------------------------------------------ */

  // The key list is built from the data file, so adding a style there is enough.
  for (const [name, style] of Object.entries(SEIUCHY_DATA.keys)) {
    el.key.add(new Option(style.label, name));
  }
  el.key.add(new Option('Random key', 'random'));

  loadSettings();
  toggleContestOptions();
  showScore();

  if ('serviceWorker' in navigator && location.protocol.startsWith('http')) {
    navigator.serviceWorker.register('sw.js').catch(() => { /* offline anyway */ });
  }

})();
