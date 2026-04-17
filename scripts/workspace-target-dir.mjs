#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

function resolveExistingManagedWindowsTargetRoot(workspaceRoot, env = process.env) {
  const candidateRoots = [
    env.SDKWORK_WINDOWS_TARGET_ROOT,
    env.TEMP,
    env.TMP,
  ];

  for (const candidateRoot of candidateRoots) {
    const configuredRoot = String(candidateRoot ?? '').trim();
    if (configuredRoot.length === 0) {
      continue;
    }

    const resolvedRoot = path.isAbsolute(configuredRoot)
      ? configuredRoot
      : path.resolve(workspaceRoot, configuredRoot);
    if (existsSync(resolvedRoot)) {
      return resolvedRoot;
    }
  }

  return '';
}

function managedWindowsWorkspaceTargetLeaf(workspaceRoot) {
  const resolvedWorkspaceRoot = path.resolve(workspaceRoot);
  const normalizedWorkspaceRoot = resolvedWorkspaceRoot.replaceAll('\\', '/').toLowerCase();
  const workspaceName = path.basename(resolvedWorkspaceRoot)
    .toLowerCase()
    .replaceAll(/[^a-z0-9]+/g, '')
    .slice(0, 12) || 'workspace';
  const workspaceHash = createHash('sha1')
    .update(normalizedWorkspaceRoot)
    .digest('hex')
    .slice(0, 10);

  return `${workspaceName}-${workspaceHash}`;
}

export function resolveWorkspaceTargetDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
} = {}) {
  if (typeof workspaceRoot !== 'string' || workspaceRoot.trim().length === 0) {
    throw new Error('workspaceRoot is required.');
  }

  const requestedTargetDir = String(env.CARGO_TARGET_DIR ?? '').trim();
  if (requestedTargetDir.length > 0) {
    return path.isAbsolute(requestedTargetDir)
      ? requestedTargetDir
      : path.resolve(workspaceRoot, requestedTargetDir);
  }

  if (platform !== 'win32') {
    return path.join(workspaceRoot, 'target');
  }

  const managedWindowsTargetRoot = resolveExistingManagedWindowsTargetRoot(workspaceRoot, env);
  if (managedWindowsTargetRoot.length > 0) {
    return path.join(
      managedWindowsTargetRoot,
      'sdkwork-target',
      managedWindowsWorkspaceTargetLeaf(workspaceRoot),
    );
  }

  return path.join(workspaceRoot, 'bin', '.sdkwork-target-vs2022');
}

export function withManagedWorkspaceTargetDir({
  workspaceRoot,
  env = process.env,
  platform = process.platform,
} = {}) {
  const nextEnv = { ...env };
  if (platform === 'win32' && String(nextEnv.CARGO_TARGET_DIR ?? '').trim().length === 0) {
    nextEnv.CARGO_TARGET_DIR = resolveWorkspaceTargetDir({
      workspaceRoot,
      env: nextEnv,
      platform,
    });
  }

  return nextEnv;
}
