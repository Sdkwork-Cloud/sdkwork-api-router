import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const releaseWorkflowStepContractCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'release', 'release-workflow-step-contract-catalog.mjs'),
  ).href,
);
const DEFAULT_RELEASE_WORKFLOW_STEP_CONTRACTS = releaseWorkflowStepContractCatalog.listReleaseWorkflowStepContracts();
const NON_PNPM_SETUP_NODE_CACHE_DISABLED_CONTRACTS =
  releaseWorkflowStepContractCatalog.listReleaseWorkflowStepContractsByIds([
    'rust-dependency-audit-cache-disabled',
    'publish-setup-node-cache-disabled',
  ]);

function read(relativePath) {
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

function extractNamedStepBlock(containerText, stepName) {
  const stepPattern = new RegExp(
    String.raw`^\s+- name: ${stepName}\r?\n[\s\S]*?(?=^\s+- name:|(?![\s\S]))`,
    'im',
  );
  const match = containerText.match(stepPattern);
  assert.ok(match, `missing ${stepName} step`);
  return match[0];
}

function selectReleaseWorkflowContractTarget(workflow, contract) {
  if (contract.target === 'job') {
    return extractTopLevelJobBlock(workflow, contract.jobName);
  }

  return workflow;
}

function writeModule(filePath, source) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, source, 'utf8');
}

function withNode24JavaScriptActionsEnv(workflowText) {
  let normalizedWorkflow = workflowText;
  if (!/permissions:\s*[\s\S]*?packages:\s*write/m.test(normalizedWorkflow)) {
    normalizedWorkflow = normalizedWorkflow.replace(
      /artifact-metadata:\s*write\r?\n/,
      `artifact-metadata: write\n  packages: write\n`,
    );
  }

  if (/FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'/.test(normalizedWorkflow)) {
    return normalizedWorkflow;
  }

  return normalizedWorkflow.replace(
    /packages:\s*write\r?\n/,
    `packages: write\n\nenv:\n  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: 'true'\n`,
  );
}

function withWorkflowDispatchInputs(workflowText) {
  if (/workflow_dispatch:\s*[\s\S]*?inputs:\s*[\s\S]*?release_tag:\s*[\s\S]*?git_ref:/m.test(workflowText)) {
    return workflowText;
  }

  return workflowText.replace(
    /workflow_dispatch:\s*(\r?\n)/,
    `workflow_dispatch:\n    inputs:\n      release_tag:\n        description: Existing release tag to publish\n        required: true\n        type: string\n      git_ref:\n        description: Git ref to build; defaults to refs/tags/<release_tag>\n        required: false\n        type: string$1`,
  );
}

function withWorkflowConcurrency(workflowText) {
  if (/concurrency:\s*[\s\S]*?cancel-in-progress:\s*false/m.test(workflowText)) {
    return workflowText;
  }

  return workflowText.replace(
    /\npermissions:\n/,
    `\nconcurrency:\n  group: release-\${{ github.workflow }}-\${{ github.event.inputs.release_tag || github.ref_name || github.run_id }}\n  cancel-in-progress: false\n\npermissions:\n`,
  );
}

