import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function writeText(root, relativePath, value) {
  const targetPath = path.join(root, relativePath);
  mkdirSync(path.dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, value, 'utf8');
  return targetPath;
}

function writeJson(root, relativePath, payload) {
  return writeText(root, relativePath, `${JSON.stringify(payload, null, 2)}\n`);
}

test('release catalog materializer aggregates official server and desktop assets into one machine-readable catalog', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-catalog.mjs'),
    ).href,
  );

  assert.equal(typeof module.collectReleaseCatalogEntries, 'function');
  assert.equal(typeof module.createReleaseCatalog, 'function');
  assert.equal(typeof module.materializeReleaseCatalog, 'function');
  assert.equal(typeof module.readReleaseCatalogFile, 'function');
  assert.equal(typeof module.findReleaseCatalogVariant, 'function');
  assert.equal(typeof module.resolveReleaseCatalogVariantPaths, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-catalog-'));

  try {
    writeText(
      fixtureRoot,
      'release-assets/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
      'server-archive',
    );
    writeText(
      fixtureRoot,
      'release-assets/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
      'serverdigest  sdkwork-api-router-product-server-linux-x64.tar.gz\n',
    );
    writeJson(
      fixtureRoot,
      'release-assets/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.manifest.json',
      {
        type: 'product-server-archive',
        productId: 'sdkwork-api-router-product-server',
        platform: 'linux',
        arch: 'x64',
        target: 'x86_64-unknown-linux-gnu',
        archiveFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
        checksumFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
        embeddedManifestFile: 'release-manifest.json',
        services: ['router-product-service'],
        sites: ['admin', 'portal'],
        bootstrapDataRoots: ['data'],
        deploymentAssetRoots: ['deploy'],
      },
    );

    writeText(
      fixtureRoot,
      'release-assets/native/windows/x64/desktop/portal/sdkwork-router-portal-desktop-windows-x64.exe',
      'desktop-installer',
    );
    writeText(
      fixtureRoot,
      'release-assets/native/windows/x64/desktop/portal/sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt',
      'desktopdigest  sdkwork-router-portal-desktop-windows-x64.exe\n',
    );
    writeJson(
      fixtureRoot,
      'release-assets/native/windows/x64/desktop/portal/sdkwork-router-portal-desktop-windows-x64.manifest.json',
      {
        type: 'portal-desktop-installer',
        productId: 'sdkwork-router-portal-desktop',
        appId: 'portal',
        platform: 'windows',
        arch: 'x64',
        target: 'x86_64-pc-windows-msvc',
        artifactKind: 'nsis',
        installerFile: 'sdkwork-router-portal-desktop-windows-x64.exe',
        checksumFile: 'sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt',
        sourceBundlePath: 'nsis/Portal Setup.exe',
        embeddedRuntime: {
          routerBinary: 'router-product/bin/router-product-service.exe',
          adminSiteDir: 'router-product/sites/admin/dist',
          portalSiteDir: 'router-product/sites/portal/dist',
        },
      },
    );

    const result = module.materializeReleaseCatalog({
      assetsRoot: path.join(fixtureRoot, 'release-assets'),
      releaseTag: 'release-2026-04-18-1',
      generatedAt: '2026-04-18T12:34:56.789Z',
      outputPath: path.join(fixtureRoot, 'release-assets', 'release-catalog.json'),
    });

    assert.equal(result.generatedAt, '2026-04-18T12:34:56.789Z');
    assert.equal(result.productCount, 2);
    assert.equal(result.variantCount, 2);
    assert.equal(existsSync(result.outputPath), true);

    const catalog = JSON.parse(readFileSync(result.outputPath, 'utf8'));
    assert.deepEqual(catalog, {
      version: 1,
      type: 'sdkwork-release-catalog',
      releaseTag: 'release-2026-04-18-1',
      generatedAt: '2026-04-18T12:34:56.789Z',
      productCount: 2,
      variantCount: 2,
      products: [
        {
          productId: 'sdkwork-api-router-product-server',
          variants: [
            {
              platform: 'linux',
              arch: 'x64',
              outputDirectory: 'native/linux/x64/bundles',
              variantKind: 'server-archive',
              primaryFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
              primaryFileSizeBytes: 14,
              checksumFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
              checksumAlgorithm: 'sha256',
              manifestFile: 'sdkwork-api-router-product-server-linux-x64.manifest.json',
              sha256: 'serverdigest',
              manifest: {
                type: 'product-server-archive',
                productId: 'sdkwork-api-router-product-server',
                platform: 'linux',
                arch: 'x64',
                target: 'x86_64-unknown-linux-gnu',
                archiveFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
                checksumFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
                embeddedManifestFile: 'release-manifest.json',
                services: ['router-product-service'],
                sites: ['admin', 'portal'],
                bootstrapDataRoots: ['data'],
                deploymentAssetRoots: ['deploy'],
              },
            },
          ],
        },
        {
          productId: 'sdkwork-router-portal-desktop',
          variants: [
            {
              platform: 'windows',
              arch: 'x64',
              outputDirectory: 'native/windows/x64/desktop/portal',
              variantKind: 'desktop-installer',
              primaryFile: 'sdkwork-router-portal-desktop-windows-x64.exe',
              primaryFileSizeBytes: 17,
              checksumFile: 'sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt',
              checksumAlgorithm: 'sha256',
              manifestFile: 'sdkwork-router-portal-desktop-windows-x64.manifest.json',
              sha256: 'desktopdigest',
              manifest: {
                type: 'portal-desktop-installer',
                productId: 'sdkwork-router-portal-desktop',
                appId: 'portal',
                platform: 'windows',
                arch: 'x64',
                target: 'x86_64-pc-windows-msvc',
                artifactKind: 'nsis',
                installerFile: 'sdkwork-router-portal-desktop-windows-x64.exe',
                checksumFile: 'sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt',
                sourceBundlePath: 'nsis/Portal Setup.exe',
                embeddedRuntime: {
                  routerBinary: 'router-product/bin/router-product-service.exe',
                  adminSiteDir: 'router-product/sites/admin/dist',
                  portalSiteDir: 'router-product/sites/portal/dist',
                },
              },
            },
          ],
        },
      ],
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('release catalog helpers resolve a unique variant into concrete asset paths', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-catalog.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-catalog-helpers-'));

  try {
    const releaseCatalogPath = writeJson(fixtureRoot, 'release-assets/release-catalog.json', {
      version: 1,
      type: 'sdkwork-release-catalog',
      releaseTag: 'release-2026-04-18-1',
      generatedAt: '2026-04-18T12:34:56.789Z',
      productCount: 2,
      variantCount: 2,
      products: [
        {
          productId: 'sdkwork-api-router-product-server',
          variants: [
            {
              platform: 'linux',
              arch: 'x64',
              outputDirectory: 'native/linux/x64/bundles',
              variantKind: 'server-archive',
              primaryFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
              primaryFileSizeBytes: 14,
              checksumFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
              checksumAlgorithm: 'sha256',
              manifestFile: 'sdkwork-api-router-product-server-linux-x64.manifest.json',
              sha256: 'serverdigest',
              manifest: {
                type: 'product-server-archive',
                productId: 'sdkwork-api-router-product-server',
                platform: 'linux',
                arch: 'x64',
              },
            },
          ],
        },
        {
          productId: 'sdkwork-router-portal-desktop',
          variants: [
            {
              platform: 'windows',
              arch: 'x64',
              outputDirectory: 'native/windows/x64/desktop/portal',
              variantKind: 'desktop-installer',
              primaryFile: 'sdkwork-router-portal-desktop-windows-x64.exe',
              primaryFileSizeBytes: 17,
              checksumFile: 'sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt',
              checksumAlgorithm: 'sha256',
              manifestFile: 'sdkwork-router-portal-desktop-windows-x64.manifest.json',
              sha256: 'desktopdigest',
              manifest: {
                type: 'portal-desktop-installer',
                productId: 'sdkwork-router-portal-desktop',
                platform: 'windows',
                arch: 'x64',
              },
            },
          ],
        },
      ],
    });

    const catalog = module.readReleaseCatalogFile({
      releaseCatalogPath,
    });
    const variant = module.findReleaseCatalogVariant(catalog, {
      productId: 'sdkwork-api-router-product-server',
      variantKind: 'server-archive',
      platform: 'linux',
      arch: 'x64',
    });
    const resolvedPaths = module.resolveReleaseCatalogVariantPaths({
      releaseCatalogPath,
      productId: 'sdkwork-api-router-product-server',
      variantKind: 'server-archive',
      platform: 'linux',
      arch: 'x64',
    });

    assert.equal(variant?.primaryFile, 'sdkwork-api-router-product-server-linux-x64.tar.gz');
    assert.equal(
      resolvedPaths.primaryPath,
      path.join(fixtureRoot, 'release-assets', 'native', 'linux', 'x64', 'bundles', 'sdkwork-api-router-product-server-linux-x64.tar.gz'),
    );
    assert.equal(
      resolvedPaths.manifestPath,
      path.join(fixtureRoot, 'release-assets', 'native', 'linux', 'x64', 'bundles', 'sdkwork-api-router-product-server-linux-x64.manifest.json'),
    );
    assert.equal(
      resolvedPaths.checksumPath,
      path.join(fixtureRoot, 'release-assets', 'native', 'linux', 'x64', 'bundles', 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt'),
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
