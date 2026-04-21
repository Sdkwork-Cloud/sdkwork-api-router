import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');
const releaseGovernanceWorkflowWatchCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'release-governance-workflow-watch-catalog.mjs'),
  ).href,
);
const releaseGovernanceWorkflowStepContractCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'release-governance-workflow-step-contract-catalog.mjs'),
  ).href,
);
const DEFAULT_RELEASE_GOVERNANCE_WORKFLOW_WATCH_PATHS = releaseGovernanceWorkflowWatchCatalog.listReleaseGovernanceWorkflowWatchPaths();
const DEFAULT_RELEASE_GOVERNANCE_WORKFLOW_STEP_CONTRACTS = releaseGovernanceWorkflowStepContractCatalog.listReleaseGovernanceWorkflowStepContracts();

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function escapeRegexLiteral(value) {
  return String(value).replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
}

test('repository exposes a pull-request release governance workflow that watches its contract surface', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release-governance.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release-governance.yml');

  const workflow = read('.github/workflows/release-governance.yml');

  assert.match(workflow, /permissions:\s*contents:\s*read/);
  assert.doesNotMatch(
    workflow,
    /^\s+(?:contents|id-token|attestations|artifact-metadata|packages):\s*write$/m,
  );
  for (const contract of DEFAULT_RELEASE_GOVERNANCE_WORKFLOW_STEP_CONTRACTS) {
    assert.match(workflow, new RegExp(contract.patternSource));
  }
  for (const watchedPath of DEFAULT_RELEASE_GOVERNANCE_WORKFLOW_WATCH_PATHS) {
    assert.match(workflow, new RegExp(escapeRegexLiteral(watchedPath)));
  }
});

test('release governance workflow contract helper rejects workflows that do not force JavaScript actions to Node24', async () => {
  const contractSource = read('scripts/release-governance-workflow-contracts.mjs');
  assert.match(contractSource, /release-governance-node-test-catalog\.mjs/);
  assert.match(contractSource, /release-governance-workflow-watch-catalog\.mjs/);
  assert.match(contractSource, /release-governance-workflow-step-contract-catalog\.mjs/);

  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  const workflow = read('.github/workflows/release-governance.yml');

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    workflow.replace(/env:\r?\n\s+FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'\r?\n\r?\n/, ''),
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /node24/i,
  );
});

test('release governance workflow contract helper rejects workflows that omit the explicit read-only token permissions', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  const workflow = read('.github/workflows/release-governance.yml');

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    workflow.replace(/permissions:\r?\n\s+contents:\s*read\r?\n\r?\n/, ''),
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /read-only GITHUB_TOKEN baseline|permissions/i,
  );
});

test('release governance workflow contract helper rejects workflows that do not watch the contract module', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    `
name: release-governance

on:
  pull_request:
    paths:
      - '.github/workflows/release.yml'
      - '.github/workflows/release-governance.yml'
      - 'scripts/release/**'
      - 'scripts/strict-contract-catalog.mjs'
      - 'scripts/strict-contract-catalog.test.mjs'
      - 'scripts/smoke-bind-retry-lib.mjs'
      - 'scripts/smoke-bind-retry-lib.test.mjs'
      - 'scripts/release-governance-node-test-catalog.mjs'
      - 'scripts/release-governance-node-test-catalog.test.mjs'
      - 'scripts/run-release-governance-node-tests.mjs'
      - 'scripts/run-release-governance-node-tests.test.mjs'
      - 'scripts/release-governance-workflow.test.mjs'
      - 'bin/**'
      - 'docs/release/**'
  workflow_dispatch:

permissions:
  contents: read

env:
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'

jobs:
  release-governance:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false

      - name: Run release governance node tests
        run: node scripts/run-release-governance-node-tests.mjs

      - name: Run release governance checks
        run: node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /contract module/i,
  );
});

test('release governance workflow contract helper rejects workflows that do not disable setup-node package-manager auto-cache', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  const workflow = read('.github/workflows/release-governance.yml');

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    workflow.replace(/^\s*package-manager-cache:\s*false\r?\n/m, ''),
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /package-manager(?:\s|-)?auto-cache|package-manager-cache/i,
  );
});

test('release governance workflow contract helper rejects workflows that do not watch the governed SLO architecture docs', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    `
name: release-governance

on:
  pull_request:
    paths:
      - '.github/workflows/release.yml'
      - '.github/workflows/release-governance.yml'
      - 'scripts/release/**'
      - 'scripts/strict-contract-catalog.mjs'
      - 'scripts/strict-contract-catalog.test.mjs'
      - 'scripts/smoke-bind-retry-lib.mjs'
      - 'scripts/smoke-bind-retry-lib.test.mjs'
      - 'scripts/release-governance-node-test-catalog.mjs'
      - 'scripts/release-governance-node-test-catalog.test.mjs'
      - 'scripts/release-governance-workflow-contracts.mjs'
      - 'scripts/release-governance-workflow-step-contract-catalog.mjs'
      - 'scripts/release-governance-workflow-step-contract-catalog.test.mjs'
      - 'scripts/release-governance-workflow-watch-catalog.mjs'
      - 'scripts/release-governance-workflow-watch-catalog.test.mjs'
      - 'scripts/release-governance-workflow.test.mjs'
      - 'scripts/run-release-governance-node-tests.mjs'
      - 'scripts/run-release-governance-node-tests.test.mjs'
      - 'bin/**'
      - 'docs/release/**'
  workflow_dispatch:

permissions:
  contents: read

env:
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'

jobs:
  release-governance:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false

      - name: Run release governance node tests
        run: node scripts/run-release-governance-node-tests.mjs

      - name: Run release governance checks
        run: node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /slo architecture baseline/i,
  );
});

