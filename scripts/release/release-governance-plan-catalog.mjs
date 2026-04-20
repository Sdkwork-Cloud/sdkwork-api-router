import process from 'node:process';

import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

export const RELEASE_GOVERNANCE_CHECK_DEFINITIONS = [
  {
    id: 'release-sync-audit-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-sync-audit.test.mjs'],
  },
  {
    id: 'release-workflow-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-workflow.test.mjs'],
  },
  {
    id: 'release-workflow-publish-catalog-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-workflow-publish-catalog.test.mjs'],
  },
  {
    id: 'release-governance-workflow-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release-governance-workflow.test.mjs'],
  },
  {
    id: 'release-attestation-verify-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-attestation-verify.test.mjs'],
  },
  {
    id: 'release-observability-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-observability-contracts.test.mjs'],
  },
  {
    id: 'release-slo-governance-contracts-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-slo-governance-contracts.test.mjs'],
  },
  {
    id: 'release-slo-governance-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-slo-governance.test.mjs'],
  },
  {
    id: 'release-slo-governance',
    args: ['scripts/release/slo-governance.mjs', '--format', 'json'],
  },
  {
    id: 'release-runtime-tooling-test',
    args: ['--test', '--experimental-test-isolation=none', 'bin/tests/router-runtime-tooling.test.mjs'],
  },
  {
    id: 'release-desktop-signing-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/run-desktop-release-signing.test.mjs'],
  },
  {
    id: 'release-unix-installed-runtime-smoke-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs'],
  },
  {
    id: 'release-windows-installed-runtime-smoke-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs'],
  },
  {
    id: 'release-installed-runtime-contract-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/installed-runtime-smoke-lib.test.mjs'],
  },
  {
    id: 'release-linux-docker-compose-smoke-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/run-linux-docker-compose-smoke.test.mjs'],
  },
  {
    id: 'release-linux-helm-render-smoke-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/run-linux-helm-render-smoke.test.mjs'],
  },
  {
    id: 'release-materialize-external-deps-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-external-deps.test.mjs'],
  },
  {
    id: 'release-materialize-release-catalog-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-release-catalog.test.mjs'],
  },
  {
    id: 'release-third-party-governance-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-third-party-governance.test.mjs'],
  },
  {
    id: 'release-publish-ghcr-image-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/publish-ghcr-image.test.mjs'],
  },
  {
    id: 'release-publish-ghcr-manifest-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/publish-ghcr-manifest.test.mjs'],
  },
  {
    id: 'release-window-snapshot-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-window-snapshot.test.mjs'],
  },
  {
    id: 'release-window-snapshot-materializer-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-release-window-snapshot.test.mjs'],
  },
  {
    id: 'release-sync-audit-materializer-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-release-sync-audit.test.mjs'],
  },
  {
    id: 'release-governance-bundle-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-release-governance-bundle.test.mjs'],
  },
  {
    id: 'restore-release-governance-latest-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/restore-release-governance-latest.test.mjs'],
  },
  {
    id: 'release-telemetry-export-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-release-telemetry-export.test.mjs'],
  },
  {
    id: 'release-telemetry-snapshot-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-release-telemetry-snapshot.test.mjs'],
  },
  {
    id: 'release-slo-evidence-materializer-test',
    args: ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/materialize-slo-governance-evidence.test.mjs'],
  },
  {
    id: 'release-window-snapshot',
    args: ['scripts/release/compute-release-window-snapshot.mjs', '--format', 'json', '--live'],
  },
  {
    id: 'release-sync-audit',
    args: ['scripts/release/verify-release-sync.mjs', '--format', 'json'],
  },
];

export const RELEASE_GOVERNANCE_PREFLIGHT_EXCLUDED_PLAN_IDS = [
  'release-slo-governance',
  'release-window-snapshot',
  'release-sync-audit',
];
export const RELEASE_GOVERNANCE_PROFILES = [
  'release',
  'preflight',
];

const preflightExcludedPlanIds = new Set(RELEASE_GOVERNANCE_PREFLIGHT_EXCLUDED_PLAN_IDS);

function cloneReleaseGovernanceCheckDefinition(definition) {
  return {
    id: definition.id,
    args: [...definition.args],
  };
}

const releaseGovernanceCheckDefinitionCatalog = createStrictKeyedCatalog({
  entries: RELEASE_GOVERNANCE_CHECK_DEFINITIONS,
  getKey: (definition) => definition.id,
  clone: cloneReleaseGovernanceCheckDefinition,
  duplicateKeyMessagePrefix: 'duplicate release governance check definition id',
  missingKeyMessagePrefix: 'missing release governance check definition',
});

const releaseGovernanceProfileCatalog = createStrictKeyedCatalog({
  entries: RELEASE_GOVERNANCE_PROFILES,
  getKey: (profile) => profile,
  duplicateKeyMessagePrefix: 'duplicate release governance profile',
  missingKeyMessagePrefix: 'missing release governance profile',
});

export function listReleaseGovernanceCheckDefinitions() {
  return releaseGovernanceCheckDefinitionCatalog.list();
}

export function findReleaseGovernanceCheckDefinition(definitionId) {
  return releaseGovernanceCheckDefinitionCatalog.find(definitionId);
}

export function listReleaseGovernanceCheckDefinitionsByIds(definitionIds = []) {
  return releaseGovernanceCheckDefinitionCatalog.listByKeys(definitionIds);
}

export function listReleaseGovernancePreflightExcludedPlanIds() {
  return [...RELEASE_GOVERNANCE_PREFLIGHT_EXCLUDED_PLAN_IDS];
}

export function listReleaseGovernanceProfiles() {
  return releaseGovernanceProfileCatalog.list();
}

export function findReleaseGovernanceProfile(profile) {
  return releaseGovernanceProfileCatalog.find(profile);
}

export function listReleaseGovernanceProfilesByIds(profileIds = []) {
  return releaseGovernanceProfileCatalog.listByKeys(profileIds);
}

export function assertSupportedReleaseGovernanceProfile(profile) {
  try {
    return findReleaseGovernanceProfile(profile);
  } catch (_error) {
    throw new Error(`unsupported release governance profile: ${profile}`);
  }
}

export function createReleaseGovernanceCheckPlans({
  nodeExecutable = process.execPath,
  profile = 'release',
} = {}) {
  const resolvedProfile = assertSupportedReleaseGovernanceProfile(profile);

  const definitions = listReleaseGovernanceCheckDefinitions();
  const filteredDefinitions = resolvedProfile === 'preflight'
    ? definitions.filter((definition) => !preflightExcludedPlanIds.has(definition.id))
    : definitions;

  return filteredDefinitions.map((definition) => ({
    id: definition.id,
    command: nodeExecutable,
    args: [...definition.args],
  }));
}
