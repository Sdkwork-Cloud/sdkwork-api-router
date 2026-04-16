import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('api key group dialog normalizes standard environment labels through admin i18n while preserving custom values', () => {
  const source = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyGroupsDialog.tsx',
  );
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const i18nTranslations = read('packages/sdkwork-router-admin-core/src/i18nTranslations.ts');

  assert.match(source, /function formatEnvironmentOptionLabel\(/);
  assert.match(source, /case 'live':\s*return t\('Live'\);/);
  assert.match(source, /case 'staging':\s*return t\('Staging'\);/);
  assert.match(source, /case 'test':\s*return t\('Test'\);/);
  assert.match(source, /case 'production':\s*return t\('Production'\);/);
  assert.match(source, /case 'development':\s*return t\('Development'\);/);
  assert.match(source, /default:\s*return value;/);
  assert.doesNotMatch(source, /label:\s*value,/);
  assert.match(source, /label: formatEnvironmentOptionLabel\(value, t\),/);

  assert.match(i18n, /["']Environment["']:/);
  assert.match(i18n, /["']Live["']:/);
  assert.match(i18n, /["']Staging["']:/);
  assert.match(i18n, /["']Test["']:/);
  assert.match(i18n, /["']Production["']:/);
  assert.match(i18n, /["']Development["']:/);

  assert.match(i18nTranslations, /"Environment":\s*"环境"/);
  assert.match(i18nTranslations, /"Live":\s*"实时"/);
  assert.match(i18nTranslations, /"Staging":\s*"预发"/);
  assert.match(i18nTranslations, /"Test":\s*"测试"/);
  assert.match(i18nTranslations, /"Production":\s*"生产"/);
  assert.match(i18nTranslations, /"Development":\s*"开发"/);
});
