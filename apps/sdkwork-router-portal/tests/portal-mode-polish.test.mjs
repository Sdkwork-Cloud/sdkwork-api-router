import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell exposes workspace mode and global quick actions', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Workspace mode/);
  assert.match(core, /Quick actions/);
  assert.match(core, /Why now/);
});

test('dashboard exposes a mode narrative and explicit decision path', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Mode narrative/);
  assert.match(dashboardPage, /Decision path/);
});
