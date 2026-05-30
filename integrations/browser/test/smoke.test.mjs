import assert from 'node:assert/strict';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import test from 'node:test';

import { initChktex, lint, lintBytes, toEditorMarkers } from '../chktex.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const nodePkg = pathToFileURL(path.join(root, 'pkg-node/chktex_wasm.js')).href;

test('lints latex via wasm (nodejs target)', async () => {
  const { version } = await initChktex({ pkgUrl: nodePkg });
  assert.match(version(), /^\d+\./);

  const result = await lint('\\foo trailing space\n', {
    filename: 'doc.tex',
  });

  assert.ok(result.warnings >= 1);
  assert.ok(result.diagnostics.some((d) => d.number === 1));
  assert.ok(result.output.includes('Warning 1'));
  assert.equal(typeof result.toJSON(), 'string');

  const markers = toEditorMarkers(result.diagnostics);
  assert.ok(markers.length >= 1);
  assert.equal(markers[0].startLineNumber, result.diagnostics[0].line);
});

test('lintBytes accepts Uint8Array', async () => {
  const bytes = new TextEncoder().encode('\\foo x\n');
  const result = await lintBytes(bytes, { filename: 'bytes.tex' });
  assert.ok(result.diagnostics.length >= 1);
});
