import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { listReleaseGovernanceNodeTestFiles } from './release-governance-node-test-catalog.mjs';
import { listReleaseGovernanceWorkflowStepContracts } from './release-governance-workflow-step-contract-catalog.mjs';
import { listReleaseGovernanceWorkflowWatchRequirements } from './release-governance-workflow-watch-catalog.mjs';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function escapeRegexLiteral(value) {
  return String(value).replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
}

function createLiteralPattern(value) {
  return new RegExp(escapeRegexLiteral(value));
}

export async function assertReleaseGovernanceWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release-governance.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release-governance.yml');

  const workflow = read(repoRoot, '.github/workflows/release-governance.yml');

  assert.match(
    workflow,
    /permissions:\s*contents:\s*read/,
    'release governance workflow must declare an explicit read-only GITHUB_TOKEN baseline',
  );
  assert.doesNotMatch(
    workflow,
    /^\s+(?:contents|id-token|attestations|artifact-metadata|packages):\s*write$/m,
    'release governance workflow must not request release-grade write permissions',
  );
  for (const contract of listReleaseGovernanceWorkflowStepContracts()) {
    assert.match(
      workflow,
      new RegExp(contract.patternSource),
      contract.message,
    );
  }
  for (const requirement of listReleaseGovernanceWorkflowWatchRequirements()) {
    assert.match(
      workflow,
      createLiteralPattern(requirement.path),
      requirement.message,
    );
  }

  const releaseGovernanceNodeRunner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-release-governance-node-tests.mjs'),
    ).href,
  );

  assert.equal(typeof releaseGovernanceNodeRunner.listReleaseGovernanceNodeTests, 'function');
  assert.equal(typeof releaseGovernanceNodeRunner.createReleaseGovernanceNodeTestPlan, 'function');
  assert.equal(typeof releaseGovernanceNodeRunner.runReleaseGovernanceNodeTests, 'function');
  const governedNodeTests = releaseGovernanceNodeRunner.listReleaseGovernanceNodeTests();
  assert.ok(
    governedNodeTests.includes('scripts/release/run-service-release-build.test.mjs'),
    'release-governance workflow contracts must include the managed service release runner contract test',
  );
  assert.deepEqual(
    governedNodeTests,
    listReleaseGovernanceNodeTestFiles(),
    'release-governance workflow contracts must be backed by the exact governed node test set',
  );
  assert.deepEqual(
    releaseGovernanceNodeRunner.createReleaseGovernanceNodeTestPlan({
      cwd: '.',
      env: {},
      nodeExecutable: 'node',
    }).args,
    ['--test', '--experimental-test-isolation=none', ...listReleaseGovernanceNodeTestFiles()],
    'release-governance workflow contracts must use the governed node test isolation mode in the repository-owned runner plan',
  );
}
