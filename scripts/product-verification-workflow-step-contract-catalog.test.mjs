import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'product-verification-workflow-step-contract-catalog.mjs'),
    ).href,
  );
}

test('product verification workflow step contract catalog publishes the exact governed workflow step assertions', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listProductVerificationWorkflowStepContracts, 'function');
  assert.deepEqual(
    module.listProductVerificationWorkflowStepContracts(),
    [
      {
        id: 'run-product-governance-node-tests',
        patternSource: String.raw`Run product governance node tests[\s\S]*?run:\s*node scripts\/run-product-governance-node-tests\.mjs`,
        message: 'product verification workflow must delegate governance node tests to the repository-owned runner before the main product gate',
      },
      {
        id: 'materialize-referenced-release-deps-before-installs',
        patternSource: String.raw`Materialize external release dependencies[\s\S]*?env:[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile`,
        message: 'product verification workflow must materialize only referenced external release dependencies before frozen frontend installs so workspace-linked packages resolve on GitHub runners without cloning unrelated governance-only repositories',
      },
      {
        id: 'frozen-admin-and-portal-installs',
        patternSource: String.raw`Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile`,
        message: 'product verification workflow must use explicit frozen installs for the official admin and portal workspaces',
      },
      {
        id: 'strict-product-verification-gate',
        patternSource: String.raw`Run product verification gate[\s\S]*?env:[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?run:\s*node scripts\/check-router-product\.mjs`,
        message: 'strict frontend install mode must be exported before the product verification gate runs',
      },
      {
        id: 'frozen-docs-install',
        patternSource: String.raw`Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile`,
        message: 'product verification workflow must use an explicit frozen install for the docs workspace before building the public docs site',
      },
      {
        id: 'build-docs-site-before-node-contracts',
        patternSource: String.raw`Build docs site[\s\S]*?pnpm --dir docs build`,
        message: 'product verification workflow must build the public docs site before the node contract suite runs',
      },
    ],
  );
});

test('product verification workflow step contract catalog exposes strict id-based lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findProductVerificationWorkflowStepContract, 'function');
  assert.equal(typeof module.listProductVerificationWorkflowStepContractsByIds, 'function');

  const docsBuildContract = module.findProductVerificationWorkflowStepContract(
    'build-docs-site-before-node-contracts',
  );
  assert.deepEqual(
    docsBuildContract,
    module
      .listProductVerificationWorkflowStepContracts()
      .find(({ id }) => id === 'build-docs-site-before-node-contracts'),
  );

  docsBuildContract.message = 'mutated locally';
  assert.notEqual(
    module.findProductVerificationWorkflowStepContract('build-docs-site-before-node-contracts').message,
    'mutated locally',
  );

  assert.deepEqual(
    module.listProductVerificationWorkflowStepContractsByIds([
      'run-product-governance-node-tests',
      'build-docs-site-before-node-contracts',
    ]).map(({ id }) => id),
    [
      'run-product-governance-node-tests',
      'build-docs-site-before-node-contracts',
    ],
  );

  assert.throws(
    () => module.findProductVerificationWorkflowStepContract('missing-product-workflow-contract'),
    /missing product verification workflow step contract.*missing-product-workflow-contract/i,
  );
});
