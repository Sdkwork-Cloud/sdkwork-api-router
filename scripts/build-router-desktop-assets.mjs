#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { assertFrontendBudgets } from './check-router-frontend-budgets.mjs';
import {
  checkFrontendViteConfig,
  ensureFrontendDependenciesReady,
  pnpmProcessSpec,
  pnpmSpawnOptions,
} from './dev/pnpm-launch-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

export function createDesktopAssetBuildPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
} = {}) {
  const nodeCommand = process.execPath;
  const appRoots = [
    path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  ];

  return appRoots.map((cwd) => ({
    cwd,
    command: nodeCommand,
    args: [
      path.join(workspaceRoot, 'scripts', 'dev', 'run-vite-cli.mjs'),
      'build',
    ],
    shell: false,
    windowsHide: platform === 'win32',
  }));
}

async function runBuild(step) {
  runBuildStepWithDependencyRecovery(step);
}

function writeCommandOutput(result) {
  if (result.stdout) {
    process.stdout.write(String(result.stdout));
  }
  if (result.stderr) {
    process.stderr.write(String(result.stderr));
  }
  if (result.error) {
    process.stderr.write(`${String(result.error.stack ?? result.error.message ?? result.error)}\n`);
  }
}

export function shouldReinstallFrontendDependenciesAfterBuildFailure({
  status = 0,
  stdout = '',
  stderr = '',
  errorMessage = '',
} = {}) {
  if ((status ?? 0) === 0) {
    return false;
  }

  const combinedOutput = [stdout, stderr, errorMessage]
    .map((value) => String(value ?? ''))
    .join('\n');

  return /rollup failed to resolve import/i.test(combinedOutput)
    && /node_modules[\\/]/i.test(combinedOutput);
}

function runPnpmInstall(appRoot, {
  env = process.env,
  platform = process.platform,
  execPath = process.execPath,
  spawnSyncImpl = spawnSync,
} = {}) {
  const installProcess = pnpmProcessSpec(['install'], {
    platform,
    execPath,
  });
  const result = spawnSyncImpl(installProcess.command, installProcess.args, {
    ...pnpmSpawnOptions({
      platform,
      env,
      execPath,
      cwd: appRoot,
      stdio: 'pipe',
    }),
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
  writeCommandOutput(result);

  if (result.error) {
    throw result.error;
  }
  if ((result.status ?? 1) !== 0) {
    throw new Error(`pnpm install exited with code ${result.status ?? 1} for ${appRoot}`);
  }
}

function runBuildAttempt(step, {
  env = process.env,
  platform = process.platform,
  spawnSyncImpl = spawnSync,
} = {}) {
  const result = spawnSyncImpl(step.command, step.args, {
    cwd: step.cwd,
    env: step.env ?? env,
    stdio: 'pipe',
    shell: step.shell ?? false,
    windowsHide: step.windowsHide ?? platform === 'win32',
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
  writeCommandOutput(result);
  return result;
}

export function runBuildStepWithDependencyRecovery(step, options = {}) {
  const buildResult = runBuildAttempt(step, options);
  if ((buildResult.status ?? 1) === 0) {
    return;
  }

  if (shouldReinstallFrontendDependenciesAfterBuildFailure({
    status: buildResult.status ?? 1,
    stdout: buildResult.stdout,
    stderr: buildResult.stderr,
    errorMessage: buildResult.error?.message ?? '',
  })) {
    console.warn(
      `[build-router-desktop-assets] build in ${step.cwd} hit a missing frontend dependency; rebuilding node_modules with pnpm install and retrying once`,
    );
    runPnpmInstall(step.cwd, options);
    const retryResult = runBuildAttempt(step, options);
    if ((retryResult.status ?? 1) === 0) {
      return;
    }
    throw new Error(`build in ${step.cwd} exited with code ${retryResult.status ?? 1} after dependency recovery`);
  }

  if (buildResult.error) {
    throw buildResult.error;
  }
  throw new Error(`build in ${step.cwd} exited with code ${buildResult.status ?? 1}`);
}

export async function runPostBuildChecks({
  workspaceRoot = path.resolve(__dirname, '..'),
} = {}) {
  return assertFrontendBudgets({
    workspaceRoot,
  });
}

async function main() {
  const plan = createDesktopAssetBuildPlan();
  for (const step of plan) {
    ensureFrontendDependenciesReady({
      appRoot: step.cwd,
      requiredPackages: ['vite', 'typescript'],
      requiredBinCommands: ['vite', 'tsc'],
      verifyInstalled: () => checkFrontendViteConfig({
        appRoot: step.cwd,
        command: 'build',
      }),
    });
  }

  for (const step of plan) {
    // eslint-disable-next-line no-await-in-loop
    await runBuild(step);
  }

  await runPostBuildChecks();
}

if (__filename === process.argv[1]) {
  main().catch((error) => {
    console.error(`[build-router-desktop-assets] ${error.message}`);
    process.exit(1);
  });
}
