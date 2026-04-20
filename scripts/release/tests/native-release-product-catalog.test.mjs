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
      path.join(repoRoot, 'scripts', 'release', 'native-release-product-catalog.mjs'),
    ).href,
  );
}

test('native release product catalog exposes strict governed official product metadata', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listNativeReleaseProductSpecs, 'function');
  assert.equal(typeof module.findNativeReleaseProductSpec, 'function');
  assert.equal(typeof module.listNativeReleaseProductSpecsByIds, 'function');
  assert.equal(typeof module.findNativeDesktopReleaseProductSpecByAppId, 'function');

  assert.deepEqual(
    module.listNativeReleaseProductSpecs(),
    [
      {
        id: 'product-server',
        productId: 'sdkwork-api-router-product-server',
        packageKind: 'server-archive',
        baseNamePrefix: 'sdkwork-api-router-product-server',
        archiveFileExtension: '.tar.gz',
        outputPathSegments: ['bundles'],
        archiveManifestType: 'product-server-archive',
        embeddedManifestType: 'product-server-bundle',
      },
      {
        id: 'portal-desktop',
        productId: 'sdkwork-router-portal-desktop',
        packageKind: 'desktop-installer',
        appId: 'portal',
        baseNamePrefix: 'sdkwork-router-portal-desktop',
        outputPathSegments: ['desktop', 'portal'],
        manifestType: 'portal-desktop-installer',
      },
    ],
  );

  const portalSpec = module.findNativeReleaseProductSpec('portal-desktop');
  portalSpec.outputPathSegments.push('mutated-locally');
  assert.deepEqual(
    module.findNativeReleaseProductSpec('portal-desktop'),
    {
      id: 'portal-desktop',
      productId: 'sdkwork-router-portal-desktop',
      packageKind: 'desktop-installer',
      appId: 'portal',
      baseNamePrefix: 'sdkwork-router-portal-desktop',
      outputPathSegments: ['desktop', 'portal'],
      manifestType: 'portal-desktop-installer',
    },
  );

  assert.deepEqual(
    module.listNativeReleaseProductSpecsByIds([
      'portal-desktop',
      'product-server',
    ]).map(({ id }) => id),
    [
      'portal-desktop',
      'product-server',
    ],
  );

  assert.deepEqual(
    module.findNativeDesktopReleaseProductSpecByAppId('portal'),
    {
      id: 'portal-desktop',
      productId: 'sdkwork-router-portal-desktop',
      packageKind: 'desktop-installer',
      appId: 'portal',
      baseNamePrefix: 'sdkwork-router-portal-desktop',
      outputPathSegments: ['desktop', 'portal'],
      manifestType: 'portal-desktop-installer',
    },
  );

  assert.throws(
    () => module.findNativeReleaseProductSpec('missing-native-release-product'),
    /missing native release product spec.*missing-native-release-product/i,
  );
  assert.throws(
    () => module.findNativeDesktopReleaseProductSpecByAppId('admin'),
    /missing native desktop release product app id.*admin/i,
  );
});

test('native release packager and workflow publish catalog consume the shared native release product catalog', () => {
  const nativePackager = read('scripts/release/package-release-assets.mjs');
  const workflowPublishCatalog = read('scripts/release/release-workflow-publish-catalog.mjs');

  assert.match(
    nativePackager,
    /native-release-product-catalog\.mjs/,
    'native packager must consume the shared native release product catalog',
  );
  assert.match(
    workflowPublishCatalog,
    /native-release-product-catalog\.mjs/,
    'release workflow publish catalog must consume the shared native release product catalog',
  );
  assert.doesNotMatch(
    nativePackager,
    /productId:\s*'sdkwork-router-portal-desktop'|productId:\s*'sdkwork-api-router-product-server'|type:\s*'portal-desktop-installer'|type:\s*'product-server-archive'|type:\s*'product-server-bundle'/,
    'native packager must not inline official product ids or manifest types outside the shared catalog',
  );
  assert.doesNotMatch(
    workflowPublishCatalog,
    /sdkwork-api-router-product-server-|sdkwork-router-portal-desktop-/,
    'release workflow publish catalog must not inline official base-name prefixes outside the shared catalog',
  );
});
