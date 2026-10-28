import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const web = new URL('../', import.meta.url);
const read = (path, base = web) => readFile(new URL(path, base));
const manifest = JSON.parse(await read('manifest.webmanifest'));

// What Chrome checks before it offers "Install": a name, a start URL, an app display mode, and
// a 192 px and a 512 px icon.
test('the manifest carries what installability needs', () => {
  assert.ok(manifest.name && manifest.short_name);
  assert.equal(manifest.start_url, './');
  assert.ok(['standalone', 'fullscreen', 'minimal-ui'].includes(manifest.display));
  for (const size of ['192x192', '512x512']) {
    assert.ok(manifest.icons.some(i => i.sizes === size && (i.purpose ?? 'any').includes('any')), size);
  }
  assert.ok(manifest.icons.some(i => i.purpose === 'maskable'));
});

// A PNG's width and height sit at bytes 16..24 of its IHDR chunk.
test('every manifest icon exists at its declared size', async () => {
  for (const icon of manifest.icons) {
    const bytes = await read(icon.src);
    if (icon.type === 'image/svg+xml') {
      assert.equal(icon.sizes, 'any');
      assert.match(bytes.toString('utf8'), /<svg[\s>]/);
      continue;
    }
    assert.equal(icon.type, 'image/png');
    assert.equal(bytes.toString('latin1', 1, 4), 'PNG', icon.src);
    assert.equal(`${bytes.readUInt32BE(16)}x${bytes.readUInt32BE(20)}`, icon.sizes, icon.src);
  }
});

test('the dev page links the manifest and its touch icon', async () => {
  const html = (await read('index.html')).toString('utf8');
  assert.match(html, /<link rel="manifest" href="manifest\.webmanifest">/);
  const touch = html.match(/<link rel="apple-touch-icon" href="([^"]+)">/);
  assert.ok(touch);
  await read(touch[1]);
});

// What ships is the `cp` lines in web-build.sh (AGENTS.md): a file not on them never reaches
// web/dist, and the page would link a manifest that 404s.
test('web-build.sh copies the manifest and every icon into web/dist', async () => {
  const script = (await read('scripts/web-build.sh', new URL('../', web))).toString('utf8');
  assert.match(script, /cp web\/manifest\.webmanifest "\$\{DIST\}\/"/);
  assert.match(script, /cp web\/icons\/\*\.png "\$\{DIST\}\/icons\/"/);
  for (const icon of manifest.icons.filter(i => i.src.startsWith('icons/'))) {
    assert.ok(icon.src.endsWith('.png'), `${icon.src} is not covered by icons/*.png`);
  }
});
