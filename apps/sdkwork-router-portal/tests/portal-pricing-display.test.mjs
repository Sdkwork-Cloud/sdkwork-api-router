import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal pricing surface exposes rich commercial pricing semantics for tenant-facing billing posture', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');

  assert.match(portalTypes, /export type CommercialPricingChargeUnit/);
  assert.match(portalTypes, /'input_token'/);
  assert.match(portalTypes, /'image'/);
  assert.match(portalTypes, /'video_minute'/);
  assert.match(portalTypes, /'music_track'/);
  assert.match(portalTypes, /export type CommercialPricingMethod/);
  assert.match(portalTypes, /'per_unit'/);
  assert.match(portalTypes, /'flat'/);
  assert.match(portalTypes, /'included_then_per_unit'/);
  assert.match(portalTypes, /capability_code\?: string \| null;/);
  assert.match(portalTypes, /charge_unit: CommercialPricingChargeUnit;/);
  assert.match(portalTypes, /pricing_method: CommercialPricingMethod;/);
  assert.match(portalTypes, /display_price_unit: string;/);
  assert.match(portalTypes, /minimum_charge: number;/);
  assert.match(portalTypes, /included_quantity: number;/);
  assert.match(portalTypes, /status: string;/);
  assert.match(portalTypes, /updated_at_ms: number;/);
  assert.match(portalTypes, /effective_from_ms: number;/);
  assert.match(portalTypes, /effective_to_ms\?: number \| null;/);

  assert.match(billingPage, /Pricing posture/);
  assert.match(billingPage, /Billing method/);
  assert.match(billingPage, /Effective from/);
  assert.match(billingPage, /Effective to/);
  assert.match(billingPage, /Input token/);
  assert.match(billingPage, /Image/);
  assert.match(billingPage, /Music track/);
  assert.match(billingPage, /USD \/ 1M input tokens/);
});
