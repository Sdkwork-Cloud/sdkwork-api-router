#!/usr/bin/env node

import path from 'node:path';
import process from 'node:process';

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

  const userProfile = String(env.USERPROFILE ?? '').trim();
  if (userProfile.length > 0) {
    const workspaceName = path.basename(workspaceRoot).trim().toLowerCase() || 'workspace';
    return path.join(userProfile, '.sdkwork-target', workspaceName);
  }

  const tempRoot = String(env.TEMP ?? env.TMP ?? '').trim()
    || (env.USERPROFILE ? path.join(env.USERPROFILE, 'AppData', 'Local', 'Temp') : '');
  if (tempRoot.length === 0) {
    return path.join(workspaceRoot, 'target');
  }

  const workspaceName = path.basename(workspaceRoot).trim().toLowerCase() || 'workspace';
  return path.join(tempRoot, 'srt', workspaceName);
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
