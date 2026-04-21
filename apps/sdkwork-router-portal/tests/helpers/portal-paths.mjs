import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

export function resolvePortalAppRoot(importMetaUrl) {
  const testFilePath = fileURLToPath(importMetaUrl);
  return path.resolve(path.dirname(testFilePath), '..');
}

export function resolvePortalPackagesRoot(importMetaUrl) {
  return path.join(resolvePortalAppRoot(importMetaUrl), 'packages');
}

export function resolvePortalWorkspaceRoot(importMetaUrl) {
  return path.resolve(resolvePortalAppRoot(importMetaUrl), '..', '..');
}

export function resolvePortalAppbaseRoot(importMetaUrl, env = process.env) {
  if (env.SDKWORK_APPBASE_ROOT) {
    return path.resolve(env.SDKWORK_APPBASE_ROOT);
  }

  return path.resolve(resolvePortalWorkspaceRoot(importMetaUrl), '..', 'sdkwork-appbase');
}
