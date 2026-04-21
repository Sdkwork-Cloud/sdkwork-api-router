import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { listReleaseWorkflowStepContracts } from './release-workflow-step-contract-catalog.mjs';

function read(repoRoot, relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function extractTopLevelJobBlock(workflow, jobName) {
  const blockPattern = new RegExp(
    String.raw`^  ${jobName}:\r?\n[\s\S]*?(?=^  [a-z0-9][a-z0-9-]*:|(?![\s\S]))`,
    'im',
  );
  const match = workflow.match(blockPattern);
  assert.ok(match, `missing ${jobName} job definition`);
  return match[0];
}

function selectReleaseWorkflowContractTarget(workflow, contract) {
  if (contract.target === 'job') {
    return extractTopLevelJobBlock(workflow, contract.jobName);
  }

  return workflow;
}

export async function assertReleaseWorkflowContracts({
  repoRoot,
} = {}) {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read(repoRoot, path.join('.github', 'workflows', 'release.yml'));
  for (const contract of listReleaseWorkflowStepContracts()) {
    const targetText = selectReleaseWorkflowContractTarget(workflow, contract);
    if (contract.assertion === 'doesNotMatch') {
      assert.doesNotMatch(
        targetText,
        new RegExp(contract.patternSource),
        contract.message,
      );
      continue;
    }

    assert.match(
      targetText,
      new RegExp(contract.patternSource),
      contract.message,
    );
  }

  const helper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );
  const releaseWindowHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-window-snapshot.mjs'),
    ).href,
  );
  const releaseSyncHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-sync-audit.mjs'),
    ).href,
  );
  const telemetryExportHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-export.mjs'),
    ).href,
  );
  const telemetrySnapshotHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-telemetry-snapshot.mjs'),
    ).href,
  );
  const sloHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-slo-governance-evidence.mjs'),
    ).href,
  );
  const thirdPartyGovernanceHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-third-party-governance.mjs'),
    ).href,
  );
  const governanceBundleHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-governance-bundle.mjs'),
    ).href,
  );
  const releaseCatalogHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-release-catalog.mjs'),
    ).href,
  );
  const unixRuntimeSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-unix-installed-runtime-smoke.mjs'),
    ).href,
  );
  const windowsRuntimeSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-windows-installed-runtime-smoke.mjs'),
    ).href,
  );
  const linuxDockerComposeSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-docker-compose-smoke.mjs'),
    ).href,
  );
  const linuxHelmRenderSmokeHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-linux-helm-render-smoke.mjs'),
    ).href,
  );
  const publishGhcrImageHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-image.mjs'),
    ).href,
  );
  const publishGhcrManifestHelper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-manifest.mjs'),
    ).href,
  );

  assert.equal(typeof helper.listExternalReleaseDependencySpecs, 'function');
  assert.equal(typeof helper.buildExternalReleaseClonePlan, 'function');
  assert.equal(typeof helper.auditExternalReleaseDependencyCoverage, 'function');
  assert.equal(typeof helper.selectExternalReleaseDependencySpecsForMaterialization, 'function');
  assert.equal(typeof releaseWindowHelper.resolveReleaseWindowSnapshotProducerInput, 'function');
  assert.equal(typeof releaseWindowHelper.materializeReleaseWindowSnapshot, 'function');
  assert.equal(typeof releaseSyncHelper.resolveReleaseSyncAuditProducerInput, 'function');
  assert.equal(typeof releaseSyncHelper.materializeReleaseSyncAudit, 'function');
  assert.equal(typeof telemetryExportHelper.resolveReleaseTelemetryExportProducerInput, 'function');
  assert.equal(typeof telemetryExportHelper.materializeReleaseTelemetryExport, 'function');
  assert.equal(typeof telemetrySnapshotHelper.resolveReleaseTelemetryExportInput, 'function');
  assert.equal(typeof telemetrySnapshotHelper.resolveReleaseTelemetrySnapshotInput, 'function');
  assert.equal(typeof telemetrySnapshotHelper.deriveReleaseTelemetrySnapshotFromExport, 'function');
  assert.equal(typeof telemetrySnapshotHelper.validateReleaseTelemetrySnapshotShape, 'function');
  assert.equal(typeof telemetrySnapshotHelper.materializeReleaseTelemetrySnapshot, 'function');
  assert.equal(typeof sloHelper.resolveSloGovernanceEvidenceInput, 'function');
  assert.equal(typeof sloHelper.validateSloGovernanceEvidenceShape, 'function');
  assert.equal(typeof sloHelper.materializeSloGovernanceEvidence, 'function');
  assert.equal(typeof thirdPartyGovernanceHelper.materializeThirdPartyGovernance, 'function');
  assert.equal(typeof thirdPartyGovernanceHelper.validateThirdPartySbomArtifact, 'function');
  assert.equal(typeof thirdPartyGovernanceHelper.validateThirdPartyNoticesArtifact, 'function');
  assert.equal(typeof governanceBundleHelper.listReleaseGovernanceBundleArtifactSpecs, 'function');
  assert.equal(typeof governanceBundleHelper.createReleaseGovernanceBundleManifest, 'function');
  assert.equal(typeof governanceBundleHelper.materializeReleaseGovernanceBundle, 'function');
  assert.equal(typeof releaseCatalogHelper.collectReleaseCatalogEntries, 'function');
  assert.equal(typeof releaseCatalogHelper.createReleaseCatalog, 'function');
  assert.equal(typeof releaseCatalogHelper.materializeReleaseCatalog, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.parseArgs, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.createUnixInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.createUnixInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof unixRuntimeSmokeHelper.createUnixInstalledRuntimeSmokeEvidence, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.parseArgs, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.createWindowsInstalledRuntimeSmokeOptions, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.createWindowsInstalledRuntimeSmokePlan, 'function');
  assert.equal(typeof windowsRuntimeSmokeHelper.createWindowsInstalledRuntimeSmokeEvidence, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.parseArgs, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.createLinuxDockerComposeSmokeOptions, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.createLinuxDockerComposeSmokePlan, 'function');
  assert.equal(typeof linuxDockerComposeSmokeHelper.createLinuxDockerComposeSmokeEvidence, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.parseArgs, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.createLinuxHelmRenderSmokeOptions, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.createLinuxHelmRenderSmokePlan, 'function');
  assert.equal(typeof linuxHelmRenderSmokeHelper.createLinuxHelmRenderSmokeEvidence, 'function');
  assert.equal(typeof publishGhcrImageHelper.publishGhcrImage, 'function');
  assert.equal(typeof publishGhcrImageHelper.createImagePublishMetadata, 'function');
  assert.equal(
    typeof publishGhcrImageHelper.createGhcrImagePublishPlan,
    'function',
    'release workflow publish-ghcr-image helper must export createGhcrImagePublishPlan',
  );
  assert.equal(typeof publishGhcrManifestHelper.publishGhcrManifest, 'function');
  assert.equal(typeof publishGhcrManifestHelper.createGhcrManifestPublishMetadata, 'function');
  assert.equal(typeof publishGhcrManifestHelper.createGhcrManifestPublishPlan, 'function');

  const specs = helper.listExternalReleaseDependencySpecs();
  assert.equal(specs.length, 4);
  assert.deepEqual(
    specs.map((spec) => spec.id),
    ['sdkwork-core', 'sdkwork-ui', 'sdkwork-appbase', 'sdkwork-im-sdk'],
  );

  const coverage = helper.auditExternalReleaseDependencyCoverage();
  assert.equal(coverage.covered, true);
  assert.deepEqual(coverage.uncoveredReferences, []);
  assert.deepEqual(coverage.externalDependencyIds, ['sdkwork-ui']);

  const referencedSpecs = helper.selectExternalReleaseDependencySpecsForMaterialization({
    specs,
    coverage,
    env: {
      SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: 'referenced',
    },
  });
  assert.deepEqual(
    referencedSpecs.map((spec) => spec.id),
    ['sdkwork-ui'],
  );
}
