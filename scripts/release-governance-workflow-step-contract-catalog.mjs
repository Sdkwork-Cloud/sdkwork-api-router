import { createStrictContractCatalog } from './strict-contract-catalog.mjs';

export const RELEASE_GOVERNANCE_WORKFLOW_STEP_CONTRACTS = [
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
];

const releaseGovernanceWorkflowStepContractCatalog = createStrictContractCatalog({
  contracts: RELEASE_GOVERNANCE_WORKFLOW_STEP_CONTRACTS,
  duplicateIdMessagePrefix: 'duplicate release governance workflow step contract id',
  missingIdMessagePrefix: 'missing release governance workflow step contract',
});

export function listReleaseGovernanceWorkflowStepContracts() {
  return releaseGovernanceWorkflowStepContractCatalog.list();
}

export function findReleaseGovernanceWorkflowStepContract(contractId) {
  return releaseGovernanceWorkflowStepContractCatalog.find(contractId);
}

export function listReleaseGovernanceWorkflowStepContractsByIds(contractIds = []) {
  return releaseGovernanceWorkflowStepContractCatalog.listByIds(contractIds);
}
