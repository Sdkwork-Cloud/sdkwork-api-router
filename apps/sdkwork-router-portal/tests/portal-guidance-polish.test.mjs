import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell keeps a persistent launch-journey guide visible', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Launch journey/);
  assert.match(core, /Current blocker/);
  assert.match(core, /Next milestone/);
});

test('dashboard exposes journey progress and milestone map', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Journey progress/);
  assert.match(dashboardPage, /Milestone map/);
});
