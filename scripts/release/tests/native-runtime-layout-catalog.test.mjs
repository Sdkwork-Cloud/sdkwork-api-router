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
      path.join(repoRoot, 'scripts', 'release', 'native-runtime-layout-catalog.mjs'),
    ).href,
  );
}

test('native runtime layout catalog exposes strict governed desktop and server runtime layouts', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listNativeRuntimeLayoutSpecs, 'function');
  assert.equal(typeof module.findNativeRuntimeLayoutSpec, 'function');
  assert.equal(typeof module.listNativeRuntimeLayoutSpecsByIds, 'function');
  assert.equal(typeof module.findNativePortalDesktopEmbeddedRuntimeLayoutSpec, 'function');
  assert.equal(typeof module.findNativeProductServerBundleRuntimeLayoutSpec, 'function');

  assert.deepEqual(
    module.listNativeRuntimeLayoutSpecs(),
    [
      {
        id: 'portal-desktop-embedded-runtime',
        layoutKind: 'desktop-embedded-runtime',
        serviceBinaryName: 'router-product-service',
        serviceBinaryDir: 'router-product/bin',
        siteTargetDirs: {
          admin: 'router-product/sites/admin/dist',
          portal: 'router-product/sites/portal/dist',
        },
        bootstrapDataRootDirs: {
          data: 'router-product/data',
        },
        deploymentAssetRootDirs: {},
        releaseManifestFile: 'router-product/release-manifest.json',
        readmeFile: 'router-product/README.txt',
      },
      {
        id: 'product-server-bundle',
        layoutKind: 'product-server-bundle',
        serviceBinaryName: 'router-product-service',
        serviceBinaryDir: 'bin',
        bundleInstallers: {
          shell: 'install.sh',
          powershell: 'install.ps1',
        },
        controlScriptDir: 'control/bin',
        siteTargetDirs: {
          admin: 'sites/admin/dist',
          portal: 'sites/portal/dist',
        },
        bootstrapDataRootDirs: {
          data: 'data',
        },
        deploymentAssetRootDirs: {
          deploy: 'deploy',
        },
        releaseManifestFile: 'release-manifest.json',
        readmeFile: 'README.txt',
      },
    ],
  );

  const desktopSpec = module.findNativePortalDesktopEmbeddedRuntimeLayoutSpec();
  desktopSpec.siteTargetDirs.admin = 'mutated-locally';
  assert.deepEqual(
    module.findNativePortalDesktopEmbeddedRuntimeLayoutSpec(),
    {
      id: 'portal-desktop-embedded-runtime',
      layoutKind: 'desktop-embedded-runtime',
      serviceBinaryName: 'router-product-service',
      serviceBinaryDir: 'router-product/bin',
      siteTargetDirs: {
        admin: 'router-product/sites/admin/dist',
        portal: 'router-product/sites/portal/dist',
      },
      bootstrapDataRootDirs: {
        data: 'router-product/data',
      },
      deploymentAssetRootDirs: {},
      releaseManifestFile: 'router-product/release-manifest.json',
      readmeFile: 'router-product/README.txt',
    },
  );

  assert.deepEqual(
    module.listNativeRuntimeLayoutSpecsByIds([
      'product-server-bundle',
      'portal-desktop-embedded-runtime',
    ]).map(({ id }) => id),
    [
      'product-server-bundle',
      'portal-desktop-embedded-runtime',
    ],
  );

  assert.deepEqual(
    module.findNativeProductServerBundleRuntimeLayoutSpec(),
    {
      id: 'product-server-bundle',
      layoutKind: 'product-server-bundle',
      serviceBinaryName: 'router-product-service',
      serviceBinaryDir: 'bin',
      bundleInstallers: {
        shell: 'install.sh',
        powershell: 'install.ps1',
      },
      controlScriptDir: 'control/bin',
      siteTargetDirs: {
        admin: 'sites/admin/dist',
        portal: 'sites/portal/dist',
      },
      bootstrapDataRootDirs: {
        data: 'data',
      },
      deploymentAssetRootDirs: {
        deploy: 'deploy',
      },
      releaseManifestFile: 'release-manifest.json',
      readmeFile: 'README.txt',
    },
  );

  assert.throws(
    () => module.findNativeRuntimeLayoutSpec('missing-native-runtime-layout'),
    /missing native runtime layout spec.*missing-native-runtime-layout/i,
  );
});

test('native release packager consumes the shared runtime layout catalog instead of hardcoding internal bundle paths', () => {
  const nativePackager = read('scripts/release/package-release-assets.mjs');

  assert.match(
    nativePackager,
    /native-runtime-layout-catalog\.mjs/,
    'native packager must consume the shared runtime layout catalog',
  );
  assert.doesNotMatch(
    nativePackager,
    /['"`]router-product\/sites\/admin\/dist['"`]|['"`]router-product\/sites\/portal\/dist['"`]|['"`]router-product\/data['"`]|['"`]router-product\/release-manifest\.json['"`]|['"`]router-product\/README\.txt['"`]/,
    'native packager must not inline portal desktop embedded runtime layout paths after catalog extraction',
  );
  assert.doesNotMatch(
    nativePackager,
    /path\.join\(archiveRoot,\s*['"`]README\.txt['"`]\)|path\.join\(archiveRoot,\s*['"`]release-manifest\.json['"`]\)|['"`]sites\/admin\/dist['"`]|['"`]sites\/portal\/dist['"`]|['"`]data['"`]\s*&&\s*set SDKWORK_ADMIN_SITE_DIR/,
    'native packager must not inline product-server bundle layout paths after catalog extraction',
  );
});
