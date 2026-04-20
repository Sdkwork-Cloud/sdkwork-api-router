import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'native-build-root-catalog.mjs'),
    ).href,
  );
}

test('native build-root catalog exposes strict desktop and service candidate-root definitions', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listNativeDesktopBuildRootCandidateSpecs, 'function');
  assert.equal(typeof module.findNativeDesktopBuildRootCandidateSpec, 'function');
  assert.equal(typeof module.listNativeDesktopBuildRootCandidateSpecsByIds, 'function');
  assert.equal(typeof module.listNativeServiceReleaseRootCandidateSpecs, 'function');
  assert.equal(typeof module.findNativeServiceReleaseRootCandidateSpec, 'function');
  assert.equal(typeof module.listNativeServiceReleaseRootCandidateSpecsByIds, 'function');
  assert.equal(typeof module.materializeNativeDesktopBuildRootCandidates, 'function');
  assert.equal(typeof module.materializeNativeServiceReleaseRootCandidates, 'function');

  assert.deepEqual(
    module.listNativeDesktopBuildRootCandidateSpecs(),
    [
      {
        id: 'app-target-root',
        description: 'app-root target directory used by tauri v2 project layouts',
        rootKind: 'app-target-root',
        releasePathSegments: ['release', 'bundle'],
        targetScoped: true,
        platforms: [],
      },
      {
        id: 'workspace-target-root',
        description: 'managed workspace target directory used by repository-wide builds',
        rootKind: 'workspace-target-root',
        releasePathSegments: ['release', 'bundle'],
        targetScoped: true,
        platforms: [],
      },
      {
        id: 'managed-windows-tauri-target-root',
        description: 'managed Windows tauri sidecar target directory',
        rootKind: 'managed-windows-tauri-target-root',
        releasePathSegments: ['release', 'bundle'],
        targetScoped: true,
        platforms: ['windows'],
      },
      {
        id: 'workspace-product-target-root',
        description: 'repository product target directory dedicated to the desktop app',
        rootKind: 'workspace-product-target-root',
        releasePathSegments: ['release', 'bundle'],
        targetScoped: true,
        platforms: [],
      },
      {
        id: 'src-tauri-target-root',
        description: 'src-tauri target directory used by default tauri layouts',
        rootKind: 'src-tauri-target-root',
        releasePathSegments: ['release', 'bundle'],
        targetScoped: true,
        platforms: [],
      },
    ],
  );

  const desktopSpec = module.findNativeDesktopBuildRootCandidateSpec('workspace-target-root');
  desktopSpec.releasePathSegments.push('mutated-locally');
  assert.deepEqual(
    module.findNativeDesktopBuildRootCandidateSpec('workspace-target-root'),
    {
      id: 'workspace-target-root',
      description: 'managed workspace target directory used by repository-wide builds',
      rootKind: 'workspace-target-root',
      releasePathSegments: ['release', 'bundle'],
      targetScoped: true,
      platforms: [],
    },
  );

  assert.deepEqual(
    module.listNativeDesktopBuildRootCandidateSpecsByIds([
      'managed-windows-tauri-target-root',
      'src-tauri-target-root',
    ]).map(({ id }) => id),
    [
      'managed-windows-tauri-target-root',
      'src-tauri-target-root',
    ],
  );

  assert.deepEqual(
    module.listNativeServiceReleaseRootCandidateSpecs(),
    [
      {
        id: 'workspace-target-root',
        description: 'managed workspace target directory used by repository-wide service builds',
        rootKind: 'workspace-target-root',
        releasePathSegments: ['release'],
        targetScoped: true,
      },
      {
        id: 'repository-target-root',
        description: 'repository target directory used by cargo release builds',
        rootKind: 'repository-target-root',
        releasePathSegments: ['release'],
        targetScoped: true,
      },
    ],
  );
  assert.deepEqual(
    module.listNativeServiceReleaseRootCandidateSpecsByIds([
      'repository-target-root',
    ]),
    [
      {
        id: 'repository-target-root',
        description: 'repository target directory used by cargo release builds',
        rootKind: 'repository-target-root',
        releasePathSegments: ['release'],
        targetScoped: true,
      },
    ],
  );

  assert.throws(
    () => module.findNativeDesktopBuildRootCandidateSpec('missing-desktop-root-candidate'),
    /missing native desktop build root candidate spec.*missing-desktop-root-candidate/i,
  );
  assert.throws(
    () => module.findNativeServiceReleaseRootCandidateSpec('missing-service-root-candidate'),
    /missing native service release root candidate spec.*missing-service-root-candidate/i,
  );
});