function writeReleaseWorkflowContractFixture({
  workflowText,
  coverage = {
    covered: true,
    uncoveredReferences: [],
    externalDependencyIds: ['sdkwork-ui'],
  },
  includeNode24JavaScriptActionsEnv = true,
} = {}) {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-workflow-'));
  mkdirSync(path.join(fixtureRoot, '.github', 'workflows'), { recursive: true });
  mkdirSync(path.join(fixtureRoot, 'scripts', 'release'), { recursive: true });

  writeFileSync(
    path.join(fixtureRoot, '.github', 'workflows', 'release.yml'),
    includeNode24JavaScriptActionsEnv
      ? withNode24JavaScriptActionsEnv(withWorkflowConcurrency(withWorkflowDispatchInputs(workflowText)))
      : withWorkflowConcurrency(withWorkflowDispatchInputs(workflowText)),
    'utf8',
  );

  writeModule(
    path.join(fixtureRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    `
export function listExternalReleaseDependencySpecs() {
  return [
    { id: 'sdkwork-core', repository: 'Sdkwork-Cloud/sdkwork-core', envRefKey: 'SDKWORK_CORE_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-ui', repository: 'Sdkwork-Cloud/sdkwork-ui', envRefKey: 'SDKWORK_UI_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-appbase', repository: 'Sdkwork-Cloud/sdkwork-appbase', envRefKey: 'SDKWORK_APPBASE_GIT_REF', defaultRef: 'main' },
    { id: 'sdkwork-im-sdk', repository: 'Sdkwork-Cloud/sdkwork-im-sdk', envRefKey: 'SDKWORK_IM_SDK_GIT_REF', defaultRef: 'main' },
  ];
}
export function buildExternalReleaseClonePlan() {
  return { command: 'git', args: [] };
}
export function auditExternalReleaseDependencyCoverage() {
  return ${JSON.stringify(coverage, null, 2)};
}
export function selectExternalReleaseDependencySpecsForMaterialization({ specs = [] } = {}) {
  return specs.filter((spec) => spec.id === 'sdkwork-ui');
}
`,
  );

  for (const [name, body] of Object.entries({
    'materialize-release-window-snapshot.mjs': `
export function resolveReleaseWindowSnapshotProducerInput() { return { source: 'json', snapshot: {} }; }
export function materializeReleaseWindowSnapshot() { return { outputPath: 'docs/release/release-window-snapshot-latest.json' }; }
`,
    'materialize-release-sync-audit.mjs': `
export function resolveReleaseSyncAuditProducerInput() { return { source: 'json', summary: {} }; }
export function materializeReleaseSyncAudit() { return { outputPath: 'docs/release/release-sync-audit-latest.json' }; }
`,
    'materialize-release-telemetry-export.mjs': `
export function resolveReleaseTelemetryExportProducerInput() { return { source: 'json', payload: {} }; }
export function materializeReleaseTelemetryExport() { return { outputPath: 'docs/release/release-telemetry-export-latest.json' }; }
`,
    'materialize-release-telemetry-snapshot.mjs': `
export function resolveReleaseTelemetryExportInput() { return { source: 'json', payload: {} }; }
export function resolveReleaseTelemetrySnapshotInput() { return { source: 'json', payload: {} }; }
export function deriveReleaseTelemetrySnapshotFromExport() { return { generatedAt: '2026-04-18T10:00:00Z', source: { kind: 'release-telemetry-export' }, targets: {} }; }
export function validateReleaseTelemetrySnapshotShape() { return { snapshotId: 'release-telemetry-snapshot-v1', targetCount: 3 }; }
export function materializeReleaseTelemetrySnapshot() { return { outputPath: 'docs/release/release-telemetry-snapshot-latest.json' }; }
`,
    'materialize-slo-governance-evidence.mjs': `
export function resolveSloGovernanceEvidenceInput() { return { source: 'json', payload: {} }; }
export function validateSloGovernanceEvidenceShape() { return { baselineId: 'release-slo-governance-baseline', targetCount: 3 }; }
export function materializeSloGovernanceEvidence() { return { outputPath: 'docs/release/slo-governance-latest.json' }; }
`,
    'materialize-third-party-governance.mjs': `
export function materializeThirdPartyGovernance() { return { sbomOutputPath: 'docs/release/third-party-sbom-latest.spdx.json', noticesOutputPath: 'docs/release/third-party-notices-latest.json' }; }
export function validateThirdPartySbomArtifact() { return { spdxVersion: 'SPDX-2.3', packageCount: 2 }; }
export function validateThirdPartyNoticesArtifact() { return { version: 1, packageCount: 2, noticeLength: 42 }; }
`,
    'materialize-release-governance-bundle.mjs': `
export function listReleaseGovernanceBundleArtifactSpecs() { return []; }
export function createReleaseGovernanceBundleManifest() { return { version: 1, bundleEntryCount: 0, artifacts: [] }; }
export function materializeReleaseGovernanceBundle() { return { outputDir: 'artifacts/release-governance-bundle', bundleEntryCount: 0, manifestPath: 'artifacts/release-governance-bundle/release-governance-bundle-manifest.json' }; }
`,
    'run-unix-installed-runtime-smoke.mjs': `
export function parseArgs() { return {}; }
export function createUnixInstalledRuntimeSmokeOptions() { return {}; }
export function createUnixInstalledRuntimeSmokePlan() { return {}; }
export function createUnixInstalledRuntimeSmokeEvidence() { return {}; }
`,
    'run-windows-installed-runtime-smoke.mjs': `
export function parseArgs() { return {}; }
export function createWindowsInstalledRuntimeSmokeOptions() { return {}; }
export function createWindowsInstalledRuntimeSmokePlan() { return {}; }
export function createWindowsInstalledRuntimeSmokeEvidence() { return {}; }
`,
    'run-linux-docker-compose-smoke.mjs': `
export function parseArgs() { return {}; }
export function createLinuxDockerComposeSmokeOptions() { return {}; }
export function createLinuxDockerComposeSmokePlan() { return {}; }
export function createLinuxDockerComposeSmokeEvidence() { return {}; }
`,
    'run-linux-helm-render-smoke.mjs': `
export function parseArgs() { return {}; }
export function createLinuxHelmRenderSmokeOptions() { return {}; }
export function createLinuxHelmRenderSmokePlan() { return {}; }
export function createLinuxHelmRenderSmokeEvidence() { return {}; }
`,
    'materialize-release-catalog.mjs': `
export function collectReleaseCatalogEntries() { return []; }
export function createReleaseCatalog() { return { version: 1, products: [] }; }
export function materializeReleaseCatalog() { return { outputPath: 'artifacts/release/release-catalog.json', productCount: 2, variantCount: 2 }; }
`,
    'publish-ghcr-image.mjs': `
export function publishGhcrImage() { return { imageRef: 'ghcr.io/example/sdkwork-api-router:release-test-linux-x64', digest: 'sha256:test' }; }
export function createImagePublishMetadata() { return { imageRef: 'ghcr.io/example/sdkwork-api-router:release-test-linux-x64', digest: 'sha256:test' }; }
export function createGhcrImagePublishPlan() { return { imageRef: 'ghcr.io/example/sdkwork-api-router:release-test-linux-x64', bundlePath: 'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz' }; }
`,
    'publish-ghcr-manifest.mjs': `
export function publishGhcrManifest() { return { targetImageRef: 'ghcr.io/example/sdkwork-api-router:release-test', digest: 'sha256:test' }; }
export function createGhcrManifestPublishMetadata() { return { targetImageRef: 'ghcr.io/example/sdkwork-api-router:release-test', digest: 'sha256:test' }; }
export function createGhcrManifestPublishPlan() { return { targetImageRef: 'ghcr.io/example/sdkwork-api-router:release-test', sourceImageRefs: [] }; }
`,
  })) {
    writeModule(path.join(fixtureRoot, 'scripts', 'release', name), body);
  }

  return fixtureRoot;
}

test('release workflow publishes only official server and portal desktop products', () => {
  const workflowPath = path.join(repoRoot, '.github', 'workflows', 'release.yml');
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');
  const contractSource = read('scripts/release/release-workflow-contracts.mjs');

  assert.match(contractSource, /release-workflow-step-contract-catalog\.mjs/);

  for (const contract of DEFAULT_RELEASE_WORKFLOW_STEP_CONTRACTS) {
    const targetText = selectReleaseWorkflowContractTarget(workflow, contract);
    if (contract.assertion === 'doesNotMatch') {
      assert.doesNotMatch(targetText, new RegExp(contract.patternSource));
      continue;
    }

    assert.match(targetText, new RegExp(contract.patternSource));
  }
});

test('release workflow helper extracts the full native-release job block when the body contains literal Z characters', () => {
  const nativeReleaseJob = extractTopLevelJobBlock(
    read('.github/workflows/release.yml'),
    'native-release',
  );

  assert.match(nativeReleaseJob, /Materialize external release dependencies/);
  assert.match(nativeReleaseJob, /Upload Windows installed runtime smoke evidence/);
  assert.match(nativeReleaseJob, /Upload Linux Helm render smoke evidence/);
});

test('release workflow step helper extracts a final step block through EOF when the body contains literal Z characters', () => {
  const stepBlock = extractNamedStepBlock(
    `    - name: Final governed step
      run: echo "Z-runtime-marker"
`,
    'Final governed step',
  );

  assert.match(stepBlock, /Z-runtime-marker/);
});

test('release workflow contract helper accepts the repository workflow', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  await contracts.assertReleaseWorkflowContracts({
    repoRoot,
  });
});

