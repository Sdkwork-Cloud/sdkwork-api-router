import assert from 'node:assert/strict';
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createBundleFixture({
  platform = 'linux',
  target = 'x86_64-unknown-linux-gnu',
  fileName = 'sdkwork-router-portal_0.1.0_amd64.deb',
} = {}) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-desktop-sign-'));
  const bundleDirName = platform === 'windows' ? 'nsis' : platform === 'macos' ? 'dmg' : 'deb';
  const bundleDir = path.join(root, target, 'release', 'bundle', bundleDirName);
  mkdirSync(bundleDir, { recursive: true });
  const installerPath = path.join(bundleDir, fileName);
  writeFileSync(installerPath, 'fixture installer', 'utf8');
  return {
    root,
    installerPath,
  };
}

test('desktop release signing script exposes a parseable CLI contract and discovers official installer files', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-signing.mjs'),
    ).href,
  );

  assert.equal(typeof module.parseArgs, 'function');
  assert.equal(typeof module.createDesktopReleaseSigningPlan, 'function');
  assert.equal(typeof module.resolveDesktopSigningBundleFiles, 'function');
  assert.equal(typeof module.createDesktopReleaseSigningEvidence, 'function');

  const fixture = createBundleFixture();
  const evidencePath = path.join(fixture.root, 'desktop-release-signing-linux-x64.json');

  try {
    const options = module.parseArgs([
      '--app',
      'portal',
      '--platform',
      'linux',
      '--arch',
      'x64',
      '--target',
      'x86_64-unknown-linux-gnu',
      '--evidence-path',
      evidencePath,
    ]);

    assert.deepEqual(options, {
      appId: 'portal',
      platform: 'linux',
      arch: 'x64',
      targetTriple: 'x86_64-unknown-linux-gnu',
      evidencePath: path.resolve(evidencePath),
    });

    const plan = module.createDesktopReleaseSigningPlan({
      repoRoot,
      appId: options.appId,
      platform: options.platform,
      arch: options.arch,
      targetTriple: options.targetTriple,
      evidencePath: options.evidencePath,
      buildRoots: [fixture.root],
      env: {},
    });

    assert.equal(plan.appId, 'portal');
    assert.equal(plan.platform, 'linux');
    assert.equal(plan.arch, 'x64');
    assert.equal(plan.targetTriple, 'x86_64-unknown-linux-gnu');
    assert.equal(plan.required, false);
    assert.equal(plan.hook.kind, 'none');
    assert.deepEqual(plan.bundleFiles, [fixture.installerPath]);

    const evidence = module.createDesktopReleaseSigningEvidence({
      plan,
      status: 'skipped',
      commandCount: 0,
    });

    assert.equal(evidence.status, 'skipped');
    assert.equal(evidence.required, false);
    assert.deepEqual(evidence.bundleFiles, [
      path.relative(repoRoot, fixture.installerPath).replaceAll('\\', '/'),
    ]);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});

test('desktop release signing plan rejects required signing when no hook is configured', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-signing.mjs'),
    ).href,
  );

  const fixture = createBundleFixture();
  const evidencePath = path.join(fixture.root, 'desktop-release-signing-linux-x64.json');

  try {
    assert.throws(
      () => module.createDesktopReleaseSigningPlan({
        repoRoot,
        appId: 'portal',
        platform: 'linux',
        arch: 'x64',
        targetTriple: 'x86_64-unknown-linux-gnu',
        evidencePath,
        buildRoots: [fixture.root],
        env: {
          SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED: 'true',
        },
      }),
      /signing.*required/i,
    );
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});

test('desktop release signing executes the configured hook and writes evidence', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-signing.mjs'),
    ).href,
  );

  const fixture = createBundleFixture();
  const hookScriptPath = path.join(fixture.root, 'hook.mjs');
  const evidencePath = path.join(fixture.root, 'desktop-release-signing-linux-x64.json');
  const markerPath = path.join(fixture.root, 'signed-marker.txt');
  writeFileSync(
    hookScriptPath,
    [
      "import { appendFileSync } from 'node:fs';",
      "appendFileSync(process.argv[2], `${process.argv[3]}\\n`, 'utf8');",
    ].join('\n'),
    'utf8',
  );

  try {
    const plan = module.createDesktopReleaseSigningPlan({
      repoRoot,
      appId: 'portal',
      platform: 'linux',
      arch: 'x64',
      targetTriple: 'x86_64-unknown-linux-gnu',
      evidencePath,
      buildRoots: [fixture.root],
      env: {
        SDKWORK_RELEASE_DESKTOP_SIGN_HOOK: `"${process.execPath}" "${hookScriptPath}" "${markerPath}" "{file}"`,
      },
    });

    const result = module.executeDesktopReleaseSigningPlan(plan);
    assert.equal(result.status, 'signed');
    assert.equal(readFileSync(markerPath, 'utf8').trim(), fixture.installerPath);
    const evidence = JSON.parse(readFileSync(evidencePath, 'utf8'));
    assert.equal(evidence.status, 'signed');
    assert.equal(evidence.commandCount, 1);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});

test('desktop release signing prefers platform-specific hooks over the generic fallback', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-signing.mjs'),
    ).href,
  );

  const hook = module.resolveDesktopSigningHook({
    platform: 'windows',
    env: {
      SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK: 'signtool.exe sign "{file}"',
      SDKWORK_RELEASE_DESKTOP_SIGN_HOOK: 'generic-sign "{file}"',
    },
  });

  assert.deepEqual(hook, {
    kind: 'command',
    command: 'signtool.exe sign "{file}"',
    envVar: 'SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK',
  });
});

test('desktop release signing records failed evidence when the configured hook exits non-zero', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-signing.mjs'),
    ).href,
  );

  const fixture = createBundleFixture();
  const evidencePath = path.join(fixture.root, 'desktop-release-signing-linux-x64.json');

  try {
    const plan = module.createDesktopReleaseSigningPlan({
      repoRoot,
      appId: 'portal',
      platform: 'linux',
      arch: 'x64',
      targetTriple: 'x86_64-unknown-linux-gnu',
      evidencePath,
      buildRoots: [fixture.root],
      env: {
        SDKWORK_RELEASE_DESKTOP_SIGN_HOOK: `"${process.execPath}" -e "process.stderr.write('boom'); process.exit(9)"`,
      },
    });

    assert.throws(
      () => module.executeDesktopReleaseSigningPlan(plan),
      /Desktop signing hook failed/i,
    );

    const evidence = JSON.parse(readFileSync(evidencePath, 'utf8'));
    assert.equal(evidence.status, 'failed');
    assert.equal(evidence.commandCount, 0);
    assert.equal(evidence.hook.envVar, 'SDKWORK_RELEASE_DESKTOP_SIGN_HOOK');
    assert.equal(evidence.failure.message.includes('boom'), true);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
});
