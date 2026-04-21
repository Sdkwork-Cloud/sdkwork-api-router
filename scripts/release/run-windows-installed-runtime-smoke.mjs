#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import {
  applyInstallPlan,
  assertInstallInputsExist,
  createInstallPlan,
  renderRuntimeConfigTemplate,
  renderRuntimeEnvTemplate,
} from '../../bin/lib/router-runtime-tooling.mjs';
import {
  assertInstalledReleasePayloadContract,
  assertInstalledRuntimeBackupBundle,
  assertInstalledPackagedBootstrapData,
  createInstalledRuntimeSmokeLayout,
  resolveInstalledBootstrapDataRoot,
} from './installed-runtime-smoke-lib.mjs';
import { resolveDesktopReleaseTarget } from './desktop-targets.mjs';
import {
  findAvailableTcpPort,
  runWithBindConflictRetry,
} from '../smoke-bind-retry-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_WAIT_SECONDS = 120;
const DEFAULT_HEALTH_ATTEMPTS = 12;
const DEFAULT_HEALTH_DELAY_MS = 1000;

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new Error(`${token} requires a value`);
  }

  return next;
}

function resolveRuntimeHome(repoRoot, runtimeHome, { platform, arch }) {
  if (runtimeHome) {
    return path.isAbsolute(runtimeHome)
      ? runtimeHome
      : path.resolve(repoRoot, runtimeHome);
  }

  return path.resolve(repoRoot, 'artifacts', 'release-smoke', `${platform}-${arch}`);
}

function resolveEvidencePath(repoRoot, evidencePath, { platform, arch }) {
  if (evidencePath) {
    return path.isAbsolute(evidencePath)
      ? evidencePath
      : path.resolve(repoRoot, evidencePath);
  }

  return path.resolve(
    repoRoot,
    'artifacts',
    'release-governance',
    `windows-installed-runtime-smoke-${platform}-${arch}.json`,
  );
}

function resolveReleaseOutputDir(repoRoot, releaseOutputDir) {
  if (releaseOutputDir) {
    return path.isAbsolute(releaseOutputDir)
      ? releaseOutputDir
      : path.resolve(repoRoot, releaseOutputDir);
  }

  return path.resolve(repoRoot, 'artifacts', 'release');
}

function assertWindowsRuntimeSmokePorts(ports) {
  for (const key of ['web', 'gateway', 'admin', 'portal']) {
    const value = ports?.[key];
    if (!Number.isInteger(value) || value <= 0) {
      throw new Error(`missing windows runtime smoke port: ${key}`);
    }
  }
}

function renderWindowsInstalledRuntimeSmokeEnvContents({
  runtimeHome,
  ports,
} = {}) {
  assertWindowsRuntimeSmokePorts(ports);

  let contents = renderRuntimeEnvTemplate({
    installRoot: runtimeHome,
    platform: 'win32',
  });

  const replacements = new Map([
    ['SDKWORK_WEB_BIND', `SDKWORK_WEB_BIND="127.0.0.1:${ports.web}"`],
    ['SDKWORK_GATEWAY_BIND', `SDKWORK_GATEWAY_BIND="127.0.0.1:${ports.gateway}"`],
    ['SDKWORK_ADMIN_BIND', `SDKWORK_ADMIN_BIND="127.0.0.1:${ports.admin}"`],
    ['SDKWORK_PORTAL_BIND', `SDKWORK_PORTAL_BIND="127.0.0.1:${ports.portal}"`],
  ]);

  for (const [key, replacement] of replacements.entries()) {
    contents = contents.replace(new RegExp(`^${key}=.*$`, 'm'), replacement);
  }

  return contents;
}

function renderWindowsInstalledRuntimeSmokeConfigContents({
  runtimeHome,
  ports,
} = {}) {
  assertWindowsRuntimeSmokePorts(ports);

  let contents = renderRuntimeConfigTemplate({
    installRoot: runtimeHome,
    platform: 'win32',
  });

  const replacements = new Map([
    ['web_bind', `web_bind: "127.0.0.1:${ports.web}"`],
    ['gateway_bind', `gateway_bind: "127.0.0.1:${ports.gateway}"`],
    ['admin_bind', `admin_bind: "127.0.0.1:${ports.admin}"`],
    ['portal_bind', `portal_bind: "127.0.0.1:${ports.portal}"`],
  ]);

  for (const [key, replacement] of replacements.entries()) {
    contents = contents.replace(new RegExp(`^${key}:.*$`, 'm'), replacement);
  }

  return contents;
}