test('release workflow contract helper rejects workflows that omit the governance release job', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  rust-dependency-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v5
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Build docs site
        run: pnpm --dir docs build
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs

  native-release:
    runs-on: ubuntu-latest
    steps:
      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}
      - name: Upload official release assets
        uses: actions/upload-artifact@v6
        with:
          name: release-assets-native-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: |
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/desktop/portal/**/*
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that still publish web release assets', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  rust-dependency-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v5
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Build docs site
        run: pnpm --dir docs build
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs

  governance-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
    runs-on: ubuntu-latest
    steps:
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Materialize release window snapshot
        run: node scripts/release/materialize-release-window-snapshot.mjs
      - name: Materialize release sync audit
        run: node scripts/release/materialize-release-sync-audit.mjs
      - name: Materialize release telemetry export
        run: node scripts/release/materialize-release-telemetry-export.mjs
      - name: Materialize release telemetry snapshot
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs
      - name: Materialize SLO governance evidence
        run: node scripts/release/materialize-slo-governance-evidence.mjs
      - name: Materialize release governance bundle
        run: node scripts/release/materialize-release-governance-bundle.mjs
      - name: Upload release governance bundle artifact
        uses: actions/upload-artifact@v6
        with:
          name: release-governance-bundle
          path: artifacts/release-governance-bundle/**/*
      - name: Run release governance gate
        run: node scripts/release/run-release-governance-checks.mjs --format json

  native-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
      - governance-release
    runs-on: ubuntu-latest
    steps:
      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}
      - name: Package web release assets
        run: node scripts/release/package-release-assets.mjs web --release-tag \${{ needs.prepare.outputs.release_tag }} --output-dir artifacts/release
      - name: Upload web release assets
        uses: actions/upload-artifact@v6
        with:
          name: release-assets-web
          path: artifacts/release/**/*
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that do not attest the governance bundle artifact', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml').replace(
      /^\s*- name: Generate governance bundle attestation\r?\n[\s\S]*?(?=^\s*- name: Run release governance gate)/m,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /governance bundle|attest/i,
  );
});

