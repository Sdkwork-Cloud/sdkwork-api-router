import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

export const PRODUCT_GOVERNANCE_NODE_TEST_FILES = [
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
];

const productGovernanceNodeTestCatalog = createStrictKeyedCatalog({
  entries: PRODUCT_GOVERNANCE_NODE_TEST_FILES,
  getKey: (filePath) => filePath,
  duplicateKeyMessagePrefix: 'duplicate product governance node test file',
  missingKeyMessagePrefix: 'missing product governance node test file',
});

export function listProductGovernanceNodeTestFiles() {
  return productGovernanceNodeTestCatalog.list();
}

export function findProductGovernanceNodeTestFile(filePath) {
  return productGovernanceNodeTestCatalog.find(filePath);
}

export function listProductGovernanceNodeTestFilesByPaths(filePaths = []) {
  return productGovernanceNodeTestCatalog.listByKeys(filePaths);
}
