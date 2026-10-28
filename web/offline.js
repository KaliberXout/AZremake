// offline.js — the switch for the opt-in offline mode (web/sw.js has the what and the why).
//
//   ?offline=1   turn it on: remember it, register sw.js, store the page and the boot data
//   ?offline=0   turn it off: unregister sw.js and delete everything it stored
//
// The choice is remembered in localStorage, so later visits need no switch; a visit without one
// re-registers (which is how an edited sw.js gets picked up) or, when off, removes a worker left
// behind. Like platform.js this is imported dynamically, unawaited and never load-bearing: a
// browser without service workers, or a failure here, plays exactly as before.

const FLAG = 'wenilla:offline';
const PREFIX = 'wenilla-offline-'; // sw.js's cache names
const WORKER = 'sw.js';

export async function syncOfflineMode({ search = location.search, scriptUrl = './' + WORKER } = {}) {
  if (!('serviceWorker' in navigator)) return false;
  const q = new URLSearchParams(search).get('offline');
  if (q === '1') remember(true);
  else if (q === '0') remember(false);
  const on = q === '1' || (q !== '0' && remembered());
  try {
    if (on) {
      navigator.serviceWorker.addEventListener('message', onProgress);
      await navigator.serviceWorker.register(scriptUrl);
      await navigator.serviceWorker.ready;
      console.log(`offline: on — ${await status()} (?offline=0 turns it off)`);
      return true;
    }
    await removeWorker(q === '0');
  } catch (e) {
    console.warn('offline:', e);
  }
  return false;
}

// What is stored, for the console line: the number of game files and the origin's footprint.
export async function status() {
  let files = 0;
  for (const name of await caches.keys()) {
    if (name.startsWith(PREFIX + 'data-')) files += (await (await caches.open(name)).keys()).length;
  }
  const est = await navigator.storage?.estimate?.().catch(() => null);
  const mb = (n) => `${Math.round(n / 1e6)} MB`;
  return `${files} game files stored` + (est ? `, ${mb(est.usage)} used of ${mb(est.quota)}` : '');
}

async function removeWorker(announce) {
  let removed = false;
  for (const reg of await navigator.serviceWorker.getRegistrations()) {
    const script = (reg.active || reg.waiting || reg.installing)?.scriptURL ?? '';
    if (script.endsWith('/' + WORKER)) removed = (await reg.unregister()) || removed;
  }
  for (const name of await caches.keys()) {
    if (name.startsWith(PREFIX)) removed = (await caches.delete(name)) || removed;
  }
  if (removed || announce) console.log('offline: off — stored files deleted');
}

function onProgress(event) {
  const { type, done, total } = event.data ?? {};
  if (type === 'wenilla-offline') console.log(`offline: storing boot data ${done}/${total}`);
}

function remembered() {
  try { return localStorage.getItem(FLAG) === '1'; } catch (_) { return false; }
}

function remember(on) {
  try {
    if (on) localStorage.setItem(FLAG, '1');
    else localStorage.removeItem(FLAG);
  } catch (_) { /* storage blocked: the switch then lasts for this visit only */ }
}
