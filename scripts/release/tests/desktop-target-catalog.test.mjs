import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'desktop-targets.mjs'),
    ).href,
  );
}

test('desktop target helper exposes strict platform, arch, target-triple, and installer-rule catalogs', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listDesktopReleasePlatformSpecs, 'function');
  assert.equal(typeof module.findDesktopReleasePlatformSpec, 'function');
  assert.equal(typeof module.listDesktopReleasePlatformSpecsByIds, 'function');
  assert.equal(typeof module.listDesktopReleaseArchSpecs, 'function');
  assert.equal(typeof module.findDesktopReleaseArchSpec, 'function');
  assert.equal(typeof module.listDesktopReleaseArchSpecsByIds, 'function');
  assert.equal(typeof module.listDesktopReleaseTargetTripleSpecs, 'function');
  assert.equal(typeof module.findDesktopReleaseTargetTripleSpec, 'function');
  assert.equal(typeof module.listDesktopReleaseTargetTripleSpecsByIds, 'function');
  assert.equal(typeof module.resolveDesktopReleaseBundleKinds, 'function');
  assert.equal(typeof module.resolveDesktopOfficialInstallerRule, 'function');

  assert.deepEqual(
    module.listDesktopReleasePlatformSpecs(),
    [
      {
        platform: 'windows',
        aliases: ['win32', 'windows'],
        bundles: ['nsis'],
        officialInstaller: {
          artifactKind: 'nsis',
          expectedBundleDirectory: 'nsis',
          expectedFileSuffix: '.exe',
        },
      },
      {
        platform: 'linux',
        aliases: ['linux'],
        bundles: ['deb'],
        officialInstaller: {
          artifactKind: 'deb',
          expectedBundleDirectory: 'deb',
          expectedFileSuffix: '.deb',
        },
      },
      {
        platform: 'macos',
        aliases: ['darwin', 'macos'],
        bundles: ['dmg'],
        officialInstaller: {
          artifactKind: 'dmg',
          expectedBundleDirectory: 'dmg',
          expectedFileSuffix: '.dmg',
        },
      },
    ],
  );

  const windowsSpec = module.findDesktopReleasePlatformSpec('windows');
  windowsSpec.aliases.push('mutated-locally');
  windowsSpec.bundles.push('zip');
  windowsSpec.officialInstaller.expectedFileSuffix = '.msi';
  assert.deepEqual(
    module.findDesktopReleasePlatformSpec('windows'),
    {
      platform: 'windows',
      aliases: ['win32', 'windows'],
      bundles: ['nsis'],
      officialInstaller: {
        artifactKind: 'nsis',
        expectedBundleDirectory: 'nsis',
        expectedFileSuffix: '.exe',
      },
    },
  );

  assert.deepEqual(
    module.listDesktopReleasePlatformSpecsByIds([
      'linux',
      'macos',
    ]).map(({ platform }) => platform),
    ['linux', 'macos'],
  );

  assert.deepEqual(
    module.listDesktopReleaseArchSpecs(),
    [
      {
        arch: 'x64',
        aliases: ['x64', 'x86_64', 'amd64'],
      },
      {
        arch: 'arm64',
        aliases: ['arm64', 'aarch64'],
      },
    ],
  );
  assert.deepEqual(
    module.findDesktopReleaseArchSpec('arm64'),
    {
      arch: 'arm64',
      aliases: ['arm64', 'aarch64'],
    },
  );
  assert.deepEqual(
    module.listDesktopReleaseArchSpecsByIds([
      'x64',
      'arm64',
    ]).map(({ arch }) => arch),
    ['x64', 'arm64'],
  );

  assert.deepEqual(
    module.findDesktopReleaseTargetTripleSpec('aarch64-pc-windows-msvc'),
    {
      targetTriple: 'aarch64-pc-windows-msvc',
      platform: 'windows',
      arch: 'arm64',
    },
  );
  assert.deepEqual(
    module.listDesktopReleaseTargetTripleSpecsByIds([
      'x86_64-unknown-linux-gnu',
      'aarch64-apple-darwin',
    ]).map(({ targetTriple }) => targetTriple),
    [
      'x86_64-unknown-linux-gnu',
      'aarch64-apple-darwin',
    ],
  );

  assert.equal(module.normalizeDesktopPlatform('darwin'), 'macos');
  assert.equal(module.normalizeDesktopArch('aarch64'), 'arm64');
  assert.equal(
    module.buildDesktopTargetTriple({
      platform: 'linux',
      arch: 'arm64',
    }),
    'aarch64-unknown-linux-gnu',
  );
  assert.deepEqual(
    module.parseDesktopTargetTriple('x86_64-apple-darwin'),
    {
      platform: 'macos',
      arch: 'x64',
      targetTriple: 'x86_64-apple-darwin',
    },
  );
  assert.deepEqual(
    module.resolveDesktopReleaseBundleKinds({
      platform: 'win32',
    }),
    ['nsis'],
  );
  assert.deepEqual(
    module.resolveDesktopOfficialInstallerRule({
      platform: 'darwin',
    }),
    {
      artifactKind: 'dmg',
      expectedBundleDirectory: 'dmg',
      expectedFileSuffix: '.dmg',
    },
  );

  assert.throws(
    () => module.findDesktopReleasePlatformSpec('android'),
    /missing desktop release platform spec.*android/i,
  );
  assert.throws(
    () => module.findDesktopReleaseArchSpec('ppc64'),
    /missing desktop release arch spec.*ppc64/i,
  );
  assert.throws(
    () => module.findDesktopReleaseTargetTripleSpec('unsupported-target-triple'),
    /missing desktop release target triple spec.*unsupported-target-triple/i,
  );
});

test('desktop packaging consumers derive bundle and installer rules from the shared desktop target helper', async () => {
  const desktopBuildRunner = readFileSync(
    path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-build.mjs'),
    'utf8',
  );
  const nativePackager = readFileSync(
    path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    'utf8',
  );

  assert.match(
    desktopBuildRunner,
    /resolveDesktopReleaseBundleKinds/,
    'desktop release build runner must import the governed bundle-kind helper',
  );
  assert.doesNotMatch(
    desktopBuildRunner,
    /return \['nsis'\]|return \['deb'\]|return \['dmg'\]/,
    'desktop release build runner must not hardcode per-platform bundle kinds outside the shared helper',
  );

  assert.match(
    nativePackager,
    /resolveDesktopOfficialInstallerRule/,
    'native release packager must import the governed official-installer rule helper',
  );
  assert.doesNotMatch(
    nativePackager,
    /const DESKTOP_RELEASE_ARTIFACT_RULES =/,
    'native release packager must not define a second local installer-rule table',
  );
});
