import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const ORIGIN = 'http://127.0.0.1:8090';
const swSource = await readFile(new URL('../sw.js', import.meta.url), 'utf8');
const offlineSource = await readFile(new URL('../offline.js', import.meta.url), 'utf8');
let loads = 0;
// A fresh module instance per test: the comment makes each data: URL unique.
const load = (source) =>
  import('data:text/javascript;base64,' + Buffer.from(`${source}\n//${loads++}`).toString('base64'));

function override(t, globals) {
  for (const [key, value] of Object.entries(globals)) {
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, key);
    Object.defineProperty(globalThis, key, { configurable: true, writable: true, value });
    t.after(() => descriptor ? Object.defineProperty(globalThis, key, descriptor) : delete globalThis[key]);
  }
}

// Just enough of CacheStorage: keys are absolute URLs, a non-GET request matches only with
// ignoreMethod, relative URLs resolve against the worker's location — as in a browser.
class FakeCache {
  entries = new Map();
  puts = [];
  key(req) { return new URL(typeof req === 'string' ? req : req.url, ORIGIN + '/sw.js').href; }
  async match(req, opts = {}) {
    if (typeof req !== 'string' && req.method && req.method !== 'GET' && !opts.ignoreMethod) return undefined;
    return this.entries.get(this.key(req))?.clone();
  }
  async put(req, res) {
    const body = await res.arrayBuffer();
    this.puts.push(this.key(req));
    this.entries.set(this.key(req), new Response(body, { status: res.status, headers: res.headers }));
  }
  async addAll(urls) {
    for (const url of urls) {
      const res = await fetch(new URL(url, ORIGIN + '/sw.js').href);
      if (!res.ok) throw new TypeError(`addAll: ${url} ${res.status}`);
      await this.put(url, res);
    }
  }
  async keys() { return [...this.entries.keys()]; }
}

class FakeCaches {
  stores = new Map();
  async open(name) {
    if (!this.stores.has(name)) this.stores.set(name, new FakeCache());
    return this.stores.get(name);
  }
  async keys() { return [...this.stores.keys()]; }
  async delete(name) { return this.stores.delete(name); }
}

// The host: `files` maps a path to [body, headers]; `online` false makes every fetch reject.
function fakeHost(files) {
  const host = { online: true, requests: [] };
  host.fetch = async (input) => {
    const url = new URL(typeof input === 'string' ? input : input.url, ORIGIN + '/sw.js');
    const method = typeof input === 'string' ? 'GET' : input.method ?? 'GET';
    host.requests.push(`${method} ${url.pathname}`);
    if (!host.online) throw new TypeError('Failed to fetch');
    const file = files[url.pathname];
    if (!file) return new Response('', { status: 404 });
    return new Response(method === 'HEAD' ? null : file[0], { status: 200, headers: file[1] ?? {} });
  };
  return host;
}

async function worker(t, files) {
  const handlers = {};
  const announced = [];
  const host = fakeHost(files);
  const caches = new FakeCaches();
  override(t, {
    self: {
      location: new URL(ORIGIN + '/sw.js'),
      addEventListener: (type, fn) => { handlers[type] = fn; },
      skipWaiting: async () => {},
      clients: {
        claim: async () => {},
        matchAll: async () => [{ postMessage: (m) => announced.push(m) }],
      },
    },
    caches,
    fetch: host.fetch,
  });
  await load(swSource);
  const lifecycle = async (type) => {
    const waits = [];
    handlers[type]({ waitUntil: (p) => waits.push(p) });
    await Promise.all(waits);
  };
  // Returns the Response the worker answered with, or null when it let the request through.
  const request = async (path, { method = 'GET', mode = 'cors', headers = {} } = {}) => {
    let answer = null;
    const waits = [];
    handlers.fetch({
      request: { url: ORIGIN + path, method, mode, headers: new Headers(headers) },
      respondWith: (p) => { answer = p; },
      waitUntil: (p) => waits.push(p),
    });
    const res = answer && await answer;
    await Promise.all(waits);
    return res;
  };
  const data = (name) => '/data/' + encodeURIComponent(name);
  return { host, caches, announced, lifecycle, request, data };
}

const SHELL = {
  '/': ['<!doctype html>page', { 'content-type': 'text/html', etag: '"p1"' }],
  '/wenilla.js': ['glue', { etag: '"g1"' }],
  '/wenilla_bg.wasm': ['wasm-v1', { 'content-type': 'application/wasm', etag: '"w1"' }],
  '/wasi_stubs.js': ['', {}], '/boot.js': ['', {}], '/platform.js': ['', {}],
  '/offline.js': ['', {}], '/favicon.svg': ['<svg/>', {}], '/manifest.webmanifest': ['{}', {}],
};

