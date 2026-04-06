import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('pricing module keeps editable example copy and status labels behind i18n helpers', () => {
  const source = read('packages/sdkwork-router-admin-pricing/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');

  assert.doesNotMatch(source, /placeholder="retail-pro"/);
  assert.doesNotMatch(source, /placeholder="Retail Pro"/);
  assert.doesNotMatch(source, /placeholder="USD"/);
  assert.doesNotMatch(source, /placeholder="credit"/);
  assert.doesNotMatch(source, /placeholder="token\.input"/);
  assert.doesNotMatch(source, /placeholder="responses"/);
  assert.doesNotMatch(source, /placeholder="gpt-4\.1"/);
  assert.doesNotMatch(source, /placeholder="provider-openai-official"/);
  assert.doesNotMatch(source, /placeholder="Retail text input pricing"/);
  assert.doesNotMatch(source, /<option value="draft">draft<\/option>/);
  assert.doesNotMatch(source, /<option value="planned">planned<\/option>/);
  assert.doesNotMatch(source, /<option value="active">active<\/option>/);
  assert.doesNotMatch(source, /<option value="archived">archived<\/option>/);

  assert.match(source, /placeholder=\{t\('Example: retail-pro'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: Retail Pro'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: USD'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: credit'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: token\.input'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: responses'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: gpt-4\.1'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: provider-openai-official'\)\}/);
  assert.match(source, /placeholder=\{t\('Example: Retail text input pricing'\)\}/);
  assert.match(source, /<option value="draft">\{t\('Draft'\)\}<\/option>/);
  assert.match(source, /<option value="planned">\{t\('Planned'\)\}<\/option>/);
  assert.match(source, /<option value="active">\{t\('Active'\)\}<\/option>/);
  assert.match(source, /<option value="archived">\{t\('Archived'\)\}<\/option>/);

  assert.match(i18n, /'Draft':/);
  assert.match(i18n, /'Planned':/);
  assert.match(i18n, /'Example: retail-pro':/);
  assert.match(i18n, /'Example: Retail Pro':/);
  assert.match(i18n, /'Example: USD':/);
  assert.match(i18n, /'Example: credit':/);
  assert.match(i18n, /'Example: token\.input':/);
  assert.match(i18n, /'Example: responses':/);
  assert.match(i18n, /'Example: provider-openai-official':/);
  assert.match(i18n, /'Example: Retail text input pricing':/);
});
