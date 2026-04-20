import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-watch-catalog.mjs'),
    ).href,
  );
}

test('product verification workflow watch catalog publishes the exact governed watch surface', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listProductVerificationWorkflowWatchPaths, 'function');
  assert.deepEqual(
    module.listProductVerificationWorkflowWatchPaths(),
    [
      '.github/workflows/product-verification.yml',
      '.github/workflows/release.yml',
      'Cargo.toml',
      'Cargo.lock',
      '*.sh',
      '*.ps1',
      'package.json',
      'README.md',
      'README.zh-CN.md',
      'apps/sdkwork-router-admin/**',
      'apps/sdkwork-router-portal/**',
      'bin/**',
      'crates/**',
      'data/**',
      'docs/**',
      'scripts/build-router-desktop-assets.mjs',
      'scripts/build-router-desktop-assets.test.mjs',
      'scripts/check-router-docs-safety.mjs',
      'scripts/check-router-docs-safety.test.mjs',
      'scripts/check-router-frontend-budgets.mjs',
      'scripts/check-router-frontend-budgets.test.mjs',
      'scripts/strict-contract-catalog.mjs',
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/browser-runtime-smoke.mjs',
      'scripts/browser-runtime-smoke.test.mjs',
      'scripts/check-admin-browser-runtime.mjs',
      'scripts/check-admin-browser-runtime.test.mjs',
      'scripts/check-portal-browser-runtime.mjs',
      'scripts/check-portal-browser-runtime.test.mjs',
      'scripts/check-server-dev-workspace.mjs',
      'scripts/check-server-dev-workspace.test.mjs',
      'scripts/check-router-product.mjs',
      'scripts/check-router-product.test.mjs',
      'scripts/check-rust-dependency-audit.mjs',
      'scripts/check-rust-dependency-audit.policy.json',
      'scripts/check-rust-dependency-audit.test.mjs',
      'scripts/dev/pnpm-launch-lib.mjs',
      'scripts/dev/tests/pnpm-launch-lib.test.mjs',
      'scripts/prepare-router-portal-desktop-runtime.mjs',
      'scripts/prepare-router-portal-desktop-runtime.test.mjs',
      'scripts/product-governance-node-test-catalog.mjs',
      'scripts/product-governance-node-test-catalog.test.mjs',
      'scripts/product-verification-workflow-contracts.mjs',
      'scripts/product-verification-workflow-step-contract-catalog.mjs',
      'scripts/product-verification-workflow-step-contract-catalog.test.mjs',
      'scripts/product-verification-workflow-watch-catalog.mjs',
      'scripts/product-verification-workflow-watch-catalog.test.mjs',
      'scripts/product-verification-workflow.test.mjs',
      'scripts/run-product-governance-node-tests.mjs',
      'scripts/run-product-governance-node-tests.test.mjs',
      'scripts/release/**',
      'scripts/release-flow-contract.test.mjs',
      'scripts/run-router-product.mjs',
      'scripts/run-router-product.test.mjs',
      'scripts/run-router-product-service.mjs',
      'scripts/run-router-product-service.test.mjs',
      'scripts/run-tauri-cli.mjs',
      'scripts/release/desktop-targets.mjs',
      'services/**',
      'vendor/**',
    ],
  );
});

test('product verification workflow watch catalog exposes strict path lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findProductVerificationWorkflowWatchRequirement, 'function');
  assert.equal(typeof module.listProductVerificationWorkflowWatchRequirementsByPaths, 'function');

  const strictHelperRequirement = module.findProductVerificationWorkflowWatchRequirement(
    'scripts/strict-contract-catalog.mjs',
  );
  assert.deepEqual(
    strictHelperRequirement,
    module
      .listProductVerificationWorkflowWatchRequirements()
      .find(({ path }) => path === 'scripts/strict-contract-catalog.mjs'),
  );

  strictHelperRequirement.message = 'mutated locally';
  assert.notEqual(
    module.findProductVerificationWorkflowWatchRequirement('scripts/strict-contract-catalog.mjs').message,
    'mutated locally',
  );

  assert.deepEqual(
    module.listProductVerificationWorkflowWatchRequirementsByPaths([
      'scripts/strict-contract-catalog.mjs',
      'scripts/strict-contract-catalog.test.mjs',
    ]).map(({ path }) => path),
    [
      'scripts/strict-contract-catalog.mjs',
      'scripts/strict-contract-catalog.test.mjs',
    ],
  );
});
