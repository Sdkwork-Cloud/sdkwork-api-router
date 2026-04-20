import path from 'node:path';
import process from 'node:process';

import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';
import { normalizeDesktopPlatform } from './desktop-targets.mjs';
import { resolveManagedWindowsTauriTargetDir } from '../run-tauri-cli.mjs';
import { resolveWorkspaceTargetDir } from '../workspace-target-dir.mjs';

const NATIVE_DESKTOP_BUILD_ROOT_CANDIDATE_SPECS = [
  Object.freeze({
    id: 'app-target-root',
    description: 'app-root target directory used by tauri v2 project layouts',
    rootKind: 'app-target-root',
    releasePathSegments: Object.freeze(['release', 'bundle']),
    targetScoped: true,
    platforms: Object.freeze([]),
  }),
  Object.freeze({
    id: 'workspace-target-root',
    description: 'managed workspace target directory used by repository-wide builds',
    rootKind: 'workspace-target-root',
    releasePathSegments: Object.freeze(['release', 'bundle']),
    targetScoped: true,
    platforms: Object.freeze([]),
  }),
  Object.freeze({
    id: 'managed-windows-tauri-target-root',
    description: 'managed Windows tauri sidecar target directory',
    rootKind: 'managed-windows-tauri-target-root',
    releasePathSegments: Object.freeze(['release', 'bundle']),
    targetScoped: true,
    platforms: Object.freeze(['windows']),
  }),
  Object.freeze({
    id: 'workspace-product-target-root',
    description: 'repository product target directory dedicated to the desktop app',
    rootKind: 'workspace-product-target-root',
    releasePathSegments: Object.freeze(['release', 'bundle']),
    targetScoped: true,
    platforms: Object.freeze([]),
  }),
  Object.freeze({
    id: 'src-tauri-target-root',
    description: 'src-tauri target directory used by default tauri layouts',
    rootKind: 'src-tauri-target-root',
    releasePathSegments: Object.freeze(['release', 'bundle']),
    targetScoped: true,
    platforms: Object.freeze([]),
  }),
];

const NATIVE_SERVICE_RELEASE_ROOT_CANDIDATE_SPECS = [
  Object.freeze({
    id: 'workspace-target-root',
    description: 'managed workspace target directory used by repository-wide service builds',
    rootKind: 'workspace-target-root',
    releasePathSegments: Object.freeze(['release']),
    targetScoped: true,
  }),
  Object.freeze({
    id: 'repository-target-root',
    description: 'repository target directory used by cargo release builds',
    rootKind: 'repository-target-root',
    releasePathSegments: Object.freeze(['release']),
    targetScoped: true,
  }),
];

function cloneDesktopBuildRootCandidateSpec(spec) {
  return {
    ...spec,
    releasePathSegments: [...spec.releasePathSegments],
    platforms: [...spec.platforms],
  };
}

function cloneServiceReleaseRootCandidateSpec(spec) {
  return {
    ...spec,
    releasePathSegments: [...spec.releasePathSegments],
  };
}

const nativeDesktopBuildRootCandidateSpecCatalog = createStrictKeyedCatalog({
  entries: NATIVE_DESKTOP_BUILD_ROOT_CANDIDATE_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneDesktopBuildRootCandidateSpec,
  duplicateKeyMessagePrefix: 'duplicate native desktop build root candidate spec',
  missingKeyMessagePrefix: 'missing native desktop build root candidate spec',
});

const nativeServiceReleaseRootCandidateSpecCatalog = createStrictKeyedCatalog({
  entries: NATIVE_SERVICE_RELEASE_ROOT_CANDIDATE_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneServiceReleaseRootCandidateSpec,
  duplicateKeyMessagePrefix: 'duplicate native service release root candidate spec',
  missingKeyMessagePrefix: 'missing native service release root candidate spec',
});

export function listNativeDesktopBuildRootCandidateSpecs() {
  return nativeDesktopBuildRootCandidateSpecCatalog.list();
}

export function findNativeDesktopBuildRootCandidateSpec(candidateId) {
  return nativeDesktopBuildRootCandidateSpecCatalog.find(candidateId);
}

export function listNativeDesktopBuildRootCandidateSpecsByIds(candidateIds = []) {
  return nativeDesktopBuildRootCandidateSpecCatalog.listByKeys(candidateIds);
}

export function listNativeServiceReleaseRootCandidateSpecs() {
  return nativeServiceReleaseRootCandidateSpecCatalog.list();
}

