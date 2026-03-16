import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell exposes a mission strip across authenticated routes', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Mission strip/);
  assert.match(core, /Primary mission/);
  assert.match(core, /Immediate next move/);
  assert.match(core, /Lead risk/);
});
