import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell exposes a daily brief for independent operators', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Daily brief/);
  assert.match(core, /Top focus/);
  assert.match(core, /Risk watch/);
});

test('dashboard exposes focus board and risk watchlist', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Focus board/);
  assert.match(dashboardPage, /Risk watchlist/);
});
