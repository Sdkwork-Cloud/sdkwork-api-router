import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function extractMap(source, name) {
  const start = source.indexOf(`const ${name}: Record<string, string> = {`);
  assert.notEqual(start, -1, `missing map ${name}`);

  const open = source.indexOf('{', start);
  const close = source.indexOf('\n};', open);
  assert.notEqual(close, -1, `missing closing brace for ${name}`);

  const body = source.slice(open + 1, close);
  return new Map(
    [...body.matchAll(/\n\s*"([^"]+)":\s*(?:"([^"]*)"|\n\s*"([^"]*)"),/g)].map((match) => [
      match[1],
      match[2] ?? match[3] ?? '',
    ]),
  );
}

function buildTranslationUsagePattern(key) {
  const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`t\\(\\s*'${escapedKey}'\\s*(?:,|\\))`, 's');
}

test('pricing display units and settings shell label are localized instead of falling back to English literals', () => {
  const pricingHelpers = read('packages/sdkwork-router-admin-core/src/commercialPricing.ts');
  const settingsSource = read('packages/sdkwork-router-admin-settings/src/Settings.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const pricingTranslations = extractMap(i18n, 'ADMIN_ZH_PRICING_TRANSLATIONS');

  const pricingKeys = [
    '{count} x {unit}',
    'USD / 1M input tokens',
    'USD / image',
    'USD / input token',
    'USD / music track',
    'USD / request',
  ];

  for (const key of pricingKeys) {
    assert.match(
      pricingHelpers,
      buildTranslationUsagePattern(key),
      `expected commercial pricing helpers to render ${key} through t(...)`,
    );
    assert.ok(
      pricingTranslations.has(key),
      `expected pricing translation key ${key}`,
    );
    assert.notEqual(
      pricingTranslations.get(key),
      key,
      `expected pricing translation ${key} to be localized instead of English`,
    );
  }

  assert.match(settingsSource, buildTranslationUsagePattern('Shell'));
  assert.match(
    i18n,
    /^\s*Shell:\s*'(?!Shell')[^']+'/m,
    'expected Shell label to be translated instead of falling back to English',
  );
});
