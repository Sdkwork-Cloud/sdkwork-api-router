import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

export const RELEASE_GOVERNANCE_NODE_TEST_FILES = [
  'scripts/release-governance-node-test-catalog.test.mjs',
  'scripts/release-governance-workflow-step-contract-catalog.test.mjs',
  'scripts/release-governance-workflow-watch-catalog.test.mjs',
  'scripts/release-governance-workflow.test.mjs',
  'scripts/run-release-governance-node-tests.test.mjs',
  'scripts/strict-contract-catalog.test.mjs',
  'scripts/release/tests/installed-runtime-smoke-lib.test.mjs',
  'scripts/release/tests/release-cli-format-catalog.test.mjs',
  'scripts/release/tests/release-governance-plan-catalog.test.mjs',
  'scripts/release/tests/release-governance-runner.test.mjs',
  'scripts/release/tests/materialize-third-party-governance.test.mjs',
];

const releaseGovernanceNodeTestCatalog = createStrictKeyedCatalog({
  entries: RELEASE_GOVERNANCE_NODE_TEST_FILES,
  getKey: (filePath) => filePath,
  duplicateKeyMessagePrefix: 'duplicate release governance node test file',
  missingKeyMessagePrefix: 'missing release governance node test file',
});

export function listReleaseGovernanceNodeTestFiles() {
  return releaseGovernanceNodeTestCatalog.list();
}

export function findReleaseGovernanceNodeTestFile(filePath) {
  return releaseGovernanceNodeTestCatalog.find(filePath);
}

export function listReleaseGovernanceNodeTestFilesByPaths(filePaths = []) {
  return releaseGovernanceNodeTestCatalog.listByKeys(filePaths);
}
