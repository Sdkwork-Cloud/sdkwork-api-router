import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('rate limit dialog keeps route and model placeholders behind i18n helpers', () => {
  const source = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/rate-limits/GatewayRateLimitPolicyDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.doesNotMatch(source, /placeholder="\/v1\/chat\/completions"/);
  assert.doesNotMatch(source, /placeholder="gpt-4\.1"/);
  assert.match(source, /placeholder=\{t\('Example: \/v1\/chat\/completions'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: gpt-4\.1'\)\}/);
  assert.match(i18n, /'Example: \/v1\/chat\/completions':/);
  assert.match(i18n, /'Example: gpt-4\.1':/);
});
