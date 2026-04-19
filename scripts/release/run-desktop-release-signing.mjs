#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { resolveDesktopReleaseTarget } from './desktop-targets.mjs';
import {
  resolveAvailableNativeBuildRoot,
  resolveNativeBuildRootCandidates,
  shouldIncludeDesktopBundleFile,
} from './package-release-assets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..', '..');

export const DESKTOP_SIGNING_REQUIRED_ENV = 'SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED';
export const DESKTOP_GENERIC_SIGN_HOOK_ENV = 'SDKWORK_RELEASE_DESKTOP_SIGN_HOOK';

export const DESKTOP_PLATFORM_SIGN_HOOK_ENVS = {
  windows: 'SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK',
  linux: 'SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK',
  macos: 'SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK',
};

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new Error(`${token} requires a value`);
  }
  return next;
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    appId: '',
    platform: '',
    arch: '',
    targetTriple: '',
    evidencePath: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    switch (token) {
      case '--app':
        options.appId = String(readOptionValue(token, next)).trim();
        index += 1;
        break;
      case '--platform':
        options.platform = String(readOptionValue(token, next)).trim();
        index += 1;
        break;
      case '--arch':
        options.arch = String(readOptionValue(token, next)).trim();
        index += 1;
        break;
      case '--target':
        options.targetTriple = String(readOptionValue(token, next)).trim();
        index += 1;
        break;
      case '--evidence-path':
        options.evidencePath = path.resolve(readOptionValue(token, next));
        index += 1;
        break;
      default:
        throw new Error(`unknown option: ${token}`);
    }
  }

  if (!options.appId) {
    throw new Error('--app is required');
  }
  if (!options.targetTriple && (!options.platform || !options.arch)) {
    throw new Error('--target is required unless both --platform and --arch are provided');
  }
  if (!options.evidencePath) {
    throw new Error('--evidence-path is required');
  }

  return options;
}

function parseBooleanEnv(value, fallback = false) {
  const normalized = String(value ?? '').trim().toLowerCase();
  if (!normalized) {
    return fallback;
  }
  if (['1', 'true', 'yes', 'on'].includes(normalized)) {
    return true;
  }
  if (['0', 'false', 'no', 'off'].includes(normalized)) {
    return false;
  }
  return fallback;
}

export function resolveDesktopSigningHook({
  platform,
  env = process.env,
} = {}) {
  const platformEnvVar = DESKTOP_PLATFORM_SIGN_HOOK_ENVS[platform];
  const platformCommand = String(env[platformEnvVar] ?? '').trim();
  if (platformCommand) {
    return {
      kind: 'command',
      command: platformCommand,
      envVar: platformEnvVar,
    };
  }

  const genericCommand = String(env[DESKTOP_GENERIC_SIGN_HOOK_ENV] ?? '').trim();
  if (genericCommand) {
    return {
      kind: 'command',
      command: genericCommand,
      envVar: DESKTOP_GENERIC_SIGN_HOOK_ENV,
    };
  }

  return {
    kind: 'none',
    command: '',
    envVar: platformEnvVar,
  };
}

function listFilesRecursively(root) {
  const results = [];
  if (!root) {
    return results;
  }

  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const absolutePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      results.push(...listFilesRecursively(absolutePath));
      continue;
    }
    if (entry.isFile()) {
      results.push(absolutePath);
    }
  }

  return results;
}

function normalizeBundleRelativePath(relativePath) {
  const normalizedPath = String(relativePath ?? '').replaceAll('\\', '/');
  const bundleMarker = '/release/bundle/';
  const bundleIndex = normalizedPath.lastIndexOf(bundleMarker);
  if (bundleIndex >= 0) {
    return normalizedPath.slice(bundleIndex + bundleMarker.length);
  }

  if (normalizedPath.startsWith('release/bundle/')) {
    return normalizedPath.slice('release/bundle/'.length);
  }

  return normalizedPath;
}

export function resolveDesktopSigningBundleFiles({
  appId,
  platform,
  targetTriple = '',
  buildRoots,
} = {}) {
  const candidates = Array.isArray(buildRoots) && buildRoots.length > 0
    ? [...new Set(buildRoots.map((root) => path.resolve(root)))]
    : resolveNativeBuildRootCandidates({
        appId,
        targetTriple,
        platform,
      });
  const buildRoot = resolveAvailableNativeBuildRoot({
    appId,
    targetTriple,
    buildRoots: candidates,
  });

  if (!buildRoot) {
    throw new Error(`No desktop bundle output was found for ${appId} ${targetTriple || platform}`);
  }

  const bundleFiles = listFilesRecursively(buildRoot)
    .filter((absolutePath) => shouldIncludeDesktopBundleFile(
      platform,
      normalizeBundleRelativePath(path.relative(buildRoot, absolutePath)),
    ))
    .sort();

  if (bundleFiles.length === 0) {
    throw new Error(`No official desktop installer was found under ${buildRoot}`);
  }

  return bundleFiles;
}