test('release governance workflow contract helper rejects workflows that inline raw node test lists instead of the repository runner', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    `
name: release-governance

on:
  pull_request:
    paths:
      - '.github/workflows/release.yml'
      - '.github/workflows/release-governance.yml'
      - 'scripts/release/**'
      - 'scripts/strict-contract-catalog.mjs'
      - 'scripts/strict-contract-catalog.test.mjs'
      - 'scripts/release-governance-node-test-catalog.mjs'
      - 'scripts/release-governance-node-test-catalog.test.mjs'
      - 'scripts/release-governance-workflow-contracts.mjs'
      - 'scripts/release-governance-workflow-watch-catalog.mjs'
      - 'scripts/release-governance-workflow-watch-catalog.test.mjs'
      - 'scripts/release-governance-workflow.test.mjs'
      - 'scripts/run-release-governance-node-tests.mjs'
      - 'scripts/run-release-governance-node-tests.test.mjs'
      - 'bin/**'
      - 'docs/架构/135-可观测性与SLO治理设计-2026-04-07.md'
      - 'docs/架构/143-全局架构对齐与收口计划-2026-04-08.md'
      - 'docs/release/**'
  workflow_dispatch:

permissions:
  contents: read

env:
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'

jobs:
  release-governance:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v5

      - name: Setup Node.js
        uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false

      - name: Run release governance node tests
        run: node --test --experimental-test-isolation=none scripts/release-governance-workflow.test.mjs scripts/release/tests/release-governance-runner.test.mjs

      - name: Run release governance checks
        run: node scripts/release/run-release-governance-checks.mjs --profile preflight --format json
`,
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-release-governance-node-tests.mjs'),
    `
export function listReleaseGovernanceNodeTests() {
  return [
    'scripts/release-governance-workflow.test.mjs',
    'scripts/run-release-governance-node-tests.test.mjs',
    'scripts/release/run-service-release-build.test.mjs',
    'scripts/release/tests/release-governance-plan-catalog.test.mjs',
    'scripts/release/tests/release-governance-runner.test.mjs',
  ];
}

export function createReleaseGovernanceNodeTestPlan() {
  return {
    command: 'node',
    args: ['--test', '--experimental-test-isolation=none', ...listReleaseGovernanceNodeTests()],
  };
}

export function runReleaseGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /repository-owned runner|run-release-governance-node-tests/i,
  );
});

test('release governance workflow contract helper rejects workflows that do not run the managed service release runner contract test', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    read('.github/workflows/release-governance.yml'),
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-release-governance-node-tests.mjs'),
    `
export function listReleaseGovernanceNodeTests() {
  return [
    'scripts/release-governance-node-test-catalog.test.mjs',
    'scripts/release-governance-workflow-step-contract-catalog.test.mjs',
    'scripts/release-governance-workflow-watch-catalog.test.mjs',
    'scripts/release-governance-workflow.test.mjs',
    'scripts/run-release-governance-node-tests.test.mjs',
    'scripts/strict-contract-catalog.test.mjs',
    'scripts/smoke-bind-retry-lib.test.mjs',
    'scripts/release/tests/installed-runtime-smoke-lib.test.mjs',
    'scripts/release/tests/release-cli-format-catalog.test.mjs',
    'scripts/release/tests/release-governance-plan-catalog.test.mjs',
    'scripts/release/tests/release-governance-runner.test.mjs',
    'scripts/release/tests/materialize-third-party-governance.test.mjs',
  ];
}

export function createReleaseGovernanceNodeTestPlan() {
  return {
    command: 'node',
    args: ['--test', '--experimental-test-isolation=none', ...listReleaseGovernanceNodeTests()],
  };
}

export function runReleaseGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /managed service release runner|run-service-release-build|exact governed node test set/i,
  );
});

test('release governance workflow contract helper rejects runners that omit the governed node test isolation mode', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release-governance.yml'),
    read('.github/workflows/release-governance.yml'),
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-release-governance-node-tests.mjs'),
    `
export function listReleaseGovernanceNodeTests() {
  return [
    'scripts/release-governance-node-test-catalog.test.mjs',
    'scripts/release-governance-workflow-step-contract-catalog.test.mjs',
    'scripts/release-governance-workflow-watch-catalog.test.mjs',
    'scripts/release-governance-workflow.test.mjs',
    'scripts/run-release-governance-node-tests.test.mjs',
    'scripts/strict-contract-catalog.test.mjs',
    'scripts/smoke-bind-retry-lib.test.mjs',
    'scripts/release/run-service-release-build.test.mjs',
    'scripts/release/tests/installed-runtime-smoke-lib.test.mjs',
    'scripts/release/tests/release-cli-format-catalog.test.mjs',
    'scripts/release/tests/release-governance-plan-catalog.test.mjs',
    'scripts/release/tests/release-governance-runner.test.mjs',
    'scripts/release/tests/materialize-third-party-governance.test.mjs',
  ];
}

export function createReleaseGovernanceNodeTestPlan() {
  return {
    command: 'node',
    args: ['--test', ...listReleaseGovernanceNodeTests()],
  };
}

export function runReleaseGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertReleaseGovernanceWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /test isolation|experimental-test-isolation/i,
  );
});
