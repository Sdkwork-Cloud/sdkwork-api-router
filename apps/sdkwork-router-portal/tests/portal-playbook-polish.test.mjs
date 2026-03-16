import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell exposes an operating rhythm for independent users', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Operating rhythm/);
  assert.match(core, /Before traffic/);
  assert.match(core, /If risk appears/);
});

test('dashboard exposes review cadence and playbook lanes', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Review cadence/);
  assert.match(dashboardPage, /Playbook lane/);
});