export function findNativeServiceReleaseRootCandidateSpec(candidateId) {
  return nativeServiceReleaseRootCandidateSpecCatalog.find(candidateId);
}

export function listNativeServiceReleaseRootCandidateSpecsByIds(candidateIds = []) {
  return nativeServiceReleaseRootCandidateSpecCatalog.listByKeys(candidateIds);
}

export function materializeNativeDesktopBuildRootCandidates({
  appDir,
  workspaceTargetDirName,
  targetTriple = '',
  env = process.env,
  platform = process.platform,
  workspaceRoot,
  resolveWorkspaceTargetDirImpl = resolveWorkspaceTargetDir,
  resolveManagedWindowsTauriTargetDirImpl = resolveManagedWindowsTauriTargetDir,
} = {}) {
  const normalizedPlatform = normalizeDesktopPlatform(platform);
  const roots = [];

  for (const spec of listNativeDesktopBuildRootCandidateSpecs()) {
    if (spec.platforms.length > 0 && !spec.platforms.includes(normalizedPlatform)) {
      continue;
    }

    const rootPath = resolveDesktopBuildRootCandidateRoot({
      spec,
      appDir,
      workspaceTargetDirName,
      env,
      platform,
      workspaceRoot,
      resolveWorkspaceTargetDirImpl,
      resolveManagedWindowsTauriTargetDirImpl,
    });
    if (!rootPath) {
      continue;
    }

    roots.push(...buildReleasePathVariants({
      rootPath,
      targetTriple,
      targetScoped: spec.targetScoped,
      releasePathSegments: spec.releasePathSegments,
    }));
  }

  return [...new Set(roots)];
}

export function materializeNativeServiceReleaseRootCandidates({
  targetTriple = '',
  env = process.env,
  platform = process.platform,
  workspaceRoot,
  resolveWorkspaceTargetDirImpl = resolveWorkspaceTargetDir,
} = {}) {
  const roots = [];

  for (const spec of listNativeServiceReleaseRootCandidateSpecs()) {
    const rootPath = resolveServiceReleaseRootCandidateRoot({
      spec,
      env,
      platform,
      workspaceRoot,
      resolveWorkspaceTargetDirImpl,
    });
    if (!rootPath) {
      continue;
    }

    roots.push(...buildReleasePathVariants({
      rootPath,
      targetTriple,
      targetScoped: spec.targetScoped,
      releasePathSegments: spec.releasePathSegments,
    }));
  }

  return [...new Set(roots)];
}

function resolveDesktopBuildRootCandidateRoot({
  spec,
  appDir,
  workspaceTargetDirName,
  env,
  platform,
  workspaceRoot,
  resolveWorkspaceTargetDirImpl,
  resolveManagedWindowsTauriTargetDirImpl,
} = {}) {
  switch (spec.rootKind) {
    case 'app-target-root':
      return path.join(appDir, 'target');
    case 'workspace-target-root':
      return resolveWorkspaceTargetDirImpl({
        workspaceRoot,
        env,
        platform: normalizeNodePlatform(platform),
      });
    case 'managed-windows-tauri-target-root':
      return resolveManagedWindowsTauriTargetDirImpl({
        cwd: appDir,
        env,
        platform: 'win32',
      });
    case 'workspace-product-target-root':
      return path.join(workspaceRoot, 'target', workspaceTargetDirName);
    case 'src-tauri-target-root':
      return path.join(appDir, 'src-tauri', 'target');
    default:
      return '';
  }
}

function resolveServiceReleaseRootCandidateRoot({
  spec,
  env,
  platform,
  workspaceRoot,
  resolveWorkspaceTargetDirImpl,
} = {}) {
  switch (spec.rootKind) {
    case 'workspace-target-root':
      return resolveWorkspaceTargetDirImpl({
        workspaceRoot,
        env,
        platform: normalizeNodePlatform(platform),
      });
    case 'repository-target-root':
      return path.join(workspaceRoot, 'target');
    default:
      return '';
  }
}

function buildReleasePathVariants({
  rootPath,
  targetTriple = '',
  targetScoped = true,
  releasePathSegments = [],
} = {}) {
  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const variants = [];

  if (targetScoped && normalizedTargetTriple.length > 0) {
    variants.push(path.join(rootPath, normalizedTargetTriple, ...releasePathSegments));
  }
  variants.push(path.join(rootPath, ...releasePathSegments));

  return variants;
}

function normalizeNodePlatform(platform = process.platform) {
  if (platform === 'windows') {
    return 'win32';
  }
  if (platform === 'macos') {
    return 'darwin';
  }

  return platform;
}
