import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadAdminApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-admin-api',
      'src',
      'index.ts',
    ),
  );
}

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function installAdminApiTestEnvironment() {
  const requests = [];
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;
  const previousWindow = globalThis.window;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.router.admin.session-token' ? 'admin-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.window = {
    location: {
      origin: 'http://127.0.0.1:3000',
      port: '3000',
    },
  };
  globalThis.fetch = async (input, init) => {
    requests.push({
      url: String(input),
      method: init?.method ?? 'GET',
      authorization: init?.headers?.authorization ?? init?.headers?.Authorization ?? null,
    });

    return {
      ok: true,
      status: 200,
      async json() {
        return {};
      },
    };
  };

  return {
    requests,
    restore() {
      globalThis.fetch = previousFetch;
      globalThis.localStorage = previousLocalStorage;
      globalThis.window = previousWindow;
    },
  };
}

test('admin marketing api client exposes canonical coupon template, campaign, budget, and redemption methods', async () => {
  const adminApi = loadAdminApi();
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const env = installAdminApiTestEnvironment();

  assert.match(types, /export type CouponTemplateStatus/);
  assert.match(types, /export type CouponDistributionKind/);
  assert.match(types, /export type MarketingCampaignStatus/);
  assert.match(types, /export type CampaignBudgetStatus/);
  assert.match(types, /export type CouponCodeStatus/);
  assert.match(types, /export type CouponReservationStatus/);
  assert.match(types, /export type CouponRedemptionStatus/);
  assert.match(types, /export type CouponRollbackType/);
  assert.match(types, /export interface CouponTemplateRecord/);
  assert.match(types, /export interface MarketingCampaignRecord/);
  assert.match(types, /export interface CampaignBudgetRecord/);
  assert.match(types, /export interface CouponCodeRecord/);
  assert.match(types, /export interface CouponReservationRecord/);
  assert.match(types, /export interface CouponRedemptionRecord/);
  assert.match(types, /export interface CouponRollbackRecord/);

  try {
    await adminApi.listMarketingCouponTemplates();
    await adminApi.saveMarketingCouponTemplate({
      coupon_template_id: 'tpl_launch',
      template_key: 'launch-buffer',
      display_name: 'Launch Buffer',
      status: 'active',
      distribution_kind: 'shared_code',
      benefit: {
        benefit_kind: 'grant_units',
        subsidy_percent: null,
        subsidy_amount_minor: null,
        grant_units: 12000,
        currency_code: null,
      },
      restriction: {
        subject_scope: 'project',
        min_order_amount_minor: null,
        first_order_only: false,
        new_customer_only: true,
        exclusive_group: 'launch',
        stacking_policy: 'exclusive',
        max_redemptions_per_subject: 1,
        eligible_target_kinds: ['coupon_redemption'],
      },
      created_at_ms: 1717171717000,
      updated_at_ms: 1717171717000,
    });
    await adminApi.updateMarketingCouponTemplateStatus('tpl_launch', 'archived');
    await adminApi.listMarketingCampaigns();
    await adminApi.saveMarketingCampaign({
      marketing_campaign_id: 'campaign_launch',
      coupon_template_id: 'tpl_launch',
      display_name: 'Launch Week',
      status: 'active',
      start_at_ms: 1717171717000,
      end_at_ms: 1719773717000,
      created_at_ms: 1717171717000,
      updated_at_ms: 1717171717000,
    });
    await adminApi.updateMarketingCampaignStatus('campaign_launch', 'paused');
    await adminApi.listMarketingCampaignBudgets();
    await adminApi.saveMarketingCampaignBudget({
      campaign_budget_id: 'budget_launch',
      marketing_campaign_id: 'campaign_launch',
      status: 'active',
      total_budget_minor: 200000,
      reserved_budget_minor: 25000,
      consumed_budget_minor: 50000,
      created_at_ms: 1717171717000,
      updated_at_ms: 1717171717000,
    });
    await adminApi.updateMarketingCampaignBudgetStatus('budget_launch', 'closed');
    await adminApi.listMarketingCouponCodes();
    await adminApi.saveMarketingCouponCode({
      coupon_code_id: 'code_launch_a',
      coupon_template_id: 'tpl_launch',
      code_value: 'LAUNCHA',
      status: 'available',
      claimed_subject_scope: 'project',
      claimed_subject_id: 'project_launch',
      expires_at_ms: 1719773717000,
      created_at_ms: 1717171717000,
      updated_at_ms: 1717171717000,
    });
    await adminApi.updateMarketingCouponCodeStatus('code_launch_a', 'disabled');
    await adminApi.listMarketingCouponReservations();
    await adminApi.listMarketingCouponRedemptions();
    await adminApi.listMarketingCouponRollbacks();

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/admin/marketing/coupon-templates',
        '/api/admin/marketing/coupon-templates',
        '/api/admin/marketing/coupon-templates/tpl_launch/status',
        '/api/admin/marketing/campaigns',
        '/api/admin/marketing/campaigns',
        '/api/admin/marketing/campaigns/campaign_launch/status',
        '/api/admin/marketing/budgets',
        '/api/admin/marketing/budgets',
        '/api/admin/marketing/budgets/budget_launch/status',
        '/api/admin/marketing/codes',
        '/api/admin/marketing/codes',
        '/api/admin/marketing/codes/code_launch_a/status',
        '/api/admin/marketing/reservations',
        '/api/admin/marketing/redemptions',
        '/api/admin/marketing/rollbacks',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.method),
      ['GET', 'POST', 'POST', 'GET', 'POST', 'POST', 'GET', 'POST', 'POST', 'GET', 'POST', 'POST', 'GET', 'GET', 'GET'],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(15).fill('Bearer admin-session-token'),
    );
  } finally {
    env.restore();
  }
});