test('a game file is stored on first read and served from the store once the host is gone', async t => {
  const w = await worker(t, { [`/data/${encodeURIComponent('DBFilesClient\\Map.dbc')}`]: ['map bytes'] });
  const path = w.data('DBFilesClient\\Map.dbc');
  assert.equal(await (await w.request(path)).text(), 'map bytes');
  w.host.online = false;
  const res = await w.request(path);
  assert.equal(res.status, 200);
  assert.equal(await res.text(), 'map bytes');
});

test('offline, a game file never read answers 503, and an existence check of it too', async t => {
  const w = await worker(t, {});
  w.host.online = false;
  assert.equal((await w.request(w.data('World\\never.adt'))).status, 503);
  assert.equal((await w.request(w.data('World\\never.adt'), { method: 'HEAD' })).status, 503);
});

test('a HEAD is answered from the stored GET, without touching the host', async t => {
  const w = await worker(t, { [`/data/${encodeURIComponent('a\\b.blp')}`]: ['texture'] });
  await w.request(w.data('a\\b.blp'));
  w.host.requests.length = 0;
  const res = await w.request(w.data('a\\b.blp'), { method: 'HEAD' });
  assert.equal(res.status, 200);
  assert.equal(await res.text(), '');
  assert.deepEqual(w.host.requests, []);
});

test('a host error is passed through and not stored', async t => {
  const w = await worker(t, {});
  assert.equal((await w.request(w.data('missing'))).status, 404);
  for (const cache of w.caches.stores.values()) assert.equal(cache.puts.length, 0);
});

test('byte ranges and WebSocket-free paths are left to the network', async t => {
  const w = await worker(t, {});
  assert.equal(await w.request(w.data('x'), { headers: { range: 'bytes=0-9' } }), null);
  assert.equal(await w.request(w.data('x'), { method: 'POST' }), null);
  assert.equal(await w.request('/', { method: 'POST' }), null);
});

test('the name index follows the host while it is up, and the stored copy when it is not', async t => {
  const files = { '/data/__index': ['["a"]'] };
  const w = await worker(t, files);
  assert.equal(await (await w.request('/data/__index')).text(), '["a"]');
  files['/data/__index'] = ['["a","b"]'];
  assert.equal(await (await w.request('/data/__index')).text(), '["a","b"]');
  w.host.online = false;
  assert.equal(await (await w.request('/data/__index')).text(), '["a","b"]');
});

test('a navigation is stored by path, found offline whatever its query, and falls back to the page', async t => {
  const w = await worker(t, { ...SHELL, '/harness.html': ['harness', {}] });
  await w.request('/?offline=1', { mode: 'navigate' });
  await w.request('/harness.html?x=1', { mode: 'navigate' });
  w.host.online = false;
  assert.equal(await (await w.request('/?bridge=1', { mode: 'navigate' })).text(), '<!doctype html>page');
  assert.equal(await (await w.request('/harness.html', { mode: 'navigate' })).text(), 'harness');
  assert.equal(await (await w.request('/index.html', { mode: 'navigate' })).text(), '<!doctype html>page');
});

test('the page is fetched first while the host is up; an unchanged wasm is not rewritten', async t => {
  const files = { ...SHELL };
  const w = await worker(t, files);
  await w.lifecycle('install');
  const shell = [...w.caches.stores.entries()].find(([name]) => name.includes('shell'))[1];
  const wasm = ORIGIN + '/wenilla_bg.wasm';
  assert.equal(shell.puts.filter((k) => k === wasm).length, 1);
  assert.equal(await (await w.request('/wenilla_bg.wasm')).text(), 'wasm-v1');
  assert.equal(shell.puts.filter((k) => k === wasm).length, 1, 'same etag: no second write');
  files['/wenilla_bg.wasm'] = ['wasm-v2', { 'content-type': 'application/wasm', etag: '"w2"' }];
  assert.equal(await (await w.request('/wenilla_bg.wasm')).text(), 'wasm-v2');
  assert.equal(shell.puts.filter((k) => k === wasm).length, 2, 'new etag: rewritten');
  w.host.online = false;
  const res = await w.request('/wenilla_bg.wasm');
  assert.equal(await res.text(), 'wasm-v2');
  assert.equal(res.headers.get('content-type'), 'application/wasm');
});

test('install stores the page and warms the boot and world read-sets, with progress', async t => {
  const names = ['__index', 'DBFilesClient\\A.dbc', 'Interface\\B C.blp'];
  const files = {
    ...SHELL,
    '/boot-manifest.json': [JSON.stringify({ version: 1, names: names.slice(0, 2) })],
    '/world-manifest.json': [JSON.stringify({ version: 1, names: names.slice(1) })],
  };
  for (const name of names) files['/data/' + encodeURIComponent(name)] = [name];
  const w = await worker(t, files);
  await w.lifecycle('install');
  w.host.online = false;
  for (const name of names.slice(1)) {
    assert.equal(await (await w.request(w.data(name))).text(), name);
  }
  assert.ok(!w.host.requests.includes('GET /data/__index'), '__index is not warmed as a file');
  assert.deepEqual(w.announced.at(-1), { type: 'wenilla-offline', done: 2, total: 2 });
});

