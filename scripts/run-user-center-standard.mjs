#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, '..');

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

export function resolveUserCenterStandardTestFile({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
} = {}) {
  return path.join(
    resolvedWorkspaceRoot,
    'apps',
    'sdkwork-router-portal',
    'tests',
    'portal-user-center-standard.test.mjs',
  );
}

export function resolveServerUserCenterContractTestFile({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
} = {}) {
  return path.join(
    resolvedWorkspaceRoot,
    'scripts',
    'router-product-service-user-center-contract.test.mjs',
  );
}

export function resolveServerUserCenterEntrypointContractTestFile({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
} = {}) {
  return path.join(
    resolvedWorkspaceRoot,
    'scripts',
    'server-user-center-entrypoint-contract.test.mjs',
  );
}

export function resolveSdkworkAppbaseContractsRunner({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
  sdkworkAppbaseRoot = process.env.SDKWORK_APPBASE_ROOT,
} = {}) {
  const resolvedAppbaseRoot = sdkworkAppbaseRoot
    ? path.resolve(sdkworkAppbaseRoot)
    : path.join(resolvedWorkspaceRoot, '..', 'sdkwork-appbase');

  return path.join(
    resolvedAppbaseRoot,
    'scripts',
    'run-user-center-standard-contracts.mjs',
  );
}

export function createUserCenterStandardTestPlan({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
  cwd = resolvedWorkspaceRoot,
  env = process.env,
  nodeExecutable = process.execPath,
  platform = process.platform,
} = {}) {
  return {
    command: nodeExecutable,
    args: [resolveUserCenterStandardTestFile({ workspaceRoot: resolvedWorkspaceRoot })],
    cwd,
    env,
    shell: false,
    windowsHide: platform === 'win32',
  };
}

export function createUserCenterStandardCommandPlan({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
  sdkworkAppbaseRoot = process.env.SDKWORK_APPBASE_ROOT,
  cwd = resolvedWorkspaceRoot,
  env = process.env,
  nodeExecutable = process.execPath,
  platform = process.platform,
} = {}) {
  const appbaseRunner = resolveSdkworkAppbaseContractsRunner({
    workspaceRoot: resolvedWorkspaceRoot,
    sdkworkAppbaseRoot,
  });
  const portalPlan = createUserCenterStandardTestPlan({
    workspaceRoot: resolvedWorkspaceRoot,
    cwd,
    env,
    nodeExecutable,
    platform,
  });

  return [
    {
      label: 'sdkwork-appbase user-center standard contracts',
      command: nodeExecutable,
      args: [appbaseRunner],
      cwd,
      env,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      ...portalPlan,
      label: 'router portal user-center standard',
    },
    {
      label: 'router product service user-center contract',
      command: nodeExecutable,
      args: [resolveServerUserCenterContractTestFile({ workspaceRoot: resolvedWorkspaceRoot })],
      cwd,
      env,
      shell: false,
      windowsHide: platform === 'win32',
    },
    {
      label: 'router deployment user-center entrypoint contract',
      command: nodeExecutable,
      args: [resolveServerUserCenterEntrypointContractTestFile({
        workspaceRoot: resolvedWorkspaceRoot,
      })],
      cwd,
      env,
      shell: false,
      windowsHide: platform === 'win32',
    },
  ];
}

function buildCommandFailure(plan, result) {
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

  return new Error(
    `${plan.label ?? 'user-center standard step'} failed with exit code ${result?.status ?? 'unknown'} while executing ${plan.command} ${plan.args.join(' ')}${fragments.length > 0 ? `\n${fragments.join('\n')}` : ''}`,
  );
}

export function runUserCenterStandardTest({
  workspaceRoot: resolvedWorkspaceRoot = workspaceRoot,
  cwd = resolvedWorkspaceRoot,
  env = process.env,
  nodeExecutable = process.execPath,
  platform = process.platform,
  spawnSyncImpl = spawnSync,
} = {}) {
  const commandPlan = createUserCenterStandardCommandPlan({
    workspaceRoot: resolvedWorkspaceRoot,
    cwd,
    env,
    nodeExecutable,
    platform,
  });

  const results = [];

  for (const plan of commandPlan) {
    const result = spawnSyncImpl(plan.command, plan.args, {
      cwd: plan.cwd,
      env: plan.env,
      shell: plan.shell,
      stdio: 'inherit',
      windowsHide: plan.windowsHide,
    });

    if (result?.error || result?.status !== 0) {
      throw buildCommandFailure(plan, result);
    }

    results.push(result);
  }

  return results;
}

function isDirectExecution({
  argv1 = process.argv[1] ?? '',
  moduleFile = __filename,
  platform = process.platform,
} = {}) {
  if (!argv1) {
    return false;
  }

  const resolvedArgv1 = path.resolve(argv1);
  const resolvedModuleFile = path.resolve(moduleFile);
  if (platform === 'win32') {
    return resolvedArgv1.toLowerCase() === resolvedModuleFile.toLowerCase();
  }

  return resolvedArgv1 === resolvedModuleFile;
}

if (isDirectExecution()) {
  runUserCenterStandardTest();
}
