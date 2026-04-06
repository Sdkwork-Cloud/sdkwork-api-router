import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin workbench surfaces richer commercial pricing semantics across commercial and gateway pages', () => {
  const coreIndex = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const commercialPricing = read('packages/sdkwork-router-admin-core/src/commercialPricing.ts');
  const commercialPage = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  const accessPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const workbenchActions = read('packages/sdkwork-router-admin-core/src/workbenchActions.ts');

  assert.match(coreIndex, /commercialPricingChargeUnitLabel/);
  assert.match(coreIndex, /commercialPricingMethodLabel/);
  assert.match(coreIndex, /commercialPricingDisplayUnit/);
  assert.match(coreIndex, /selectPrimaryCommercialPricingRate/);

  assert.match(commercialPricing, /Input token/);
  assert.match(commercialPricing, /USD \/ 1M input tokens/);
  assert.match(commercialPricing, /effective_from_ms/);
  assert.match(commercialPricing, /effective_to_ms/);
  assert.match(workbench, /handleScheduleCommercialPricingPlan/);
  assert.match(workbench, /handleSynchronizeCommercialPricingLifecycle/);
  assert.match(workbenchActions, /handleScheduleCommercialPricingPlan/);
  assert.match(workbenchActions, /handleSynchronizeCommercialPricingLifecycle/);

  assert.match(commercialPage, /Billing method/);
  assert.match(commercialPage, /Price unit/);
  assert.match(commercialPage, /commercialPricingChargeUnitLabel/);
  assert.match(commercialPage, /commercialPricingDisplayUnit/);

  assert.match(accessPage, /Billing method/);
  assert.match(accessPage, /Price unit/);
  assert.match(accessPage, /commercialPricingChargeUnitLabel/);
  assert.match(accessPage, /commercialPricingDisplayUnit/);

  assert.match(usagePage, /Billing method/);
  assert.match(usagePage, /Price unit/);
  assert.match(usagePage, /commercialPricingChargeUnitLabel/);
  assert.match(usagePage, /commercialPricingDisplayUnit/);
});
