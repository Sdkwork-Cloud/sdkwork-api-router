import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('commercial route metadata keys are present in the zh-CN translation catalog', () => {
  const routes = read('packages/sdkwork-router-admin-core/src/routes.ts');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.match(routes, /label:\s*'Commercial'/);
  assert.match(routes, /eyebrow:\s*'Revenue'/);
  assert.match(routes, /detail:\s*'Commercial accounts, settlement explorer, and pricing governance'/);
  assert.match(routes, /label:\s*'Pricing'/);
  assert.match(routes, /eyebrow:\s*'Finops'/);
  assert.match(routes, /detail:\s*'Pricing plans, charge units, and billing method governance'/);

  assert.match(i18n, /'Commercial':/);
  assert.match(i18n, /'Revenue':/);
  assert.match(i18n, /'Commercial accounts, settlement explorer, and pricing governance':/);
  assert.match(i18n, /'Pricing':/);
  assert.match(i18n, /'Finops':/);
  assert.match(i18n, /'Pricing plans, charge units, and billing method governance':/);
});

test('api key group color input keeps placeholder copy behind i18n helpers', () => {
  const source = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyGroupsDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.doesNotMatch(source, /placeholder="#2563eb"/);
  assert.match(source, /placeholder=\{t\('Example: #2563eb'\)\}/);
  assert.match(i18n, /'Example: #2563eb':/);
});