export function createDesktopReleaseSigningPlan({
  repoRoot: currentRepoRoot = repoRoot,
  appId,
  platform,
  arch,
  targetTriple = '',
  evidencePath,
  env = process.env,
  buildRoots,
} = {}) {
  const target = resolveDesktopReleaseTarget({
    platform,
    arch,
    targetTriple,
    env,
  });
  const hook = resolveDesktopSigningHook({
    platform: target.platform,
    env,
  });
  const required = parseBooleanEnv(env[DESKTOP_SIGNING_REQUIRED_ENV], false);
  const bundleFiles = resolveDesktopSigningBundleFiles({
    appId,
    platform: target.platform,
    targetTriple: target.targetTriple,
    buildRoots,
  });

  if (required && hook.kind === 'none') {
    throw new Error(
      `Desktop release signing is required for ${target.platform}/${target.arch}, but no signing hook is configured`,
    );
  }

  return {
    repoRoot: currentRepoRoot,
    appId,
    platform: target.platform,
    arch: target.arch,
    targetTriple: target.targetTriple,
    evidencePath: path.resolve(evidencePath),
    env: { ...env },
    required,
    hook,
    bundleFiles,
  };
}

function renderHookCommand(commandTemplate, plan, bundleFile) {
  return String(commandTemplate)
    .replaceAll('{app}', plan.appId)
    .replaceAll('{platform}', plan.platform)
    .replaceAll('{arch}', plan.arch)
    .replaceAll('{target}', plan.targetTriple)
    .replaceAll('{file}', bundleFile)
    .replaceAll('{evidence}', plan.evidencePath);
}

export function createDesktopReleaseSigningEvidence({
  plan,
  status,
  commandCount = 0,
  failure = null,
} = {}) {
  const toRepoRelative = (value) => path.relative(plan.repoRoot, value).replaceAll('\\', '/');
  return {
    version: 1,
    type: 'sdkwork-desktop-release-signing',
    appId: plan.appId,
    platform: plan.platform,
    arch: plan.arch,
    targetTriple: plan.targetTriple,
    required: plan.required,
    status,
    hook: {
      kind: plan.hook.kind,
      envVar: plan.hook.envVar,
    },
    bundleFiles: plan.bundleFiles.map((bundleFile) => toRepoRelative(bundleFile)),
    evidencePath: toRepoRelative(plan.evidencePath),
    commandCount,
    failure: failure
      ? {
          message: failure instanceof Error ? failure.message : String(failure),
        }
      : null,
  };
}

function writeEvidenceFile(evidencePath, evidence) {
  mkdirSync(path.dirname(evidencePath), { recursive: true });
  writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, 'utf8');
}

export function executeDesktopReleaseSigningPlan(plan, {
  spawn = spawnSync,
} = {}) {
  if (plan.hook.kind === 'none') {
    const evidence = createDesktopReleaseSigningEvidence({
      plan,
      status: 'skipped',
      commandCount: 0,
    });
    writeEvidenceFile(plan.evidencePath, evidence);
    return {
      status: 'skipped',
      evidence,
    };
  }

  let commandCount = 0;
  for (const bundleFile of plan.bundleFiles) {
    const command = renderHookCommand(plan.hook.command, plan, bundleFile);
    const result = spawn(command, {
      cwd: plan.repoRoot,
      env: plan.env,
      shell: true,
      stdio: 'pipe',
      encoding: 'utf8',
    });

    if ((result.status ?? 0) !== 0) {
      const failure = new Error(
        `Desktop signing hook failed for ${bundleFile}: ${String(result.stderr ?? result.stdout ?? '').trim() || `exit ${result.status}`}`,
      );
      const evidence = createDesktopReleaseSigningEvidence({
        plan,
        status: 'failed',
        commandCount,
        failure,
      });
      writeEvidenceFile(plan.evidencePath, evidence);
      throw failure;
    }

    commandCount += 1;
  }

  const evidence = createDesktopReleaseSigningEvidence({
    plan,
    status: 'signed',
    commandCount,
  });
  writeEvidenceFile(plan.evidencePath, evidence);
  return {
    status: 'signed',
    evidence,
  };
}

function handleFatalError(error) {
  console.error(error instanceof Error ? error.stack ?? error.message : String(error));
  process.exit(1);
}

function runCli() {
  const options = parseArgs();
  const plan = createDesktopReleaseSigningPlan({
    repoRoot,
    appId: options.appId,
    platform: options.platform,
    arch: options.arch,
    targetTriple: options.targetTriple,
    evidencePath: options.evidencePath,
  });
  executeDesktopReleaseSigningPlan(plan);
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    runCli();
  } catch (error) {
    handleFatalError(error);
  }
}