test('install fails when the page cannot be stored', async t => {
  const w = await worker(t, {});
  await assert.rejects(w.lifecycle('install'));
});

test('activate drops older versions of its own caches and nothing else', async t => {
  const w = await worker(t, {});
  for (const name of ['wenilla-offline-data-v0', 'wenilla-offline-shell-v0', 'someone-else']) await w.caches.open(name);
  await w.request(w.data('x'));
  await w.lifecycle('activate');
  const left = await w.caches.keys();
  assert.ok(left.includes('someone-else'));
  assert.ok(!left.some((name) => name.endsWith('-v0')));
});

// offline.js: the switch.
function page(t, { search = '', flag = null, registrations = [], cacheNames = [] } = {}) {
  const state = {
    store: new Map(flag ? [['wenilla:offline', flag]] : []),
    registered: [],
    unregistered: [],
    deleted: [],
    cacheNames: [...cacheNames],
  };
  override(t, {
    location: { search },
    localStorage: {
      getItem: (k) => state.store.get(k) ?? null,
      setItem: (k, v) => state.store.set(k, v),
      removeItem: (k) => state.store.delete(k),
    },
    navigator: {
      serviceWorker: {
        register: async (url) => { state.registered.push(url); },
        ready: Promise.resolve(),
        addEventListener() {},
        getRegistrations: async () => registrations.map((scriptURL) => ({
          active: { scriptURL },
          unregister: async () => { state.unregistered.push(scriptURL); return true; },
        })),
      },
      storage: { estimate: async () => ({ usage: 5e6, quota: 1e9 }) },
    },
    caches: {
      keys: async () => state.cacheNames,
      open: async () => ({ keys: async () => [1, 2, 3] }),
      delete: async (name) => {
        state.deleted.push(name);
        state.cacheNames = state.cacheNames.filter((n) => n !== name);
        return true;
      },
    },
    console: { ...console, log() {} },
  });
  return state;
}

test('?offline=1 remembers the choice and registers the worker', async t => {
  const state = page(t, { search: '?offline=1' });
  const { syncOfflineMode } = await load(offlineSource);
  assert.equal(await syncOfflineMode(), true);
  assert.deepEqual(state.registered, ['./sw.js']);
  assert.equal(state.store.get('wenilla:offline'), '1');
});

test('a remembered choice re-registers without the switch', async t => {
  const state = page(t, { flag: '1' });
  const { syncOfflineMode } = await load(offlineSource);
  assert.equal(await syncOfflineMode(), true);
  assert.deepEqual(state.registered, ['./sw.js']);
});

test('?offline=0 forgets, unregisters only this worker and deletes only its caches', async t => {
  const state = page(t, {
    search: '?offline=0',
    flag: '1',
    registrations: [`${ORIGIN}/sw.js`, `${ORIGIN}/other-worker.js`],
    cacheNames: ['wenilla-offline-shell-v1', 'wenilla-offline-data-v1', 'unrelated'],
  });
  const { syncOfflineMode } = await load(offlineSource);
  assert.equal(await syncOfflineMode(), false);
  assert.deepEqual(state.registered, []);
  assert.deepEqual(state.unregistered, [`${ORIGIN}/sw.js`]);
  assert.deepEqual(state.deleted, ['wenilla-offline-shell-v1', 'wenilla-offline-data-v1']);
  assert.equal(state.store.has('wenilla:offline'), false);
});

test('off by default: nothing is registered', async t => {
  const state = page(t);
  const { syncOfflineMode } = await load(offlineSource);
  assert.equal(await syncOfflineMode(), false);
  assert.deepEqual(state.registered, []);
});

test('?offline=1 still works for the visit when storage is blocked', async t => {
  const state = page(t, { search: '?offline=1' });
  globalThis.localStorage.setItem = () => { throw new Error('blocked'); };
  globalThis.localStorage.getItem = () => { throw new Error('blocked'); };
  const { syncOfflineMode } = await load(offlineSource);
  assert.equal(await syncOfflineMode(), true);
  assert.deepEqual(state.registered, ['./sw.js']);
});

test('the status line counts the stored game files', async t => {
  page(t, { cacheNames: ['wenilla-offline-data-v1', 'wenilla-offline-shell-v1'] });
  const { status } = await load(offlineSource);
  assert.equal(await status(), '3 game files stored, 5 MB used of 1000 MB');
});

// What ships is the `cp` line in web-build.sh (AGENTS.md): without both files in web/dist the
// switch would register a worker that 404s.
test('web-build.sh copies the worker and its switch into web/dist', async () => {
  const script = await readFile(new URL('../../scripts/web-build.sh', import.meta.url), 'utf8');
  const cp = script.split('\n').find((line) => line.startsWith('cp web/index.html'));
  assert.match(cp, / web\/offline\.js /);
  assert.match(cp, / web\/sw\.js /);
});