function truncateText(value, maxLength = 1600) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function quoteForPowerShellSingleQuotedString(value) {
  return String(value).replaceAll("'", "''");
}

function toPortableRelativePath(repoRoot, targetPath) {
  return (path.relative(repoRoot, targetPath) || '.').replaceAll('\\', '/');
}

function readLogExcerpt(filePath, maxLines = 40) {
  if (!existsSync(filePath)) {
    return '';
  }

  const lines = readFileSync(filePath, 'utf8').trim().split(/\r?\n/u);
  return lines.slice(-maxLines).join('\n').trim();
}

function buildFailureContext(plan) {
  const contexts = [];

  const stdoutExcerpt = readLogExcerpt(plan.stdoutLogPath);
  if (stdoutExcerpt) {
    contexts.push(`stdout log (${plan.stdoutLogPath}):\n${truncateText(stdoutExcerpt)}`);
  }

  const stderrExcerpt = readLogExcerpt(plan.stderrLogPath);
  if (stderrExcerpt) {
    contexts.push(`stderr log (${plan.stderrLogPath}):\n${truncateText(stderrExcerpt)}`);
  }

  return contexts.length > 0 ? `\n${contexts.join('\n\n')}` : '';
}

export { assertInstalledReleasePayloadContract, resolveInstalledBootstrapDataRoot };

function buildCommandFailure(label, result, plan) {
  const fragments = [];

  if (result?.error) {
    fragments.push(`error: ${result.error.message}`);
  }

  if (String(result?.stdout ?? '').trim()) {
    fragments.push(`stdout: ${truncateText(result.stdout)}`);
  }

  if (String(result?.stderr ?? '').trim()) {
    fragments.push(`stderr: ${truncateText(result.stderr)}`);
  }

  const exitText = result?.status == null ? 'unknown' : String(result.status);
  return new Error(
    `${label} failed with exit code ${exitText}${fragments.length > 0 ? `\n${fragments.join('\n')}` : ''}${buildFailureContext(plan)}`,
  );
}

function runScriptCommand(command, args, { cwd, env, label, plan, stdio = 'pipe' } = {}) {
  const result = spawnSync(command, args, {
    cwd,
    env,
    encoding: 'utf8',
    shell: false,
    stdio,
  });

  if (result.error || result.status !== 0) {
    throw buildCommandFailure(label, result, plan);
  }

  return result;
}

async function allocateLoopbackPorts() {
  return {
    web: await findAvailableTcpPort(),
    gateway: await findAvailableTcpPort(),
    admin: await findAvailableTcpPort(),
    portal: await findAvailableTcpPort(),
  };
}

async function assertHealthyResponse(url) {
  const urlLiteral = quoteForPowerShellSingleQuotedString(url);
  const result = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-Command',
      [
        '$ErrorActionPreference = \'Stop\'',
        `try { $response = Invoke-WebRequest -UseBasicParsing -Uri '${urlLiteral}' -TimeoutSec 5; Write-Output ('status=' + $response.StatusCode); Write-Output ('body=' + ([string]$response.Content).Trim()) } catch { Write-Error $_.Exception.Message; exit 1 }`,
      ].join('; '),
    ],
    {
      encoding: 'utf8',
    },
  );

  if (result.error || result.status !== 0) {
    throw new Error(
      `${url} health probe failed: ${truncateText(`${result.stdout}${result.stderr}${result.error?.message ?? ''}`, 400)}`,
    );
  }

  const output = String(result.stdout ?? '');
  const status = Number(output.match(/^status=(\d+)$/m)?.[1] ?? 0);
  const body = String(output.match(/^body=(.*)$/m)?.[1] ?? '').trim();

  if (status < 200 || status >= 300) {
    throw new Error(`${url} returned HTTP ${status}: ${truncateText(body, 400)}`);
  }

  if (body.length > 0 && body.toLowerCase() !== 'ok') {
    throw new Error(`${url} returned unexpected body: ${truncateText(body, 400)}`);
  }
}

