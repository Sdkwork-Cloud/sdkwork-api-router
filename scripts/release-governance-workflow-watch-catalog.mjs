import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

function createWatchRequirement(path, message) {
  return {
    path,
    message,
  };
}

export const RELEASE_GOVERNANCE_WORKFLOW_WATCH_REQUIREMENTS = [
  createWatchRequirement(
    '.github/workflows/release.yml',
    'release-governance workflow must watch the release workflow',
  ),
  createWatchRequirement(
    '.github/workflows/release-governance.yml',
    'release-governance workflow must watch its own workflow file',
  ),
  createWatchRequirement(
    'scripts/release/**',
    'release-governance workflow must watch the governed release helper subtree',
  ),
  createWatchRequirement(
    'scripts/strict-contract-catalog.mjs',
    'release-governance workflow must watch the shared strict contract catalog helper',
  ),
  createWatchRequirement(
    'scripts/strict-contract-catalog.test.mjs',
    'release-governance workflow must watch the shared strict contract catalog contract test',
  ),
  createWatchRequirement(
    'scripts/smoke-bind-retry-lib.mjs',
    'release-governance workflow must watch the shared bind-retry smoke helper implementation',
  ),
  createWatchRequirement(
    'scripts/smoke-bind-retry-lib.test.mjs',
    'release-governance workflow must watch the shared bind-retry smoke helper contract test',
  ),
  createWatchRequirement(
    'scripts/release-governance-node-test-catalog.mjs',
    'release-governance workflow must watch the governed node test catalog module',
  ),
  createWatchRequirement(
    'scripts/release-governance-node-test-catalog.test.mjs',
    'release-governance workflow must watch the governed node test catalog contract test',
  ),
  createWatchRequirement(
    'scripts/release-governance-workflow-contracts.mjs',
    'release-governance workflow must watch the contract module',
  ),
  createWatchRequirement(
    'scripts/release-governance-workflow-step-contract-catalog.mjs',
    'release-governance workflow must watch the governed workflow step contract catalog module',
  ),
  createWatchRequirement(
    'scripts/release-governance-workflow-step-contract-catalog.test.mjs',
    'release-governance workflow must watch the governed workflow step contract catalog contract test',
  ),
  createWatchRequirement(
    'scripts/release-governance-workflow-watch-catalog.mjs',
    'release-governance workflow must watch the governed workflow watch catalog module',
  ),
  createWatchRequirement(
    'scripts/release-governance-workflow-watch-catalog.test.mjs',
    'release-governance workflow must watch the governed workflow watch catalog contract test',
  ),
  createWatchRequirement(
    'scripts/release-governance-workflow.test.mjs',
    'release-governance workflow must watch the workflow contract test',
  ),
  createWatchRequirement(
    'scripts/run-release-governance-node-tests.mjs',
    'release-governance workflow must watch the repository-owned runner entrypoint',
  ),
  createWatchRequirement(
    'scripts/run-release-governance-node-tests.test.mjs',
    'release-governance workflow must watch the repository-owned runner contract test',
  ),
  createWatchRequirement(
    'bin/**',
    'release-governance workflow must watch the governed runtime binary subtree',
  ),
  createWatchRequirement(
    'docs/架构/135-可观测性与SLO治理设计-2026-04-07.md',
    'release-governance workflow must watch the governed SLO architecture baseline',
  ),
  createWatchRequirement(
    'docs/架构/143-全局架构对齐与收口计划-2026-04-08.md',
    'release-governance workflow must watch the global architecture closure baseline',
  ),
  createWatchRequirement(
    'docs/release/**',
    'release-governance workflow must watch the governed release documentation subtree',
  ),
];

const releaseGovernanceWorkflowWatchCatalog = createStrictKeyedCatalog({
  entries: RELEASE_GOVERNANCE_WORKFLOW_WATCH_REQUIREMENTS,
  getKey: ({ path }) => path,
  duplicateKeyMessagePrefix: 'duplicate release governance workflow watch requirement path',
  missingKeyMessagePrefix: 'missing release governance workflow watch requirement',
});

export function listReleaseGovernanceWorkflowWatchRequirements() {
  return releaseGovernanceWorkflowWatchCatalog.list();
}

export function findReleaseGovernanceWorkflowWatchRequirement(watchPath) {
  return releaseGovernanceWorkflowWatchCatalog.find(watchPath);
}

export function listReleaseGovernanceWorkflowWatchRequirementsByPaths(watchPaths = []) {
  return releaseGovernanceWorkflowWatchCatalog.listByKeys(watchPaths);
}

export function listReleaseGovernanceWorkflowWatchPaths() {
  return releaseGovernanceWorkflowWatchCatalog.list().map(({ path }) => path);
}
