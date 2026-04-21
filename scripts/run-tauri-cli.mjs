#!/usr/bin/env node

import fs from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  buildDesktopReleaseEnv,
  DESKTOP_TARGET_ENV_VAR,
} from './release/desktop-targets.mjs';
import {
  resolveWorkspaceTargetDir,
  withManagedWorkspaceTempDir,
} from './workspace-target-dir.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const BACKGROUND_LAUNCH_ENV = 'SDKWORK_ROUTER_BACKGROUND';

function normalizeCliArgs(args = []) {
  return args.filter((arg) => arg !== '--');
}

function normalizeWorkspaceRelativePath(candidate = '') {
  return path.relative(rootDir, path.resolve(candidate)).replaceAll('\\', '/');
}

function shouldLaunchInBackground(commandName, args = [], env = process.env) {
  if (commandName !== 'dev') {
    return false;
  }

  if (String(env[BACKGROUND_LAUNCH_ENV] ?? '').trim() === '1') {
    return true;
  }

  return normalizeCliArgs(args).some((arg) => arg === '--service' || arg === '--start-hidden');
}

function isVisualStudioCmakeGenerator(value = '') {
  return /visual studio/i.test(String(value ?? '').trim());
}

function sanitizeNonWindowsCmakeEnv(baseEnv = process.env, platform = process.platform) {
  const env = { ...baseEnv };
  if (platform === 'win32') {
    return env;
  }

  delete env.HOST_CMAKE_GENERATOR;
  delete env.CMAKE_GENERATOR_PLATFORM;
  delete env.CMAKE_GENERATOR_TOOLSET;

  if (isVisualStudioCmakeGenerator(env.CMAKE_GENERATOR)) {
    delete env.CMAKE_GENERATOR;
  }

  return env;
}

export function withSupportedWindowsCmakeGenerator(
  baseEnv = process.env,
  platform = process.platform,
) {
  const env = sanitizeNonWindowsCmakeEnv(baseEnv, platform);
  if (platform !== 'win32') {
    return env;
  }

  const requestedGenerator = String(env.CMAKE_GENERATOR ?? '').trim();
  if (requestedGenerator.length > 0 && !requestedGenerator.includes('2026')) {
    return env;
  }

  env.CMAKE_GENERATOR = 'Visual Studio 17 2022';
  env.HOST_CMAKE_GENERATOR = 'Visual Studio 17 2022';
  return env;
}

function resolveCargoBinDir(baseEnv = process.env, platform = process.platform) {
  if (platform === 'win32') {
    const cargoHome = String(baseEnv.CARGO_HOME ?? '').trim()
      || (baseEnv.USERPROFILE ? path.join(baseEnv.USERPROFILE, '.cargo') : '');
    return cargoHome ? path.join(cargoHome, 'bin') : null;
  }

  const home = String(baseEnv.HOME ?? '').trim();
  return home ? path.join(home, '.cargo', 'bin') : null;
}

function withCargoToolchainOnPath(baseEnv = process.env, platform = process.platform) {
  const env = { ...baseEnv };
  const cargoBinDir = resolveCargoBinDir(baseEnv, platform);
  if (!cargoBinDir || !fs.existsSync(cargoBinDir)) {
    return env;
  }

  const currentPath = String(env.PATH ?? env.Path ?? '').trim();
  const pathEntries = currentPath ? currentPath.split(path.delimiter) : [];
  if (!pathEntries.some((entry) => entry.toLowerCase() === cargoBinDir.toLowerCase())) {
    const joinedPath = [cargoBinDir, ...pathEntries].filter(Boolean).join(path.delimiter);
    env.PATH = joinedPath;
    env.Path = joinedPath;
  }

  return env;
}

function resolveLocalTauriCliCommand({
  cwd = process.cwd(),
  platform = process.platform,
} = {}) {
  const localBinName = platform === 'win32' ? 'tauri.CMD' : 'tauri';
  const localBinPath = path.join(cwd, 'node_modules', '.bin', localBinName);
  if (fs.existsSync(localBinPath)) {
    return localBinPath;
  }

  return platform === 'win32' ? 'tauri.cmd' : 'tauri';
}

export function resolveManagedWindowsTauriTargetDir({
  env = process.env,
  cwd = process.cwd(),
  platform = process.platform,
} = {}) {
  if (platform !== 'win32') {
    return null;
  }

  return resolveWorkspaceTargetDir({
    workspaceRoot: rootDir,
    env,
    platform,
  });
}

function normalizeComparablePath(candidate = '') {
  return path.normalize(String(candidate ?? '').trim()).replace(/[\\/]+$/u, '').toLowerCase();
}

