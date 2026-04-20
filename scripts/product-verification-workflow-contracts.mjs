import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { listProductVerificationWorkflowStepContracts } from './product-verification-workflow-step-contract-catalog.mjs';
import { listProductVerificationWorkflowWatchRequirements } from './product-verification-workflow-watch-catalog.mjs';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function escapeRegexLiteral(value) {
  return String(value).replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
}

function createLiteralPattern(value) {
  return new RegExp(escapeRegexLiteral(value));
}

export async function assertProductVerificationWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'product-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/product-verification.yml');

  const workflow = read(repoRoot, path.join('.github', 'workflows', 'product-verification.yml'));

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(
    workflow,
    /permissions:\s*contents:\s*read/,
    'product verification workflow must declare an explicit read-only GITHUB_TOKEN baseline',
  );
  assert.doesNotMatch(
    workflow,
    /^\s+(?:contents|id-token|attestations|artifact-metadata|packages):\s*write$/m,
    'product verification workflow must not request release-grade write permissions',
  );
  assert.match(
    workflow,
    /workflow_dispatch:\s*[\s\S]*?env:\s*[\s\S]*?FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'[\s\S]*?jobs:/,
    'product verification workflow must opt GitHub JavaScript actions into the Node 24 runtime to avoid Node 20 deprecation drift on hosted runners',
  );
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /pnpm\/action-setup@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  for (const requirement of listProductVerificationWorkflowWatchRequirements()) {
    assert.match(
      workflow,
      createLiteralPattern(requirement.path),
      requirement.message,
    );
  }
  assert.doesNotMatch(
    workflow,
    /console\/\*\*/,
    'product verification workflow should not treat the legacy console workspace as an official product verification trigger surface',
  );
  for (const contract of listProductVerificationWorkflowStepContracts()) {
    assert.match(
      workflow,
      new RegExp(contract.patternSource),
      contract.message,
    );
  }
  assert.doesNotMatch(
    workflow,
    /console\/tests\/sdk-transport-unsafe-integer\.test\.mjs|console\/pnpm-lock\.yaml/,
    'product verification workflow must not include legacy console-specific test or dependency inputs',
  );
  assert.match(
    workflow,
    /docs\/pnpm-lock\.yaml/,
    'product verification workflow must cache the docs lockfile because docs build is part of the governed public documentation surface',
  );

  const productGovernanceCatalog = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-governance-node-test-catalog.mjs'),
    ).href,
  );
  const productGovernanceRunner = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-product-governance-node-tests.mjs'),
    ).href,
  );

  assert.equal(typeof productGovernanceCatalog.listProductGovernanceNodeTestFiles, 'function');
  assert.equal(typeof productGovernanceRunner.listProductGovernanceNodeTests, 'function');
  assert.equal(typeof productGovernanceRunner.createProductGovernanceNodeTestPlan, 'function');
  assert.equal(typeof productGovernanceRunner.runProductGovernanceNodeTests, 'function');
  const governedNodeTests = productGovernanceRunner.listProductGovernanceNodeTests();
  assert.ok(
    governedNodeTests.includes('scripts/dev/tests/pnpm-launch-lib.test.mjs'),
    'product governance node test runner must include the shared pnpm helper contract test',
  );
  assert.ok(
    governedNodeTests.includes('scripts/check-rust-dependency-audit.test.mjs'),
    'product governance node test runner must include the Rust dependency audit contract test',
  );
  assert.ok(
    governedNodeTests.includes('scripts/run-router-product.test.mjs')
      && governedNodeTests.includes('scripts/run-router-product-service.test.mjs'),
    'product governance node test runner must include the root product entrypoint tests',
  );
  assert.ok(
    governedNodeTests.includes('bin/tests/root-entrypoint-wrappers.test.mjs'),
    'product governance node test runner must include the root entrypoint wrapper contract test',
  );
  assert.ok(
    governedNodeTests.includes('scripts/release/tests/publish-ghcr-image.test.mjs')
      && governedNodeTests.includes('scripts/release/tests/publish-ghcr-manifest.test.mjs'),
    'product governance node test runner must include the GHCR publish contract tests',
  );
  assert.deepEqual(
    governedNodeTests,
    productGovernanceCatalog.listProductGovernanceNodeTestFiles(),
    'product governance node test runner must own the exact governed test set, including GHCR publish coverage',
  );
  assert.deepEqual(
    productGovernanceRunner.createProductGovernanceNodeTestPlan({
      cwd: '.',
      env: {},
      nodeExecutable: 'node',
    }).args,
    ['--test', '--experimental-test-isolation=none', ...productGovernanceCatalog.listProductGovernanceNodeTestFiles()],
    'product governance node test runner must use the governed node test isolation mode in the repository-owned runner plan',
  );
}
