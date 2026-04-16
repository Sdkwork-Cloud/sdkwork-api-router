import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('routing profile dialog keeps numeric posture placeholders behind i18n helpers', () => {
  const source = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.doesNotMatch(source, /placeholder="0\.00"/);
  assert.doesNotMatch(source, /placeholder="1200"/);
  assert.match(source, /placeholder=\{t\('Example: 0\.00 maximum spend'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: 1200 ms ceiling'\)\}/);
  assert.match(i18n, /'Example: 0\.00 maximum spend':/);
  assert.match(i18n, /'Example: 1200 ms ceiling':/);
});

test('routing profile dialog keeps strategy labels behind admin i18n helpers', () => {
  const source = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutingProfilesDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.doesNotMatch(source, /label: 'Deterministic priority'/);
  assert.doesNotMatch(source, /label: 'Weighted random'/);
  assert.doesNotMatch(source, /label: 'SLO aware'/);
  assert.doesNotMatch(source, /label: 'Geo affinity'/);
  assert.match(source, /label: t\('Deterministic priority'\)/);
  assert.match(source, /label: t\('Weighted random'\)/);
  assert.match(source, /label: t\('SLO aware'\)/);
  assert.match(source, /label: t\('Geo affinity'\)/);
  assert.match(i18n, /["']Deterministic priority["']:/);
  assert.match(i18n, /["']Weighted random["']:/);
  assert.match(i18n, /["']SLO aware["']:/);
  assert.match(i18n, /["']Geo affinity["']:/);
});