function shouldOverrideWindowsCargoTargetDir({
  env = process.env,
  platform = process.platform,
} = {}) {
  if (platform !== 'win32') {
    return false;
  }

  const existingTargetDir = String(env.CARGO_TARGET_DIR ?? '').trim();
  if (!existingTargetDir) {
    return true;
  }

  const managedWorkspaceTargetDir = resolveWorkspaceTargetDir({
    workspaceRoot: rootDir,
    env: {
      ...env,
      CARGO_TARGET_DIR: '',
    },
    platform,
  });

  return normalizeComparablePath(existingTargetDir) === normalizeComparablePath(managedWorkspaceTargetDir);
}

function extractTargetTriple(args, env = process.env) {
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === '--target') {
      return String(args[index + 1] ?? '').trim();
    }
  }

  return String(env?.[DESKTOP_TARGET_ENV_VAR] ?? '').trim();
}

function shouldPreflightPortalDesktopRuntime({
  commandName,
  cwd = process.cwd(),
} = {}) {
  return commandName === 'dev'
    && normalizeWorkspaceRelativePath(cwd) === 'apps/sdkwork-router-portal';
}

function createTauriCliPreflightSteps({
  commandName,
  cwd = process.cwd(),
  env = process.env,
  platform = process.platform,
} = {}) {
  if (!shouldPreflightPortalDesktopRuntime({
    commandName,
    cwd,
  })) {
    return [];
  }

  return [{
    label: 'portal desktop runtime preflight',
    command: process.execPath,
    args: [path.join(rootDir, 'scripts', 'prepare-router-portal-desktop-runtime.mjs')],
    cwd: rootDir,
    env,
    shell: false,
    windowsHide: platform === 'win32',
  }];
}

export function createTauriCliPlan({
  commandName,
  args = [],
  cwd = process.cwd(),
  env = process.env,
  platform = process.platform,
} = {}) {
  if (typeof commandName !== 'string' || commandName.trim().length === 0) {
    throw new Error('commandName is required.');
  }

  const background = shouldLaunchInBackground(commandName, args, env);
  const requestedTargetTriple = extractTargetTriple(args, env);
  const resolvedEnv = requestedTargetTriple
    ? buildDesktopReleaseEnv({
        env,
        targetTriple: requestedTargetTriple,
      })
    : { ...env };
  const managedEnv = withManagedWorkspaceTempDir({
    workspaceRoot: rootDir,
    env: resolvedEnv,
    platform,
  });
  if (shouldOverrideWindowsCargoTargetDir({
    env: managedEnv,
    platform,
  })) {
    const shortTargetDir = resolveManagedWindowsTauriTargetDir({
      env: managedEnv,
      cwd,
      platform,
    });
    if (shortTargetDir) {
      managedEnv.CARGO_TARGET_DIR = shortTargetDir;
    }
  }

  const finalEnv = withSupportedWindowsCmakeGenerator(
    withCargoToolchainOnPath(managedEnv, platform),
    platform,
  );

  return {
    command: resolveLocalTauriCliCommand({
      cwd,
      platform,
    }),
    args: [commandName, ...args],
    cwd,
    env: finalEnv,
    shell: platform === 'win32',
    detached: background,
    windowsHide: platform === 'win32',
    preflightSteps: createTauriCliPreflightSteps({
      commandName,
      cwd,
      env: finalEnv,
      platform,
    }),
  };
}

function runPreflightSteps(preflightSteps = []) {
  for (const step of preflightSteps) {
    const result = spawnSync(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell ?? false,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    if (result.error) {
      throw new Error(`${step.label} failed: ${result.error.message}`);
    }
    if ((result.status ?? 1) !== 0) {
      throw new Error(`${step.label} exited with code ${result.status ?? 1}`);
    }
  }
}

function runCli() {
  const [commandName, ...args] = process.argv.slice(2);
  const plan = createTauriCliPlan({
    commandName,
    args,
  });
  try {
    runPreflightSteps(plan.preflightSteps ?? []);
  } catch (error) {
    console.error(`[run-tauri-cli] ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
  }
  const child = spawn(plan.command, plan.args, {
    cwd: plan.cwd,
    env: plan.env,
    stdio: plan.detached ? 'ignore' : 'inherit',
    detached: plan.detached ?? false,
    shell: plan.shell,
    windowsHide: plan.windowsHide ?? process.platform === 'win32',
  });

  child.on('error', (error) => {
    console.error(`[run-tauri-cli] ${error.message}`);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      console.error(`[run-tauri-cli] command exited with signal ${signal}`);
      process.exit(1);
    }

    process.exit(code ?? 0);
  });

  if (plan.detached) {
    child.unref();
    process.exit(0);
  }
}

if (__filename === process.argv[1]) {
  runCli();
}