test('release workflow contract helper rejects workflows that do not publish GHCR manifest metadata evidence', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml').replace(
      /^\s*- name: Publish multi-arch container image manifest\r?\n[\s\S]*?(?=^\s*- name: Publish release assets)/m,
      `      - name: Publish multi-arch container image manifest\n        shell: bash\n        run: |\n          docker buildx imagetools create -t ghcr.io/example/sdkwork-api-router:\${{ needs.prepare.outputs.release_tag }} ghcr.io/example/sdkwork-api-router:\${{ needs.prepare.outputs.release_tag }}-linux-x64 ghcr.io/example/sdkwork-api-router:\${{ needs.prepare.outputs.release_tag }}-linux-arm64\n`,
    ),
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /ghcr.*manifest.*metadata|repository-owned publish script/i,
  );
});

test('release workflow contract helper rejects workflows whose GHCR image helper omits the publish plan export', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml'),
  });
  writeModule(
    path.join(fixtureRoot, 'scripts', 'release', 'publish-ghcr-image.mjs'),
    `
export function publishGhcrImage() { return { imageRef: 'ghcr.io/example/sdkwork-api-router:release-test-linux-x64', digest: 'sha256:test' }; }
export function createImagePublishMetadata() { return { imageRef: 'ghcr.io/example/sdkwork-api-router:release-test-linux-x64', digest: 'sha256:test' }; }
`,
  );

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /ghcr.*image.*plan|createGhcrImagePublishPlan/i,
  );
});

