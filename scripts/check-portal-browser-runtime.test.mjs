import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('portal browser runtime smoke exposes home and unsafe-id commercial route checks', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-portal-browser-runtime.mjs')).href,
  );

  assert.equal(typeof module.createPortalBrowserRuntimeSmokePlan, 'function');
  assert.equal(typeof module.runPortalBrowserRuntimeSmoke, 'function');

  const plan = module.createPortalBrowserRuntimeSmokePlan({
    workspaceRoot,
    portalAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
    platform: 'win32',
    env: {},
    previewPort: 4174,
  });

  assert.equal(plan.previewUrl, 'http://127.0.0.1:4174/portal/');
  assert.deepEqual(plan.expectedTexts, [
    'Unified AI gateway workspace',
    'Operate routing, credentials, usage, and downloads from one product surface.',
  ]);
  assert.deepEqual(plan.expectedSelectors, [
    '[data-slot="portal-home-page"]',
    '[data-slot="portal-home-metrics"]',
  ]);
  assert.ok(Array.isArray(plan.routeChecks));
  assert.deepEqual(
    plan.routeChecks.map((check) => check.id),
    ['home', 'account-unsafe-id', 'billing-unsafe-id', 'settlements-unsafe-id'],
  );
  assert.deepEqual(
    plan.routeChecks.slice(1).map((check) => check.url),
    [
      'http://127.0.0.1:4174/portal/console/account',
      'http://127.0.0.1:4174/portal/console/billing',
      'http://127.0.0.1:4174/portal/console/settlements',
    ],
  );
  assert.ok(
    plan.routeChecks.slice(1).every((check) =>
      check.expectedTexts.includes('1950809575122113173')),
    'commercial portal routes must assert the exact high-bit account id',
  );
  assert.ok(
    plan.routeChecks.slice(1).every((check) =>
      check.forbiddenTexts.includes('1950809575122113300')),
    'commercial portal routes must reject the rounded account id',
  );
  assert.match(plan.buildStep.args.join(' '), /run-vite-cli\.mjs build/);
  assert.match(plan.previewStep.args.join(' '), /run-vite-cli\.mjs preview/);
  assert.match(plan.previewStep.args.join(' '), /--port 4174/);
});
