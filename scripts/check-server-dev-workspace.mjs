#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import {
  readJsonResponse,
  runBrowserRuntimeSmoke,
} from './browser-runtime-smoke.mjs';
import {
  checkFrontendViteConfig,
  ensureFrontendDependenciesReady,
} from './dev/pnpm-launch-lib.mjs';
import { withSupportedWindowsCmakeGenerator } from './run-tauri-cli.mjs';
import {
  withManagedWorkspaceTargetDir,
  withManagedWorkspaceTempDir,
} from './workspace-target-dir.mjs';
import {
  allocateAvailableTcpPorts,
  createChildFailureWatcher,
  raceAgainstChildFailure,
  resolvePositiveInteger,
  runWithBindConflictRetry,
  isBindConflictError,
} from './smoke-bind-retry-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DEFAULT_TIMEOUT_MS = 180_000;
const HEALTH_POLL_TIMEOUT_MS = 4_000;
const ADMIN_EXPECTED_SELECTORS = [
  'input[type="email"]',
  'input[type="password"]',
  'button[type="submit"]',
];
const PORTAL_EXPECTED_TEXTS = [];
const PORTAL_EXPECTED_SELECTORS = [
  '[data-slot="portal-home-page"]',
  '[data-slot="portal-home-metrics"]',
];

function truncateText(value, maxLength = 1200) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 15))}...[truncated]`;
}

function sqliteUrlForPath(databasePath) {
  const normalized = path.resolve(databasePath).replaceAll('\\', '/');
  return normalized.startsWith('/')
    ? `sqlite://${normalized}`
    : `sqlite:///${normalized}`;
}

function managedWorkspaceEnv({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  env = process.env,
} = {}) {
  return withManagedWorkspaceTargetDir({
    workspaceRoot,
    platform,
    env: withManagedWorkspaceTempDir({
      workspaceRoot,
      platform,
      env: withSupportedWindowsCmakeGenerator(env, platform),
    }),
  });
}

function killProcessTree(child, platform = process.platform) {
  if (!child?.pid) {
    return;
  }

  if (platform === 'win32') {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    });
    return;
  }

  child.kill('SIGTERM');
}

async function allocateSmokeBinds() {
  const [gatewayPort, adminPort, portalPort, webPort, adminSitePort, portalSitePort] =
    await allocateAvailableTcpPorts(6);

  return {
    gatewayBind: `127.0.0.1:${gatewayPort}`,
    adminBind: `127.0.0.1:${adminPort}`,
    portalBind: `127.0.0.1:${portalPort}`,
    webBind: `127.0.0.1:${webPort}`,
    adminSiteTarget: `127.0.0.1:${adminSitePort}`,
    portalSiteTarget: `127.0.0.1:${portalSitePort}`,
  };
}

export const isServerDevWorkspaceBindConflictError = isBindConflictError;

async function waitForHttpCheck(check, timeoutMs = DEFAULT_TIMEOUT_MS) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      const response = await fetch(check.url, {
        signal: AbortSignal.timeout(HEALTH_POLL_TIMEOUT_MS),
      });
      if (!response.ok) {
        throw new Error(`${check.url} returned HTTP ${response.status}`);
      }

      if (check.kind === 'openapi') {
        const payload = await readJsonResponse(response);
        if (typeof payload?.openapi !== 'string' || payload.openapi.trim().length === 0) {
          throw new Error(`${check.url} did not expose an OpenAPI document`);
        }
      }

      return {
        id: check.id,
        url: check.url,
        kind: check.kind,
      };
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await delay(500);
    }
  }

  throw new Error(
    `${check.id} did not become ready within ${timeoutMs}ms: ${lastError?.message ?? 'unknown error'}`,
  );
}

function waitForChildExit(child, timeoutMs = 15_000) {
  return new Promise((resolve) => {
    let settled = false;
    const timeout = setTimeout(() => {
      if (settled) {
        return;
      }
      settled = true;
      resolve({
        exited: false,
        code: null,
        signal: null,
      });
    }, timeoutMs);

    child.once('exit', (code, signal) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timeout);
      resolve({
        exited: true,
        code,
        signal,
      });
    });
  });
}

export function createServerDevWorkspaceSmokePlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  env = process.env,
  binds,
  siteTargets,
  databaseUrl,
  stopFile = '',
} = {}) {
  if (!binds?.gatewayBind || !binds?.adminBind || !binds?.portalBind || !binds?.webBind) {
    throw new Error('binds.gatewayBind, binds.adminBind, binds.portalBind, and binds.webBind are required');
  }
  if (!siteTargets?.adminSiteTarget || !siteTargets?.portalSiteTarget) {
    throw new Error('siteTargets.adminSiteTarget and siteTargets.portalSiteTarget are required');
  }
  if (!databaseUrl) {
    throw new Error('databaseUrl is required');
  }

  const launchArgs = [
    path.join(workspaceRoot, 'scripts', 'run-router-product.mjs'),
    'server',
    '--gateway-bind',
    binds.gatewayBind,
    '--admin-bind',
    binds.adminBind,
    '--portal-bind',
    binds.portalBind,
    '--web-bind',
    binds.webBind,
    '--admin-site-target',
    siteTargets.adminSiteTarget,
    '--portal-site-target',
    siteTargets.portalSiteTarget,
    '--database-url',
    databaseUrl,
  ];

  if (stopFile) {
    launchArgs.push('--stop-file', stopFile);
  }

  return {
    workspaceRoot,
    platform,
    env,
    binds,
    siteTargets,
    databaseUrl,
    launchStep: {
      label: 'server development workspace',
      command: process.execPath,
      args: launchArgs,
      cwd: workspaceRoot,
      env,
      shell: false,
      windowsHide: platform === 'win32',
    },
    healthChecks: [
      {
        id: 'unified-gateway-health',
        kind: 'health',
        url: `http://${binds.webBind}/api/v1/health`,
      },
      {
        id: 'direct-gateway-health',
        kind: 'health',
        url: `http://${binds.gatewayBind}/health`,
      },
      {
        id: 'direct-admin-health',
        kind: 'health',
        url: `http://${binds.adminBind}/admin/health`,
      },
      {
        id: 'direct-portal-health',
        kind: 'health',
        url: `http://${binds.portalBind}/portal/health`,
      },
      {
        id: 'direct-gateway-openapi',
        kind: 'openapi',
        url: `http://${binds.gatewayBind}/openapi.json`,
      },
      {
        id: 'direct-admin-openapi',
        kind: 'openapi',
        url: `http://${binds.adminBind}/admin/openapi.json`,
      },
      {
        id: 'direct-portal-openapi',
        kind: 'openapi',
        url: `http://${binds.portalBind}/portal/openapi.json`,
      },
    ],
    routeChecks: [
      {
        id: 'unified-admin-login',
        url: `http://${binds.webBind}/admin/`,
        expectedTexts: [],
        expectedSelectors: ADMIN_EXPECTED_SELECTORS,
        forbiddenTexts: [],
        expectedRequestIncludes: [],
        setupScript: '',
      },
      {
        id: 'unified-portal-home',
        url: `http://${binds.webBind}/portal/`,
        expectedTexts: PORTAL_EXPECTED_TEXTS,
        expectedSelectors: PORTAL_EXPECTED_SELECTORS,
        forbiddenTexts: [],
        expectedRequestIncludes: [],
        setupScript: '',
      },
    ],
  };
}

export async function runServerDevWorkspaceSmoke({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  env = process.env,
  timeoutMs = resolvePositiveInteger(
    env.SDKWORK_SERVER_DEV_WORKSPACE_SMOKE_TIMEOUT_MS,
    DEFAULT_TIMEOUT_MS,
  ),
} = {}) {
  return await runServerDevWorkspaceSmokeWithDependencies({
    workspaceRoot,
    platform,
    env,
    timeoutMs,
  });
}

async function ensureServerDevWorkspaceReady({
  workspaceRoot,
  platform,
  env,
}) {
  for (const appRoot of [
    path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  ]) {
    ensureFrontendDependenciesReady({
      appRoot,
      requiredPackages: ['vite', 'typescript', 'jiti'],
      requiredBinCommands: ['vite', 'tsc'],
      verifyInstalled: () => checkFrontendViteConfig({
        appRoot,
        command: 'serve',
        env,
        platform,
      }),
      platform,
      env,
    });
  }
}