test('native build-root catalog materializes governed desktop and service candidate paths in canonical order', async () => {
  const module = await loadModule();

  assert.deepEqual(
    module.materializeNativeDesktopBuildRootCandidates({
      appDir: path.join(repoRoot, 'apps', 'sdkwork-router-admin'),
      workspaceTargetDirName: 'sdkwork-router-admin-tauri',
      targetTriple: 'x86_64-pc-windows-msvc',
      env: {
        USERPROFILE: 'C:/Users/admin',
        TEMP: 'C:/Temp',
      },
      platform: 'win32',
      workspaceRoot: repoRoot,
      resolveWorkspaceTargetDirImpl() {
        return 'C:/managed-workspace-target';
      },
      resolveManagedWindowsTauriTargetDirImpl() {
        return 'C:/managed-windows-tauri-target';
      },
    }).map((entry) => entry.replaceAll('\\', '/')),
    [
      path.join(repoRoot, 'apps', 'sdkwork-router-admin', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle').replaceAll('\\', '/'),
      path.join(repoRoot, 'apps', 'sdkwork-router-admin', 'target', 'release', 'bundle').replaceAll('\\', '/'),
      'C:/managed-workspace-target/x86_64-pc-windows-msvc/release/bundle',
      'C:/managed-workspace-target/release/bundle',
      'C:/managed-windows-tauri-target/x86_64-pc-windows-msvc/release/bundle',
      'C:/managed-windows-tauri-target/release/bundle',
      path.join(repoRoot, 'target', 'sdkwork-router-admin-tauri', 'x86_64-pc-windows-msvc', 'release', 'bundle').replaceAll('\\', '/'),
      path.join(repoRoot, 'target', 'sdkwork-router-admin-tauri', 'release', 'bundle').replaceAll('\\', '/'),
      path.join(repoRoot, 'apps', 'sdkwork-router-admin', 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle').replaceAll('\\', '/'),
      path.join(repoRoot, 'apps', 'sdkwork-router-admin', 'src-tauri', 'target', 'release', 'bundle').replaceAll('\\', '/'),
    ],
  );

  assert.deepEqual(
    module.materializeNativeServiceReleaseRootCandidates({
      targetTriple: 'x86_64-unknown-linux-gnu',
      env: {},
      platform: 'linux',
      workspaceRoot: repoRoot,
      resolveWorkspaceTargetDirImpl() {
        return '/managed-workspace-target';
      },
    }).map((entry) => entry.replaceAll('\\', '/')),
    [
      '/managed-workspace-target/x86_64-unknown-linux-gnu/release',
      '/managed-workspace-target/release',
      path.join(repoRoot, 'target', 'x86_64-unknown-linux-gnu', 'release').replaceAll('\\', '/'),
      path.join(repoRoot, 'target', 'release').replaceAll('\\', '/'),
    ],
  );
});

test('native release packager consumes the shared build-root catalog instead of hardcoding candidate search order locally', () => {
  const nativePackager = read('scripts/release/package-release-assets.mjs');

  assert.match(
    nativePackager,
    /native-build-root-catalog\.mjs/,
    'native packager must consume the shared build-root catalog',
  );
  assert.doesNotMatch(
    nativePackager,
    /resolveManagedWindowsTauriTargetDir|resolveWorkspaceTargetDir/,
    'native packager must not import low-level root resolvers directly after build-root catalog extraction',
  );
  assert.doesNotMatch(
    nativePackager,
    /const roots = \[\]|const candidates = \[\]/,
    'native packager must not maintain local candidate-root accumulation lists outside the shared build-root catalog',
  );
});
