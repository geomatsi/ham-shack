/*
 * Seiuchy-NG -- service worker.
 *
 * Only useful when the app is served over http(s): it keeps a copy of every
 * file so the page also opens with the network switched off, and so the browser
 * offers to install it on the home screen.  Opening index.html straight from
 * the filesystem works without any of this.
 *
 * Bump CACHE when you change any of the files below, otherwise an installed
 * copy will keep serving the old ones.
 */
'use strict';

const CACHE = 'seiuchy-ng-v3';

const FILES = [
  './',
  'index.html',
  'seiuchy.css',
  'data.js',
  'random.js',
  'morse.js',
  'qso.js',
  'app.js',
  'icon.svg',
  'icon-192.png',
  'icon-512.png',
  'manifest.webmanifest'
];

self.addEventListener('install', (event) => {
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE);
    // One file at a time rather than addAll: addAll is atomic, so a single
    // missing or renamed file throws the whole cache away and the app loses
    // its offline copy without saying anything.
    const results = await Promise.allSettled(FILES.map((file) => cache.add(file)));
    const missing = FILES.filter((file, i) => results[i].status === 'rejected');
    if (missing.length) console.warn('seiuchy: could not cache ' + missing.join(', '));
  })());
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys()
      .then((names) => Promise.all(names.filter((n) => n !== CACHE).map((n) => caches.delete(n))))
      .then(() => self.clients.claim())
  );
});

/* Cache first: the app never needs the network, and this makes it instant. */
self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;
  event.respondWith((async () => {
    const hit = await caches.match(event.request);
    if (hit) return hit;
    try {
      return await fetch(event.request);
    } catch (error) {
      // Offline and not in the cache.  For a page load, hand back the app
      // instead of the browser's dinosaur -- this is what makes launching the
      // installed icon work once the machine that served it has gone away.
      if (event.request.mode === 'navigate') {
        const page = await caches.match('index.html');
        if (page) return page;
      }
      throw error;
    }
  })());
});