async function runServerDevWorkspaceSmokeAttempt({
  workspaceRoot,
  platform,
  env,
  timeoutMs,
  binds,
}) {
  const baseEnv = env;
  const runtimeRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-server-dev-workspace-'));
  const stopFile = path.join(runtimeRoot, 'workspace.stop');
  const databaseUrl = sqliteUrlForPath(path.join(runtimeRoot, 'sdkwork-api-router.db'));
  const plan = createServerDevWorkspaceSmokePlan({
    workspaceRoot,
    platform,
    env: baseEnv,
    binds: {
      gatewayBind: binds.gatewayBind,
      adminBind: binds.adminBind,
      portalBind: binds.portalBind,
      webBind: binds.webBind,
    },
    siteTargets: {
      adminSiteTarget: binds.adminSiteTarget,
      portalSiteTarget: binds.portalSiteTarget,
    },
    databaseUrl,
    stopFile,
  });

  const child = spawn(plan.launchStep.command, plan.launchStep.args, {
    cwd: plan.launchStep.cwd,
    env: plan.launchStep.env,
    stdio: 'pipe',
    shell: plan.launchStep.shell,
    windowsHide: plan.launchStep.windowsHide,
  });
  let stdout = '';
  let stderr = '';
  const childFailureWatcher = createChildFailureWatcher(child, {
    label: 'server development workspace',
  });

  child.stdout?.on('data', (chunk) => {
    stdout += String(chunk);
  });
  child.stderr?.on('data', (chunk) => {
    stderr += String(chunk);
  });

  try {
    const healthChecks = [];
    for (const check of plan.healthChecks) {
      // eslint-disable-next-line no-await-in-loop
      healthChecks.push(await raceAgainstChildFailure(
        waitForHttpCheck(check, timeoutMs),
        childFailureWatcher,
      ));
    }

    const routeChecks = [];
    for (const routeCheck of plan.routeChecks) {
      // eslint-disable-next-line no-await-in-loop
      routeChecks.push(await raceAgainstChildFailure(
        runBrowserRuntimeSmoke({
          url: routeCheck.url,
          expectedTexts: routeCheck.expectedTexts,
          expectedSelectors: routeCheck.expectedSelectors,
          forbiddenTexts: routeCheck.forbiddenTexts,
          expectedRequestIncludes: routeCheck.expectedRequestIncludes,
          setupScript: routeCheck.setupScript,
          timeoutMs,
          platform,
          env: baseEnv,
        }),
        childFailureWatcher,
      ));
    }

    return {
      binds: {
        gatewayBind: binds.gatewayBind,
        adminBind: binds.adminBind,
        portalBind: binds.portalBind,
        webBind: binds.webBind,
      },
      siteTargets: {
        adminSiteTarget: binds.adminSiteTarget,
        portalSiteTarget: binds.portalSiteTarget,
      },
      databaseUrl,
      healthChecks,
      routeChecks,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(
      `${message}\nserver stdout:\n${truncateText(stdout)}\nserver stderr:\n${truncateText(stderr)}`,
    );
  } finally {
    childFailureWatcher.stop();

    try {
      writeFileSync(stopFile, 'stop\n', 'utf8');
    } catch {
      // best effort cooperative shutdown
    }

    const exitResult = await waitForChildExit(child, 10_000);
    if (!exitResult.exited) {
      killProcessTree(child, platform);
      await delay(500).catch(() => {});
    }

    rmSync(runtimeRoot, { recursive: true, force: true });
  }
}

export async function runServerDevWorkspaceSmokeWithDependencies({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  env = process.env,
  timeoutMs = resolvePositiveInteger(
    env.SDKWORK_SERVER_DEV_WORKSPACE_SMOKE_TIMEOUT_MS,
    DEFAULT_TIMEOUT_MS,
  ),
  maxAttempts = resolvePositiveInteger(
    env.SDKWORK_SERVER_DEV_WORKSPACE_SMOKE_BIND_RETRY_ATTEMPTS,
    3,
  ),
  retryDelayMs = resolvePositiveInteger(
    env.SDKWORK_SERVER_DEV_WORKSPACE_SMOKE_BIND_RETRY_DELAY_MS,
    250,
  ),
  prepareEnv = ({ workspaceRoot: targetWorkspaceRoot, platform: targetPlatform, env: sourceEnv }) => managedWorkspaceEnv({
    workspaceRoot: targetWorkspaceRoot,
    platform: targetPlatform,
    env: sourceEnv,
  }),
  ensureReady = ensureServerDevWorkspaceReady,
  allocateBinds = allocateSmokeBinds,
  attemptRunner = runServerDevWorkspaceSmokeAttempt,
  delayImpl = delay,
} = {}) {
  const baseEnv = prepareEnv({
    workspaceRoot,
    platform,
    env,
  });

  await ensureReady({
    workspaceRoot,
    platform,
    env: baseEnv,
  });

  return await runWithBindConflictRetry({
    label: 'check-server-dev-workspace',
    maxAttempts,
    retryDelayMs,
    delayImpl,
    allocate: async ({ attempt, maxAttempts: attemptLimit }) =>
      await allocateBinds({ attempt, maxAttempts: attemptLimit }),
    run: async ({ allocation: binds }) =>
      await attemptRunner({
        workspaceRoot,
        platform,
        env: baseEnv,
        timeoutMs,
        binds,
      }),
    shouldRetry: isServerDevWorkspaceBindConflictError,
  });
}

async function main() {
  const result = await runServerDevWorkspaceSmoke();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
