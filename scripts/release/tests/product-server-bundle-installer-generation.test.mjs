import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'product-server-bundle-installer-generation.mjs'),
    ).href,
  );
}

function createRuntimeLayout(overrides = {}) {
  return {
    id: 'product-server-bundle',
    layoutKind: 'product-server-bundle',
    serviceBinaryName: 'custom-router',
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
    ...overrides,
  };
}

test('bundle installer generators honor runtimeLayout.serviceBinaryName instead of hardcoding router-product-service', async () => {
  const module = await loadModule();
  const runtimeLayout = createRuntimeLayout();

  const shellScript = module.renderProductServerBundleInstallShellScript({
    releaseVersion: '1.2.3',
    runtimeLayout,
    platformId: 'linux',
    archId: 'x64',
    targetTriple: 'x86_64-unknown-linux-gnu',
    serviceBinaryNames: ['custom-router', 'gateway-service'],
  });
  assert.match(shellScript, /ROUTER_BINARY="\$RELEASE_BIN_DIR\/custom-router"/);
  assert.match(shellScript, /custom-router\.launchd\.stdout\.log/);
  assert.match(shellScript, /custom-router\.launchd\.stderr\.log/);
  assert.doesNotMatch(shellScript, /router-product-service(?![^"]*gateway-service)/);

  const powerShellScript = module.renderProductServerBundleInstallPowerShellScript({
    releaseVersion: '1.2.3',
    runtimeLayout,
    platformId: 'windows',
    archId: 'x64',
    targetTriple: 'x86_64-pc-windows-msvc',
    serviceBinaryNames: ['custom-router', 'gateway-service'],
  });
  assert.match(powerShellScript, /\$routerBinary = Join-Path \$releaseBinDir 'custom-router\.exe'/);
  assert.doesNotMatch(powerShellScript, /router-product-service\.exe/);
});

test('bundle installer generators consume the shared installed runtime layout catalog instead of inlining installed paths', () => {
  const source = readFileSync(
    path.join(repoRoot, 'scripts', 'release', 'product-server-bundle-installer-generation.mjs'),
    'utf8',
  );

  assert.match(source, /installed-runtime-layout-catalog\.mjs/);
  assert.doesNotMatch(source, /'CONTROL_ROOT="\$PRODUCT_ROOT\/current"',/);
  assert.doesNotMatch(source, /'RELEASE_BIN_DIR="\$RELEASE_ROOT\/bin"',/);
  assert.doesNotMatch(source, /'STATIC_DATA_DIR="\$RELEASE_ROOT\/data"',/);
  assert.doesNotMatch(source, /'DEPLOY_DIR="\$RELEASE_ROOT\/deploy"',/);
  assert.doesNotMatch(source, /'ADMIN_SITE_DIR="\$RELEASE_ROOT\/sites\/admin\/dist"',/);
  assert.doesNotMatch(source, /'PORTAL_SITE_DIR="\$RELEASE_ROOT\/sites\/portal\/dist"',/);
  assert.doesNotMatch(source, /'RELEASE_PAYLOAD_MANIFEST="\$RELEASE_ROOT\/release-manifest\.json"',/);
  assert.doesNotMatch(source, /'RELEASE_PAYLOAD_README="\$RELEASE_ROOT\/README\.txt"',/);
  assert.doesNotMatch(source, /'CURRENT_MANIFEST="\$CONTROL_ROOT\/release-manifest\.json"',/);
  assert.doesNotMatch(source, /"\$releasePayloadManifest = Join-Path \$releaseRoot 'release-manifest\.json'"/);
  assert.doesNotMatch(source, /"\$releasePayloadReadme = Join-Path \$releaseRoot 'README\.txt'"/);
  assert.doesNotMatch(source, /"\$currentManifest = Join-Path \$controlRoot 'release-manifest\.json'"/);
});

test('bundle installer generators write a scalar target triple in the installed runtime manifest contract', async () => {
  const module = await loadModule();
  const runtimeLayout = createRuntimeLayout();

  const shellScript = module.renderProductServerBundleInstallShellScript({
    releaseVersion: '1.2.3',
    runtimeLayout,
    platformId: 'linux',
    archId: 'x64',
    targetTriple: 'x86_64-unknown-linux-gnu',
    serviceBinaryNames: ['custom-router', 'gateway-service'],
  });
  assert.match(shellScript, /"target": "\$\{TARGET_TRIPLE\}"/);
  assert.doesNotMatch(shellScript, /"target": \{/);

  const powerShellScript = module.renderProductServerBundleInstallPowerShellScript({
    releaseVersion: '1.2.3',
    runtimeLayout,
    platformId: 'windows',
    archId: 'x64',
    targetTriple: 'x86_64-pc-windows-msvc',
    serviceBinaryNames: ['custom-router', 'gateway-service'],
  });
  assert.match(powerShellScript, /target = \$targetTriple/);
  assert.doesNotMatch(powerShellScript, /target = \[ordered\]@\{/);
});