async function waitForHealthUrls(urls) {
  let lastError = null;

  for (let attempt = 0; attempt < DEFAULT_HEALTH_ATTEMPTS; attempt += 1) {
    try {
      for (const url of urls) {
        // eslint-disable-next-line no-await-in-loop
        await assertHealthyResponse(url);
      }

      return;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (attempt + 1 >= DEFAULT_HEALTH_ATTEMPTS) {
        break;
      }

      // eslint-disable-next-line no-await-in-loop
      await delay(DEFAULT_HEALTH_DELAY_MS);
    }
  }

  throw new Error(
    `installed runtime health checks did not stabilize after ${DEFAULT_HEALTH_ATTEMPTS} attempts: ${lastError?.message ?? 'unknown error'}`,
  );
}

export function createWindowsInstalledRuntimeSmokeOptions({
  repoRoot = rootDir,
  platform = process.platform,
  arch = process.arch,
  target = '',
  releaseOutputDir = '',
  runtimeHome = '',
  evidencePath = '',
} = {}) {
  const resolvedTarget = resolveDesktopReleaseTarget({
    targetTriple: target,
    platform,
    arch,
  });

  if (resolvedTarget.platform !== 'windows') {
    throw new Error('run-windows-installed-runtime-smoke only supports windows release lanes');
  }

  return {
    platform: resolvedTarget.platform,
    arch: resolvedTarget.arch,
    target: resolvedTarget.targetTriple,
    releaseOutputDir: resolveReleaseOutputDir(repoRoot, releaseOutputDir),
    runtimeHome: resolveRuntimeHome(repoRoot, runtimeHome, resolvedTarget),
    evidencePath: resolveEvidencePath(repoRoot, evidencePath, resolvedTarget),
  };
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    platform: '',
    arch: '',
    target: '',
    releaseOutputDir: '',
    runtimeHome: '',
    evidencePath: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--platform') {
      options.platform = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--arch') {
      options.arch = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--target') {
      options.target = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--runtime-home') {
      options.runtimeHome = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--release-output-dir') {
      options.releaseOutputDir = readOptionValue(token, next);
      index += 1;
      continue;
    }

    if (token === '--evidence-path') {
      options.evidencePath = readOptionValue(token, next);
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!options.platform) {
    throw new Error('--platform is required');
  }
  if (!options.arch) {
    throw new Error('--arch is required');
  }
  if (!options.target) {
    throw new Error('--target is required');
  }

  return createWindowsInstalledRuntimeSmokeOptions({
    repoRoot: rootDir,
    ...options,
  });
}

export function createWindowsInstalledRuntimeSmokePlan({
  repoRoot = rootDir,
  platform,
  arch,
  target,
  releaseOutputDir,
  runtimeHome,
  evidencePath,
  env = process.env,
  ports = {
    web: 9983,
    gateway: 9980,
    admin: 9981,
    portal: 9982,
  },
} = {}) {
  const options = createWindowsInstalledRuntimeSmokeOptions({
    repoRoot,
    platform,
    arch,
    target,
    releaseOutputDir,
    runtimeHome,
    evidencePath,
  });

  assertWindowsRuntimeSmokePorts(ports);

  const installPlan = createInstallPlan({
    repoRoot,
    installRoot: options.runtimeHome,
    platform: options.platform,
    arch: options.arch,
    releaseOutputDir: options.releaseOutputDir,
    env: {
      ...env,
      SDKWORK_DESKTOP_TARGET: options.target,
    },
  });
  const runtimeLayout = createInstalledRuntimeSmokeLayout({
    installPlan,
    runtimeHome: options.runtimeHome,
  });

  return {
    ...options,
    installPlan,
    controlHome: runtimeLayout.controlHome,
    backupBundlePath: path.join(options.runtimeHome, 'backup-smoke'),
    routerConfigPath: path.join(runtimeLayout.configDir, 'router.yaml'),
    routerConfigContents: renderWindowsInstalledRuntimeSmokeConfigContents({
      runtimeHome: options.runtimeHome,
      ports,
    }),
    routerEnvPath: path.join(runtimeLayout.configDir, 'router.env'),
    routerEnvContents: renderWindowsInstalledRuntimeSmokeEnvContents({
      runtimeHome: options.runtimeHome,
      ports,
    }),
    startCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(runtimeLayout.controlHome, 'bin', 'start.ps1'),
        '-Home',
        runtimeLayout.controlHome,
        '-WaitSeconds',
        String(DEFAULT_WAIT_SECONDS),
      ],
      stdio: 'ignore',
    },
    stopCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(runtimeLayout.controlHome, 'bin', 'stop.ps1'),
        '-Home',
        runtimeLayout.controlHome,
        '-WaitSeconds',
        String(DEFAULT_WAIT_SECONDS),
      ],
    },
    backupDryRunCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(runtimeLayout.controlHome, 'bin', 'backup.ps1'),
        '-Home',
        runtimeLayout.controlHome,
        '-OutputPath',
        path.join(options.runtimeHome, 'backup-smoke'),
        '-DryRun',
      ],
    },
    backupCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(runtimeLayout.controlHome, 'bin', 'backup.ps1'),
        '-Home',
        runtimeLayout.controlHome,
        '-OutputPath',
        path.join(options.runtimeHome, 'backup-smoke'),
        '-Force',
      ],
    },
    restoreDryRunCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(runtimeLayout.controlHome, 'bin', 'restore.ps1'),
        '-Home',
        runtimeLayout.controlHome,
        '-SourcePath',
        path.join(options.runtimeHome, 'backup-smoke'),
        '-Force',
        '-DryRun',
      ],
    },
    restoreCommand: {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        path.join(runtimeLayout.controlHome, 'bin', 'restore.ps1'),
        '-Home',
        runtimeLayout.controlHome,
        '-SourcePath',
        path.join(options.runtimeHome, 'backup-smoke'),
        '-Force',
      ],
    },
    pidFilePath: path.join(runtimeLayout.runDir, 'router-product-service.pid'),
    stdoutLogPath: path.join(runtimeLayout.logDir, 'router-product-service.stdout.log'),
    stderrLogPath: path.join(runtimeLayout.logDir, 'router-product-service.stderr.log'),
    healthUrls: [
      `http://127.0.0.1:${ports.web}/api/v1/health`,
      `http://127.0.0.1:${ports.web}/api/admin/health`,
      `http://127.0.0.1:${ports.web}/api/portal/health`,
    ],
  };
}

