// sw.js — the opt-in offline mode's service worker. Registered only by web/offline.js, and only
// after `?offline=1`; without that switch no worker exists and nothing below runs.
//
// What it buys: the game keeps playing when `wenilla-host` is not running. Everything the page
// reads comes from this origin — the page and its modules, the wasm, and `/data/*`, the game files
// the host answers from the player's own client — so a worker that keeps copies in the Cache API
// can stand in for the host. It cannot stand in for the client itself: a `/data/*` file is
// available offline only once it was read online, either by the install-time warmup below (the
// boot and world-entry read-sets, boot-manifest.json and world-manifest.json) or by playing.
// A zone never played online is not playable offline.
//
// What it costs: a controlled page sends every request through the `fetch` event below,
// including the chain's synchronous XHRs (benilla-formats/src/web.rs) on the client's one
// CPU-bound thread. That is the reason it is opt-in, and `?offline=0` removes it entirely.
//
// The routes mirror wenilla-host's:
//   /data/<name>   immutable (the host serves it max-age=1y): cache first, network on a miss,
//                  and the miss is stored. GET only; HEAD (the chain's existence fallback) is
//                  answered from a stored GET. Offline and not stored: 503.
//   /data/__index  the chain's name list, which follows the player's Data folder: network first.
//   /ws/*          never seen here (WebSockets bypass service workers).
//   everything else is the page (served no-cache, rewritten by every build): network first, and
//                  the stored copy only when the host is unreachable.

const VERSION = 1;
const PREFIX = 'wenilla-offline-';
const SHELL = `${PREFIX}shell-v${VERSION}`;
const DATA = `${PREFIX}data-v${VERSION}`;

// What the dev page loads before the client draws its first glue screen. bridge.js and the
// examples are opt-in (`?bridge=`) and get stored on first use like any other page file.
const SHELL_FILES = [
  './', 'wenilla.js', 'wenilla_bg.wasm', 'wasi_stubs.js', 'boot.js', 'platform.js', 'offline.js',
  'favicon.svg', 'manifest.webmanifest',
];
const MANIFESTS = ['boot-manifest.json', 'world-manifest.json'];
const WARM_CONCURRENCY = 8;

self.addEventListener('install', (event) => {
  event.waitUntil((async () => {
    // addAll is all-or-nothing on purpose: a worker that cannot boot the page offline must not
    // install. The data warmup after it is best-effort, file by file.
    await (await caches.open(SHELL)).addAll(SHELL_FILES);
    await warmManifests();
    await self.skipWaiting();
  })());
});

self.addEventListener('activate', (event) => {
  event.waitUntil((async () => {
    for (const name of await caches.keys()) {
      if (name.startsWith(PREFIX) && name !== SHELL && name !== DATA) await caches.delete(name);
    }
    // Take the page that registered us right away, so the reads it makes from here on are
    // stored, rather than only those of the next visit.
    await self.clients.claim();
  })());
});

self.addEventListener('fetch', (event) => {
  const req = event.request;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  // A byte range is not what a stored whole file answers; leave it to the network.
  if (req.headers.has('range')) return;
  if (url.pathname === '/data/__index') {
    if (req.method === 'GET') event.respondWith(networkFirst(event, DATA, req));
    return;
  }
  if (url.pathname.startsWith('/data/')) {
    if (req.method === 'GET') event.respondWith(dataGet(event));
    else if (req.method === 'HEAD') event.respondWith(dataHead(req));
    return;
  }
  if (req.method !== 'GET') return;
  // A navigation is stored under its path alone: `?offline=1`, `?bridge=1` or a pinned CVar is
  // still the same page, and offline it must be found whatever query string it is opened with.
  event.respondWith(networkFirst(event, SHELL, req.mode === 'navigate' ? url.origin + url.pathname : req));
});

