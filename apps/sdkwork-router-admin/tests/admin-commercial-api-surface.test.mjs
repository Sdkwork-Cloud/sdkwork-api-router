import assert from 'node:assert/strict';
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

test('admin commercial api client exposes canonical investigation methods', async () => {
  const adminApi = loadAdminApi();
  const env = installAdminApiTestEnvironment();

  try {
    await adminApi.listCommercialAccounts();
    await adminApi.getCommercialAccountBalance(7001);
    await adminApi.listCommercialAccountBenefitLots(7001);
    await adminApi.listCommercialAccountLedger(7001);
    await adminApi.listCommercialAccountHolds();
    await adminApi.listCommercialRequestSettlements();
    await adminApi.listCommercialPricingPlans();
    await adminApi.listCommercialPricingRates();
    await adminApi.listRecentCommerceOrders(12);
    await adminApi.listCommercePaymentEvents('order-1');
    await adminApi.getCommerceOrderAudit('order-1');

    assert.deepEqual(
      env.requests.map((request) => request.url),
      [
        '/api/admin/billing/accounts',
        '/api/admin/billing/accounts/7001/balance',
        '/api/admin/billing/accounts/7001/benefit-lots',
        '/api/admin/billing/accounts/7001/ledger',
        '/api/admin/billing/account-holds',
        '/api/admin/billing/request-settlements',
        '/api/admin/billing/pricing-plans',
        '/api/admin/billing/pricing-rates',
        '/api/admin/commerce/orders?limit=12',
        '/api/admin/commerce/orders/order-1/payment-events',
        '/api/admin/commerce/orders/order-1/audit',
      ],
    );
    assert.deepEqual(
      env.requests.map((request) => request.authorization),
      Array(11).fill('Bearer admin-session-token'),
    );
  } finally {
    env.restore();
  }
});

test('admin commercial api client preserves backend order-audit lookup errors', async () => {
  const adminApi = loadAdminApi();
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
  globalThis.fetch = async () => ({
    ok: false,
    status: 404,
    async json() {
      return {
        error: {
          message: 'commerce order order-missing not found',
        },
      };
    },
  });

  try {
    await assert.rejects(
      adminApi.getCommerceOrderAudit('order-missing'),
      /commerce order order-missing not found/,
    );
  } finally {
    globalThis.fetch = previousFetch;
    globalThis.localStorage = previousLocalStorage;
    globalThis.window = previousWindow;
  }
});
