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

test('apirouter usage shell copy is overridden by the dedicated zh-CN surface slice', () => {
  const usageSource = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const i18n = read('packages/sdkwork-router-admin-core/src/i18n.tsx');
  const apirouterSurfaceTranslations = extractMap(
    i18n,
    'ADMIN_ZH_APIROUTER_SURFACE_TRANSLATIONS',
  );

  const expectedKeys = [
    'Compiled snapshot',
    'Fallback reason',
    'Request settlements',
    '{amount} charge',
    '{amount} upstream',
    'Active plans',
    'Audio sec',
    'Billing events',
    'Capability mix',
    'Captured credits',
    'Charge distribution across routed multimodal capabilities.',
    'Commercial pricing plans and rates stay visible alongside usage analytics and billing events.',
    'Compare platform credit, BYOK, and passthrough posture.',
    'Customer charge',
    'Event-level chargeback stays aligned with the active usage filters.',
    'Export billing events CSV',
    'Group chargeback',
    'Images',
    'Multimodal signals',
    'Music sec',
    'No accounting-mode breakdown is available yet.',
    'No billing events match the current filters.',
    'No capability billing mix is available yet.',
    'No recent billing events yet',
    'None',
    'Operators can audit profile application, compiled snapshots, and fallback posture without leaving usage review.',
    'Operators can inspect hold-to-settlement posture without leaving multimodal usage review.',
    'Plans',
    'Pricing posture',
    'Rates',
    'Recent billing events',
    'Recent billing events appear once routed requests create billable multimodal traffic.',
    'Recent billing events keep multimodal chargeback, provider cost, and routing evidence in one operator review table.',
    'Routing evidence',
    'Signal',
    'Top API key groups by visible customer charge.',
    'Total accounts',
    'Track token, image, audio, video, and music exposure from routed billing events.',
    'Ungrouped',
    'Usage review now stays anchored to the canonical commercial account inventory.',
    'Video sec',
  ];

  for (const key of expectedKeys) {
    assert.match(
      usageSource,
      buildTranslationUsagePattern(key),
      `expected gateway usage page to render ${key} through t(...)`,
    );
    assert.ok(
      apirouterSurfaceTranslations.has(key),
      `expected dedicated apirouter surface translation key ${key}`,
    );
    assert.notEqual(
      apirouterSurfaceTranslations.get(key),
      key,
      `expected apirouter surface translation ${key} to be localized instead of English`,
    );
  }
});
