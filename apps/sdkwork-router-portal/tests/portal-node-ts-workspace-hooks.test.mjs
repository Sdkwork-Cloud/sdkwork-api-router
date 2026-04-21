import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

import {
  createPortalNodeTsWorkspaceHooks,
} from './helpers/node-ts-workspace-hooks.mjs';
import {
  resolvePortalAppRoot,
  resolvePortalAppbaseRoot,
} from './helpers/portal-paths.mjs';

const appRoot = resolvePortalAppRoot(import.meta.url);
const appbaseRoot = resolvePortalAppbaseRoot(import.meta.url);

function createResolveEcho() {
  return function nextResolve(specifier) {
    return { url: specifier };
  };
}

function createLocalFallbackResolve() {
  return function nextResolve(specifier) {
    if (specifier === './userCenter') {
      throw new Error('unresolved');
    }

    return { url: specifier };
  };
}

test('router portal node ts workspace hooks resolve workspace package aliases to source entrypoints', async () => {
  const hooks = createPortalNodeTsWorkspaceHooks({
    appRoot,
    appbaseRoot,
  });
  const parentURL = pathToFileURL(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-portal-api',
      'src',
      'index.ts',
    ),
  ).href;

  const resolved = await hooks.resolve(
    'sdkwork-router-portal-types',
    { parentURL },
    createResolveEcho(),
  );

  assert.equal(
    resolved.url,
    pathToFileURL(
      path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-types',
        'src',
        'index.ts',
      ),
    ).href,
  );
});

test('router portal node ts workspace hooks resolve extensionless local ts imports', async () => {
  const hooks = createPortalNodeTsWorkspaceHooks({
    appRoot,
    appbaseRoot,
  });
  const parentURL = pathToFileURL(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-types',
      'src',
      'validation.ts',
    ),
  ).href;

  const resolved = await hooks.resolve(
    './userCenter',
    { parentURL },
    createLocalFallbackResolve(),
  );

  assert.equal(
    resolved.url,
    pathToFileURL(
      path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-types',
        'src',
        'userCenter.ts',
      ),
    ).href,
  );
});

test('router portal node ts workspace hooks transform ts modules into executable esm without jiti', async () => {
  const hooks = createPortalNodeTsWorkspaceHooks({
    appRoot,
    appbaseRoot,
  });
  const portalApiUrl = pathToFileURL(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-portal-api',
      'src',
      'index.ts',
    ),
  ).href;

  const loaded = await hooks.load(
    portalApiUrl,
    {},
    async () => {
      throw new Error('nextLoad should not run for ts modules');
    },
  );

  assert.equal(loaded.format, 'module');
  assert.equal(loaded.shortCircuit, true);
  assert.match(loaded.source, /class PortalApiError extends Error/);
  assert.match(loaded.source, /constructor\(message, status\)/);
  assert.doesNotMatch(loaded.source, /readonly status:/);
});
