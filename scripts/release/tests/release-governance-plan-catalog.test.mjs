import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

const EXPECTED_RELEASE_GOVERNANCE_CHECK_DEFINITIONS = [
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

const EXPECTED_RELEASE_GOVERNANCE_PREFLIGHT_EXCLUDED_PLAN_IDS = [
  'release-slo-governance',
  'release-window-snapshot',
  'release-sync-audit',
];

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-governance-plan-catalog.mjs'),
    ).href,
  );
}

test('release governance plan catalog publishes the exact governed definitions and preflight exclusions', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listReleaseGovernanceCheckDefinitions, 'function');
  assert.equal(typeof module.listReleaseGovernancePreflightExcludedPlanIds, 'function');
  assert.equal(typeof module.createReleaseGovernanceCheckPlans, 'function');

  assert.deepEqual(
    module.listReleaseGovernanceCheckDefinitions(),
    EXPECTED_RELEASE_GOVERNANCE_CHECK_DEFINITIONS,
  );
  assert.deepEqual(
    module.listReleaseGovernancePreflightExcludedPlanIds(),
    EXPECTED_RELEASE_GOVERNANCE_PREFLIGHT_EXCLUDED_PLAN_IDS,
  );
});

test('release governance plan catalog binds the governed definitions to a node executable and filters preflight lanes', async () => {
  const module = await loadModule();

  const releasePlans = module.createReleaseGovernanceCheckPlans({
    nodeExecutable: 'node-custom',
  });
  assert.deepEqual(
    releasePlans,
    EXPECTED_RELEASE_GOVERNANCE_CHECK_DEFINITIONS.map((definition) => ({
      id: definition.id,
      command: 'node-custom',
      args: definition.args,
    })),
  );

  const preflightPlans = module.createReleaseGovernanceCheckPlans({
    nodeExecutable: 'node-custom',
    profile: 'preflight',
  });
  assert.deepEqual(
    preflightPlans,
    EXPECTED_RELEASE_GOVERNANCE_CHECK_DEFINITIONS
      .filter(
        (definition) => !EXPECTED_RELEASE_GOVERNANCE_PREFLIGHT_EXCLUDED_PLAN_IDS.includes(definition.id),
      )
      .map((definition) => ({
        id: definition.id,
        command: 'node-custom',
        args: definition.args,
      })),
  );

  assert.throws(
    () => module.createReleaseGovernanceCheckPlans({
      nodeExecutable: 'node-custom',
      profile: 'invalid-profile',
    }),
    /unsupported release governance profile/i,
  );
});

test('release governance plan catalog exposes strict definition lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findReleaseGovernanceCheckDefinition, 'function');
  assert.equal(typeof module.listReleaseGovernanceCheckDefinitionsByIds, 'function');
  assert.equal(typeof module.listReleaseGovernanceProfiles, 'function');
  assert.equal(typeof module.findReleaseGovernanceProfile, 'function');
  assert.equal(typeof module.listReleaseGovernanceProfilesByIds, 'function');
  assert.equal(typeof module.assertSupportedReleaseGovernanceProfile, 'function');

  const workflowDefinition = module.findReleaseGovernanceCheckDefinition('release-workflow-test');
  assert.deepEqual(
    workflowDefinition,
    EXPECTED_RELEASE_GOVERNANCE_CHECK_DEFINITIONS.find(({ id }) => id === 'release-workflow-test'),
  );

  workflowDefinition.args.push('--mutated-locally');
  assert.deepEqual(
    module.findReleaseGovernanceCheckDefinition('release-workflow-test').args,
    ['--test', '--experimental-test-isolation=none', 'scripts/release/tests/release-workflow.test.mjs'],
  );

  assert.deepEqual(
    module.listReleaseGovernanceCheckDefinitionsByIds([
      'release-workflow-test',
      'release-publish-ghcr-manifest-test',
    ]).map(({ id }) => id),
    [
      'release-workflow-test',
      'release-publish-ghcr-manifest-test',
    ],
  );

  assert.deepEqual(
    module.listReleaseGovernanceProfiles(),
    ['release', 'preflight'],
  );
  assert.equal(
    module.findReleaseGovernanceProfile('release'),
    'release',
  );
  assert.deepEqual(
    module.listReleaseGovernanceProfilesByIds([
      'preflight',
      'release',
    ]),
    ['preflight', 'release'],
  );
  assert.equal(
    module.assertSupportedReleaseGovernanceProfile('preflight'),
    'preflight',
  );

  assert.throws(
    () => module.findReleaseGovernanceCheckDefinition('missing-release-governance-definition'),
    /missing release governance check definition.*missing-release-governance-definition/i,
  );
  assert.throws(
    () => module.findReleaseGovernanceProfile('invalid-profile'),
    /missing release governance profile.*invalid-profile/i,
  );
  assert.throws(
    () => module.assertSupportedReleaseGovernanceProfile('invalid-profile'),
    /unsupported release governance profile/i,
  );
});
