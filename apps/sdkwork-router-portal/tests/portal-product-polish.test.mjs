import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal auth entry exposes a launch-cockpit narrative instead of plain auth copy', () => {
  const authComponents = read('packages/sdkwork-router-portal-auth/src/components/index.tsx');
  const authPages = read('packages/sdkwork-router-portal-auth/src/pages/index.tsx');

  assert.match(authComponents, /Start in four moves/);
  assert.match(authComponents, /Why teams trust this portal/);
  assert.match(authPages, /Preview the first launch path/);
});

test('portal shell keeps a global workspace pulse and help lane visible', () => {
  const core = read('packages/sdkwork-router-portal-core/src/index.tsx');

  assert.match(core, /Workspace pulse/);
  assert.match(core, /Need help\?/);
  assert.match(core, /Command center/);
});

test('dashboard includes an action queue and explicit launch checklist', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Action queue/);
  assert.match(dashboardPage, /Launch checklist/);
  assert.match(dashboardPage, /Production readiness/);
});

test('credits and billing pages expose runway and guardrail decision support', () => {
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');

  assert.match(creditsPage, /Redemption guardrails/);
  assert.match(creditsPage, /Recommended offer/);
  assert.match(billingPage, /Estimated runway/);
  assert.match(billingPage, /Recommended bundle/);
});
