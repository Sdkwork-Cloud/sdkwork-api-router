import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('server development workspace smoke exposes a root-entrypoint-backed launch plan', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-server-dev-workspace.mjs')).href,
  );

  assert.equal(typeof module.createServerDevWorkspaceSmokePlan, 'function');
  assert.equal(typeof module.runServerDevWorkspaceSmoke, 'function');

  const plan = module.createServerDevWorkspaceSmokePlan({
    workspaceRoot,
    platform: 'win32',
    env: {},
    binds: {
      gatewayBind: '127.0.0.1:19080',
      adminBind: '127.0.0.1:19081',
      portalBind: '127.0.0.1:19082',
      webBind: '127.0.0.1:13001',
    },
    siteTargets: {
      adminSiteTarget: '127.0.0.1:15173',
      portalSiteTarget: '127.0.0.1:15174',
    },
    databaseUrl: 'sqlite:///tmp/sdkwork-router-server-dev-workspace.db',
  });

  assert.equal(plan.launchStep.command, process.execPath);
  assert.deepEqual(plan.launchStep.args, [
    path.join(workspaceRoot, 'scripts', 'run-router-product.mjs'),
    'server',
    '--gateway-bind',
    '127.0.0.1:19080',
    '--admin-bind',
    '127.0.0.1:19081',
    '--portal-bind',
    '127.0.0.1:19082',
    '--web-bind',
    '127.0.0.1:13001',
    '--admin-site-target',
    '127.0.0.1:15173',
    '--portal-site-target',
    '127.0.0.1:15174',
    '--database-url',
    'sqlite:///tmp/sdkwork-router-server-dev-workspace.db',
  ]);
  assert.equal(plan.launchStep.shell, false);

  assert.deepEqual(plan.healthChecks.map(({ id }) => id), [
    'unified-gateway-health',
    'direct-gateway-health',
    'direct-admin-health',
    'direct-portal-health',
    'direct-gateway-openapi',
    'direct-admin-openapi',
    'direct-portal-openapi',
  ]);
  assert.equal(plan.routeChecks[0].url, 'http://127.0.0.1:13001/admin/');
  assert.deepEqual(plan.routeChecks[0].expectedSelectors, [
    'input[type="email"]',
    'input[type="password"]',
    'button[type="submit"]',
  ]);
  assert.equal(plan.routeChecks[1].url, 'http://127.0.0.1:13001/portal/');
  assert.deepEqual(plan.routeChecks[1].expectedTexts, []);
  assert.deepEqual(plan.routeChecks[1].expectedSelectors, [
    '[data-slot="portal-home-page"]',
    '[data-slot="portal-home-metrics"]',
  ]);
});

test('server development workspace smoke classifies cross-platform bind collision messages', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-server-dev-workspace.mjs')).href,
  );

  assert.equal(typeof module.isServerDevWorkspaceBindConflictError, 'function');
  assert.equal(
    module.isServerDevWorkspaceBindConflictError(
      new Error('listener failed: Address already in use (os error 98)'),
    ),
    true,
  );
  assert.equal(
    module.isServerDevWorkspaceBindConflictError(
      new Error('Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)'),
    ),
    true,
  );
  assert.equal(
    module.isServerDevWorkspaceBindConflictError(
      new Error('router boot failed because the database migration is invalid'),
    ),
    false,
  );
});

test('server development workspace smoke retries once after a bind collision and returns the later successful result', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-server-dev-workspace.mjs')).href,
  );

  assert.equal(typeof module.runServerDevWorkspaceSmokeWithDependencies, 'function');

  const allocatedBinds = [
    {
      gatewayBind: '127.0.0.1:19080',
      adminBind: '127.0.0.1:19081',
      portalBind: '127.0.0.1:19082',
      webBind: '127.0.0.1:19083',
      adminSiteTarget: '127.0.0.1:19084',
      portalSiteTarget: '127.0.0.1:19085',
    },
    {
      gatewayBind: '127.0.0.1:19180',
      adminBind: '127.0.0.1:19181',
      portalBind: '127.0.0.1:19182',
      webBind: '127.0.0.1:19183',
      adminSiteTarget: '127.0.0.1:19184',
      portalSiteTarget: '127.0.0.1:19185',
    },
  ];
  const attemptBinds = [];
  let ensureReadyCalls = 0;
  let delayCalls = 0;

  const result = await module.runServerDevWorkspaceSmokeWithDependencies({
    workspaceRoot,
    platform: 'linux',
    env: {},
    timeoutMs: 5_000,
    maxAttempts: 2,
    retryDelayMs: 0,
    prepareEnv: ({ env }) => env,
    ensureReady: async () => {
      ensureReadyCalls += 1;
    },
    allocateBinds: async () => allocatedBinds[attemptBinds.length],
    attemptRunner: async ({ binds }) => {
      attemptBinds.push(binds.gatewayBind);
      if (attemptBinds.length === 1) {
        throw new Error(
          `workspace launch failed\nserver stderr:\nlistener failed: Address already in use for ${binds.gatewayBind}`,
        );
      }

      return {
        ok: true,
        binds,
      };
    },
    delayImpl: async () => {
      delayCalls += 1;
    },
  });

  assert.equal(ensureReadyCalls, 1);
  assert.equal(delayCalls, 1);
  assert.deepEqual(attemptBinds, ['127.0.0.1:19080', '127.0.0.1:19180']);
  assert.deepEqual(result, {
    ok: true,
    binds: allocatedBinds[1],
  });
});

test('server development workspace smoke surfaces non-bind failures without retrying', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-server-dev-workspace.mjs')).href,
  );

  let allocateCalls = 0;
  let delayCalls = 0;

  await assert.rejects(
    () => module.runServerDevWorkspaceSmokeWithDependencies({
      workspaceRoot,
      platform: 'linux',
      env: {},
      timeoutMs: 5_000,
      maxAttempts: 3,
      retryDelayMs: 0,
      prepareEnv: ({ env }) => env,
      ensureReady: async () => {},
      allocateBinds: async () => {
        allocateCalls += 1;
        return {
          gatewayBind: '127.0.0.1:19280',
          adminBind: '127.0.0.1:19281',
          portalBind: '127.0.0.1:19282',
          webBind: '127.0.0.1:19283',
          adminSiteTarget: '127.0.0.1:19284',
          portalSiteTarget: '127.0.0.1:19285',
        };
      },
      attemptRunner: async () => {
        throw new Error('database bootstrap failed');
      },
      delayImpl: async () => {
        delayCalls += 1;
      },
    }),
    /database bootstrap failed/,
  );

  assert.equal(allocateCalls, 1);
  assert.equal(delayCalls, 0);
});
