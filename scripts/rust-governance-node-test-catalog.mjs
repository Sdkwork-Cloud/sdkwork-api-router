import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

export const RUST_GOVERNANCE_NODE_TEST_FILES = [
  'scripts/check-rust-dependency-audit.test.mjs',
  'scripts/check-rust-verification-matrix.test.mjs',
  'scripts/rust-governance-node-test-catalog.test.mjs',
  'scripts/rust-verification-workflow.test.mjs',
  'scripts/run-rust-governance-node-tests.test.mjs',
];

const rustGovernanceNodeTestCatalog = createStrictKeyedCatalog({
  entries: RUST_GOVERNANCE_NODE_TEST_FILES,
  getKey: (filePath) => filePath,
  duplicateKeyMessagePrefix: 'duplicate rust governance node test file',
  missingKeyMessagePrefix: 'missing rust governance node test file',
});

export function listRustGovernanceNodeTestFiles() {
  return rustGovernanceNodeTestCatalog.list();
}

export function findRustGovernanceNodeTestFile(filePath) {
  return rustGovernanceNodeTestCatalog.find(filePath);
}

export function listRustGovernanceNodeTestFilesByPaths(filePaths = []) {
  return rustGovernanceNodeTestCatalog.listByKeys(filePaths);
}
