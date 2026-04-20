import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-watch-catalog.mjs'),
    ).href,
  );
}

test('release governance workflow watch catalog publishes the exact governed watch surface', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listReleaseGovernanceWorkflowWatchPaths, 'function');
  assert.deepEqual(
    module.listReleaseGovernanceWorkflowWatchPaths(),
    [
      '.github/workflows/release.yml',
      '.github/workflows/release-governance.yml',
      'scripts/release/**',
      'scripts/strict-contract-catalog.mjs',
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/release-governance-node-test-catalog.mjs',
      'scripts/release-governance-node-test-catalog.test.mjs',
      'scripts/release-governance-workflow-contracts.mjs',
      'scripts/release-governance-workflow-step-contract-catalog.mjs',
      'scripts/release-governance-workflow-step-contract-catalog.test.mjs',
      'scripts/release-governance-workflow-watch-catalog.mjs',
      'scripts/release-governance-workflow-watch-catalog.test.mjs',
      'scripts/release-governance-workflow.test.mjs',
      'scripts/run-release-governance-node-tests.mjs',
      'scripts/run-release-governance-node-tests.test.mjs',
      'bin/**',
      'docs/架构/135-可观测性与SLO治理设计-2026-04-07.md',
      'docs/架构/143-全局架构对齐与收口计划-2026-04-08.md',
      'docs/release/**',
    ],
  );
});

test('release governance workflow watch catalog exposes strict path lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findReleaseGovernanceWorkflowWatchRequirement, 'function');
  assert.equal(typeof module.listReleaseGovernanceWorkflowWatchRequirementsByPaths, 'function');

  const strictHelperRequirement = module.findReleaseGovernanceWorkflowWatchRequirement(
    'scripts/strict-contract-catalog.mjs',
  );
  assert.deepEqual(
    strictHelperRequirement,
    module
      .listReleaseGovernanceWorkflowWatchRequirements()
      .find(({ path }) => path === 'scripts/strict-contract-catalog.mjs'),
  );

  strictHelperRequirement.message = 'mutated locally';
  assert.notEqual(
    module.findReleaseGovernanceWorkflowWatchRequirement('scripts/strict-contract-catalog.mjs').message,
    'mutated locally',
  );

  assert.deepEqual(
    module.listReleaseGovernanceWorkflowWatchRequirementsByPaths([
      'scripts/strict-contract-catalog.mjs',
      'scripts/strict-contract-catalog.test.mjs',
    ]).map(({ path }) => path),
    [
      'scripts/strict-contract-catalog.mjs',
      'scripts/strict-contract-catalog.test.mjs',
    ],
  );
});