test('release workflow contract helper rejects workflows that still build the admin desktop product', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  rust-dependency-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v5
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Build docs site
        run: pnpm --dir docs build
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs

  governance-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
    runs-on: ubuntu-latest
    steps:
      - name: Materialize external release dependencies
        run: node scripts/release/materialize-external-deps.mjs
      - name: Materialize release window snapshot
        run: node scripts/release/materialize-release-window-snapshot.mjs
      - name: Materialize release sync audit
        run: node scripts/release/materialize-release-sync-audit.mjs
      - name: Materialize release telemetry export
        run: node scripts/release/materialize-release-telemetry-export.mjs
      - name: Materialize release telemetry snapshot
        run: node scripts/release/materialize-release-telemetry-snapshot.mjs
      - name: Materialize SLO governance evidence
        run: node scripts/release/materialize-slo-governance-evidence.mjs
      - name: Materialize release governance bundle
        run: node scripts/release/materialize-release-governance-bundle.mjs
      - name: Upload release governance bundle artifact
        uses: actions/upload-artifact@v6
        with:
          name: release-governance-bundle
          path: artifacts/release-governance-bundle/**/*
      - name: Run release governance gate
        run: node scripts/release/run-release-governance-checks.mjs --format json

  native-release:
    needs:
      - prepare
      - rust-dependency-audit
      - product-verification
      - governance-release
    runs-on: ubuntu-latest
    steps:
      - name: Build admin desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app admin --target \${{ matrix.target }}
      - name: Build portal desktop release
        run: node scripts/release/run-desktop-release-build.mjs --app portal --target \${{ matrix.target }}
      - name: Upload official release assets
        uses: actions/upload-artifact@v6
        with:
          name: release-assets-native-\${{ matrix.platform }}-\${{ matrix.arch }}
          path: |
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/desktop/portal/**/*
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
  );
});

test('release workflow contract helper rejects workflows that do not build the docs site before release product verification', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: `
name: release

on:
  push:
    tags:
      - 'release-*'
  workflow_dispatch:

permissions:
  contents: write
  id-token: write
  attestations: write
  artifact-metadata: write

jobs:
  prepare:
    runs-on: ubuntu-latest
    outputs:
      release_tag: \${{ steps.resolve.outputs.release_tag }}
      git_ref: \${{ steps.resolve.outputs.git_ref }}
    steps:
      - name: Resolve release target
        id: resolve
        shell: bash
        run: |
          if [[ "\${GITHUB_EVENT_NAME}" == "push" ]]; then
            release_tag="\${GITHUB_REF_NAME}"
            git_ref="\${GITHUB_REF}"
          else
            release_tag="\${{ github.event.inputs.release_tag }}"
            git_ref="\${{ github.event.inputs.git_ref }}"
            if [[ -z "$git_ref" ]]; then
              git_ref="refs/tags/$release_tag"
            fi
          fi

          echo "release_tag=$release_tag" >> "$GITHUB_OUTPUT"
          echo "git_ref=$git_ref" >> "$GITHUB_OUTPUT"

  rust-dependency-audit:
    needs: prepare
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          package-manager-cache: false
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - run: node scripts/check-rust-dependency-audit.mjs

  product-verification:
    needs:
      - prepare
      - rust-dependency-audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: \${{ needs.prepare.outputs.git_ref }}
      - uses: pnpm/action-setup@v5
        with:
          version: 10
      - uses: actions/setup-node@v5
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: |
            apps/sdkwork-router-admin/pnpm-lock.yaml
            apps/sdkwork-router-portal/pnpm-lock.yaml
            docs/pnpm-lock.yaml
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-audit
      - name: Materialize external release dependencies
        env:
          SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE: referenced
        run: node scripts/release/materialize-external-deps.mjs
      - name: Install product verification workspace dependencies
        run: |
          pnpm --dir apps/sdkwork-router-admin install --frozen-lockfile
          pnpm --dir apps/sdkwork-router-portal install --frozen-lockfile
          pnpm --dir docs install --frozen-lockfile
      - name: Run release product verification
        env:
          SDKWORK_STRICT_FRONTEND_INSTALLS: '1'
        run: node scripts/check-router-product.mjs
`,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /docs site/i,
  );
});

