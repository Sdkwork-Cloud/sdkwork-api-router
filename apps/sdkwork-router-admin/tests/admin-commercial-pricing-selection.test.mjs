import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadCommercialPricing() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-core',
      'src',
      'commercialPricing.ts',
    ),
  );
}

test('admin commercial pricing selection prefers currently effective active plans over future active versions', () => {
  const { selectPrimaryCommercialPricingPlan } = loadCommercialPricing();
  const now = 1_717_171_730_000;

  const primaryPlan = selectPrimaryCommercialPricingPlan(
    [
      {
        pricing_plan_id: 9102,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 2,
        display_name: 'Workspace Retail Future',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: now + 86_400_000,
        effective_to_ms: null,
        created_at_ms: now - 1000,
        updated_at_ms: now + 1000,
      },
      {
        pricing_plan_id: 9101,
        tenant_id: 1001,
        organization_id: 2002,
        plan_code: 'workspace-retail',
        plan_version: 1,
        display_name: 'Workspace Retail Current',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        effective_from_ms: now - 86_400_000,
        effective_to_ms: now + 86_400_000,
        created_at_ms: now - 2000,
        updated_at_ms: now,
      },
    ],
    now,
  );

  assert.equal(primaryPlan?.pricing_plan_id, 9101);
  assert.equal(primaryPlan?.display_name, 'Workspace Retail Current');
});
