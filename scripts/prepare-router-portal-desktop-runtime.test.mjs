import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('portal desktop runtime preparation stages a release-like router-product resource tree', async () => {
  const module = await import(
    pathToFileURL(
      path.join(workspaceRoot, 'scripts', 'prepare-router-portal-desktop-runtime.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolvePortalDesktopRuntimeResourceRoot, 'function');
  assert.equal(typeof module.createPortalDesktopRuntimeBuildPlan, 'function');
  assert.equal(typeof module.resolvePortalDesktopRuntimeResourceLayout, 'function');
  assert.equal(typeof module.stagePortalDesktopRuntimeResources, 'function');

  const resourceRoot = module.resolvePortalDesktopRuntimeResourceRoot({
    workspaceRoot,
  });
  assert.equal(
    resourceRoot.replaceAll('\\', '/'),
    path.join(workspaceRoot, 'bin', 'portal-rt').replaceAll('\\', '/'),
  );

  const layout = module.resolvePortalDesktopRuntimeResourceLayout({
    workspaceRoot,
    platform: 'win32',
    targetTriple: 'x86_64-pc-windows-msvc',
  });

  assert.equal(
    layout.routerProductRoot.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product').replaceAll('\\', '/'),
  );
  assert.equal(
    layout.routerBinaryPath.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product', 'bin', 'router-product-service.exe').replaceAll('\\', '/'),
  );
  assert.equal(
    layout.adminSiteDir.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product', 'sites', 'admin', 'dist').replaceAll('\\', '/'),
  );
  assert.equal(
    layout.portalSiteDir.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product', 'sites', 'portal', 'dist').replaceAll('\\', '/'),
  );
  assert.equal(
    layout.bootstrapDataDir.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product', 'data').replaceAll('\\', '/'),
  );
  assert.equal(
    layout.releasePayloadManifestPath.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product', 'release-manifest.json').replaceAll('\\', '/'),
  );
  assert.equal(
    layout.releasePayloadReadmePath.replaceAll('\\', '/'),
    path.join(resourceRoot, 'router-product', 'README.txt').replaceAll('\\', '/'),
  );

  const plan = module.createPortalDesktopRuntimeBuildPlan({
    workspaceRoot,
    platform: 'win32',
    targetTriple: 'x86_64-pc-windows-msvc',
    env: {},
  });

  assert.equal(plan.length, 4);
  assert.deepEqual(
    plan.map((step) => step.label),
    [
      'build admin frontend',
      'build portal frontend',
      'build router-product-service',
      'stage portal desktop router-product resources',
    ],
  );
  assert.equal(plan[2].command, 'cargo');
  assert.deepEqual(plan[2].args, [
    'build',
    '--release',
    '--target',
    'x86_64-pc-windows-msvc',
    '-p',
    'router-product-service',
  ]);
});

test('portal desktop runtime staging materializes bootstrap data and payload metadata beside the sidecar binary', async () => {
  const module = await import(
    pathToFileURL(
      path.join(workspaceRoot, 'scripts', 'prepare-router-portal-desktop-runtime.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-portal-desktop-runtime-'));
  const routerBinaryPath = path.join(fixtureRoot, 'target', 'x86_64-unknown-linux-gnu', 'release', 'router-product-service');
  const adminSiteDir = path.join(fixtureRoot, 'apps', 'sdkwork-router-admin', 'dist');
  const portalSiteDir = path.join(fixtureRoot, 'apps', 'sdkwork-router-portal', 'dist');
  const bootstrapDataDir = path.join(fixtureRoot, 'data', 'profiles');

  try {
    mkdirSync(path.dirname(routerBinaryPath), { recursive: true });
    mkdirSync(adminSiteDir, { recursive: true });
    mkdirSync(portalSiteDir, { recursive: true });
    mkdirSync(bootstrapDataDir, { recursive: true });

    writeFileSync(routerBinaryPath, '#!/usr/bin/env sh\nexit 0\n', 'utf8');
    writeFileSync(path.join(adminSiteDir, 'index.html'), '<html>admin</html>\n', 'utf8');
    writeFileSync(path.join(portalSiteDir, 'index.html'), '<html>portal</html>\n', 'utf8');
    writeFileSync(path.join(bootstrapDataDir, 'prod.json'), '{"profile":"prod"}\n', 'utf8');

    const layout = module.stagePortalDesktopRuntimeResources({
      workspaceRoot: fixtureRoot,
      platform: 'linux',
      targetTriple: 'x86_64-unknown-linux-gnu',
      env: {},
    });

    assert.equal(
      readFileSync(layout.routerBinaryPath, 'utf8'),
      '#!/usr/bin/env sh\nexit 0\n',
    );
    assert.equal(
      readFileSync(path.join(layout.adminSiteDir, 'index.html'), 'utf8'),
      '<html>admin</html>\n',
    );
    assert.equal(
      readFileSync(path.join(layout.portalSiteDir, 'index.html'), 'utf8'),
      '<html>portal</html>\n',
    );
    assert.equal(
      readFileSync(path.join(layout.bootstrapDataDir, 'profiles', 'prod.json'), 'utf8'),
      '{"profile":"prod"}\n',
    );

    const releaseManifest = JSON.parse(readFileSync(layout.releasePayloadManifestPath, 'utf8'));
    assert.deepEqual(releaseManifest, {
      type: 'portal-desktop-router-product',
      productId: 'sdkwork-router-portal-desktop',
      platform: 'linux',
      arch: 'x64',
      target: 'x86_64-unknown-linux-gnu',
      routerBinary: 'bin/router-product-service',
      sites: ['admin', 'portal'],
      bootstrapDataRoots: ['data'],
    });
    assert.match(
      readFileSync(layout.releasePayloadReadmePath, 'utf8'),
      /fixed public port 3001/i,
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('portal desktop runtime preparation consumes the shared embedded runtime layout catalog', () => {
  const source = readFileSync(
    path.join(workspaceRoot, 'scripts', 'prepare-router-portal-desktop-runtime.mjs'),
    'utf8',
  );

  assert.match(
    source,
    /native-runtime-layout-catalog\.mjs/,
    'portal desktop runtime preparation must consume the shared embedded runtime layout catalog',
  );
  assert.doesNotMatch(
    source,
    /path\.join\(routerProductRoot,\s*['"`]bin['"`],\s*withExecutable\(|path\.join\(routerProductRoot,\s*['"`]sites['"`],\s*['"`]admin['"`],\s*['"`]dist['"`]\)|path\.join\(routerProductRoot,\s*['"`]sites['"`],\s*['"`]portal['"`],\s*['"`]dist['"`]\)|path\.join\(routerProductRoot,\s*['"`]data['"`]\)|path\.join\(routerProductRoot,\s*['"`]release-manifest\.json['"`]\)|path\.join\(routerProductRoot,\s*['"`]README\.txt['"`]\)/,
    'portal desktop runtime preparation must not hardcode embedded runtime resource paths after catalog extraction',
  );
  assert.doesNotMatch(
    source,
    /-\s+sites\/admin\/dist: bundled admin site assets|-\s+sites\/portal\/dist: bundled portal site assets|-\s+data: bootstrap data packs for first-start initialization|-\s+release-manifest\.json: embedded runtime payload contract metadata|-\s+README\.txt: operator-facing payload notes/,
    'portal desktop runtime readme content must derive embedded layout paths from the shared catalog',
  );
});

test('portal desktop tauri bundle resources and rust sidecar lookup stay aligned to the embedded runtime catalog', async () => {
  const module = await import(
    pathToFileURL(
      path.join(workspaceRoot, 'scripts', 'prepare-router-portal-desktop-runtime.mjs'),
    ).href,
  );
  const runtimeCatalog = await import(
    pathToFileURL(
      path.join(workspaceRoot, 'scripts', 'release', 'native-runtime-layout-catalog.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolvePortalDesktopRuntimeTauriResourceMap, 'function');

  const portalAppRoot = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal');
  const tauriConfig = JSON.parse(
    readFileSync(path.join(portalAppRoot, 'src-tauri', 'tauri.conf.json'), 'utf8'),
  );
  const desktopRuntimeSource = readFileSync(
    path.join(portalAppRoot, 'src-tauri', 'src', 'desktop_runtime.rs'),
    'utf8',
  );
  const runtimeLayout = runtimeCatalog.findNativePortalDesktopEmbeddedRuntimeLayoutSpec();
  const resourceMap = module.resolvePortalDesktopRuntimeTauriResourceMap({
    workspaceRoot,
    appRoot: portalAppRoot,
  });

  assert.deepEqual(tauriConfig.bundle.resources, {
    [resourceMap.sourceRelativePath]: resourceMap.targetRelativePath,
  });

  const embeddedRootName = runtimeLayout.readmeFile.split('/')[0];
  const routerBinaryDir = path.posix.relative(embeddedRootName, runtimeLayout.serviceBinaryDir);
  const adminSiteDir = path.posix.relative(embeddedRootName, runtimeLayout.siteTargetDirs.admin);
  const portalSiteDir = path.posix.relative(embeddedRootName, runtimeLayout.siteTargetDirs.portal);
  const bootstrapDataDir = path.posix.relative(
    embeddedRootName,
    runtimeLayout.bootstrapDataRootDirs.data,
  );
  const releaseManifestFile = path.posix.relative(
    embeddedRootName,
    runtimeLayout.releaseManifestFile,
  );
  const readmeFile = path.posix.relative(embeddedRootName, runtimeLayout.readmeFile);

  assert.match(desktopRuntimeSource, new RegExp(String.raw`join\("${embeddedRootName}"\)`));
  assert.match(
    desktopRuntimeSource,
    new RegExp(String.raw`root\.join\("${routerBinaryDir}"\)\.join\(ROUTER_BINARY_NAME\)`),
  );
  assert.match(
    desktopRuntimeSource,
    new RegExp(
      String.raw`root\.join\("${adminSiteDir.split('/')[0]}"\)\.join\("${adminSiteDir.split('/')[1]}"\)\.join\("${adminSiteDir.split('/')[2]}"\)`,
    ),
  );
  assert.match(
    desktopRuntimeSource,
    new RegExp(
      String.raw`root\.join\("${portalSiteDir.split('/')[0]}"\)\.join\("${portalSiteDir.split('/')[1]}"\)\.join\("${portalSiteDir.split('/')[2]}"\)`,
    ),
  );
  assert.match(desktopRuntimeSource, new RegExp(String.raw`root\.join\("${bootstrapDataDir}"\)`));
  assert.match(
    desktopRuntimeSource,
    new RegExp(String.raw`root\.join\("${releaseManifestFile.replace('.', '\\.')}"\)`),
  );
  assert.match(
    desktopRuntimeSource,
    new RegExp(String.raw`root\.join\("${readmeFile.replace('.', '\\.')}"\)`),
  );
});
