import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-workflow-step-contract-catalog.mjs'),
    ).href,
  );
}

test('release governance workflow step contract catalog publishes the exact governed workflow step assertions', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listReleaseGovernanceWorkflowStepContracts, 'function');
  assert.deepEqual(
    module.listReleaseGovernanceWorkflowStepContracts(),
    [
      {
        id: 'pull-request-and-dispatch-triggers',
        patternSource: String.raw`pull_request:[\s\S]*?workflow_dispatch:`,
        message: 'release-governance workflow must expose pull_request and workflow_dispatch triggers',
      },
      {
        id: 'force-node24-javascript-actions',
        patternSource: String.raw`workflow_dispatch:\s*[\s\S]*?env:\s*[\s\S]*?FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'[\s\S]*?jobs:`,
        message: 'release-governance workflow must force JavaScript actions onto the Node24 runtime',
      },
      {
        id: 'checkout-repository',
        patternSource: String.raw`Checkout repository[\s\S]*?uses:\s*actions\/checkout@v5`,
        message: 'release-governance workflow must checkout the repository with actions/checkout@v5',
      },
      {
        id: 'setup-node-without-auto-cache',
        patternSource: String.raw`Setup Node\.js[\s\S]*?uses:\s*actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?package-manager-cache:\s*false`,
        message: 'release-governance workflow must disable setup-node package-manager auto-cache in its non-pnpm job',
      },
      {
        id: 'run-release-governance-node-tests',
        patternSource: String.raw`Run release governance node tests[\s\S]*?run:\s*node scripts\/run-release-governance-node-tests\.mjs`,
        message: 'release-governance workflow must delegate governance node tests to the repository-owned runner',
      },
      {
        id: 'run-release-governance-checks',
        patternSource: String.raw`Run release governance checks[\s\S]*?run:\s*node scripts\/release\/run-release-governance-checks\.mjs --profile preflight --format json`,
        message: 'release-governance workflow must run the governed preflight release governance checks',
      },
    ],
  );
});

test('release governance workflow step contract catalog exposes strict id-based lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findReleaseGovernanceWorkflowStepContract, 'function');
  assert.equal(typeof module.listReleaseGovernanceWorkflowStepContractsByIds, 'function');

  const nodeTestsContract = module.findReleaseGovernanceWorkflowStepContract(
    'run-release-governance-node-tests',
  );
  assert.deepEqual(
    nodeTestsContract,
    module
      .listReleaseGovernanceWorkflowStepContracts()
      .find(({ id }) => id === 'run-release-governance-node-tests'),
  );

  nodeTestsContract.message = 'mutated locally';
  assert.notEqual(
    module.findReleaseGovernanceWorkflowStepContract('run-release-governance-node-tests').message,
    'mutated locally',
  );

  assert.deepEqual(
    module.listReleaseGovernanceWorkflowStepContractsByIds([
      'setup-node-without-auto-cache',
      'run-release-governance-checks',
    ]).map(({ id }) => id),
    [
      'setup-node-without-auto-cache',
      'run-release-governance-checks',
    ],
  );

  assert.throws(
    () => module.findReleaseGovernanceWorkflowStepContract('missing-release-governance-contract'),
    /missing release governance workflow step contract.*missing-release-governance-contract/i,
  );
});
