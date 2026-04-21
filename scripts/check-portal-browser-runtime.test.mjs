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

test('portal browser runtime smoke retries after a preview-port bind conflict and returns the later result', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-portal-browser-runtime.mjs')).href,
  );

  assert.equal(typeof module.runPortalBrowserRuntimeSmokeWithDependencies, 'function');

  const attemptedPorts = [];
  let readyCalls = 0;
  let buildCalls = 0;
  let delayCalls = 0;

  const result = await module.runPortalBrowserRuntimeSmokeWithDependencies({
    workspaceRoot,
    portalAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
    platform: 'linux',
    env: {},
    timeoutMs: 5_000,
    maxAttempts: 2,
    retryDelayMs: 0,
    ensureReady: async () => {
      readyCalls += 1;
    },
    allocatePreviewPort: async ({ attempt }) => (attempt === 1 ? 4174 : 4176),
    runBuildStep: async () => {
      buildCalls += 1;
    },
    attemptRunner: async ({ plan }) => {
      attemptedPorts.push(plan.previewUrl);
      if (plan.previewUrl.includes(':4174/')) {
        throw new Error('Port 4174 is already in use');
      }

      return {
        previewUrl: plan.previewUrl,
        checks: [{ id: 'home' }],
      };
    },
    delayImpl: async () => {
      delayCalls += 1;
    },
  });

  assert.equal(readyCalls, 1);
  assert.equal(buildCalls, 1);
  assert.equal(delayCalls, 1);
  assert.deepEqual(attemptedPorts, [
    'http://127.0.0.1:4174/portal/',
    'http://127.0.0.1:4176/portal/',
  ]);
  assert.deepEqual(result, {
    previewUrl: 'http://127.0.0.1:4176/portal/',
    checks: [{ id: 'home' }],
  });
});

test('portal browser runtime smoke surfaces non-bind failures without retrying', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-portal-browser-runtime.mjs')).href,
  );

  let allocationCalls = 0;
  let delayCalls = 0;

  await assert.rejects(
    () => module.runPortalBrowserRuntimeSmokeWithDependencies({
      workspaceRoot,
      portalAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
      platform: 'linux',
      env: {},
      timeoutMs: 5_000,
      maxAttempts: 3,
      retryDelayMs: 0,
      ensureReady: async () => {},
      allocatePreviewPort: async () => {
        allocationCalls += 1;
        return 4174;
      },
      runBuildStep: async () => {},
      attemptRunner: async () => {
        throw new Error('unexpected portal HTML');
      },
      delayImpl: async () => {
        delayCalls += 1;
      },
    }),
    /unexpected portal HTML/,
  );

  assert.equal(allocationCalls, 1);
  assert.equal(delayCalls, 0);
});
