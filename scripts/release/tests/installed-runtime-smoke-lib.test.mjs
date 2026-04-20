import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'installed-runtime-smoke-lib.mjs'),
    ).href,
  );
}

function createTempDirectory(prefix) {
  return mkdtempSync(path.join(os.tmpdir(), `${prefix}-`));
}

test('assertInstalledRuntimeBackupBundle resolves required bundle contents from manifest-declared paths', async () => {
  const module = await loadModule();
  const backupRoot = createTempDirectory('installed-runtime-backup-bundle');

  try {
    mkdirSync(path.join(backupRoot, 'bundle-state', 'control'), { recursive: true });
    mkdirSync(path.join(backupRoot, 'bundle-state', 'config-snapshot'), { recursive: true });
    mkdirSync(path.join(backupRoot, 'bundle-state', 'data-snapshot'), { recursive: true });

    writeFileSync(
      path.join(backupRoot, 'backup-manifest.json'),
      `${JSON.stringify({
        formatVersion: 2,
        createdAt: '2026-04-20T00:00:00Z',
        runtimeHome: '/opt/sdkwork-api-router/current',
        configRoot: '/etc/sdkwork-api-router',
        mutableDataRoot: '/var/lib/sdkwork-api-router',
        logRoot: '/var/log/sdkwork-api-router',
        runRoot: '/run/sdkwork-api-router',
        bundle: {
          controlManifestFile: 'bundle-state/control/runtime-manifest.json',
          configSnapshotRoot: 'bundle-state/config-snapshot',
          mutableDataSnapshotRoot: 'bundle-state/data-snapshot',
        },
        database: {
          kind: 'sqlite',
          strategy: 'file-copy',
          dumpFile: null,
        },
      }, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(
      path.join(backupRoot, 'bundle-state', 'control', 'runtime-manifest.json'),
      '{}\n',
      'utf8',
    );

    const result = module.assertInstalledRuntimeBackupBundle(backupRoot);

    assert.equal(
      result.controlManifestPath,
      path.join(backupRoot, 'bundle-state', 'control', 'runtime-manifest.json'),
    );
    assert.equal(
      result.configSnapshotRoot,
      path.join(backupRoot, 'bundle-state', 'config-snapshot'),
    );
    assert.equal(
      result.mutableDataSnapshotRoot,
      path.join(backupRoot, 'bundle-state', 'data-snapshot'),
    );
  } finally {
    rmSync(backupRoot, { recursive: true, force: true });
  }
});

test('assertInstalledRuntimeBackupBundle rejects backup manifests that omit the bundle path contract', async () => {
  const module = await loadModule();
  const backupRoot = createTempDirectory('installed-runtime-backup-manifest');

  try {
    writeFileSync(
      path.join(backupRoot, 'backup-manifest.json'),
      `${JSON.stringify({
        formatVersion: 2,
        createdAt: '2026-04-20T00:00:00Z',
        runtimeHome: '/opt/sdkwork-api-router/current',
        configRoot: '/etc/sdkwork-api-router',
        mutableDataRoot: '/var/lib/sdkwork-api-router',
        logRoot: '/var/log/sdkwork-api-router',
        runRoot: '/run/sdkwork-api-router',
        database: {
          kind: 'sqlite',
          strategy: 'file-copy',
          dumpFile: null,
        },
      }, null, 2)}\n`,
      'utf8',
    );

    assert.throws(
      () => module.assertInstalledRuntimeBackupBundle(backupRoot),
      /bundle.*controlManifestFile|bundle.*configSnapshotRoot|bundle.*mutableDataSnapshotRoot/i,
    );
  } finally {
    rmSync(backupRoot, { recursive: true, force: true });
  }
});

test('assertInstalledReleasePayloadContract validates the installed current release manifest against the active release payload layout', async () => {
  const module = await loadModule();
  const runtimeHome = createTempDirectory('installed-runtime-release-contract');
  const currentRoot = path.join(runtimeHome, 'current');
  const releaseRoot = path.join(runtimeHome, 'releases', '1.2.3');
  const routerBinary = path.join(releaseRoot, 'bin', process.platform === 'win32' ? 'router-product-service.exe' : 'router-product-service');
  const bootstrapDataRoot = path.join(releaseRoot, 'data');
  const deploymentAssetRoot = path.join(releaseRoot, 'deploy');
  const releasePayloadManifest = path.join(releaseRoot, 'release-manifest.json');
  const releasePayloadReadmeFile = path.join(releaseRoot, 'README.txt');
  const adminSiteDistDir = path.join(releaseRoot, 'sites', 'admin', 'dist');
  const portalSiteDistDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const configRoot = path.join(runtimeHome, 'config');
  const configFile = path.join(configRoot, 'router.yaml');
  const mutableDataRoot = path.join(runtimeHome, 'data');
  const logRoot = path.join(runtimeHome, 'log');
  const runRoot = path.join(runtimeHome, 'run');

  try {
    mkdirSync(currentRoot, { recursive: true });
    mkdirSync(path.dirname(routerBinary), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'channels'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'providers'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'routing'), { recursive: true });
    mkdirSync(deploymentAssetRoot, { recursive: true });
    mkdirSync(adminSiteDistDir, { recursive: true });
    mkdirSync(portalSiteDistDir, { recursive: true });
    mkdirSync(configRoot, { recursive: true });
    mkdirSync(mutableDataRoot, { recursive: true });
    mkdirSync(logRoot, { recursive: true });
    mkdirSync(runRoot, { recursive: true });

    writeFileSync(routerBinary, '#!/usr/bin/env sh\nexit 0\n', 'utf8');
    writeFileSync(releasePayloadManifest, '{}\n', 'utf8');
    writeFileSync(releasePayloadReadmeFile, 'release readme\n', 'utf8');
    writeFileSync(configFile, 'router: {}\n', 'utf8');
    writeFileSync(path.join(adminSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');
    writeFileSync(path.join(portalSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');

    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        runtime: 'sdkwork-api-router',
        layoutVersion: 2,
        installMode: 'portable',
        productRoot: runtimeHome,
        controlRoot: currentRoot,
        releaseVersion: '1.2.3',
        releasesRoot: path.join(runtimeHome, 'releases'),
        releaseRoot,
        target: process.platform === 'win32' ? 'x86_64-pc-windows-msvc' : 'x86_64-unknown-linux-gnu',
        installedBinaries: ['router-product-service'],
        bootstrapDataRoot,
        deploymentAssetRoot,
        releasePayloadManifest,
        releasePayloadReadmeFile,
        adminSiteDistDir,
        portalSiteDistDir,
        routerBinary,
        configRoot,
        configFile,
        mutableDataRoot,
        logRoot,
        runRoot,
        installedAt: '2026-04-20T00:00:00.000Z',
      }, null, 2)}\n`,
      'utf8',
    );

    const result = module.assertInstalledReleasePayloadContract(runtimeHome);

    assert.equal(result.manifest.releaseVersion, '1.2.3');
    assert.equal(result.releaseRoot, releaseRoot);
    assert.equal(result.routerBinary, routerBinary);
    assert.equal(result.adminSiteDistDir, adminSiteDistDir);
    assert.equal(result.portalSiteDistDir, portalSiteDistDir);
    assert.equal(result.releasePayloadManifest, releasePayloadManifest);
    assert.equal(result.releasePayloadReadmeFile, releasePayloadReadmeFile);
  } finally {
    rmSync(runtimeHome, { recursive: true, force: true });
  }
});

