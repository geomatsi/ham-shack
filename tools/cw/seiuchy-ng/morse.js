/*
 * Seiuchy-NG -- Morse code generation and playback.
 *
 * Timing follows the usual PARIS convention: one dot is 1.2/wpm seconds, a dash
 * is three dots, elements inside a character are one dot apart, characters are
 * three dots apart and words seven.  Every one of those durations is then
 * multiplied by a factor taken from the keying style (see "keys" in data.js),
 * which is what makes a simulated bug sound like a bug.
 */
'use strict';

const MORSE_TABLE = {
  a: '.-',    b: '-...',  c: '-.-.',  d: '-..',   e: '.',     f: '..-.',
  g: '--.',   h: '....',  i: '..',    j: '.---',  k: '-.-',   l: '.-..',
  m: '--',    n: '-.',    o: '---',   p: '.--.',  q: '--.-',  r: '.-.',
  s: '...',   t: '-',     u: '..-',   v: '...-',  w: '.--',   x: '-..-',
  y: '-.--',  z: '--..',
  0: '-----', 1: '.----', 2: '..---', 3: '...--', 4: '....-',
  5: '.....', 6: '-....', 7: '--...', 8: '---..', 9: '----.',
  '.': '.-.-.-', ',': '--..--', '?': '..--..', '/': '-..-.',
  '=': '-...-',  '+': '.-.-.',  '>': '...-.-'   // > is <SK>
};

// Characters a heavy-handed operator is most likely to add a stray dot to.
const SLOPPY_CHARS = ',?=34567>';

const ATTACK = 0.004;  // seconds of rise/fall, so the tone does not click

let sharedContext = null;

function audioContext() {
  if (!sharedContext) {
    const Ctor = window.AudioContext || window.webkitAudioContext;
    sharedContext = new Ctor();
  }
  // Browsers start the context suspended until a user gesture touches it.
  if (sharedContext.state === 'suspended') sharedContext.resume();
  return sharedContext;
}

/* A factor is stored as [base, spread] and drawn fresh every time it is used. */
function factor(pair) {
  return pair[1] ? pair[0] + Random.fraction() * pair[1] : pair[0];
}

class MorsePlayer {
  constructor() {
    this.osc = null;
    this.gain = null;
    this.endsAt = 0;
  }

  /* Send `text`, optionally not before `startAt` (an absolute context time, as
     returned by a previous play()).  Returns the time at which it ends. */
  play(text, settings, startAt) {
    const ctx = audioContext();
    this.stop();

    const style = settings.key;
    const unit = 1.2 / settings.wpm;
    const level = settings.volume;

    this.gain = ctx.createGain();
    this.gain.gain.value = 0;
    this.gain.connect(ctx.destination);
    this.osc = ctx.createOscillator();
    this.osc.type = 'sine';
    this.osc.frequency.value = settings.freq;
    this.osc.connect(this.gain);

    let t = Math.max(ctx.currentTime + 0.05, startAt || 0);
    for (const ch of text.toLowerCase()) {
      if (ch === ' ') {
        // The character gap has already been added, four more units make seven.
        t += 4 * unit * factor(style.word);
        continue;
      }
      const elements = MORSE_TABLE[ch];
      if (!elements) continue;
      for (const el of elements) {
        const on = el === '.' ? unit * factor(style.dit) : 3 * unit * factor(style.dah);
        this.keyDown(t, on, level);
        t += on + unit * factor(style.gap);
        if (style.sloppyDots && el === '.' && SLOPPY_CHARS.includes(ch) &&
            Random.fraction() < 0.07) {
          const extra = unit * factor(style.dit);
          this.keyDown(t, extra, level);
          t += extra + unit * factor(style.gap);
        }
      }
      t += 2 * unit * factor(style.letter);  // one element gap was added above
    }

    this.osc.start(ctx.currentTime);
    this.osc.stop(t + 0.05);
    this.endsAt = t;
    return t;
  }

  keyDown(at, duration, level) {
    const rise = Math.min(ATTACK, duration / 4);
    const g = this.gain.gain;
    g.setValueAtTime(0, at);
    g.linearRampToValueAtTime(level, at + rise);
    g.setValueAtTime(level, at + duration - rise);
    g.linearRampToValueAtTime(0, at + duration);
  }

  stop() {
    if (!this.osc) return;
    const now = sharedContext.currentTime;
    this.gain.gain.cancelScheduledValues(now);
    this.gain.gain.setValueAtTime(0, now);
    try { this.osc.stop(now); } catch (e) { /* already stopped */ }
    this.osc.disconnect();
    this.gain.disconnect();
    this.osc = null;
    this.gain = null;
    this.endsAt = 0;
  }

  get playing() {
    return this.osc !== null && sharedContext.currentTime < this.endsAt;
  }
}

/* Wake the audio hardware up from inside a click handler.  iOS in particular
   refuses to make any sound until this has happened at least once. */
function unlockAudio() {
  audioContext();
}
