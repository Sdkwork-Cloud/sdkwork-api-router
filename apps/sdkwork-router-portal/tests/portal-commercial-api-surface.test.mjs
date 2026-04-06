import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadPortalApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-portal-api',
      'src',
      'index.ts',
    ),
  );
}

function loadBillingRepository() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-billing',
      'src',
      'repository',
      'index.ts',
    ),
  );
}

function installPortalApiTestEnvironment(responseMap = {}) {
  const requests = [];
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;
  const previousWindow = globalThis.window;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.router.portal.session-token' ? 'portal-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.window = {
    location: {
      origin: 'http://127.0.0.1:3001',
      port: '3001',
    },
  };
  globalThis.fetch = async (input, init) => {
    const url = String(input);
    requests.push({
      url,
      method: init?.method ?? 'GET',
      authorization: init?.headers?.authorization ?? init?.headers?.Authorization ?? null,
    });

    const payload = responseMap[url] ?? {};
    return {
      ok: true,
      status: 200,
      async json() {
        return payload;
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

test('portal commercial api client exposes canonical commercial billing methods', async () => {
  const portalApi = loadPortalApi();
  const env = installPortalApiTestEnvironment();

  try {
    await portalApi.getPortalCommercialAccount();
    await portalApi.getPortalCommercialAccountHistory();
    await portalApi.getPortalCommercialAccountBalance();
    await portalApi.listPortalCommercialBenefitLots();
    await portalApi.listPortalCommercialHolds();
    await portalApi.listPortalCommercialRequestSettlements();
    await portalApi.listPortalCommercialPricingPlans();
    await portalApi.listPortalCommercialPricingRates();
    await portalApi.getPortalCommerceOrderCenter();

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/billing/account',
        '/api/portal/billing/account-history',
        '/api/portal/billing/account/balance',
        '/api/portal/billing/account/benefit-lots',
        '/api/portal/billing/account/holds',
        '/api/portal/billing/account/request-settlements',
        '/api/portal/billing/pricing-plans',
        '/api/portal/billing/pricing-rates',
        '/api/portal/commerce/order-center',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(9).fill('Bearer portal-session-token'),
    );
  } finally {
    env.restore();
  }
});

test('portal billing repository collapses commercial workspace reads onto aggregate endpoints', async () => {
  const billingRepository = loadBillingRepository();
  const env = installPortalApiTestEnvironment({
    '/api/portal/billing/summary': {
      project_id: 'project-demo',
      entry_count: 0,
      used_units: 0,
      booked_amount: 0,
      remaining_units: 0,
      exhausted: false,
    },
    '/api/portal/usage/records': [],
    '/api/portal/billing/events/summary': {
      total_events: 0,
      project_count: 0,
      group_count: 0,
      capability_count: 0,
      total_request_count: 0,
      total_units: 0,
      total_input_tokens: 0,
      total_output_tokens: 0,
      total_tokens: 0,
      total_image_count: 0,
      total_audio_seconds: 0,
      total_video_seconds: 0,
      total_music_seconds: 0,
      total_upstream_cost: 0,
      total_customer_charge: 0,
      projects: [],
      groups: [],
      capabilities: [],
      accounting_modes: [],
    },
    '/api/portal/billing/events': [],
    '/api/portal/commerce/catalog': {
      plans: [],
      packs: [],
      recharge_options: [],
      custom_recharge_policy: null,
      coupons: [],
    },
    '/api/portal/commerce/order-center': {
      project_id: 'project-demo',
      membership: {
        membership_id: 'membership-pro',
        project_id: 'project-demo',
        user_id: 'user-demo',
        plan_id: 'pro',
        plan_name: 'Pro',
        price_cents: 9900,
        price_label: '$99.00',
        cadence: 'monthly',
        included_units: 100000,
        status: 'active',
        source: 'workspace_seed',
        activated_at_ms: 1,
        updated_at_ms: 2,
      },
      reconciliation: {
        account_id: 7001,
        last_reconciled_order_id: 'order-0',
        last_reconciled_order_updated_at_ms: 1,
        last_reconciled_order_created_at_ms: 1,
        last_reconciled_at_ms: 1,
        backlog_order_count: 1,
        checkpoint_lag_ms: 1,
        healthy: false,
      },
      orders: [
        {
          order: {
            order_id: 'order-1',
            project_id: 'project-demo',
            user_id: 'user-demo',
            target_kind: 'recharge_pack',
            target_id: 'pack-100k',
            target_name: '100k Pack',
            list_price_cents: 1000,
            payable_price_cents: 1000,
            list_price_label: '$10.00',
            payable_price_label: '$10.00',
            granted_units: 100000,
            bonus_units: 0,
            status: 'refunded',
            source: 'workspace_seed',
            created_at_ms: 1,
            updated_at_ms: 5,
          },
          payment_events: [
            {
              payment_event_id: 'payevt-order-1-settled',
              order_id: 'order-1',
              project_id: 'project-demo',
              user_id: 'user-demo',
              provider: 'stripe',
              provider_event_id: 'evt_stripe_1',
              dedupe_key: 'stripe_evt_1',
              event_type: 'settled',
              payload_json: '{"amount_cents":1000}',
              processing_status: 'processed',
              processing_message: null,
              received_at_ms: 3,
              processed_at_ms: 4,
              order_status_after: 'fulfilled',
            },
          ],
          latest_payment_event: {
            payment_event_id: 'payevt-order-1-settled',
            order_id: 'order-1',
            project_id: 'project-demo',
            user_id: 'user-demo',
            provider: 'stripe',
            provider_event_id: 'evt_stripe_1',
            dedupe_key: 'stripe_evt_1',
            event_type: 'settled',
            payload_json: '{"amount_cents":1000}',
            processing_status: 'processed',
            processing_message: null,
            received_at_ms: 3,
            processed_at_ms: 4,
            order_status_after: 'fulfilled',
          },
          checkout_session: {
            order_id: 'order-1',
            order_status: 'refunded',
            session_status: 'refunded',
            provider: 'manual_lab',
            mode: 'closed',
            reference: 'PAY-order-1',
            payable_price_label: '$10.00',
            guidance: 'refunded',
            methods: [],
          },
        },
      ],
    },
    '/api/portal/billing/account-history': {
      account: {
        account_id: 7001,
        tenant_id: 1001,
        organization_id: 2002,
        user_id: 9001,
        account_type: 'primary',
        currency_code: 'USD',
        credit_unit_code: 'credit',
        status: 'active',
        allow_overdraft: false,
        overdraft_limit: 0,
        created_at_ms: 1,
        updated_at_ms: 2,
      },
      balance: {
        account_id: 7001,
        available_balance: 150,
        held_balance: 10,
        consumed_balance: 40,
        grant_balance: 240,
        active_lot_count: 1,
        lots: [],
      },
      benefit_lots: [{ lot_id: 8001 }],
      holds: [{ hold_id: 8101 }],
      request_settlements: [{ request_settlement_id: 8301 }],
      ledger: [],
    },
    '/api/portal/billing/pricing-plans': [{ pricing_plan_id: 9101 }],
    '/api/portal/billing/pricing-rates': [{ pricing_rate_id: 9201 }],
  });

  try {
    const result = await billingRepository.loadBillingPageData();

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/portal/billing/summary',
        '/api/portal/usage/records',
        '/api/portal/billing/events/summary',
        '/api/portal/billing/events',
        '/api/portal/commerce/catalog',
        '/api/portal/commerce/order-center',
        '/api/portal/billing/account-history',
        '/api/portal/billing/pricing-plans',
        '/api/portal/billing/pricing-rates',
      ],
    );
    assert.equal(result.orders.length, 1);
    assert.equal(result.orders[0].order_id, 'order-1');
    assert.equal(result.membership?.membership_id, 'membership-pro');
    assert.equal(result.payment_history.length, 2);
    assert.equal(result.payment_history[0].row_kind, 'refunded_order_state');
    assert.equal(result.payment_history[0].order_id, 'order-1');
    assert.equal(result.payment_history[0].event_type, 'refunded');
    assert.equal(result.payment_history[0].provider, 'manual_lab');
    assert.equal(result.payment_history[0].checkout_reference, 'PAY-order-1');
    assert.equal(result.payment_history[1].row_kind, 'payment_event');
    assert.equal(result.payment_history[1].payment_event_id, 'payevt-order-1-settled');
    assert.equal(result.payment_history[1].provider, 'stripe');
    assert.equal(result.payment_history[1].provider_event_id, 'evt_stripe_1');
    assert.equal(result.refund_history.length, 1);
    assert.equal(result.refund_history[0].row_kind, 'refunded_order_state');
    assert.equal(result.refund_history[0].event_type, 'refunded');
    assert.equal(result.commercial_reconciliation?.account_id, 7001);
    assert.equal(result.commercial_reconciliation?.backlog_order_count, 1);
    assert.equal(result.commercial_reconciliation?.healthy, false);
    assert.equal(result.commercial_account.account.account_id, 7001);
    assert.equal(result.commercial_balance.account_id, 7001);
    assert.equal(result.commercial_benefit_lots.length, 1);
    assert.equal(result.commercial_holds.length, 1);
    assert.equal(result.commercial_request_settlements.length, 1);
    assert.equal(result.commercial_pricing_plans.length, 1);
    assert.equal(result.commercial_pricing_rates.length, 1);
  } finally {
    env.restore();
  }
});