test('assertInstalledReleasePayloadContract rejects installed manifests that omit runtime contract metadata', async () => {
  const module = await loadModule();
  const runtimeHome = createTempDirectory('installed-runtime-release-metadata');
  const currentRoot = path.join(runtimeHome, 'current');
  const releaseRoot = path.join(runtimeHome, 'releases', '1.2.3');
  const routerBinary = path.join(releaseRoot, 'bin', process.platform === 'win32' ? 'router-product-service.exe' : 'router-product-service');
  const bootstrapDataRoot = path.join(releaseRoot, 'data');
  const deploymentAssetRoot = path.join(releaseRoot, 'deploy');
  const releasePayloadManifest = path.join(releaseRoot, 'release-manifest.json');
  const releasePayloadReadmeFile = path.join(releaseRoot, 'README.txt');
  const adminSiteDistDir = path.join(releaseRoot, 'sites', 'admin', 'dist');
  const portalSiteDistDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const configRoot = path.join(runtimeHome, 'config');
  const configFile = path.join(configRoot, 'router.yaml');
  const mutableDataRoot = path.join(runtimeHome, 'data');
  const logRoot = path.join(runtimeHome, 'log');
  const runRoot = path.join(runtimeHome, 'run');

  try {
    mkdirSync(currentRoot, { recursive: true });
    mkdirSync(path.dirname(routerBinary), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'channels'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'providers'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'routing'), { recursive: true });
    mkdirSync(deploymentAssetRoot, { recursive: true });
    mkdirSync(adminSiteDistDir, { recursive: true });
    mkdirSync(portalSiteDistDir, { recursive: true });
    mkdirSync(configRoot, { recursive: true });
    mkdirSync(mutableDataRoot, { recursive: true });
    mkdirSync(logRoot, { recursive: true });
    mkdirSync(runRoot, { recursive: true });

    writeFileSync(routerBinary, '#!/usr/bin/env sh\nexit 0\n', 'utf8');
    writeFileSync(releasePayloadManifest, '{}\n', 'utf8');
    writeFileSync(releasePayloadReadmeFile, 'release readme\n', 'utf8');
    writeFileSync(configFile, 'router: {}\n', 'utf8');
    writeFileSync(path.join(adminSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');
    writeFileSync(path.join(portalSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');

    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        installMode: 'portable',
        productRoot: runtimeHome,
        controlRoot: currentRoot,
        releaseVersion: '1.2.3',
        releasesRoot: path.join(runtimeHome, 'releases'),
        releaseRoot,
        target: process.platform === 'win32' ? 'x86_64-pc-windows-msvc' : 'x86_64-unknown-linux-gnu',
        installedBinaries: ['router-product-service'],
        bootstrapDataRoot,
        deploymentAssetRoot,
        releasePayloadManifest,
        releasePayloadReadmeFile,
        adminSiteDistDir,
        portalSiteDistDir,
        routerBinary,
        configRoot,
        configFile,
        mutableDataRoot,
        logRoot,
        runRoot,
        installedAt: '2026-04-20T00:00:00.000Z',
      }, null, 2)}\n`,
      'utf8',
    );

    let error;
    try {
      module.assertInstalledReleasePayloadContract(runtimeHome);
    } catch (thrownError) {
      error = thrownError;
    }
    assert.ok(error instanceof Error);
    assert.match(error.message, /runtime/i);
    assert.match(error.message, /layoutVersion/i);
  } finally {
    rmSync(runtimeHome, { recursive: true, force: true });
  }
});

test('assertInstalledReleasePayloadContract rejects installed manifests whose runtime contract metadata has unsupported values', async () => {
  const module = await loadModule();
  const runtimeHome = createTempDirectory('installed-runtime-release-metadata-values');
  const currentRoot = path.join(runtimeHome, 'current');
  const releaseRoot = path.join(runtimeHome, 'releases', '1.2.3');
  const routerBinary = path.join(releaseRoot, 'bin', process.platform === 'win32' ? 'router-product-service.exe' : 'router-product-service');
  const bootstrapDataRoot = path.join(releaseRoot, 'data');
  const deploymentAssetRoot = path.join(releaseRoot, 'deploy');
  const releasePayloadManifest = path.join(releaseRoot, 'release-manifest.json');
  const releasePayloadReadmeFile = path.join(releaseRoot, 'README.txt');
  const adminSiteDistDir = path.join(releaseRoot, 'sites', 'admin', 'dist');
  const portalSiteDistDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const configRoot = path.join(runtimeHome, 'config');
  const configFile = path.join(configRoot, 'router.yaml');
  const mutableDataRoot = path.join(runtimeHome, 'data');
  const logRoot = path.join(runtimeHome, 'log');
  const runRoot = path.join(runtimeHome, 'run');

  try {
    mkdirSync(currentRoot, { recursive: true });
    mkdirSync(path.dirname(routerBinary), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'channels'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'providers'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'routing'), { recursive: true });
    mkdirSync(deploymentAssetRoot, { recursive: true });
    mkdirSync(adminSiteDistDir, { recursive: true });
    mkdirSync(portalSiteDistDir, { recursive: true });
    mkdirSync(configRoot, { recursive: true });
    mkdirSync(mutableDataRoot, { recursive: true });
    mkdirSync(logRoot, { recursive: true });
    mkdirSync(runRoot, { recursive: true });

    writeFileSync(routerBinary, '#!/usr/bin/env sh\nexit 0\n', 'utf8');
    writeFileSync(releasePayloadManifest, '{}\n', 'utf8');
    writeFileSync(releasePayloadReadmeFile, 'release readme\n', 'utf8');
    writeFileSync(configFile, 'router: {}\n', 'utf8');
    writeFileSync(path.join(adminSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');
    writeFileSync(path.join(portalSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');

    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        runtime: 'custom-router',
        layoutVersion: 3,
        installMode: 'portable',
        productRoot: runtimeHome,
        controlRoot: currentRoot,
        releaseVersion: '1.2.3',
        releasesRoot: path.join(runtimeHome, 'releases'),
        releaseRoot,
        target: process.platform === 'win32' ? 'x86_64-pc-windows-msvc' : 'x86_64-unknown-linux-gnu',
        installedBinaries: ['router-product-service'],
        bootstrapDataRoot,
        deploymentAssetRoot,
        releasePayloadManifest,
        releasePayloadReadmeFile,
        adminSiteDistDir,
        portalSiteDistDir,
        routerBinary,
        configRoot,
        configFile,
        mutableDataRoot,
        logRoot,
        runRoot,
        installedAt: '2026-04-20T00:00:00.000Z',
      }, null, 2)}\n`,
      'utf8',
    );

    let error;
    try {
      module.assertInstalledReleasePayloadContract(runtimeHome);
    } catch (thrownError) {
      error = thrownError;
    }
    assert.ok(error instanceof Error);
    assert.match(error.message, /runtime|layoutVersion/i);
  } finally {
    rmSync(runtimeHome, { recursive: true, force: true });
  }
});

