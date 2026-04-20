import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('repository exposes a cached package-group Rust verification workflow', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'rust-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/rust-verification.yml');

  const workflow = read('.github/workflows/rust-verification.yml');

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /permissions:\s*contents:\s*read/);
  assert.doesNotMatch(
    workflow,
    /^\s+(?:contents|id-token|attestations|artifact-metadata|packages):\s*write$/m,
  );
  assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(
    workflow,
    /Setup Node\.js[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?package-manager-cache:\s*false/,
  );
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /group:\s*interface-openapi/);
  assert.match(workflow, /group:\s*gateway-service/);
  assert.match(workflow, /group:\s*admin-service/);
  assert.match(workflow, /group:\s*portal-service/);
  assert.match(workflow, /group:\s*dependency-audit/);
  assert.match(workflow, /group:\s*product-runtime/);
  assert.match(workflow, /vendor\/\*\*/);
  assert.match(workflow, /scripts\/check-rust-dependency-audit\.mjs/);
  assert.match(workflow, /scripts\/check-rust-dependency-audit\.policy\.json/);
  assert.match(workflow, /scripts\/check-rust-dependency-audit\.test\.mjs/);
  assert.match(workflow, /scripts\/run-rust-governance-node-tests\.mjs/);
  assert.match(workflow, /scripts\/rust-governance-node-test-catalog\.mjs/);
  assert.match(workflow, /scripts\/rust-governance-node-test-catalog\.test\.mjs/);
  assert.match(workflow, /scripts\/run-rust-governance-node-tests\.test\.mjs/);
  assert.match(workflow, /scripts\/rust-verification-workflow-contracts\.mjs/);
  assert.match(
    workflow,
    /scripts\/run-tauri-cli\.mjs/,
    'rust verification workflow must watch the shared Windows runtime helper',
  );
  assert.match(
    workflow,
    /scripts\/workspace-target-dir\.mjs/,
    'rust verification workflow must watch the shared workspace target-dir helper',
  );
  assert.match(
    workflow,
    /scripts\/release\/desktop-targets\.mjs/,
    'rust verification workflow must watch transitive desktop target helpers loaded by the matrix runner',
  );
  assert.match(workflow, /Install cargo-audit/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  assert.match(workflow, /Run rust governance node tests/);
  assert.match(
    workflow,
    /run:\s*node scripts\/run-rust-governance-node-tests\.mjs/,
  );
  assert.match(
    workflow,
    /node scripts\/check-rust-verification-matrix\.mjs --group \$\{\{ matrix\.group \}\}/,
  );
  assert.match(workflow, /rust-verification-windows-workspace:/);
  assert.match(workflow, /runs-on:\s*windows-latest/);
  assert.match(workflow, /github\.event_name == 'workflow_dispatch'/);
  assert.match(workflow, /github\.event\.inputs\.group == 'workspace'/);
  assert.match(
    workflow,
    /rust-verification-windows-workspace:[\s\S]*?Setup Node\.js[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?package-manager-cache:\s*false/,
  );
  assert.match(workflow, /node scripts\/check-rust-verification-matrix\.mjs --group workspace/);
});

test('rust verification workflow contract helper rejects workflows that omit the explicit read-only token permissions', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'rust-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-rust-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  const workflowWithoutPermissions = read('.github/workflows/rust-verification.yml').replace(
    /\r?\npermissions:\r?\n\s+contents:\s*read\r?\n/,
    '\n',
  );
  assert.doesNotMatch(workflowWithoutPermissions, /permissions:\s*contents:\s*read/);

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'rust-verification.yml'),
    workflowWithoutPermissions,
    'utf8',
  );

  await assert.rejects(
    contracts.assertRustVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /read-only GITHUB_TOKEN baseline|permissions/i,
  );
});

test('rust verification workflow contract helper rejects workflows that inline raw node test lists instead of the repository runner', async () => {
  const contractSource = read('scripts/rust-verification-workflow-contracts.mjs');
  assert.match(contractSource, /rust-governance-node-test-catalog\.mjs/);

  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'rust-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-rust-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'rust-verification.yml'),
    read('.github/workflows/rust-verification.yml').replace(
      /run:\s*node scripts\/run-rust-governance-node-tests\.mjs/,
      'run: node --test scripts/check-rust-dependency-audit.test.mjs scripts/check-rust-verification-matrix.test.mjs scripts/rust-verification-workflow.test.mjs',
    ),
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-rust-governance-node-tests.mjs'),
    `
export function listRustGovernanceNodeTests() {
  return [
    'scripts/check-rust-dependency-audit.test.mjs',
    'scripts/check-rust-verification-matrix.test.mjs',
    'scripts/rust-verification-workflow.test.mjs',
    'scripts/run-rust-governance-node-tests.test.mjs',
  ];
}

export function createRustGovernanceNodeTestPlan() {
  return { command: 'node', args: ['--test', ...listRustGovernanceNodeTests()] };
}

export function runRustGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertRustVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /repository-owned runner|run-rust-governance-node-tests/i,
  );
});

test('rust verification workflow contract helper rejects runners that omit the governed node test isolation mode', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'rust-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-rust-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'rust-verification.yml'),
    read('.github/workflows/rust-verification.yml'),
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-rust-governance-node-tests.mjs'),
    `
export function listRustGovernanceNodeTests() {
  return [
    'scripts/check-rust-dependency-audit.test.mjs',
    'scripts/check-rust-verification-matrix.test.mjs',
    'scripts/rust-governance-node-test-catalog.test.mjs',
    'scripts/rust-verification-workflow.test.mjs',
    'scripts/run-rust-governance-node-tests.test.mjs',
  ];
}

export function createRustGovernanceNodeTestPlan() {
  return { command: 'node', args: ['--test', ...listRustGovernanceNodeTests()] };
}

export function runRustGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertRustVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /test isolation|experimental-test-isolation/i,
  );
});

test('rust verification workflow contract helper rejects workflows that do not watch the rust governance catalog contract test', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'rust-verification-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-rust-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'rust-verification.yml'),
    read('.github/workflows/rust-verification.yml').replace(
      /^.*scripts\/rust-governance-node-test-catalog\.test\.mjs.*\r?\n/gm,
      '',
    ),
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-rust-governance-node-tests.mjs'),
    `
export function listRustGovernanceNodeTests() {
  return [
    'scripts/check-rust-dependency-audit.test.mjs',
    'scripts/check-rust-verification-matrix.test.mjs',
    'scripts/rust-governance-node-test-catalog.test.mjs',
    'scripts/rust-verification-workflow.test.mjs',
    'scripts/run-rust-governance-node-tests.test.mjs',
  ];
}

export function createRustGovernanceNodeTestPlan() {
  return {
    command: 'node',
    args: ['--test', '--experimental-test-isolation=none', ...listRustGovernanceNodeTests()],
  };
}

export function runRustGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertRustVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /catalog contract test|rust-governance-node-test-catalog\.test\.mjs/i,
  );
});
