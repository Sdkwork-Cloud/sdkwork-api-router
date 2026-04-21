import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';

import {
  resolvePortalAppRoot,
  resolvePortalAppbaseRoot,
  resolvePortalPackagesRoot,
  resolvePortalWorkspaceRoot,
} from './helpers/portal-paths.mjs';

test('router portal test path helpers resolve stable roots without depending on cwd', () => {
  const originalCwd = process.cwd();
  const expectedAppRoot = path.resolve(import.meta.dirname, '..');
  const expectedPackagesRoot = path.join(expectedAppRoot, 'packages');
  const expectedWorkspaceRoot = path.resolve(expectedAppRoot, '..', '..');
  const expectedAppbaseRoot = path.resolve(expectedWorkspaceRoot, '..', 'sdkwork-appbase');

  try {
    process.chdir(path.join(expectedAppRoot, 'packages'));

    assert.equal(resolvePortalAppRoot(import.meta.url), expectedAppRoot);
    assert.equal(resolvePortalPackagesRoot(import.meta.url), expectedPackagesRoot);
    assert.equal(resolvePortalWorkspaceRoot(import.meta.url), expectedWorkspaceRoot);
    assert.equal(resolvePortalAppbaseRoot(import.meta.url), expectedAppbaseRoot);
    assert.equal(existsSync(path.join(resolvePortalAppRoot(import.meta.url), 'package.json')), true);
  } finally {
    process.chdir(originalCwd);
  }
});

test('router portal appbase path helper honors SDKWORK_APPBASE_ROOT overrides for external CI checkouts', () => {
  const expectedAppbaseRoot = 'D:/workspace/external/sdkwork-appbase';

  assert.equal(
    resolvePortalAppbaseRoot(import.meta.url, {
      SDKWORK_APPBASE_ROOT: expectedAppbaseRoot,
    }),
    path.resolve(expectedAppbaseRoot),
  );
});
