import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

test('browser runtime smoke exposes a parseable CLI contract and Chromium launch plan', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createBrowserRuntimeSmokePlan, 'function');
  assert.equal(typeof module.resolveChromiumBrowserExecutable, 'function');

  const options = module.parseArgs([
    '--url',
    'http://127.0.0.1:4174/portal/',
    '--expected-text',
    'Unified AI gateway workspace',
    '--expected-text',
    'Operate routing, credentials, usage, and downloads from one product surface.',
    '--timeout-ms',
    '45000',
  ]);

  assert.deepEqual(options, {
    url: 'http://127.0.0.1:4174/portal/',
    expectedTexts: [
      'Unified AI gateway workspace',
      'Operate routing, credentials, usage, and downloads from one product surface.',
    ],
    expectedSelectors: [],
    timeoutMs: 45000,
    browserPath: '',
  });

  const plan = module.createBrowserRuntimeSmokePlan({
    ...options,
    browserPath: 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    platform: 'win32',
  });

  assert.equal(plan.url, options.url);
  assert.deepEqual(plan.expectedTexts, options.expectedTexts);
  assert.deepEqual(plan.expectedSelectors, []);
  assert.equal(plan.timeoutMs, 45000);
  assert.equal(plan.browserCommand, 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe');
  assert.ok(plan.remoteDebuggingPort > 0);
  assert.match(plan.userDataDir, /sdkwork-browser-smoke-/i);
  assert.ok(
    plan.browserArgs.includes('--headless=new'),
    'browser smoke must run in headless mode',
  );
  assert.ok(
    plan.browserArgs.includes(`--remote-debugging-port=${plan.remoteDebuggingPort}`),
    'browser smoke must expose the debugging port for CDP inspection',
  );
});

test('browser runtime smoke rejects missing required inputs', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.throws(() => module.parseArgs([]), /--url is required/i);
  assert.throws(
    () =>
      module.parseArgs([
        '--url',
        'http://127.0.0.1:4174/portal/',
      ]),
    /--expected-text or --expected-selector is required/i,
  );
});

test('browser runtime smoke can resolve a Chromium executable from a provided candidate list', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  const browserPath = module.resolveChromiumBrowserExecutable({
    platform: 'win32',
    candidatePaths: [
      'C:/missing/msedge.exe',
      'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    ],
    exists: (candidate) => candidate.includes('/Program Files (x86)/Microsoft/Edge/Application/'),
  });

  assert.equal(
    browserPath,
    'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  );
});

test('browser runtime smoke preserves unsafe integers when polling JSON endpoints', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  const payload = await module.readJsonResponse(
    new Response(
      '{"webSocketDebuggerUrl":"ws://127.0.0.1/devtools/page/1","unsafe_marker":9007199254740993}',
      {
        status: 200,
        headers: {
          'content-type': 'application/json',
        },
      },
    ),
  );

  assert.equal(payload.unsafe_marker, '9007199254740993');
});

test('browser runtime smoke plan preserves setup scripts, forbidden texts, and expected request fragments', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.createMockFetchSetupScript, 'function');

  const setupScript = module.createMockFetchSetupScript({
    localStorageEntries: {
      'sdkwork-router-portal.user-center.session-token': 'portal-token',
    },
    exactResponses: {
      '/api/portal/workspace': {
        tenant: { id: 'tenant-1', name: 'Tenant 1' },
      },
    },
    patternResponses: [{
      pattern: '^/api/admin/billing/accounts/646979632893840957/ledger$',
      body: [],
    }],
  });

  const plan = module.createBrowserRuntimeSmokePlan({
    url: 'http://127.0.0.1:4174/portal/console/account',
    expectedTexts: ['1950809575122113173'],
    expectedSelectors: ['[data-slot="portal-account-page"]'],
    forbiddenTexts: ['1950809575122113300'],
    expectedRequestIncludes: ['/api/admin/billing/accounts/646979632893840957/ledger'],
    setupScript,
    browserPath: 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    platform: 'win32',
  });

  assert.deepEqual(plan.forbiddenTexts, ['1950809575122113300']);
  assert.deepEqual(plan.expectedRequestIncludes, [
    '/api/admin/billing/accounts/646979632893840957/ledger',
  ]);
  assert.equal(plan.setupScript, setupScript);
  assert.match(setupScript, /sdkwork-router-portal\.user-center\.session-token/);
  assert.match(setupScript, /\/api\/portal\/workspace/);
  assert.match(setupScript, /646979632893840957/);
});

test('browser runtime smoke hardens Linux CI launch plans for hosted Chromium startup', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  const plan = module.createBrowserRuntimeSmokePlan({
    url: 'http://127.0.0.1:3001/admin/',
    expectedSelectors: ['input[type="email"]'],
    browserPath: '/usr/bin/google-chrome',
    platform: 'linux',
    env: {
      GITHUB_ACTIONS: 'true',
    },
  });

  assert.equal(plan.devtoolsTimeoutMs, 30000);
  assert.ok(plan.browserArgs.includes('--no-sandbox'));
  assert.ok(plan.browserArgs.includes('--disable-dev-shm-usage'));
});

