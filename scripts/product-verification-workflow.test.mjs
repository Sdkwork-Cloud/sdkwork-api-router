import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');
const productGovernanceCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'product-governance-node-test-catalog.mjs'),
  ).href,
);
const productVerificationWorkflowWatchCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'product-verification-workflow-watch-catalog.mjs'),
  ).href,
);
const productVerificationWorkflowStepContractCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'product-verification-workflow-step-contract-catalog.mjs'),
  ).href,
);
const DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS = productGovernanceCatalog.listProductGovernanceNodeTestFiles();
const DEFAULT_PRODUCT_VERIFICATION_WORKFLOW_WATCH_PATHS = productVerificationWorkflowWatchCatalog.listProductVerificationWorkflowWatchPaths();
const DEFAULT_PRODUCT_VERIFICATION_WORKFLOW_STEP_CONTRACTS = productVerificationWorkflowStepContractCatalog.listProductVerificationWorkflowStepContracts();

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function escapeRegexLiteral(value) {
  return String(value).replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
}

function extractNamedStepBlock(containerText, stepName) {
  const stepPattern = new RegExp(
    String.raw`^\s+- name: ${stepName}\r?\n[\s\S]*?(?=^\s+- name:|(?![\s\S]))`,
    'im',
  );
  const match = containerText.match(stepPattern);
  assert.ok(match, `missing ${stepName} step`);
  return match[0];
}

function withNode24JavaScriptActionsEnv(workflowText) {
  if (/FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/.test(workflowText)) {
    return workflowText;
  }

  return workflowText.replace(
    /\r?\njobs:\r?\n/,
    `\n\nenv:\n  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'\n\njobs:\n`,
  );
}

function createFixtureRoot() {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-product-verification-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts'), { recursive: true });
  return fixtureRoot;
}

function createProductGovernanceCatalogFixtureSource({
  testFiles = DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS,
} = {}) {
  return `
const PRODUCT_GOVERNANCE_NODE_TEST_FILES = ${JSON.stringify(testFiles, null, 2)};

export function listProductGovernanceNodeTestFiles() {
  return [...PRODUCT_GOVERNANCE_NODE_TEST_FILES];
}
`;
}

function createProductGovernanceRunnerFixtureSource({
  testFiles = DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS,
} = {}) {
  return `
import { listProductGovernanceNodeTestFiles } from './product-governance-node-test-catalog.mjs';

export function listProductGovernanceNodeTests() {
  return listProductGovernanceNodeTestFiles();
}

export function createProductGovernanceNodeTestPlan({
  cwd = '.',
  env = {},
  nodeExecutable = 'node',
} = {}) {
  return {
    command: nodeExecutable,
    args: ['--test', '--experimental-test-isolation=none', ...listProductGovernanceNodeTests()],
    cwd,
    env,
    shell: false,
    windowsHide: false,
  };
}

export function runProductGovernanceNodeTests() {
  return { status: 0 };
}
`;
}

function writeProductVerificationFixture({
  workflowText = read('.github/workflows/product-verification.yml'),
  testFiles = DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS,
} = {}) {
  const fixtureRoot = createFixtureRoot();
  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'product-verification.yml'),
    workflowText,
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'product-governance-node-test-catalog.mjs'),
    createProductGovernanceCatalogFixtureSource({ testFiles }),
    'utf8',
  );
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-product-governance-node-tests.mjs'),
    createProductGovernanceRunnerFixtureSource({ testFiles }),
    'utf8',
  );
  return fixtureRoot;
}

async function loadContracts() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-contracts.mjs'),
    ).href,
  );
}

test('repository exposes a pull-request product verification workflow with governed installs and strict mode', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'product-verification.yml');

  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/product-verification.yml');

  const workflow = read('.github/workflows/product-verification.yml');
  const contractSource = read('scripts/product-verification-workflow-contracts.mjs');

  assert.match(workflow, /pull_request:/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /permissions:\s*contents:\s*read/);
  assert.doesNotMatch(
    workflow,
    /^\s+(?:contents|id-token|attestations|artifact-metadata|packages):\s*write$/m,
  );
  assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /pnpm\/action-setup@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /dtolnay\/rust-toolchain@stable/);
  assert.match(workflow, /Swatinem\/rust-cache@v2/);
  assert.match(workflow, /taiki-e\/install-action@cargo-audit/);
  assert.match(workflow, /docs\/pnpm-lock\.yaml/);
  for (const watchedPath of DEFAULT_PRODUCT_VERIFICATION_WORKFLOW_WATCH_PATHS) {
    assert.match(workflow, new RegExp(escapeRegexLiteral(watchedPath)));
  }
  assert.doesNotMatch(workflow, /console\/\*\*/);
  for (const contract of DEFAULT_PRODUCT_VERIFICATION_WORKFLOW_STEP_CONTRACTS) {
    assert.match(workflow, new RegExp(contract.patternSource));
  }
  assert.doesNotMatch(workflow, /console\/tests\/sdk-transport-unsafe-integer\.test\.mjs/);
  assert.doesNotMatch(workflow, /console\/pnpm-lock\.yaml/);
  assert.match(contractSource, /product-verification-workflow-watch-catalog\.mjs/);
  assert.match(contractSource, /product-verification-workflow-step-contract-catalog\.mjs/);
});

test('product verification workflow defers pnpm version selection to the root packageManager field', () => {
  const rootPackage = JSON.parse(read('package.json'));
  const pnpmSetupStep = extractNamedStepBlock(
    read('.github/workflows/product-verification.yml'),
    'Setup pnpm',
  );

  assert.equal(rootPackage.packageManager, 'pnpm@10.30.2');
  assert.doesNotMatch(
    pnpmSetupStep,
    /^\s+version:/m,
  );
});