test('assertInstalledReleasePayloadContract rejects installed manifests whose payload paths drift away from the active release layout', async () => {
  const module = await loadModule();
  const runtimeHome = createTempDirectory('installed-runtime-release-drift');
  const currentRoot = path.join(runtimeHome, 'current');
  const releaseRoot = path.join(runtimeHome, 'releases', '1.2.3');
  const adminSiteDistDir = path.join(runtimeHome, 'broken-admin-site');
  const portalSiteDistDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const routerBinary = path.join(releaseRoot, 'bin', process.platform === 'win32' ? 'router-product-service.exe' : 'router-product-service');
  const bootstrapDataRoot = path.join(releaseRoot, 'data');
  const deploymentAssetRoot = path.join(releaseRoot, 'deploy');
  const configRoot = path.join(runtimeHome, 'config');
  const mutableDataRoot = path.join(runtimeHome, 'data');
  const logRoot = path.join(runtimeHome, 'log');
  const runRoot = path.join(runtimeHome, 'run');

  try {
    mkdirSync(currentRoot, { recursive: true });
    mkdirSync(path.dirname(routerBinary), { recursive: true });
    mkdirSync(bootstrapDataRoot, { recursive: true });
    mkdirSync(deploymentAssetRoot, { recursive: true });
    mkdirSync(adminSiteDistDir, { recursive: true });
    mkdirSync(portalSiteDistDir, { recursive: true });
    mkdirSync(configRoot, { recursive: true });
    mkdirSync(mutableDataRoot, { recursive: true });
    mkdirSync(logRoot, { recursive: true });
    mkdirSync(runRoot, { recursive: true });

    writeFileSync(routerBinary, '#!/usr/bin/env sh\nexit 0\n', 'utf8');
    writeFileSync(path.join(releaseRoot, 'release-manifest.json'), '{}\n', 'utf8');
    writeFileSync(path.join(releaseRoot, 'README.txt'), 'release readme\n', 'utf8');
    writeFileSync(path.join(portalSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');
    writeFileSync(path.join(configRoot, 'router.yaml'), 'router: {}\n', 'utf8');

    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        runtime: 'sdkwork-api-router',
        layoutVersion: 2,
        installMode: 'portable',
        productRoot: runtimeHome,
        controlRoot: currentRoot,
        releaseVersion: '1.2.3',
        releasesRoot: path.join(runtimeHome, 'releases'),
        releaseRoot,
        target: process.platform === 'win32' ? 'x86_64-pc-windows-msvc' : 'x86_64-unknown-linux-gnu',
        installedBinaries: ['router-product-service'],
        bootstrapDataRoot,
        deploymentAssetRoot,
        releasePayloadManifest: path.join(releaseRoot, 'release-manifest.json'),
        releasePayloadReadmeFile: path.join(releaseRoot, 'README.txt'),
        adminSiteDistDir,
        portalSiteDistDir,
        routerBinary,
        configRoot,
        configFile: path.join(configRoot, 'router.yaml'),
        mutableDataRoot,
        logRoot,
        runRoot,
        installedAt: '2026-04-20T00:00:00.000Z',
      }, null, 2)}\n`,
      'utf8',
    );

    assert.throws(
      () => module.assertInstalledReleasePayloadContract(runtimeHome),
      /adminSiteDistDir|active release payload layout|releases\/1\.2\.3/i,
    );
  } finally {
    rmSync(runtimeHome, { recursive: true, force: true });
  }
});