export function createWindowsInstalledRuntimeSmokeEvidence({
  repoRoot = rootDir,
  plan,
  ok,
  failure = null,
} = {}) {
  const stdoutLogExcerpt = readLogExcerpt(plan.stdoutLogPath);
  const stderrLogExcerpt = readLogExcerpt(plan.stderrLogPath);

  const evidence = {
    generatedAt: new Date().toISOString(),
    ok,
    platform: plan.platform,
    arch: plan.arch,
    target: plan.target,
    runtimeHome: toPortableRelativePath(repoRoot, plan.runtimeHome),
    evidencePath: toPortableRelativePath(repoRoot, plan.evidencePath),
    backupBundlePath: toPortableRelativePath(repoRoot, plan.backupBundlePath),
    backupRestoreVerified: Boolean(ok),
    healthUrls: plan.healthUrls,
  };

  if (stdoutLogExcerpt || stderrLogExcerpt) {
    evidence.logs = {};
    if (stdoutLogExcerpt) {
      evidence.logs.stdout = stdoutLogExcerpt;
    }
    if (stderrLogExcerpt) {
      evidence.logs.stderr = stderrLogExcerpt;
    }
  }

  if (!ok) {
    evidence.failure = {
      message: failure instanceof Error ? failure.message : String(failure ?? 'unknown error'),
    };
  }

  return evidence;
}

function writeWindowsInstalledRuntimeSmokeEvidence({
  evidencePath,
  evidence,
} = {}) {
  mkdirSync(path.dirname(evidencePath), { recursive: true });
  writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
}

function hasWindowsInstalledRuntimeSmokeEvidenceFields(plan) {
  return Boolean(
    plan
    && typeof plan.runtimeHome === 'string'
    && typeof plan.evidencePath === 'string'
    && typeof plan.backupBundlePath === 'string'
    && Array.isArray(plan.healthUrls),
  );
}

