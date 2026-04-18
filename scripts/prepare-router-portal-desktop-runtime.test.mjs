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
