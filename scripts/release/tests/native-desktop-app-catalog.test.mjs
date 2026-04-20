import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    ).href,
  );
}

test('native desktop packaging helpers expose a strict governed desktop app catalog', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listNativeDesktopAppSpecs, 'function');
  assert.equal(typeof module.findNativeDesktopAppSpec, 'function');
  assert.equal(typeof module.listNativeDesktopAppSpecsByIds, 'function');
  assert.equal(typeof module.listNativeDesktopAppIds, 'function');

  assert.deepEqual(
    module.listNativeDesktopAppSpecs(),
    [
      {
        id: 'admin',
        appDir: path.join(repoRoot, 'apps', 'sdkwork-router-admin'),
        targetDirName: 'sdkwork-router-admin-tauri',
        releaseEnabled: false,
      },
      {
        id: 'portal',
        appDir: path.join(repoRoot, 'apps', 'sdkwork-router-portal'),
        targetDirName: 'sdkwork-router-portal-tauri',
        releaseEnabled: true,
      },
    ],
  );

  const portalSpec = module.findNativeDesktopAppSpec('portal');
  portalSpec.targetDirName = 'mutated-locally';
  assert.deepEqual(
    module.findNativeDesktopAppSpec('portal'),
    {
      id: 'portal',
      appDir: path.join(repoRoot, 'apps', 'sdkwork-router-portal'),
      targetDirName: 'sdkwork-router-portal-tauri',
      releaseEnabled: true,
    },
  );

  assert.deepEqual(
    module.listNativeDesktopAppSpecsByIds([
      'portal',
      'admin',
    ]).map(({ id, releaseEnabled }) => ({ id, releaseEnabled })),
    [
      {
        id: 'portal',
        releaseEnabled: true,
      },
      {
        id: 'admin',
        releaseEnabled: false,
      },
    ],
  );

  assert.deepEqual(
    module.listNativeDesktopAppIds(),
    ['portal'],
  );

  assert.throws(
    () => module.findNativeDesktopAppSpec('console'),
    /missing native desktop app spec.*console/i,
  );
});

test('native desktop packaging internals do not maintain separate app-dir, target-dir, and release-id tables', () => {
  const source = readFileSync(
    path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    'utf8',
  );

  assert.match(
    source,
    /createStrictKeyedCatalog/,
    'native desktop app packaging should use the shared strict catalog helper',
  );
  assert.doesNotMatch(
    source,
    /const DESKTOP_APP_DIRS = \{/,
    'desktop app directories must not live in a standalone ad-hoc object',
  );
  assert.doesNotMatch(
    source,
    /const DESKTOP_APP_TARGET_DIR_NAMES = \{/,
    'desktop app target dir names must not live in a standalone ad-hoc object',
  );
  assert.doesNotMatch(
    source,
    /const NATIVE_RELEASE_DESKTOP_APP_IDS = \[/,
    'official desktop release ids must not live in a standalone ad-hoc array',
  );
});
