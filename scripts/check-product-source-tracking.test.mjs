import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'check-product-source-tracking.mjs'),
    ).href,
  );
}

test('product source tracking audit exposes governed root catalogs and untracked-source filtering helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listGovernedProductSourceRootSpecs, 'function');
  assert.equal(typeof module.findGovernedProductSourceRootSpec, 'function');
  assert.equal(typeof module.listGovernedProductSourceRootSpecsByIds, 'function');
  assert.equal(typeof module.listUntrackedGovernedProductSources, 'function');

  assert.deepEqual(
    module.listGovernedProductSourceRootSpecs(),
    [
      {
        id: 'scripts',
        relativePath: 'scripts',
      },
      {
        id: 'github-workflows',
        relativePath: '.github/workflows',
      },
      {
        id: 'bin',
        relativePath: 'bin',
      },
      {
        id: 'router-admin',
        relativePath: 'apps/sdkwork-router-admin',
      },
      {
        id: 'router-portal',
        relativePath: 'apps/sdkwork-router-portal',
      },
      {
        id: 'router-product-service',
        relativePath: 'services/router-product-service',
      },
    ],
  );
  assert.deepEqual(
    module.listGovernedProductSourceRootSpecsByIds([
      'scripts',
      'github-workflows',
      'bin',
      'router-admin',
      'router-portal',
      'router-product-service',
    ]),
    [
      {
        id: 'scripts',
        relativePath: 'scripts',
      },
      {
        id: 'github-workflows',
        relativePath: '.github/workflows',
      },
      {
        id: 'bin',
        relativePath: 'bin',
      },
      {
        id: 'router-admin',
        relativePath: 'apps/sdkwork-router-admin',
      },
      {
        id: 'router-portal',
        relativePath: 'apps/sdkwork-router-portal',
      },
      {
        id: 'router-product-service',
        relativePath: 'services/router-product-service',
      },
    ],
  );
  assert.throws(
    () => module.findGovernedProductSourceRootSpec('missing-root'),
    /missing governed product source root.*missing-root/i,
  );
});

test('product source tracking audit filters release-subtree and non-source untracked files out of the governed result', async () => {
  const module = await loadModule();

  const untracked = module.listUntrackedGovernedProductSources({
    repoRoot,
    spawnSyncImpl() {
      return {
        status: 0,
        stdout: [
          'scripts/check-product-source-tracking.mjs',
          'scripts/check-product-source-tracking.test.mjs',
          'scripts/release/ignored-release-helper.mjs',
          'scripts/dev/vite-runtime-lib.d.mts',
          '.github/workflows/product-governance.yml',
          'bin/start-dev.ps1',
          'bin/start-dev.sh',
          'bin/router-ops.mjs',
          'apps/sdkwork-router-admin/tests/admin-session-storage-standard.test.mjs',
          'apps/sdkwork-router-portal/tests/helpers/portal-paths.mjs',
          'services/router-product-service/src/main.rs',
          'services/router-product-service/Cargo.toml',
          'scripts/notes.txt',
          '',
        ].join('\n'),
        stderr: '',
      };
    },
  });

  assert.deepEqual(
    untracked,
    [
      '.github/workflows/product-governance.yml',
      'apps/sdkwork-router-admin/tests/admin-session-storage-standard.test.mjs',
      'apps/sdkwork-router-portal/tests/helpers/portal-paths.mjs',
      'bin/router-ops.mjs',
      'bin/start-dev.ps1',
      'bin/start-dev.sh',
      'scripts/check-product-source-tracking.mjs',
      'scripts/check-product-source-tracking.test.mjs',
      'scripts/dev/vite-runtime-lib.d.mts',
      'services/router-product-service/Cargo.toml',
      'services/router-product-service/src/main.rs',
    ],
  );
});

test('product source tracking audit keeps governed product source files under version control in the repository', async () => {
  const module = await loadModule();

  assert.deepEqual(
    module.listUntrackedGovernedProductSources({
      repoRoot,
    }),
    [],
    'governed product source files and contract tests must be tracked so clean checkouts can reproduce product verification behavior',
  );
});
