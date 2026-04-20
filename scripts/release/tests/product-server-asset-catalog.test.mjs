import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function writeText(root, relativePath, contents) {
  const targetPath = path.join(root, relativePath);
  mkdirSync(path.dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, contents, 'utf8');
  return targetPath;
}

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    ).href,
  );
}

function listUntrackedReleaseSources() {
  const gitResult = spawnSync(
    process.platform === 'win32' ? 'git.exe' : 'git',
    ['ls-files', '--others', '--exclude-standard', 'scripts/release', 'scripts/release/tests'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(gitResult.status, 0, `expected git ls-files to succeed, stderr=${gitResult.stderr ?? ''}`);
  return gitResult.stdout
    .split(/\r?\n/u)
    .map((entry) => entry.trim())
    .filter(Boolean)
    .filter((entry) => entry.endsWith('.mjs'));
}

test('product-server packaging helpers expose strict governed binary and asset-root catalogs', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listNativeServiceBinaryNames, 'function');
  assert.equal(typeof module.findNativeServiceBinaryName, 'function');
  assert.equal(typeof module.listNativeServiceBinaryNamesByIds, 'function');
  assert.equal(typeof module.listNativeProductServerSiteAssetRoots, 'function');
  assert.equal(typeof module.findNativeProductServerSiteAssetRoot, 'function');
  assert.equal(typeof module.listNativeProductServerSiteAssetRootsByIds, 'function');
  assert.equal(typeof module.listNativeProductServerBootstrapDataRoots, 'function');
  assert.equal(typeof module.findNativeProductServerBootstrapDataRoot, 'function');
  assert.equal(typeof module.listNativeProductServerBootstrapDataRootsByIds, 'function');
  assert.equal(typeof module.listNativeProductServerDeploymentAssetRoots, 'function');
  assert.equal(typeof module.findNativeProductServerDeploymentAssetRoot, 'function');
  assert.equal(typeof module.listNativeProductServerDeploymentAssetRootsByIds, 'function');
  assert.equal(typeof module.listNativeProductServerControlScriptNames, 'function');
  assert.equal(typeof module.findNativeProductServerControlScriptName, 'function');
  assert.equal(typeof module.listNativeProductServerControlScriptNamesByIds, 'function');
  assert.equal(typeof module.listNativeProductServerControlLibNames, 'function');
  assert.equal(typeof module.findNativeProductServerControlLibName, 'function');
  assert.equal(typeof module.listNativeProductServerControlLibNamesByIds, 'function');

  assert.deepEqual(
    module.listNativeServiceBinaryNames(),
    [
      'admin-api-service',
      'gateway-service',
      'portal-api-service',
      'router-web-service',
      'router-product-service',
    ],
  );
  assert.equal(
    module.findNativeServiceBinaryName('router-product-service'),
    'router-product-service',
  );
  assert.deepEqual(
    module.listNativeServiceBinaryNamesByIds([
      'gateway-service',
      'router-product-service',
    ]),
    [
      'gateway-service',
      'router-product-service',
    ],
  );

  const expectedSiteRoots = {
    admin: path.join(repoRoot, 'apps', 'sdkwork-router-admin', 'dist'),
    portal: path.join(repoRoot, 'apps', 'sdkwork-router-portal', 'dist'),
  };
  assert.deepEqual(
    module.listNativeProductServerSiteAssetRoots(),
    expectedSiteRoots,
  );
  const mutatedSiteRoots = module.listNativeProductServerSiteAssetRoots();
  mutatedSiteRoots.admin = 'mutated-locally';
  assert.deepEqual(
    module.listNativeProductServerSiteAssetRoots(),
    expectedSiteRoots,
  );
  assert.equal(
    module.findNativeProductServerSiteAssetRoot('portal'),
    expectedSiteRoots.portal,
  );
  assert.deepEqual(
    module.listNativeProductServerSiteAssetRootsByIds([
      'portal',
      'admin',
    ]),
    {
      portal: expectedSiteRoots.portal,
      admin: expectedSiteRoots.admin,
    },
  );

  const expectedBootstrapRoots = {
    data: path.join(repoRoot, 'data'),
  };
  assert.deepEqual(
    module.listNativeProductServerBootstrapDataRoots(),
    expectedBootstrapRoots,
  );
  const mutatedBootstrapRoots = module.listNativeProductServerBootstrapDataRoots();
  mutatedBootstrapRoots.data = 'mutated-locally';
  assert.deepEqual(
    module.listNativeProductServerBootstrapDataRoots(),
    expectedBootstrapRoots,
  );
  assert.equal(
    module.findNativeProductServerBootstrapDataRoot('data'),
    expectedBootstrapRoots.data,
  );
  assert.deepEqual(
    module.listNativeProductServerBootstrapDataRootsByIds([
      'data',
    ]),
    expectedBootstrapRoots,
  );

  const expectedDeploymentRoots = {
    deploy: path.join(repoRoot, 'deploy'),
  };
  assert.deepEqual(
    module.listNativeProductServerDeploymentAssetRoots(),
    expectedDeploymentRoots,
  );
  const mutatedDeploymentRoots = module.listNativeProductServerDeploymentAssetRoots();
  mutatedDeploymentRoots.deploy = 'mutated-locally';
  assert.deepEqual(
    module.listNativeProductServerDeploymentAssetRoots(),
    expectedDeploymentRoots,
  );
  assert.equal(
    module.findNativeProductServerDeploymentAssetRoot('deploy'),
    expectedDeploymentRoots.deploy,
  );
  assert.deepEqual(
    module.listNativeProductServerDeploymentAssetRootsByIds([
      'deploy',
    ]),
    expectedDeploymentRoots,
  );

  assert.deepEqual(
    module.listNativeProductServerControlScriptNames(),
    [
      'start.sh',
      'stop.sh',
      'backup.sh',
      'restore.sh',
      'support-bundle.sh',
      'validate-config.sh',
      'start.ps1',
      'stop.ps1',
      'backup.ps1',
      'restore.ps1',
      'support-bundle.ps1',
      'validate-config.ps1',
    ],
  );
  assert.equal(
    module.findNativeProductServerControlScriptName('start.sh'),
    'start.sh',
  );
  assert.deepEqual(
    module.listNativeProductServerControlScriptNamesByIds([
      'support-bundle.sh',
      'validate-config.ps1',
    ]),
    [
      'support-bundle.sh',
      'validate-config.ps1',
    ],
  );

  assert.deepEqual(
    module.listNativeProductServerControlLibNames(),
    [
      'runtime-common.sh',
      'runtime-common.ps1',
    ],
  );
  assert.equal(
    module.findNativeProductServerControlLibName('runtime-common.sh'),
    'runtime-common.sh',
  );
  assert.deepEqual(
    module.listNativeProductServerControlLibNamesByIds([
      'runtime-common.ps1',
    ]),
    [
      'runtime-common.ps1',
    ],
  );

  assert.throws(
    () => module.findNativeServiceBinaryName('missing-service-binary'),
    /missing native service binary name.*missing-service-binary/i,
  );
  assert.throws(
    () => module.findNativeProductServerSiteAssetRoot('missing-site-root'),
    /missing native product server site asset root.*missing-site-root/i,
  );
  assert.throws(
    () => module.findNativeProductServerBootstrapDataRoot('missing-bootstrap-root'),
    /missing native product server bootstrap data root.*missing-bootstrap-root/i,
  );
  assert.throws(
    () => module.findNativeProductServerDeploymentAssetRoot('missing-deployment-root'),
    /missing native product server deployment asset root.*missing-deployment-root/i,
  );
  assert.throws(
    () => module.findNativeProductServerControlScriptName('missing-control-script'),
    /missing native product server control script name.*missing-control-script/i,
  );
  assert.throws(
    () => module.findNativeProductServerControlLibName('missing-control-lib'),
    /missing native product server control lib name.*missing-control-lib/i,
  );
});

