import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

function createWatchRequirement(path, message) {
  return {
    path,
    message,
  };
}

export const PRODUCT_VERIFICATION_WORKFLOW_WATCH_REQUIREMENTS = [
  createWatchRequirement(
    '.github/workflows/product-verification.yml',
    'product verification workflow must watch its own workflow file',
  ),
  createWatchRequirement(
    '.github/workflows/user-center-upstream-sync.yml',
    'product verification workflow must watch the user-center upstream sync workflow because it governs cross-repository identity standard propagation',
  ),
  createWatchRequirement(
    '.github/workflows/release.yml',
    'product verification workflow must watch the release workflow because release packaging contract changes are product-surface changes',
  ),
  createWatchRequirement('Cargo.toml', 'product verification workflow must watch the Rust workspace manifest'),
  createWatchRequirement('Cargo.lock', 'product verification workflow must watch the Rust dependency lockfile'),
  createWatchRequirement(
    '*.sh',
    'product verification workflow must watch root shell wrappers because they are user-facing compatibility entrypoints',
  ),
  createWatchRequirement(
    '*.ps1',
    'product verification workflow must watch root PowerShell wrappers because they are user-facing compatibility entrypoints',
  ),
  createWatchRequirement(
    'package.json',
    'product verification workflow must watch the root workspace package because it owns the published tauri:dev and server:dev product entrypoints',
  ),
  createWatchRequirement('README.md', 'product verification workflow must watch the primary repository README'),
  createWatchRequirement('README.zh-CN.md', 'product verification workflow must watch the localized repository README'),
  createWatchRequirement('apps/sdkwork-router-admin/**', 'product verification workflow must watch the admin application workspace'),
  createWatchRequirement('apps/sdkwork-router-portal/**', 'product verification workflow must watch the portal application workspace'),
  createWatchRequirement('bin/**', 'product verification workflow must watch the managed runtime tooling subtree'),
  createWatchRequirement('crates/**', 'product verification workflow must watch the Rust crates workspace'),
  createWatchRequirement('data/**', 'product verification workflow must watch governed runtime data inputs'),
  createWatchRequirement('docs/**', 'product verification workflow must watch the public documentation workspace'),
  createWatchRequirement('scripts/build-router-desktop-assets.mjs', 'product verification workflow must watch the desktop asset build entrypoint'),
  createWatchRequirement('scripts/build-router-desktop-assets.test.mjs', 'product verification workflow must watch the desktop asset build contract test'),
  createWatchRequirement('scripts/check-router-docs-safety.mjs', 'product verification workflow must watch the public docs safety scanner'),
  createWatchRequirement('scripts/check-router-docs-safety.test.mjs', 'product verification workflow must watch the public docs safety scanner contract test'),
  createWatchRequirement('scripts/check-router-frontend-budgets.mjs', 'product verification workflow must watch the frontend bundle budget audit entrypoint'),
  createWatchRequirement('scripts/check-router-frontend-budgets.test.mjs', 'product verification workflow must watch the frontend bundle budget audit contract test'),
  createWatchRequirement('scripts/strict-contract-catalog.mjs', 'product verification workflow must watch the shared strict contract catalog helper'),
  createWatchRequirement('scripts/strict-contract-catalog.test.mjs', 'product verification workflow must watch the shared strict contract catalog contract test'),
  createWatchRequirement(
    'scripts/browser-runtime-smoke.mjs',
    'product verification workflow must watch the shared browser runtime smoke helper',
  ),
  createWatchRequirement(
    'scripts/browser-runtime-smoke.test.mjs',
    'product verification workflow must watch the shared browser runtime smoke helper test',
  ),
  createWatchRequirement(
    'scripts/check-admin-browser-runtime.mjs',
    'product verification workflow must watch the admin browser runtime smoke entrypoint',
  ),
  createWatchRequirement(
    'scripts/check-admin-browser-runtime.test.mjs',
    'product verification workflow must watch the admin browser runtime smoke test',
  ),
  createWatchRequirement(
    'scripts/check-portal-browser-runtime.mjs',
    'product verification workflow must watch the portal browser runtime smoke entrypoint',
  ),
  createWatchRequirement(
    'scripts/check-portal-browser-runtime.test.mjs',
    'product verification workflow must watch the portal browser runtime smoke test',
  ),
  createWatchRequirement(
    'scripts/check-server-dev-workspace.mjs',
    'product verification workflow must watch the combined server development workspace smoke entrypoint',
  ),
  createWatchRequirement(
    'scripts/check-server-dev-workspace.test.mjs',
    'product verification workflow must watch the combined server development workspace smoke contract test',
  ),
  createWatchRequirement(
    'scripts/check-browser-storage-governance.mjs',
    'product verification workflow must watch the browser storage governance audit entrypoint',
  ),
  createWatchRequirement(
    'scripts/check-browser-storage-governance.test.mjs',
    'product verification workflow must watch the browser storage governance audit contract test',
  ),
  createWatchRequirement(
    'scripts/check-product-source-tracking.mjs',
    'product verification workflow must watch the product source tracking audit entrypoint',
  ),
  createWatchRequirement(
    'scripts/check-product-source-tracking.test.mjs',
    'product verification workflow must watch the product source tracking audit contract test',
  ),
  createWatchRequirement('scripts/check-router-product.mjs', 'product verification workflow must watch the product verification gate entrypoint'),
  createWatchRequirement('scripts/check-router-product.test.mjs', 'product verification workflow must watch the product verification gate contract test'),
  createWatchRequirement('scripts/check-rust-dependency-audit.mjs', 'product verification workflow must watch the Rust dependency audit entrypoint'),
  createWatchRequirement('scripts/check-rust-dependency-audit.policy.json', 'product verification workflow must watch the Rust dependency audit policy baseline'),
  createWatchRequirement('scripts/check-rust-dependency-audit.test.mjs', 'product verification workflow must watch the Rust dependency audit contract test'),
  createWatchRequirement('scripts/dev/pnpm-launch-lib.mjs', 'product verification workflow must watch the shared pnpm helper implementation'),
  createWatchRequirement('scripts/dev/tests/pnpm-launch-lib.test.mjs', 'product verification workflow must watch the shared pnpm helper contract test'),
  createWatchRequirement(
    'scripts/prepare-router-portal-desktop-runtime.mjs',
    'product verification workflow must watch the portal desktop runtime helper staging script',
  ),
  createWatchRequirement(
    'scripts/prepare-router-portal-desktop-runtime.test.mjs',
    'product verification workflow must watch the portal desktop runtime helper staging contract test',
  ),
  createWatchRequirement(
    'scripts/smoke-bind-retry-lib.mjs',
    'product verification workflow must watch the shared bind-retry smoke helper implementation',
  ),
  createWatchRequirement(
    'scripts/smoke-bind-retry-lib.test.mjs',
    'product verification workflow must watch the shared bind-retry smoke helper contract test',
  ),
  createWatchRequirement(
    'scripts/product-governance-node-test-catalog.mjs',
    'product verification workflow must watch the product governance node test catalog module',
  ),
  createWatchRequirement(
    'scripts/product-governance-node-test-catalog.test.mjs',
    'product verification workflow must watch the product governance node test catalog contract test',
  ),
  createWatchRequirement(
    'scripts/product-verification-workflow-contracts.mjs',
    'product verification workflow must watch the contract module',
  ),
  createWatchRequirement(
    'scripts/product-verification-workflow-step-contract-catalog.mjs',
    'product verification workflow must watch the governed workflow step contract catalog module',
  ),
  createWatchRequirement(
    'scripts/product-verification-workflow-step-contract-catalog.test.mjs',
    'product verification workflow must watch the governed workflow step contract catalog contract test',
  ),
  createWatchRequirement(
    'scripts/product-verification-workflow-watch-catalog.mjs',
    'product verification workflow must watch the governed workflow watch catalog module',
  ),
  createWatchRequirement(
    'scripts/product-verification-workflow-watch-catalog.test.mjs',
    'product verification workflow must watch the governed workflow watch catalog contract test',
  ),
  createWatchRequirement(
    'scripts/product-verification-workflow.test.mjs',
    'product verification workflow must watch the workflow contract test',
  ),
  createWatchRequirement(
    'scripts/run-product-governance-node-tests.mjs',
    'product verification workflow must watch the repository-owned product governance node test runner',
  ),
  createWatchRequirement(
    'scripts/run-product-governance-node-tests.test.mjs',
    'product verification workflow must watch the product governance node test runner contract test',
  ),
  createWatchRequirement(
    'scripts/release/**',
    'product verification workflow must watch the shared release-packaging helper subtree',
  ),
  createWatchRequirement(
    'scripts/release-flow-contract.test.mjs',
    'product verification workflow must watch the release flow contract test',
  ),
  createWatchRequirement(
    'scripts/run-router-product.mjs',
    'product verification workflow must watch the root packaged product launcher entrypoint',
  ),
  createWatchRequirement(
    'scripts/run-router-product.test.mjs',
    'product verification workflow must watch the root packaged product launcher contract test',
  ),
  createWatchRequirement(
    'scripts/run-router-product-service.mjs',
    'product verification workflow must watch the shared product service launcher entrypoint',
  ),
  createWatchRequirement(
    'scripts/run-router-product-service.test.mjs',
    'product verification workflow must watch the shared product service launcher contract test',
  ),
  createWatchRequirement(
    'scripts/run-user-center-standard.mjs',
    'product verification workflow must watch the root user-center standard runner entrypoint',
  ),
  createWatchRequirement(
    'scripts/run-user-center-standard.test.mjs',
    'product verification workflow must watch the root user-center standard runner contract test',
  ),
  createWatchRequirement(
    'scripts/user-center-upstream-sync-payload.mjs',
    'product verification workflow must watch the upstream sync payload validator entrypoint',
  ),
  createWatchRequirement(
    'scripts/user-center-upstream-sync-payload.test.mjs',
    'product verification workflow must watch the upstream sync payload validator contract test',
  ),
  createWatchRequirement(
    'scripts/user-center-upstream-sync-workflow.test.mjs',
    'product verification workflow must watch the upstream sync workflow contract test',
  ),
  createWatchRequirement(
    'scripts/run-tauri-cli.mjs',
    'product verification workflow must watch the shared desktop runtime helper',
  ),
  createWatchRequirement(
    'scripts/release/desktop-targets.mjs',
    'product verification workflow must watch the shared desktop target helper',
  ),
  createWatchRequirement('services/**', 'product verification workflow must watch the services subtree'),
  createWatchRequirement('vendor/**', 'product verification workflow must watch the vendored dependency subtree'),
];

const productVerificationWorkflowWatchCatalog = createStrictKeyedCatalog({
  entries: PRODUCT_VERIFICATION_WORKFLOW_WATCH_REQUIREMENTS,
  getKey: ({ path }) => path,
  duplicateKeyMessagePrefix: 'duplicate product verification workflow watch requirement path',
  missingKeyMessagePrefix: 'missing product verification workflow watch requirement',
});

export function listProductVerificationWorkflowWatchRequirements() {
  return productVerificationWorkflowWatchCatalog.list();
}

export function findProductVerificationWorkflowWatchRequirement(watchPath) {
  return productVerificationWorkflowWatchCatalog.find(watchPath);
}

export function listProductVerificationWorkflowWatchRequirementsByPaths(watchPaths = []) {
  return productVerificationWorkflowWatchCatalog.listByKeys(watchPaths);
}

export function listProductVerificationWorkflowWatchPaths() {
  return productVerificationWorkflowWatchCatalog.list().map(({ path }) => path);
}
