import { createStrictContractCatalog } from './strict-contract-catalog.mjs';

export const PRODUCT_VERIFICATION_WORKFLOW_STEP_CONTRACTS = [
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
];

const productVerificationWorkflowStepContractCatalog = createStrictContractCatalog({
  contracts: PRODUCT_VERIFICATION_WORKFLOW_STEP_CONTRACTS,
  duplicateIdMessagePrefix: 'duplicate product verification workflow step contract id',
  missingIdMessagePrefix: 'missing product verification workflow step contract',
});

export function listProductVerificationWorkflowStepContracts() {
  return productVerificationWorkflowStepContractCatalog.list();
}

export function findProductVerificationWorkflowStepContract(contractId) {
  return productVerificationWorkflowStepContractCatalog.find(contractId);
}

export function listProductVerificationWorkflowStepContractsByIds(contractIds = []) {
  return productVerificationWorkflowStepContractCatalog.listByIds(contractIds);
}
