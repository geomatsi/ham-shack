/*
 * Seiuchy-NG -- the random source.
 *
 * Two things matter here.
 *
 * First, independence between sessions.  Every browser seeds Math.random from
 * system entropy per page, so reloading already gives you a different session
 * -- but that is an implementation detail nobody promises, and it has been
 * weaker in the past on some embedded WebViews.  Drawing from the platform
 * CSPRNG instead settles the question: crypto.getRandomValues is available in
 * every browser worth the name, works from a file:// page, and is seeded by the
 * operating system.  Math.random stays as a fallback so the trainer still runs
 * somewhere exotic.
 *
 * Second, no modulo bias.  Taking a 32-bit word modulo 9 quietly makes the
 * first few categories more likely than the last; below() throws away the ragged
 * tail of the range instead, so every value really is equally likely.
 */
'use strict';

const Random = (function () {

  const RANGE = 4294967296;          // 2^32
  const pool = new Uint32Array(256);
  let next = pool.length;            // forces a fill on first use

  const entropy = (typeof crypto !== 'undefined' && crypto.getRandomValues)
    ? crypto : null;

  function word() {
    if (!entropy) return Math.floor(Math.random() * RANGE);
    if (next >= pool.length) {
      entropy.getRandomValues(pool);
      next = 0;
    }
    return pool[next++];
  }

  /* A fraction in [0, 1), with 32 bits behind it. */
  function fraction() {
    return word() / RANGE;
  }

  /* An integer in [0, n), uniformly. */
  function below(n) {
    if (n <= 1) return 0;
    const limit = RANGE - (RANGE % n);   // largest exact multiple of n
    let w;
    do { w = word(); } while (w >= limit);
    return w % n;
  }

  return { fraction, below, usingCrypto: entropy !== null };

})();
