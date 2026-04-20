import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-governance-node-test-catalog.mjs'),
    ).href,
  );
}

test('product governance node test catalog publishes the exact governed test file set', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listProductGovernanceNodeTestFiles, 'function');
  assert.deepEqual(
    module.listProductGovernanceNodeTestFiles(),
    [
      'scripts/product-verification-workflow.test.mjs',
      'scripts/product-governance-node-test-catalog.test.mjs',
      'scripts/run-product-governance-node-tests.test.mjs',
      'scripts/check-router-product.test.mjs',
      'scripts/check-rust-dependency-audit.test.mjs',
      'scripts/browser-runtime-smoke.test.mjs',
      'scripts/check-admin-browser-runtime.test.mjs',
      'scripts/check-portal-browser-runtime.test.mjs',
      'scripts/check-server-dev-workspace.test.mjs',
      'scripts/build-router-desktop-assets.test.mjs',
      'scripts/check-router-docs-safety.test.mjs',
      'scripts/check-router-frontend-budgets.test.mjs',
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/run-router-product.test.mjs',
      'scripts/run-router-product-service.test.mjs',
      'bin/tests/root-entrypoint-wrappers.test.mjs',
      'scripts/dev/tests/pnpm-launch-lib.test.mjs',
      'scripts/prepare-router-portal-desktop-runtime.test.mjs',
      'scripts/release-flow-contract.test.mjs',
      'scripts/release/tests/desktop-target-catalog.test.mjs',
      'scripts/release/tests/native-build-root-catalog.test.mjs',
      'scripts/release/tests/native-runtime-layout-catalog.test.mjs',
      'scripts/release/tests/installed-runtime-layout-catalog.test.mjs',
      'scripts/release/tests/native-release-product-catalog.test.mjs',
      'scripts/release/tests/native-desktop-app-catalog.test.mjs',
      'scripts/release/tests/product-server-asset-catalog.test.mjs',
      'scripts/release/tests/materialize-release-catalog.test.mjs',
      'scripts/release/tests/release-workflow.test.mjs',
      'scripts/release/release-workflow-step-contract-catalog.test.mjs',
      'scripts/release/tests/release-workflow-publish-catalog.test.mjs',
      'scripts/release/tests/release-attestation-verify.test.mjs',
      'scripts/release/tests/publish-ghcr-image.test.mjs',
      'scripts/release/tests/publish-ghcr-manifest.test.mjs',
      'scripts/release/tests/docs-product-contract.test.mjs',
      'apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs',
    ],
  );
});

test('product governance node test catalog exposes strict file lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findProductGovernanceNodeTestFile, 'function');
  assert.equal(typeof module.listProductGovernanceNodeTestFilesByPaths, 'function');

  assert.equal(
    module.findProductGovernanceNodeTestFile('scripts/strict-contract-catalog.test.mjs'),
    'scripts/strict-contract-catalog.test.mjs',
  );
  assert.deepEqual(
    module.listProductGovernanceNodeTestFilesByPaths([
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/release/tests/release-workflow.test.mjs',
    ]),
    [
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/release/tests/release-workflow.test.mjs',
    ],
  );
  assert.throws(
    () => module.findProductGovernanceNodeTestFile('scripts/missing-governance-node-test.test.mjs'),
    /missing product governance node test file.*missing-governance-node-test/i,
  );
});