test('product-server packaging internals do not maintain legacy ad-hoc binary and asset-root tables', () => {
  const source = readFileSync(
    path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    'utf8',
  );

  assert.match(
    source,
    /createStrictKeyedCatalog/,
    'product-server packaging should use the shared strict catalog helper',
  );
  assert.doesNotMatch(
    source,
    /const SERVICE_BINARY_NAMES = \[/,
    'service binaries must not live in a standalone ad-hoc array',
  );
  assert.doesNotMatch(
    source,
    /const productServerSiteAssetRoots = \{/,
    'site asset roots must not live in a standalone ad-hoc object',
  );
  assert.doesNotMatch(
    source,
    /const productServerBootstrapDataRoots = \{/,
    'bootstrap data roots must not live in a standalone ad-hoc object',
  );
  assert.doesNotMatch(
    source,
    /const productServerDeploymentAssetRoots = \{/,
    'deployment asset roots must not live in a standalone ad-hoc object',
  );
});

test('release packaging sources and their contract coverage stay under version control', () => {
  assert.deepEqual(
    listUntrackedReleaseSources(),
    [],
    'all scripts/release source files and contract tests must be tracked so clean checkouts can reproduce packaging and governance behavior',
  );
});

test('product-server packaging materializes bundle-native install entrypoints and release installer metadata', async () => {
  const module = await loadModule();
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-server-bundle-'));
  const outputDir = path.join(fixtureRoot, 'release-assets');
  const serviceReleaseRoot = path.join(fixtureRoot, 'service-release');
  const siteRoot = path.join(fixtureRoot, 'sites');
  const bootstrapRoot = path.join(fixtureRoot, 'bootstrap');
  const deployRoot = path.join(fixtureRoot, 'deploy');
  const inspectedArchiveRoot = path.join(fixtureRoot, 'inspected-archive');

  try {
    writeText(
      serviceReleaseRoot,
      'router-product-service',
      '#!/usr/bin/env sh\nprintf \'%s\\n\' "router-product-service fixture"\n',
    );
    writeText(siteRoot, 'admin/dist/index.html', '<html>admin fixture</html>\n');
    writeText(siteRoot, 'portal/dist/index.html', '<html>portal fixture</html>\n');
    writeText(bootstrapRoot, 'data/providers/default.json', '{}\n');
    writeText(deployRoot, 'deploy/docker/docker-compose.yml', 'services:\n  router:\n    image: sdkwork\n');

    const bundle = module.packageProductServerBundle({
      platformId: 'linux',
      archId: 'x64',
      targetTriple: 'x86_64-unknown-linux-gnu',
      outputDir,
      serviceBinaryNames: ['router-product-service'],
      resolveServiceRoot: () => serviceReleaseRoot,
      siteAssetRoots: {
        admin: path.join(siteRoot, 'admin', 'dist'),
        portal: path.join(siteRoot, 'portal', 'dist'),
      },
      bootstrapDataRoots: {
        data: path.join(bootstrapRoot, 'data'),
      },
      deploymentAssetRoots: {
        deploy: path.join(deployRoot, 'deploy'),
      },
      runTar: (archivePath, stagingRoot, entryName) => {
        cpSync(path.join(stagingRoot, entryName), inspectedArchiveRoot, { recursive: true });
        writeFileSync(archivePath, 'fixture-archive', 'utf8');
      },
    });

    assert.equal(bundle.fileName, 'sdkwork-api-router-product-server-linux-x64.tar.gz');
    assert.equal(readFileSync(path.join(inspectedArchiveRoot, 'install.sh'), 'utf8').includes('router-ops.mjs'), false);
    assert.equal(readFileSync(path.join(inspectedArchiveRoot, 'install.sh'), 'utf8').includes('node '), false);
    assert.equal(readFileSync(path.join(inspectedArchiveRoot, 'install.ps1'), 'utf8').includes('router-ops.mjs'), false);

    const embeddedManifest = JSON.parse(
      readFileSync(path.join(inspectedArchiveRoot, 'release-manifest.json'), 'utf8'),
    );
    assert.equal(embeddedManifest.releaseVersion, '0.1.0');
    assert.deepEqual(embeddedManifest.installers, {
      shell: 'install.sh',
      powershell: 'install.ps1',
    });

    const externalManifest = JSON.parse(
      readFileSync(
        path.join(outputDir, 'native', 'linux', 'x64', 'bundles', 'sdkwork-api-router-product-server-linux-x64.manifest.json'),
        'utf8',
      ),
    );
    assert.equal(externalManifest.releaseVersion, '0.1.0');
    assert.deepEqual(externalManifest.installers, {
      shell: 'install.sh',
      powershell: 'install.ps1',
    });

    const readme = readFileSync(path.join(inspectedArchiveRoot, 'README.txt'), 'utf8');
    assert.match(readme, /\.\/install\.sh --mode system/);
    assert.match(readme, /install\.ps1 -Mode system/);
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
