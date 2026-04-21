import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..');
const modulePath = path.join(repoRoot, 'scripts', 'release', 'run-service-release-build.mjs');
const workspaceTargetDirPath = path.join(repoRoot, 'scripts', 'workspace-target-dir.mjs');

const serviceReleaseBuild = await import(pathToFileURL(modulePath).href);
const workspaceTargetDir = await import(pathToFileURL(workspaceTargetDirPath).href);

test('service release build runner exposes a managed official release build plan', () => {
  assert.equal(typeof serviceReleaseBuild.parseCliArgs, 'function');
  assert.equal(typeof serviceReleaseBuild.createServiceReleaseBuildPlan, 'function');
  assert.equal(typeof serviceReleaseBuild.buildServiceReleaseBuildFailureAnnotation, 'function');

  const windowsEnv = {
    USERPROFILE: 'C:/Users/admin',
    TEMP: 'C:/Temp',
  };
  const expectedTargetDir = workspaceTargetDir.resolveWorkspaceTargetDir({
    workspaceRoot: repoRoot,
    env: windowsEnv,
    platform: 'win32',
  });
  const expectedTempDir = workspaceTargetDir.resolveWorkspaceTempDir({
    workspaceRoot: repoRoot,
    env: windowsEnv,
    platform: 'win32',
  });

  const windowsPlan = serviceReleaseBuild.createServiceReleaseBuildPlan({
    repoRoot,
    targetTriple: 'x86_64-pc-windows-msvc',
    env: windowsEnv,
    platform: 'win32',
  });

  assert.match(String(windowsPlan.args.join(' ')), /build --release --target x86_64-pc-windows-msvc/);
  assert.match(String(windowsPlan.args.join(' ')), /-p admin-api-service/);
  assert.match(String(windowsPlan.args.join(' ')), /-p gateway-service/);
  assert.match(String(windowsPlan.args.join(' ')), /-p portal-api-service/);
  assert.match(String(windowsPlan.args.join(' ')), /-p router-web-service/);
  assert.match(String(windowsPlan.args.join(' ')), /-p router-product-service/);
  assert.equal(windowsPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(windowsPlan.env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(
    String(windowsPlan.env.CARGO_TARGET_DIR ?? '').replaceAll('\\', '/'),
    expectedTargetDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(windowsPlan.env.TEMP ?? '').replaceAll('\\', '/'),
    expectedTempDir.replaceAll('\\', '/'),
  );
  assert.equal(
    String(windowsPlan.env.TMP ?? '').replaceAll('\\', '/'),
    expectedTempDir.replaceAll('\\', '/'),
  );

  const linuxPlan = serviceReleaseBuild.createServiceReleaseBuildPlan({
    repoRoot,
    targetTriple: 'x86_64-unknown-linux-gnu',
    env: {
      CMAKE_GENERATOR: 'Visual Studio 17 2022',
      HOST_CMAKE_GENERATOR: 'Visual Studio 17 2022',
    },
    platform: 'linux',
  });

  assert.equal(Object.hasOwn(linuxPlan.env, 'CMAKE_GENERATOR'), false);
  assert.equal(Object.hasOwn(linuxPlan.env, 'HOST_CMAKE_GENERATOR'), false);
});

test('service release build runner requires an explicit target triple', () => {
  assert.throws(
    () => serviceReleaseBuild.parseCliArgs([]),
    /--target is required/,
  );
});

test('service release build runner emits GitHub annotation-safe failures', () => {
  assert.equal(
    serviceReleaseBuild.buildServiceReleaseBuildFailureAnnotation({
      targetTriple: 'x86_64-pc-windows-msvc',
      error: new Error('bundle missing 50%\nnext line'),
    }),
    '::error title=run-service-release-build::[x86_64-pc-windows-msvc] bundle missing 50%25%0Anext line',
  );
});