async function executeWindowsInstalledRuntimeSmokeAttempt({
  repoRoot = rootDir,
  plan,
  env = process.env,
} = {}) {
  try {
    assertInstallInputsExist(plan.installPlan);
    applyInstallPlan(plan.installPlan, {
      force: true,
    });
    assertInstalledPackagedBootstrapData(plan.runtimeHome);
    writeFileSync(plan.routerConfigPath, plan.routerConfigContents, 'utf8');
    writeFileSync(plan.routerEnvPath, plan.routerEnvContents, 'utf8');

    runScriptCommand(plan.startCommand.command, plan.startCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime start.ps1',
      plan,
      stdio: plan.startCommand.stdio,
    });

    await waitForHealthUrls(plan.healthUrls);

    runScriptCommand(plan.stopCommand.command, plan.stopCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime stop.ps1',
      plan,
    });
    runScriptCommand(plan.backupDryRunCommand.command, plan.backupDryRunCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime backup.ps1 dry-run',
      plan,
    });
    runScriptCommand(plan.backupCommand.command, plan.backupCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime backup.ps1',
      plan,
    });
    assertInstalledRuntimeBackupBundle(plan.backupBundlePath);
    runScriptCommand(plan.restoreDryRunCommand.command, plan.restoreDryRunCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime restore.ps1 dry-run',
      plan,
    });
    runScriptCommand(plan.restoreCommand.command, plan.restoreCommand.args, {
      cwd: repoRoot,
      env,
      label: 'installed runtime restore.ps1',
      plan,
    });

    return createWindowsInstalledRuntimeSmokeEvidence({
      repoRoot,
      plan,
      ok: true,
    });
  } catch (error) {
    const failure = error instanceof Error ? error : new Error(String(error));
    throw new Error(`${failure.message}${buildFailureContext(plan)}`);
  } finally {
    if (existsSync(plan.pidFilePath)) {
      try {
        runScriptCommand(plan.stopCommand.command, plan.stopCommand.args, {
          cwd: repoRoot,
          env,
          label: 'installed runtime stop.ps1',
          plan,
        });
      } catch {
        // Preserve the main smoke outcome; stop cleanup is best-effort.
      }
    }
  }
}

export async function runWindowsInstalledRuntimeSmokeWithDependencies({
  repoRoot = rootDir,
  platform,
  arch,
  target,
  releaseOutputDir,
  runtimeHome,
  evidencePath,
  env = process.env,
  maxAttempts = 3,
  retryDelayMs = 250,
  allocatePorts = async () => await allocateLoopbackPorts(),
  createPlan = (options) => createWindowsInstalledRuntimeSmokePlan(options),
  attemptRunner = executeWindowsInstalledRuntimeSmokeAttempt,
  delayImpl = delay,
} = {}) {
  let lastPlan = null;

  try {
    const evidence = await runWithBindConflictRetry({
      label: 'run-windows-installed-runtime-smoke',
      maxAttempts,
      retryDelayMs,
      delayImpl,
      allocate: async ({ attempt, maxAttempts: attemptLimit }) => {
        const ports = await allocatePorts({
          attempt,
          maxAttempts: attemptLimit,
        });
        const plan = createPlan({
          repoRoot,
          platform,
          arch,
          target,
          releaseOutputDir,
          runtimeHome,
          evidencePath,
          env,
          ports,
        });
        lastPlan = plan;
        return plan;
      },
      run: async ({ allocation: plan }) =>
        await attemptRunner({
          repoRoot,
          plan,
          env,
        }),
    });

    if (hasWindowsInstalledRuntimeSmokeEvidenceFields(lastPlan)) {
      writeWindowsInstalledRuntimeSmokeEvidence({
        evidencePath: lastPlan.evidencePath,
        evidence,
      });
    }
    return evidence;
  } catch (error) {
    const failure = error instanceof Error ? error : new Error(String(error));
    if (hasWindowsInstalledRuntimeSmokeEvidenceFields(lastPlan)) {
      writeWindowsInstalledRuntimeSmokeEvidence({
        evidencePath: lastPlan.evidencePath,
        evidence: createWindowsInstalledRuntimeSmokeEvidence({
          repoRoot,
          plan: lastPlan,
          ok: false,
          failure,
        }),
      });
    }
    throw failure;
  }
}

export async function runWindowsInstalledRuntimeSmoke({
  repoRoot = rootDir,
  platform,
  arch,
  target,
  releaseOutputDir,
  runtimeHome,
  evidencePath,
  env = process.env,
} = {}) {
  return await runWindowsInstalledRuntimeSmokeWithDependencies({
    repoRoot,
    platform,
    arch,
    target,
    releaseOutputDir,
    runtimeHome,
    evidencePath,
    env,
  });
}

async function main() {
  const options = parseArgs();
  const evidence = await runWindowsInstalledRuntimeSmoke(options);
  console.log(JSON.stringify(evidence, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