async function dataGet(event) {
  const req = event.request;
  const cache = await caches.open(DATA);
  const hit = await cache.match(req);
  if (hit) return hit;
  let res;
  try {
    res = await fetch(req);
  } catch (_) {
    return notStored();
  }
  if (res.status === 200) event.waitUntil(store(cache, req, res.clone()));
  return res;
}

async function dataHead(req) {
  const hit = await (await caches.open(DATA)).match(req, { ignoreMethod: true });
  if (hit) return new Response(null, { status: 200, headers: hit.headers });
  try {
    return await fetch(req);
  } catch (_) {
    return notStored();
  }
}

async function networkFirst(event, cacheName, key) {
  const cache = await caches.open(cacheName);
  try {
    const res = await fetch(event.request);
    if (res.status === 200) event.waitUntil(refresh(cache, key, res.clone()));
    return res;
  } catch (e) {
    const hit = (await cache.match(key)) ??
      (event.request.mode === 'navigate' ? await cache.match('./') : undefined);
    if (hit) return hit;
    throw e;
  }
}

// The host serves the page no-cache, so every online visit re-fetches it (a 304 from the HTTP
// cache when unchanged, which still reaches us as a 200). Rewriting ~90 MB of wasm into the
// Cache API on every visit for nothing is the cost to avoid: an unchanged validator means the
// stored copy is already this one, and the unread clone is cancelled so it does not buffer.
// Not awaited: a tee branch's cancel settles only once the other branch (the page's) is done.
async function refresh(cache, key, res) {
  const old = await cache.match(key);
  if (old && sameVersion(old, res)) {
    res.body?.cancel().catch(() => {});
    return;
  }
  await store(cache, key, res);
}

function sameVersion(a, b) {
  let compared = false;
  for (const h of ['etag', 'last-modified', 'content-length']) {
    const x = a.headers.get(h), y = b.headers.get(h);
    if (x === null && y === null) continue;
    if (x !== y) return false;
    compared = true;
  }
  return compared;
}

// A full disk (QuotaExceededError) must not fail the read that triggered the store: the player
// still gets the file, it just will not be there offline.
async function store(cache, key, res) {
  try {
    await cache.put(key, res);
  } catch (e) {
    console.warn('offline: not stored', typeof key === 'string' ? key : key.url, e);
  }
}

function notStored() {
  return new Response('not stored for offline play', { status: 503, statusText: 'Offline' });
}

// Store the boot and world-entry read-sets — the same lists, and the same URLs, boot.js
// prefetches (`'/data/' + encodeURIComponent(name)`, which is benilla-formats' encode_name) —
// so the first offline boot has what it reads before any zone. Progress goes to every page of
// the origin as `{type: 'wenilla-offline', done, total}` (web/offline.js prints it).
async function warmManifests() {
  const names = new Set();
  for (const manifest of MANIFESTS) {
    try {
      const res = await fetch(manifest, { cache: 'no-cache' });
      if (!res.ok) continue;
      for (const name of (await res.json()).names ?? []) names.add(name);
    } catch (_) { /* a tree without that manifest: nothing to warm from it */ }
  }
  names.delete('__index');
  const urls = [...names].map((name) => '/data/' + encodeURIComponent(name));
  const cache = await caches.open(DATA);
  let next = 0, done = 0;
  const worker = async () => {
    while (next < urls.length) {
      const url = urls[next++];
      try {
        if (!(await cache.match(url))) {
          const res = await fetch(url);
          if (res.status === 200) await store(cache, url, res);
        }
      } catch (_) { /* one file short; it is stored on first read instead */ }
      done++;
      if (done % 50 === 0 || done === urls.length) await announce({ done, total: urls.length });
    }
  };
  await Promise.all(Array.from({ length: WARM_CONCURRENCY }, worker));
}

async function announce(progress) {
  for (const client of await self.clients.matchAll({ type: 'window', includeUncontrolled: true })) {
    client.postMessage({ type: 'wenilla-offline', ...progress });
  }
}
