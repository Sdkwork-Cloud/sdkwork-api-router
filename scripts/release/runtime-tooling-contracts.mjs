import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

function toPortablePath(value) {
  return String(value).replaceAll('\\', '/');
}

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function createReleaseCatalogFixture({
  platform = 'linux',
  arch = 'x64',
} = {}) {
  const releaseOutputDir = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-runtime-tooling-'));
  mkdirSync(releaseOutputDir, { recursive: true });
  writeFileSync(
    path.join(releaseOutputDir, 'release-catalog.json'),
    `${JSON.stringify({
      version: 1,
      type: 'sdkwork-release-catalog',
      releaseTag: 'runtime-tooling-fixture',
      generatedAt: '2026-04-18T00:00:00.000Z',
      productCount: 1,
      variantCount: 1,
      products: [
        {
          productId: 'sdkwork-api-router-product-server',
          variants: [
            {
              platform,
              arch,
              outputDirectory: `native/${platform}/${arch}/bundles`,
              variantKind: 'server-archive',
              primaryFile: `sdkwork-api-router-product-server-${platform}-${arch}.tar.gz`,
              primaryFileSizeBytes: 0,
              checksumFile: `sdkwork-api-router-product-server-${platform}-${arch}.tar.gz.sha256.txt`,
              checksumAlgorithm: 'sha256',
              manifestFile: `sdkwork-api-router-product-server-${platform}-${arch}.manifest.json`,
              sha256: 'fixture',
              manifest: {
                type: 'product-server-archive',
                productId: 'sdkwork-api-router-product-server',
                platform,
                arch,
              },
            },
          ],
        },
      ],
    }, null, 2)}\n`,
    'utf8',
  );

  return releaseOutputDir;
}

export async function assertRuntimeToolingContracts({
  repoRoot,
} = {}) {
  const runtimeToolingModulePath = path.join(repoRoot, 'bin', 'lib', 'router-runtime-tooling.mjs');
  const runtimeToolingTestPath = path.join(repoRoot, 'bin', 'tests', 'router-runtime-tooling.test.mjs');

  assert.equal(existsSync(runtimeToolingModulePath), true, 'missing bin/lib/router-runtime-tooling.mjs');
  assert.equal(existsSync(runtimeToolingTestPath), true, 'missing bin/tests/router-runtime-tooling.test.mjs');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'start.sh')), true, 'missing bin/start.sh');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'stop.sh')), true, 'missing bin/stop.sh');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'start.ps1')), true, 'missing bin/start.ps1');
  assert.equal(existsSync(path.join(repoRoot, 'bin', 'stop.ps1')), true, 'missing bin/stop.ps1');

  const module = await import(pathToFileURL(runtimeToolingModulePath).href);
  assert.equal(typeof module.createReleaseBuildPlan, 'function');
  assert.equal(typeof module.createInstallPlan, 'function');
  assert.equal(typeof module.renderSystemdUnit, 'function');
  assert.equal(typeof module.renderLaunchdPlist, 'function');
  assert.equal(typeof module.renderWindowsTaskXml, 'function');
  assert.equal(typeof module.renderRuntimeEnvTemplate, 'function');

  const releaseOutputDir = createReleaseCatalogFixture({
    platform: 'linux',
    arch: 'x64',
  });
  try {
    const systemPlan = module.createInstallPlan({
      repoRoot,
      mode: 'system',
      platform: 'linux',
      releaseOutputDir,
    });
    assert.equal(systemPlan.mode, 'system');
    assert.equal(
      systemPlan.files.some((file) => toPortablePath(file.targetPath) === '/etc/sdkwork-api-router/router.yaml'),
      true,
      'expected system install plan to publish /etc/sdkwork-api-router/router.yaml',
    );
    assert.equal(
      systemPlan.files.some((file) => toPortablePath(file.targetPath) === '/etc/sdkwork-api-router/router.env'),
      true,
      'expected system install plan to publish /etc/sdkwork-api-router/router.env',
    );
  } finally {
    rmSync(releaseOutputDir, { recursive: true, force: true });
  }

  const systemEnvTemplate = module.renderRuntimeEnvTemplate({
    installRoot: '/opt/sdkwork-api-router',
    mode: 'system',
    platform: 'linux',
  });
  assert.match(systemEnvTemplate, /SDKWORK_CONFIG_FILE="\/etc\/sdkwork-api-router\/router\.yaml"/);
  assert.match(systemEnvTemplate, /postgresql:\/\/sdkwork:change-me@127\.0\.0\.1:5432\/sdkwork_api_router/);

  const runtimeToolingTests = read(repoRoot, 'bin/tests/router-runtime-tooling.test.mjs');
  assert.match(runtimeToolingTests, /function canSpawnUnixShellFromNode\(\)/);
  assert.match(
    runtimeToolingTests,
    /test\('unix runtime entrypoints default to the installed home beside the packaged scripts when binaries are colocated'/,
  );
  assert.match(
    runtimeToolingTests,
    /test\('installed unix runtime start\.sh and stop\.sh manage an installed home end-to-end'/,
  );
}

function isDirectExecution() {
  if (!process.argv[1]) {
    return false;
  }

  return pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url;
}

if (isDirectExecution()) {
  test('runtime tooling contracts stay aligned with install layout generation', async () => {
    const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
    await assertRuntimeToolingContracts({
      repoRoot,
    });
  });
}
