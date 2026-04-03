import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('downloads center exposes a productized delivery surface with metrics, deployment tracks, and rollout loop', () => {
  const downloadsPage = read('packages/sdkwork-router-portal-downloads/src/index.tsx');

  assert.match(downloadsPage, /PortalSiteHero/);
  assert.match(downloadsPage, /PortalSiteMetricCard/);
  assert.match(downloadsPage, /portal-downloads-metrics/);
  assert.match(downloadsPage, /portal-downloads-deployment-tracks/);
  assert.match(downloadsPage, /portal-downloads-rollout-loop/);
  assert.match(downloadsPage, /Launch the runtime your team will actually operate\./);
  assert.match(downloadsPage, /Desktop, background service, and shared gateway distributions stay connected to docs, console, and onboarding actions from one software center\./);
  assert.match(downloadsPage, /Deployment tracks/);
  assert.match(downloadsPage, /Choose the software path that matches local operators, background automation, or shared gateway delivery\./);
  assert.match(downloadsPage, /Rollout loop/);
  assert.match(downloadsPage, /Installation, launch, verification, and console handoff stay connected in one software delivery surface\./);
});