test('product verification workflow step helper extracts a final step block through EOF when the body contains literal Z characters', () => {
  const stepBlock = extractNamedStepBlock(
    `    - name: Final governed step
      run: echo "Z-product-marker"
`,
    'Final governed step',
  );

  assert.match(stepBlock, /Z-product-marker/);
});

test('product verification workflow contract helper rejects workflows that inline raw node test lists instead of the repository runner', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml').replace(
      /run:\s*node scripts\/run-product-governance-node-tests\.mjs/,
      `run: node --test ${DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS.join(' ')}`,
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /repository-owned runner|run-product-governance-node-tests/i,
  );
});

test('product verification workflow contract helper rejects workflows that omit the explicit read-only token permissions', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml').replace(
      /permissions:\r?\n\s+contents:\s*read\r?\n\r?\n/,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /read-only GITHUB_TOKEN baseline|permissions/i,
  );
});

test('product verification workflow contract helper rejects workflows that omit the GHCR publish contract tests', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    testFiles: DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS.filter(
      (testFile) => ![
        'scripts/release/tests/publish-ghcr-image.test.mjs',
        'scripts/release/tests/publish-ghcr-manifest.test.mjs',
      ].includes(testFile),
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /ghcr|publish coverage|governed test set/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not opt JavaScript actions into Node 24', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml').replace(
      /^env:\r?\n\s+FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'\r?\n\r?\n/m,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /node 24|javascript actions/i,
  );
});

test('product verification workflow contract helper rejects workflows without strict frontend install mode', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml').replace(
      /^\s*env:\r?\n\s*SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'\r?\n\s*run:\s*node scripts\/check-router-product\.mjs/m,
      '        run: node scripts/check-router-product.mjs',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /strict frontend install mode/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not watch the contract module', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml')
      .replace(/^.*scripts\/product-verification-workflow-contracts\.mjs.*\r?\n/gm, ''),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /contract module/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not watch product desktop/runtime helper inputs', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml')
      .replace(/^.*scripts\/run-tauri-cli\.mjs.*\r?\n/gm, '')
      .replace(/^.*scripts\/prepare-router-portal-desktop-runtime\.mjs.*\r?\n/gm, '')
      .replace(/^.*scripts\/prepare-router-portal-desktop-runtime\.test\.mjs.*\r?\n/gm, '')
      .replace(/^.*scripts\/release\/desktop-targets\.mjs.*\r?\n/gm, ''),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /desktop runtime helper/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not run the shared pnpm helper tests', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    testFiles: DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS.filter(
      (testFile) => testFile !== 'scripts/dev/tests/pnpm-launch-lib.test.mjs',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /shared pnpm helper/i,
  );
});

test('product verification workflow contract helper rejects runners that omit the governed node test isolation mode', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture();
  writeFileSync(
    path.join(fixtureRoot, 'scripts', 'run-product-governance-node-tests.mjs'),
    `
import { listProductGovernanceNodeTestFiles } from './product-governance-node-test-catalog.mjs';

export function listProductGovernanceNodeTests() {
  return listProductGovernanceNodeTestFiles();
}

export function createProductGovernanceNodeTestPlan({
  cwd = '.',
  env = {},
  nodeExecutable = 'node',
} = {}) {
  return {
    command: nodeExecutable,
    args: ['--test', ...listProductGovernanceNodeTests()],
    cwd,
    env,
    shell: false,
    windowsHide: false,
  };
}

export function runProductGovernanceNodeTests() {
  return { status: 0 };
}
`,
    'utf8',
  );

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /test isolation|experimental-test-isolation/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not run the Rust dependency audit contract test', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    testFiles: DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS.filter(
      (testFile) => testFile !== 'scripts/check-rust-dependency-audit.test.mjs',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /rust dependency audit/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not run the root product entrypoint tests', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    testFiles: DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS.filter(
      (testFile) => ![
        'scripts/run-router-product.test.mjs',
        'scripts/run-router-product-service.test.mjs',
      ].includes(testFile),
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /root product entrypoint|run-router-product/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not run the root entrypoint wrapper contract test', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    testFiles: DEFAULT_PRODUCT_GOVERNANCE_NODE_TESTS.filter(
      (testFile) => testFile !== 'bin/tests/root-entrypoint-wrappers.test.mjs',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /root wrapper|root entrypoint wrapper|root-entrypoint-wrappers/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not watch the root product entrypoint inputs', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml')
      .replace(/^.*package\.json.*\r?\n/gm, '')
      .replace(/^.*scripts\/run-router-product\.mjs.*\r?\n/gm, '')
      .replace(/^.*scripts\/run-router-product\.test\.mjs.*\r?\n/gm, '')
      .replace(/^.*scripts\/run-router-product-service\.mjs.*\r?\n/gm, '')
      .replace(/^.*scripts\/run-router-product-service\.test\.mjs.*\r?\n/gm, ''),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /root product entrypoint|root workspace package|run-router-product/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not watch the root wrapper inputs', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml')
      .replace(/^.*'\*\.sh'.*\r?\n/gm, '')
      .replace(/^.*'\*\.ps1'.*\r?\n/gm, ''),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /root .*wrapper|\*\.sh|\*\.ps1/i,
  );
});

test('product verification workflow contract helper rejects workflows that do not build the docs site', async () => {
  const contracts = await loadContracts();

  const fixtureRoot = writeProductVerificationFixture({
    workflowText: read('.github/workflows/product-verification.yml').replace(
      /^\s*- name: Build docs site\r?\n[\s\S]*?(?=^\s*- name: Run product governance node tests)/m,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertProductVerificationWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /docs site/i,
  );
});
