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
