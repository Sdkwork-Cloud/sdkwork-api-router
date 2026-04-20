#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { listProductGovernanceNodeTestFiles } from './product-governance-node-test-catalog.mjs';

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

export function listProductGovernanceNodeTests() {
  return listProductGovernanceNodeTestFiles();
}

export function createProductGovernanceNodeTestPlan({
  cwd = workspaceRoot,
  env = process.env,
  nodeExecutable = process.execPath,
  testFiles = listProductGovernanceNodeTests(),
} = {}) {
  return {
    command: nodeExecutable,
    args: ['--test', '--experimental-test-isolation=none', ...testFiles],
    cwd,
    env,
    shell: false,
    windowsHide: process.platform === 'win32',
  };
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
    `product governance node tests failed with exit code ${result?.status ?? 'unknown'} while executing ${plan.command} ${plan.args.join(' ')}${fragments.length > 0 ? `\n${fragments.join('\n')}` : ''}`,
  );
}

export function runProductGovernanceNodeTests({
  cwd = workspaceRoot,
  env = process.env,
  nodeExecutable = process.execPath,
  testFiles = listProductGovernanceNodeTests(),
  spawnSyncImpl = spawnSync,
} = {}) {
  const plan = createProductGovernanceNodeTestPlan({
    cwd,
    env,
    nodeExecutable,
    testFiles,
  });
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

  return result;
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
  runProductGovernanceNodeTests();
}
