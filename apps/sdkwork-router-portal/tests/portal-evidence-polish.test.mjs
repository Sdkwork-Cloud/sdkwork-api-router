import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell exposes a recent-activity evidence rail', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Recent activity/);
  assert.match(core, /Latest evidence/);
  assert.match(core, /Last request/);
});

test('dashboard exposes evidence timeline and confidence signals', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Evidence timeline/);
  assert.match(dashboardPage, /Confidence signals/);
});