test('browser runtime smoke formats detailed JavaScript exception diagnostics', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.formatBrowserExceptionDetails, 'function');

  const message = module.formatBrowserExceptionDetails({
    exceptionDetails: {
      text: 'Uncaught',
      url: 'http://127.0.0.1:3001/admin/@fs/D:/workspace/src/lib/utils.ts',
      lineNumber: 0,
      columnNumber: 24,
      exception: {
        description: [
          "SyntaxError: The requested module '/admin/@fs/D:/workspace/node_modules/clsx/dist/clsx.js' does not provide an export named 'clsx'",
          '    at http://127.0.0.1:3001/admin/@fs/D:/workspace/src/lib/utils.ts:1:25',
        ].join('\n'),
      },
    },
  });

  assert.match(
    message,
    /SyntaxError: The requested module .* does not provide an export named 'clsx'/,
  );
  assert.match(message, /utils\.ts:1:25/);
});

test('browser runtime smoke falls back to compact exception text when richer details are unavailable', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(
    module.formatBrowserExceptionDetails({
      exceptionDetails: {
        text: 'Uncaught ReferenceError: missingValue is not defined',
      },
    }),
    'Uncaught ReferenceError: missingValue is not defined',
  );
});

test('browser runtime smoke timeout diagnostics include recent JavaScript exceptions when present', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.formatBrowserTargetTimeoutDetails, 'function');

  const message = module.formatBrowserTargetTimeoutDetails({
    snapshot: {
      title: 'SDKWork Router Admin',
      bodyText: '',
      matchedTexts: [],
      matchedSelectors: [],
      expectedTexts: [],
      expectedSelectors: ['input[type="email"]'],
      readyState: 'complete',
    },
    exceptions: [
      "SyntaxError: The requested module '/admin/@fs/D:/workspace/node_modules/react-remove-scroll-bar/dist/es5/constants.js' does not provide an export named 'fullWidthClassName'",
      "TypeError: Cannot read properties of undefined (reading 'pathname')",
    ],
  });

  assert.match(
    message,
    /browser runtime smoke did not observe the expected runtime markers before timeout/i,
  );
  assert.match(message, /SDKWork Router Admin/);
  assert.match(message, /JavaScript exceptions observed before timeout/i);
  assert.match(message, /fullWidthClassName/);
  assert.match(message, /TypeError: Cannot read properties of undefined/);
});

test('browser runtime smoke retries after a devtools bind conflict and returns the later result', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  assert.equal(typeof module.runBrowserRuntimeSmokeWithDependencies, 'function');

  const attemptedPorts = [];
  let delayCalls = 0;

  const result = await module.runBrowserRuntimeSmokeWithDependencies({
    url: 'http://127.0.0.1:4174/portal/',
    expectedTexts: ['Unified AI gateway workspace'],
    expectedSelectors: [],
    browserPath: '/usr/bin/google-chrome',
    platform: 'linux',
    env: {},
    timeoutMs: 5_000,
    maxAttempts: 2,
    retryDelayMs: 0,
    allocateRemoteDebuggingPort: async ({ attempt }) => (attempt === 1 ? 9222 : 9223),
    createUserDataDir: ({ attempt }) => `tmp/browser-smoke-${attempt}`,
    attemptRunner: async ({ plan }) => {
      attemptedPorts.push(plan.remoteDebuggingPort);
      if (plan.remoteDebuggingPort === 9222) {
        throw new Error('DevTools listener failed: Address already in use');
      }

      return {
        ok: true,
        remoteDebuggingPort: plan.remoteDebuggingPort,
      };
    },
    delayImpl: async () => {
      delayCalls += 1;
    },
  });

  assert.deepEqual(attemptedPorts, [9222, 9223]);
  assert.equal(delayCalls, 1);
  assert.deepEqual(result, {
    ok: true,
    remoteDebuggingPort: 9223,
  });
});

test('browser runtime smoke surfaces non-bind failures without retrying', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'browser-runtime-smoke.mjs')).href,
  );

  let allocationCalls = 0;
  let delayCalls = 0;

  await assert.rejects(
    () => module.runBrowserRuntimeSmokeWithDependencies({
      url: 'http://127.0.0.1:4174/portal/',
      expectedTexts: ['Unified AI gateway workspace'],
      expectedSelectors: [],
      browserPath: '/usr/bin/google-chrome',
      platform: 'linux',
      env: {},
      timeoutMs: 5_000,
      maxAttempts: 3,
      retryDelayMs: 0,
      allocateRemoteDebuggingPort: async () => {
        allocationCalls += 1;
        return 9222;
      },
      createUserDataDir: () => 'tmp/browser-smoke',
      attemptRunner: async () => {
        throw new Error('page markers never rendered');
      },
      delayImpl: async () => {
        delayCalls += 1;
      },
    }),
    /page markers never rendered/,
  );

  assert.equal(allocationCalls, 1);
  assert.equal(delayCalls, 0);
});
