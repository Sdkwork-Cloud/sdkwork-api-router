import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { listRustGovernanceNodeTestFiles } from './rust-governance-node-test-catalog.mjs';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

export async function assertRustVerificationWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'rust-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/rust-verification.yml');

  const workflow = read(repoRoot, path.join('.github', 'workflows', 'rust-verification.yml'));

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(
    workflow,
    /permissions:\s*contents:\s*read/,
    'rust verification workflow must declare an explicit read-only GITHUB_TOKEN baseline',
  );
  assert.doesNotMatch(
    workflow,
    /^\s+(?:contents|id-token|attestations|artifact-metadata|packages):\s*write$/m,
    'rust verification workflow must not request release-grade write permissions',
  );
  assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  assert.match(
    workflow,
    /scripts\/run-rust-governance-node-tests\.mjs/,
    'rust verification workflow must watch the repository-owned runner entrypoint',
  );
  assert.match(
    workflow,
    /scripts\/run-rust-governance-node-tests\.test\.mjs/,
    'rust verification workflow must watch the repository-owned runner contract test',
  );
  assert.match(
    workflow,
    /scripts\/rust-governance-node-test-catalog\.mjs/,
    'rust verification workflow must watch the governed rust node test catalog module',
  );
  assert.match(
    workflow,
    /scripts\/rust-governance-node-test-catalog\.test\.mjs/,
    'rust verification workflow must watch the rust governance catalog contract test',
  );
  assert.match(
    workflow,
    /scripts\/rust-verification-workflow-contracts\.mjs/,
    'rust verification workflow must watch the workflow contract helper module',
  );
  assert.match(
    workflow,
    /Run rust governance node tests[\s\S]*?run:\s*node scripts\/run-rust-governance-node-tests\.mjs/,
    'rust verification workflow must delegate governance node tests to the repository-owned runner',
  );

  const rustGovernanceRunner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-rust-governance-node-tests.mjs'),
    ).href,
  );

  assert.equal(typeof rustGovernanceRunner.listRustGovernanceNodeTests, 'function');
  assert.equal(typeof rustGovernanceRunner.createRustGovernanceNodeTestPlan, 'function');
  assert.equal(typeof rustGovernanceRunner.runRustGovernanceNodeTests, 'function');
  assert.deepEqual(
    rustGovernanceRunner.listRustGovernanceNodeTests(),
    listRustGovernanceNodeTestFiles(),
    'rust verification workflow contracts must be backed by the exact governed node test set',
  );
  assert.deepEqual(
    rustGovernanceRunner.createRustGovernanceNodeTestPlan({
      cwd: '.',
      env: {},
      nodeExecutable: 'node',
    }).args,
    ['--test', '--experimental-test-isolation=none', ...listRustGovernanceNodeTestFiles()],
    'rust verification workflow contracts must use the governed node test isolation mode in the repository-owned runner plan',
  );
}
