import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('admin browser runtime smoke exposes a parseable preview build plan', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-admin-browser-runtime.mjs')).href,
  );

  assert.equal(typeof module.createAdminBrowserRuntimeSmokePlan, 'function');
  assert.equal(typeof module.runAdminBrowserRuntimeSmoke, 'function');

  const plan = module.createAdminBrowserRuntimeSmokePlan({
    workspaceRoot,
    adminAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    platform: 'win32',
    env: {},
    previewPort: 4173,
  });

  assert.equal(plan.previewUrl, 'http://127.0.0.1:4173/admin/');
  assert.deepEqual(plan.expectedSelectors, [
    'input[type="email"]',
    'input[type="password"]',
    'button[type="submit"]',
  ]);
  assert.ok(Array.isArray(plan.routeChecks));
  assert.deepEqual(
    plan.routeChecks.map((check) => check.id),
    ['login', 'commercial-unsafe-id'],
  );
  assert.equal(plan.routeChecks[1].url, 'http://127.0.0.1:4173/admin/commercial');
  assert.deepEqual(plan.routeChecks[1].expectedRequestIncludes, [
    '/api/admin/billing/accounts/646979632893840957/ledger',
    '/api/admin/billing/accounts/1950809575122113173/ledger',
  ]);
  assert.deepEqual(plan.routeChecks[1].forbiddenTexts, [
    '646979632893840900',
    '1950809575122113300',
  ]);
  assert.match(plan.buildStep.args.join(' '), /run-vite-cli\.mjs build/);
  assert.match(plan.previewStep.args.join(' '), /run-vite-cli\.mjs preview/);
  assert.match(plan.previewStep.args.join(' '), /--port 4173/);
});

test('admin browser runtime smoke retries after a preview-port bind conflict and returns the later result', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-admin-browser-runtime.mjs')).href,
  );

  assert.equal(typeof module.runAdminBrowserRuntimeSmokeWithDependencies, 'function');

  const attemptedPorts = [];
  let readyCalls = 0;
  let buildCalls = 0;
  let delayCalls = 0;

  const result = await module.runAdminBrowserRuntimeSmokeWithDependencies({
    workspaceRoot,
    adminAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    platform: 'linux',
    env: {},
    timeoutMs: 5_000,
    maxAttempts: 2,
    retryDelayMs: 0,
    ensureReady: async () => {
      readyCalls += 1;
    },
    allocatePreviewPort: async ({ attempt }) => (attempt === 1 ? 4173 : 4175),
    runBuildStep: async () => {
      buildCalls += 1;
    },
    attemptRunner: async ({ plan }) => {
      attemptedPorts.push(plan.previewUrl);
      if (plan.previewUrl.includes(':4173/')) {
        throw new Error('Port 4173 is already in use');
      }

      return {
        previewUrl: plan.previewUrl,
        checks: [{ id: 'login' }],
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
    'http://127.0.0.1:4173/admin/',
    'http://127.0.0.1:4175/admin/',
  ]);
  assert.deepEqual(result, {
    previewUrl: 'http://127.0.0.1:4175/admin/',
    checks: [{ id: 'login' }],
  });
});

test('admin browser runtime smoke surfaces non-bind failures without retrying', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-admin-browser-runtime.mjs')).href,
  );

  let allocationCalls = 0;
  let delayCalls = 0;

  await assert.rejects(
    () => module.runAdminBrowserRuntimeSmokeWithDependencies({
      workspaceRoot,
      adminAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
      platform: 'linux',
      env: {},
      timeoutMs: 5_000,
      maxAttempts: 3,
      retryDelayMs: 0,
      ensureReady: async () => {},
      allocatePreviewPort: async () => {
        allocationCalls += 1;
        return 4173;
      },
      runBuildStep: async () => {},
      attemptRunner: async () => {
        throw new Error('build manifest mismatch');
      },
      delayImpl: async () => {
        delayCalls += 1;
      },
    }),
    /build manifest mismatch/,
  );

  assert.equal(allocationCalls, 1);
  assert.equal(delayCalls, 0);
});
