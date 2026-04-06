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

test('pricing module routes option catalogs and localized surface copy through dedicated zh-CN translations', () => {
  const source = read('packages/sdkwork-router-admin-pricing/src/index.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const pricingTranslations = extractMap(i18n, 'ADMIN_ZH_PRICING_TRANSLATIONS');

  assert.doesNotMatch(source, /label: 'Input token'/);
  assert.doesNotMatch(source, /detail: 'Prompt and ingestion token pricing\.'/);
  assert.doesNotMatch(source, /label: 'Per unit'/);
  assert.doesNotMatch(source, /detail: 'Quantity times unit price\.'/);
  assert.doesNotMatch(source, /const roundingModeOptions = \['none', 'ceil', 'floor', 'half_up'\];/);
  assert.doesNotMatch(source, /`Last sync activated \$\{formatNumber\(report\.activated_plan_count\)\} plan versions and \$\{formatNumber\(report\.activated_rate_count\)\} pricing rows\.`/);
  assert.doesNotMatch(source, /`Last sync skipped \$\{formatNumber\(report\.skipped_plan_count\)\} due planned versions because no rate rows were attached\.`/);

  assert.match(source, /const chargeUnitOptions = useMemo\(\(\) => buildChargeUnitOptions\(t\), \[t\]\);/);
  assert.match(source, /const pricingMethodOptions = useMemo\(\(\) => buildPricingMethodOptions\(t\), \[t\]\);/);
  assert.match(source, /const roundingModeOptions = useMemo\(\(\) => buildRoundingModeOptions\(t\), \[t\]\);/);
  assert.match(source, /\{roundingModeOptions\.map\(\(option\) => \(\s*<option key=\{option\.value\} value=\{option\.value\}>\s*\{option\.label\}/s);
  assert.match(source, /t\(\s*'Last sync activated \{planCount\} plan versions and \{rateCount\} pricing rows\.',\s*\{/);
  assert.match(source, /t\(\s*'Last sync skipped \{count\} due planned versions because no rate rows were attached\.',\s*\{/);

  const expectedPricingKeys = [
    'Pricing plans',
    'Versioned commercial plan headers available to operators.',
    'Charge units',
    'Distinct units already represented in canonical pricing rows.',
    'Billing methods',
    'Settlement methods visible in active pricing definitions.',
    'Due planned versions',
    'Planned versions already inside their effective window and eligible for lifecycle convergence.',
    'Pricing rates',
    'Token pricing and media pricing rows currently maintained.',
    'Synchronize lifecycle',
    'Synchronizing...',
    'Last sync activated {planCount} plan versions and {rateCount} pricing rows.',
    'Last sync skipped {count} due planned versions because no rate rows were attached.',
    'Last sync found no due planned versions that required lifecycle changes.',
    'Token pricing',
    'Token pricing stays explicit for input, output, and cache-related usage.',
    'Media pricing',
    'Media pricing covers images, audio, video, and music with modality-native units.',
    'Charge units define what quantity gets billed in the commercial settlement layer.',
    'Billing methods stay standardized so settlement logic can evolve without schema churn.',
    'Input token',
    'Output token',
    'Cache read token',
    'Cache write token',
    'Request',
    'Image',
    'Audio second',
    'Audio minute',
    'Video second',
    'Video minute',
    'Music track',
    'Character',
    'Storage MB day',
    'Tool call',
    'Unit',
    'Prompt and ingestion token pricing.',
    'Completion and generation token pricing.',
    'Read-side cached token pricing.',
    'Write-side cache population pricing.',
    'Flat request admission or invocation pricing.',
    'Per-image generation pricing.',
    'Per-second audio processing pricing.',
    'Minute-based audio processing pricing.',
    'Per-second video generation pricing.',
    'Minute-based video generation pricing.',
    'Per-track music generation pricing.',
    'Per-character text or OCR pricing.',
    'Storage footprint pricing over time.',
    'Per tool or function invocation pricing.',
    'Fallback commercial unit when no specialized unit applies.',
    'Per unit',
    'Flat',
    'Step',
    'Included then per unit',
    'Quantity times unit price.',
    'One fixed charge per matched operation.',
    'Charge by normalized quantity steps.',
    'Burn included usage before overage pricing.',
    'No rounding',
    'Round up',
    'Round down',
    'Round half up',
  ];

  for (const key of expectedPricingKeys) {
    assert.ok(pricingTranslations.has(key), `expected pricing translation key ${key}`);
    assert.notEqual(
      pricingTranslations.get(key),
      key,
      `expected pricing translation ${key} to be localized instead of English`,
    );
  }
});
