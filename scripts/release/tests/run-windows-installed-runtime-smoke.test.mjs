import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createReleaseCatalogFixture({
  platform = 'windows',
  arch = 'x64',
} = {}) {
  const releaseOutputDir = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-smoke-'));
  mkdirSync(releaseOutputDir, { recursive: true });
  writeFileSync(
    path.join(releaseOutputDir, 'release-catalog.json'),
    `${JSON.stringify({
      version: 1,
      type: 'sdkwork-release-catalog',
      releaseTag: 'release-smoke-fixture',
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

test('windows installed runtime smoke script exposes a parseable CLI contract for release workflows', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createWindowsInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof module.createWindowsInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof module.createWindowsInstalledRuntimeSmokeEvidence, 'function');
  assert.equal(typeof module.resolveInstalledBootstrapDataRoot, 'function');

  const options = module.parseArgs([
    '--platform',
    'windows',
    '--arch',
    'x64',
    '--target',
    'x86_64-pc-windows-msvc',
    '--release-output-dir',
    'artifacts/release-fixture',
    '--runtime-home',
    'artifacts/release-smoke/windows-x64',
    '--evidence-path',
    'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
  ]);

  const releaseOutputDir = createReleaseCatalogFixture({
    platform: 'windows',
    arch: 'x64',
  });

  try {
    assert.deepEqual(options, {
      platform: 'windows',
      arch: 'x64',
      target: 'x86_64-pc-windows-msvc',
      releaseOutputDir: path.resolve(repoRoot, 'artifacts', 'release-fixture'),
      runtimeHome: path.resolve(repoRoot, 'artifacts', 'release-smoke', 'windows-x64'),
      evidencePath: path.resolve(repoRoot, 'artifacts', 'release-governance', 'windows-installed-runtime-smoke-windows-x64.json'),
    });

    const plan = module.createWindowsInstalledRuntimeSmokePlan({
      repoRoot,
      ...options,
      releaseOutputDir,
      ports: {
        web: 29483,
        gateway: 29480,
        admin: 29481,
        portal: 29482,
      },
    });

    assert.equal(plan.runtimeHome, options.runtimeHome);
    assert.equal(plan.evidencePath, options.evidencePath);
    assert.equal(plan.installPlan.directories[0], options.runtimeHome);
    assert.equal(plan.controlHome, path.join(options.runtimeHome, 'current'));
    assert.equal(plan.routerEnvPath, path.join(options.runtimeHome, 'config', 'router.env'));
    assert.equal(plan.pidFilePath, path.join(options.runtimeHome, 'run', 'router-product-service.pid'));
    assert.equal(plan.stdoutLogPath, path.join(options.runtimeHome, 'log', 'router-product-service.stdout.log'));
    assert.equal(plan.stderrLogPath, path.join(options.runtimeHome, 'log', 'router-product-service.stderr.log'));
    assert.equal(
      plan.installPlan.files.some((file) =>
        file.type === 'bundle-directory'
        && file.bundleEntryPath === 'bin'
        && file.bundlePath === path.join(
          releaseOutputDir,
          'native',
          'windows',
          'x64',
          'bundles',
          'sdkwork-api-router-product-server-windows-x64.tar.gz',
        )),
      true,
    );
    assert.deepEqual(plan.healthUrls, [
      'http://127.0.0.1:29483/api/v1/health',
      'http://127.0.0.1:29483/api/admin/health',
      'http://127.0.0.1:29483/api/portal/health',
    ]);
    assert.deepEqual(plan.startCommand.args, [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(options.runtimeHome, 'current', 'bin', 'start.ps1'),
      '-Home',
      path.join(options.runtimeHome, 'current'),
      '-WaitSeconds',
      '120',
    ]);
    assert.equal(plan.startCommand.stdio, 'ignore');
    assert.deepEqual(plan.stopCommand.args, [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(options.runtimeHome, 'current', 'bin', 'stop.ps1'),
      '-Home',
      path.join(options.runtimeHome, 'current'),
      '-WaitSeconds',
      '120',
    ]);
    assert.match(plan.routerEnvContents, /SDKWORK_WEB_BIND="127\.0\.0\.1:29483"/);
    assert.match(plan.routerEnvContents, /SDKWORK_GATEWAY_BIND="127\.0\.0\.1:29480"/);
    assert.match(plan.routerEnvContents, /SDKWORK_ADMIN_BIND="127\.0\.0\.1:29481"/);
    assert.match(plan.routerEnvContents, /SDKWORK_PORTAL_BIND="127\.0\.0\.1:29482"/);

    const successEvidence = module.createWindowsInstalledRuntimeSmokeEvidence({
      plan,
      ok: true,
    });
    assert.equal(successEvidence.ok, true);
    assert.equal(successEvidence.platform, 'windows');
    assert.equal(successEvidence.arch, 'x64');
    assert.equal(successEvidence.target, 'x86_64-pc-windows-msvc');
    assert.deepEqual(successEvidence.healthUrls, plan.healthUrls);
    assert.equal(successEvidence.runtimeHome, path.relative(repoRoot, options.runtimeHome).replaceAll('\\', '/'));
    assert.equal(successEvidence.evidencePath, path.relative(repoRoot, options.evidencePath).replaceAll('\\', '/'));

    const failureEvidence = module.createWindowsInstalledRuntimeSmokeEvidence({
      plan,
      ok: false,
      failure: new Error('powershell smoke failed'),
    });
    assert.equal(failureEvidence.ok, false);
    assert.equal(failureEvidence.failure.message, 'powershell smoke failed');
  } finally {
    rmSync(releaseOutputDir, { recursive: true, force: true });
  }
});

test('windows installed runtime smoke options reject unsupported Unix release lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );

  assert.throws(
    () => module.createWindowsInstalledRuntimeSmokeOptions({
      repoRoot,
      platform: 'linux',
      arch: 'x64',
      target: 'x86_64-unknown-linux-gnu',
    }),
    /only supports windows release lanes/i,
  );
});

test('windows installed runtime smoke resolves packaged bootstrap data from current release manifest', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );

  const runtimeHome = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-installed-runtime-'));
  const currentRoot = path.join(runtimeHome, 'current');
  const bootstrapDataRoot = path.join(runtimeHome, 'releases', '0.1.0', 'data');

  try {
    mkdirSync(path.join(currentRoot), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'channels'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'providers'), { recursive: true });
    mkdirSync(path.join(bootstrapDataRoot, 'routing'), { recursive: true });

    writeFileSync(
      path.join(currentRoot, 'release-manifest.json'),
      `${JSON.stringify({
        bootstrapDataRoot,
      }, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(path.join(bootstrapDataRoot, 'channels', 'default.json'), '{}\n', 'utf8');
    writeFileSync(path.join(bootstrapDataRoot, 'providers', 'default.json'), '{}\n', 'utf8');
    writeFileSync(path.join(bootstrapDataRoot, 'routing', 'default.json'), '{}\n', 'utf8');

    assert.equal(
      module.resolveInstalledBootstrapDataRoot(runtimeHome),
      bootstrapDataRoot,
    );
  } finally {
    rmSync(runtimeHome, { recursive: true, force: true });
  }
});