test('release workflow contract helper rejects workflows that do not seed telemetry export from the committed governed artifact path', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml').replace(
      /^\s*SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH:\s*docs\/release\/release-telemetry-export-latest\.json\r?\n/m,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /telemetry(?:-|\s)export/i,
  );
});

test('release workflow contract helper rejects workflows that do not materialize third-party governance artifacts', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml').replace(
      /^\s*- name: Materialize third-party governance\r?\n[\s\S]*?(?=^\s*- name: Upload third-party SBOM governance artifact)/m,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /third-party|sbom|notices/i,
  );
});

test('release workflow disables setup-node package-manager auto-cache in non-pnpm jobs', () => {
  const workflow = read('.github/workflows/release.yml');
  for (const contract of NON_PNPM_SETUP_NODE_CACHE_DISABLED_CONTRACTS) {
    assert.match(
      selectReleaseWorkflowContractTarget(workflow, contract),
      new RegExp(contract.patternSource),
    );
  }
});

test('release workflow defers pnpm version selection to the root packageManager field', () => {
  const rootPackage = JSON.parse(read('package.json'));
  const workflow = read('.github/workflows/release.yml');
  const productVerificationPnpmStep = extractNamedStepBlock(
    extractTopLevelJobBlock(workflow, 'product-verification'),
    'Setup pnpm',
  );
  const governanceReleasePnpmStep = extractNamedStepBlock(
    extractTopLevelJobBlock(workflow, 'governance-release'),
    'Setup pnpm',
  );
  const nativeReleasePnpmStep = extractNamedStepBlock(
    extractTopLevelJobBlock(workflow, 'native-release'),
    'Setup pnpm',
  );

  assert.equal(rootPackage.packageManager, 'pnpm@10.30.2');
  assert.doesNotMatch(
    productVerificationPnpmStep,
    /^\s+version:/m,
  );
  assert.doesNotMatch(
    governanceReleasePnpmStep,
    /^\s+version:/m,
  );
  assert.doesNotMatch(
    nativeReleasePnpmStep,
    /^\s+version:/m,
  );
});

test('release workflow contract helper rejects workflows that do not opt JavaScript actions into Node 24', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml').replace(
      /^env:\r?\n\s+FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'\r?\n\r?\n/m,
      '',
    ),
    includeNode24JavaScriptActionsEnv: false,
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /node 24|javascript actions/i,
  );
});

test('release workflow contract helper rejects non-pnpm setup-node jobs that do not disable package-manager auto-cache', async () => {
  const contracts = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-contracts.mjs'),
    ).href,
  );

  const fixtureRoot = writeReleaseWorkflowContractFixture({
    workflowText: read('.github/workflows/release.yml').replace(
      /^\s*package-manager-cache:\s*false\r?\n/m,
      '',
    ),
  });

  await assert.rejects(
    contracts.assertReleaseWorkflowContracts({
      repoRoot: fixtureRoot,
    }),
    /package-manager(?:\s|-)?auto-cache|package-manager-cache/i,
  );
});
