#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  resolveReleaseTelemetryExportProducerInput,
} from './materialize-release-telemetry-export.mjs';
import {
  findReleaseCatalogVariant,
  materializeReleaseCatalog,
  readReleaseCatalogFile,
} from './materialize-release-catalog.mjs';
import {
  resolveReleaseWindowSnapshotProducerInput,
} from './materialize-release-window-snapshot.mjs';
import {
  resolveReleaseSyncAuditProducerInput,
} from './materialize-release-sync-audit.mjs';
import {
  deriveReleaseTelemetrySnapshotFromExport,
  materializeReleaseTelemetrySnapshot,
  validateReleaseTelemetryExportShape,
  validateReleaseTelemetrySnapshotShape,
} from './materialize-release-telemetry-snapshot.mjs';
import {
  deriveSloGovernanceEvidenceFromReleaseTelemetrySnapshot,
  materializeSloGovernanceEvidence,
  validateSloGovernanceEvidenceShape,
} from './materialize-slo-governance-evidence.mjs';
import {
  auditExternalReleaseDependencyCoverage,
  buildExternalReleaseClonePlan,
  listExternalReleaseDependencySpecs,
} from './materialize-external-deps.mjs';
import {
  validateThirdPartyNoticesArtifact,
  validateThirdPartySbomArtifact,
} from './materialize-third-party-governance.mjs';
import {
  createReleaseGovernanceBundleManifest,
  listReleaseGovernanceBundleArtifactSpecs,
} from './materialize-release-governance-bundle.mjs';
import {
  createGhcrImagePublishPlan,
  createImagePublishMetadata,
  parseArgs as parseGhcrImagePublishArgs,
} from './publish-ghcr-image.mjs';
import {
  createGhcrManifestPublishMetadata,
  createGhcrManifestPublishPlan,
  parseArgs as parseGhcrManifestPublishArgs,
} from './publish-ghcr-manifest.mjs';
import {
  createLinuxDockerComposeSmokeEvidence,
  createLinuxDockerComposeSmokeOptions,
  createLinuxDockerComposeSmokePlan,
  createLinuxDockerRunFallbackResources,
  resolveLinuxDockerComposeSmokeExecutionMode,
} from './run-linux-docker-compose-smoke.mjs';
import {
  createLinuxHelmRenderSmokeEvidence,
  createLinuxHelmRenderSmokeOptions,
  createLinuxHelmRenderSmokePlan,
} from './run-linux-helm-render-smoke.mjs';
import {
  assertInstalledPackagedBootstrapData,
  assertInstalledReleasePayloadContract,
  assertInstalledRuntimeBackupBundle,
} from './installed-runtime-smoke-lib.mjs';
import {
  createUnixInstalledRuntimeSmokeEvidence,
  createUnixInstalledRuntimeSmokeOptions,
  createUnixInstalledRuntimeSmokePlan,
} from './run-unix-installed-runtime-smoke.mjs';
import {
  createWindowsInstalledRuntimeSmokeEvidence,
  createWindowsInstalledRuntimeSmokeOptions,
  createWindowsInstalledRuntimeSmokePlan,
} from './run-windows-installed-runtime-smoke.mjs';
import {
  collectReleaseWindowSnapshotResult,
  validateReleaseWindowSnapshotArtifact,
} from './compute-release-window-snapshot.mjs';
import { assertObservabilityContracts } from './observability-contracts.mjs';
import { assertDesktopReleaseSigningContracts } from './desktop-release-signing-contracts.mjs';
import { assertReleaseAttestationVerificationContracts } from './release-attestation-verification-contracts.mjs';
import { assertReleaseSyncAuditContracts } from './release-sync-audit-contracts.mjs';
import { assertReleaseWindowSnapshotContracts } from './release-window-snapshot-contracts.mjs';
import {
  createOrderedWorkflowLiteralPattern,
  findReleaseWorkflowPublishMetadataArtifact,
  listReleaseWorkflowNativeOfficialAssetPathsByPaths,
  listReleaseWorkflowNativeOfficialAssetPaths,
  listReleaseWorkflowPublishOfficialAssetPathsByPaths,
  listReleaseWorkflowPublishMetadataArtifactsByIds,
  listReleaseWorkflowPublishOfficialAssetPaths,
} from './release-workflow-publish-catalog.mjs';
import { assertReleaseWorkflowContracts } from './release-workflow-contracts.mjs';
import { listReleaseGovernanceLatestArtifactSpecs } from './restore-release-governance-latest.mjs';
import { assertRuntimeToolingContracts } from './runtime-tooling-contracts.mjs';
import { collectSloGovernanceResult } from './slo-governance.mjs';
import { assertSloGovernanceContracts } from './slo-governance-contracts.mjs';
import {
  auditReleaseSyncRepositories,
  validateReleaseSyncAuditArtifact,
} from './verify-release-sync.mjs';
import { assertReleaseGovernanceWorkflowContracts } from '../release-governance-workflow-contracts.mjs';
import { assertSupportedReleaseCliFormat } from './release-cli-format-catalog.mjs';
import {
  assertSupportedReleaseGovernanceProfile,
  createReleaseGovernanceCheckPlans,
} from './release-governance-plan-catalog.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');
const defaultReleaseTelemetrySnapshotPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const defaultReleaseWindowSnapshotPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-window-snapshot-latest.json',
);
const defaultReleaseSyncAuditPath = path.join(
  rootDir,
  'docs',
  'release',
  'release-sync-audit-latest.json',
);
const releaseGovernanceTestEnvPatterns = [
  /^SDKWORK_RELEASE_/,
  /^SDKWORK_SLO_GOVERNANCE_EVIDENCE_/,
  /^SDKWORK_(API_ROUTER|CORE|UI|APPBASE|IM_SDK)_GIT_REF$/,
];
const releaseGovernanceTestEnvKeys = new Set([
  'CARGO_BUILD_JOBS',
  'CARGO_TARGET_DIR',
  'CMAKE_GENERATOR',
  'HOST_CMAKE_GENERATOR',
  'SDKWORK_CC_DISABLE_BREPRO',
]);

function createSyntheticPrometheusHttpCounterSamples({
  service,
  healthyCount,
  unhealthyCount,
} = {}) {
  return [
    '# HELP sdkwork_http_requests_total Total HTTP requests observed',
    '# TYPE sdkwork_http_requests_total counter',
    `sdkwork_http_requests_total{service="${service}",method="GET",route="/health",status="200"} ${healthyCount}`,
    `sdkwork_http_requests_total{service="${service}",method="GET",route="/health",status="503"} ${unhealthyCount}`,
  ].join('\n');
}

function createSyntheticTelemetrySupplementalTargets() {
  return {
    'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
    'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
    'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
    'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
    'routing-simulation-p95-latency': { value: 420, burnRates: { '1h': 0.9, '6h': 0.5 } },
    'api-key-issuance-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    'runtime-rollout-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    'billing-event-write-success-rate': { ratio: 0.9995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    'account-hold-creation-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    'request-settlement-finalize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    'pricing-lifecycle-synchronize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
  };
}

function createSyntheticReleaseTelemetryExportBundle() {
  return resolveReleaseTelemetryExportProducerInput({
    generatedAt: '2026-04-08T10:00:00Z',
    sourceKind: 'observability-control-plane',
    sourceProvenance: 'release-governance-fallback',
    freshnessMinutes: 5,
    gatewayPrometheusText: createSyntheticPrometheusHttpCounterSamples({
      service: 'gateway-service',
      healthyCount: 9997,
      unhealthyCount: 3,
    }),
    adminPrometheusText: createSyntheticPrometheusHttpCounterSamples({
      service: 'admin-api-service',
      healthyCount: 4997,
      unhealthyCount: 3,
    }),
    portalPrometheusText: createSyntheticPrometheusHttpCounterSamples({
      service: 'portal-api-service',
      healthyCount: 9993,
      unhealthyCount: 7,
    }),
    supplementalTargetsJson: JSON.stringify({
      targets: createSyntheticTelemetrySupplementalTargets(),
    }),
  }).payload;
}

function createSyntheticReleaseTelemetrySnapshot() {
  return deriveReleaseTelemetrySnapshotFromExport({
    exportBundle: createSyntheticReleaseTelemetryExportBundle(),
  });
}

function createSyntheticReleaseWindowSnapshotArtifact() {
  return resolveReleaseWindowSnapshotProducerInput({
    snapshotJson: JSON.stringify({
      generatedAt: '2026-04-08T12:00:00Z',
      source: {
        kind: 'release-window-snapshot-fixture',
        provenance: 'release-governance-fallback',
      },
      snapshot: {
        latestReleaseTag: 'release-2026-03-28-8',
        commitsSinceLatestRelease: 16,
        workingTreeEntryCount: 631,
        hasReleaseBaseline: true,
      },
    }),
  }).artifact;
}

function createSyntheticReleaseSyncAuditArtifact() {
  return resolveReleaseSyncAuditProducerInput({
    auditJson: JSON.stringify({
      generatedAt: '2026-04-08T13:00:00Z',
      source: {
        kind: 'release-sync-audit-fixture',
        provenance: 'release-governance-fallback',
      },
      summary: {
        releasable: true,
        reports: [
          {
            id: 'sdkwork-api-router',
            targetDir: rootDir,
            expectedGitRoot: rootDir,
            topLevel: rootDir,
            remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
            localHead: 'abc123',
            remoteHead: 'abc123',
            expectedRef: 'main',
            branch: 'main',
            upstream: 'origin/main',
            ahead: 0,
            behind: 0,
            isDirty: false,
            reasons: [],
            releasable: true,
          },
        ],
      },
    }),
  }).artifact;
}

function createSyntheticThirdPartySbomArtifact() {
  return {
    spdxVersion: 'SPDX-2.3',
    dataLicense: 'CC0-1.0',
    SPDXID: 'SPDXRef-DOCUMENT',
    name: 'sdkwork-api-router-third-party-sbom',
    documentNamespace: 'https://sdkwork.local/spdx/sdkwork-api-router/release-governance-fallback',
    creationInfo: {
      created: '2026-04-18T00:00:00.000Z',
      creators: ['Tool: sdkwork-api-router/scripts/release/materialize-third-party-governance.mjs'],
    },
    documentDescribes: ['SPDXRef-Package-sdkwork-api-router-third-party-governance'],
    packages: [
      {
        SPDXID: 'SPDXRef-Package-sdkwork-api-router-third-party-governance',
        name: 'sdkwork-api-router-third-party-governance',
        versionInfo: '2026-04-18T00:00:00.000Z',
        supplier: 'Organization: SDKWork',
        downloadLocation: 'NOASSERTION',
        filesAnalyzed: false,
        licenseConcluded: 'NOASSERTION',
        licenseDeclared: 'NOASSERTION',
      },
      {
        SPDXID: 'SPDXRef-cargo-serde-1.0.218',
        name: 'serde',
        versionInfo: '1.0.218',
        supplier: 'NOASSERTION',
        downloadLocation: 'https://crates.io/',
        filesAnalyzed: false,
        licenseConcluded: 'NOASSERTION',
        licenseDeclared: 'MIT OR Apache-2.0',
        externalRefs: [
          {
            referenceCategory: 'PACKAGE-MANAGER',
            referenceType: 'purl',
            referenceLocator: 'pkg:cargo/serde@1.0.218',
          },
        ],
      },
      {
        SPDXID: 'SPDXRef-npm-react-19.2.4',
        name: 'react',
        versionInfo: '19.2.4',
        supplier: 'NOASSERTION',
        downloadLocation: 'https://react.dev/',
        filesAnalyzed: false,
        licenseConcluded: 'NOASSERTION',
        licenseDeclared: 'MIT',
        externalRefs: [
          {
            referenceCategory: 'PACKAGE-MANAGER',
            referenceType: 'purl',
            referenceLocator: 'pkg:npm/react@19.2.4',
          },
        ],
      },
    ],
    relationships: [
      {
        spdxElementId: 'SPDXRef-DOCUMENT',
        relationshipType: 'DESCRIBES',
        relatedSpdxElement: 'SPDXRef-Package-sdkwork-api-router-third-party-governance',
      },
      {
        spdxElementId: 'SPDXRef-Package-sdkwork-api-router-third-party-governance',
        relationshipType: 'DEPENDS_ON',
        relatedSpdxElement: 'SPDXRef-cargo-serde-1.0.218',
      },
      {
        spdxElementId: 'SPDXRef-Package-sdkwork-api-router-third-party-governance',
        relationshipType: 'DEPENDS_ON',
        relatedSpdxElement: 'SPDXRef-npm-react-19.2.4',
      },
    ],
  };
}

function createSyntheticThirdPartyNoticesArtifact() {
  return {
    version: 1,
    generatedAt: '2026-04-18T00:00:00.000Z',
    packageCount: 2,
    cargoPackageCount: 1,
    npmPackageCount: 1,
    packages: [
      {
        ecosystem: 'cargo',
        name: 'serde',
        version: '1.0.218',
        licenseDeclared: 'MIT OR Apache-2.0',
        downloadLocation: 'https://crates.io/',
        repositoryUrl: 'https://github.com/serde-rs/serde',
        homepage: '',
        sourcePath: 'vendor/serde-1.0.218/Cargo.toml',
        noticeFiles: ['vendor/serde-1.0.218/LICENSE-MIT'],
      },
      {
        ecosystem: 'npm',
        name: 'react',
        version: '19.2.4',
        licenseDeclared: 'MIT',
        downloadLocation: 'https://react.dev/',
        repositoryUrl: '',
        homepage: 'https://react.dev/',
        sourcePath: 'apps/sdkwork-router-portal/node_modules/react/package.json',
        noticeFiles: ['apps/sdkwork-router-portal/node_modules/react/LICENSE'],
      },
    ],
    noticeText: [
      'SDKWork API Router Third-Party Notices',
      '',
      'Generated: 2026-04-18T00:00:00.000Z',
      'Package count: 2',
      '',
      'Cargo crates:',
      '- serde 1.0.218 [MIT OR Apache-2.0] https://crates.io/',
      '',
      'NPM packages:',
      '- react 19.2.4 [MIT] https://react.dev/',
      '',
    ].join('\n'),
  };
}

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

function createSyntheticReleaseCatalogFixture({
  platform,
  arch,
} = {}) {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-assets-'));
  const releaseOutputDir = path.join(fixtureRoot, 'release');
  const bundleOutputDir = path.join(releaseOutputDir, 'native', platform, arch, 'bundles');
  const archiveBaseName = `sdkwork-api-router-product-server-${platform}-${arch}`;
  const archiveFile = `${archiveBaseName}.tar.gz`;
  const checksumFile = `${archiveFile}.sha256.txt`;
  const manifestFile = `${archiveBaseName}.manifest.json`;
  const manifest = {
    type: 'product-server-archive',
    productId: 'sdkwork-api-router-product-server',
    platform,
    arch,
    archiveFile,
    checksumFile,
    embeddedManifestFile: 'release-manifest.json',
    services: ['router-product-service'],
    sites: ['admin', 'portal'],
    bootstrapDataRoots: ['data'],
    deploymentAssetRoots: ['deploy'],
  };

  mkdirSync(bundleOutputDir, { recursive: true });
  writeFileSync(
    path.join(bundleOutputDir, archiveFile),
    'synthetic release bundle\n',
    'utf8',
  );
  writeFileSync(
    path.join(bundleOutputDir, checksumFile),
    `synthetic-sha256  ${archiveFile}\n`,
    'utf8',
  );
  writeFileSync(
    path.join(bundleOutputDir, manifestFile),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
  );
  writeFileSync(
    path.join(releaseOutputDir, 'release-catalog.json'),
    `${JSON.stringify({
      version: 1,
      type: 'sdkwork-release-catalog',
      releaseTag: 'release-governance-fallback',
      generatedAt: '2026-04-18T00:00:00.000Z',
      productCount: 1,
      variantCount: 1,
      products: [
        {
          productId: 'sdkwork-api-router-product-server',
          variants: [
            {
              platform,
              arch,
              outputDirectory: toPortablePath(path.relative(releaseOutputDir, bundleOutputDir)),
              variantKind: 'server-archive',
              primaryFile: archiveFile,
              primaryFileSizeBytes: 0,
              checksumFile,
              checksumAlgorithm: 'sha256',
              manifestFile,
              sha256: 'synthetic-sha256',
              manifest,
            },
          ],
        },
      ],
    }, null, 2)}\n`,
    'utf8',
  );

  return {
    releaseOutputDir,
    cleanup() {
      rmSync(fixtureRoot, { recursive: true, force: true });
    },
  };
}

function createSyntheticUnixInstalledRuntimeSmokeFallback() {
  const releaseCatalogFixture = createSyntheticReleaseCatalogFixture({
    platform: 'linux',
    arch: 'x64',
  });
  const options = createUnixInstalledRuntimeSmokeOptions({
    repoRoot: rootDir,
    platform: 'linux',
    arch: 'x64',
    target: 'x86_64-unknown-linux-gnu',
    releaseOutputDir: releaseCatalogFixture.releaseOutputDir,
    runtimeHome: 'artifacts/release-smoke/linux-x64',
    evidencePath: 'artifacts/release-governance/unix-installed-runtime-smoke-linux-x64.json',
  });
  try {
    const plan = createUnixInstalledRuntimeSmokePlan({
      repoRoot: rootDir,
      ...options,
      ports: {
        web: 19483,
        gateway: 19480,
        admin: 19481,
        portal: 19482,
      },
    });
    const evidence = createUnixInstalledRuntimeSmokeEvidence({
      repoRoot: rootDir,
      plan,
      ok: true,
    });

    return {
      options,
      plan,
      evidence,
    };
  } finally {
    releaseCatalogFixture.cleanup();
  }
}

function createSyntheticWindowsInstalledRuntimeSmokeFallback() {
  const releaseCatalogFixture = createSyntheticReleaseCatalogFixture({
    platform: 'windows',
    arch: 'x64',
  });
  const options = createWindowsInstalledRuntimeSmokeOptions({
    repoRoot: rootDir,
    platform: 'windows',
    arch: 'x64',
    target: 'x86_64-pc-windows-msvc',
    releaseOutputDir: releaseCatalogFixture.releaseOutputDir,
    runtimeHome: 'artifacts/release-smoke/windows-x64',
    evidencePath: 'artifacts/release-governance/windows-installed-runtime-smoke-windows-x64.json',
  });
  try {
    const plan = createWindowsInstalledRuntimeSmokePlan({
      repoRoot: rootDir,
      ...options,
      ports: {
        web: 29483,
        gateway: 29480,
        admin: 29481,
        portal: 29482,
      },
    });
    const evidence = createWindowsInstalledRuntimeSmokeEvidence({
      repoRoot: rootDir,
      plan,
      ok: true,
    });

    return {
      options,
      plan,
      evidence,
    };
  } finally {
    releaseCatalogFixture.cleanup();
  }
}

function createSyntheticInstalledRuntimeContractFallback() {
  const runtimeRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-installed-runtime-contract-'));
  const backupRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-installed-runtime-backup-'));
  const currentRoot = path.join(runtimeRoot, 'current');
  const releaseRoot = path.join(runtimeRoot, 'releases', '1.2.3');
  const routerBinary = path.join(
    releaseRoot,
    'bin',
    process.platform === 'win32' ? 'router-product-service.exe' : 'router-product-service',
  );
  const bootstrapDataRoot = path.join(releaseRoot, 'data');
  const deploymentAssetRoot = path.join(releaseRoot, 'deploy');
  const releasePayloadManifest = path.join(releaseRoot, 'release-manifest.json');
  const releasePayloadReadmeFile = path.join(releaseRoot, 'README.txt');
  const adminSiteDistDir = path.join(releaseRoot, 'sites', 'admin', 'dist');
  const portalSiteDistDir = path.join(releaseRoot, 'sites', 'portal', 'dist');
  const configRoot = path.join(runtimeRoot, 'config');
  const configFile = path.join(configRoot, 'router.yaml');
  const mutableDataRoot = path.join(runtimeRoot, 'data');
  const logRoot = path.join(runtimeRoot, 'log');
  const runRoot = path.join(runtimeRoot, 'run');

  mkdirSync(currentRoot, { recursive: true });
  mkdirSync(path.dirname(routerBinary), { recursive: true });
  mkdirSync(path.join(bootstrapDataRoot, 'channels'), { recursive: true });
  mkdirSync(path.join(bootstrapDataRoot, 'providers'), { recursive: true });
  mkdirSync(path.join(bootstrapDataRoot, 'routing'), { recursive: true });
  mkdirSync(deploymentAssetRoot, { recursive: true });
  mkdirSync(adminSiteDistDir, { recursive: true });
  mkdirSync(portalSiteDistDir, { recursive: true });
  mkdirSync(configRoot, { recursive: true });
  mkdirSync(mutableDataRoot, { recursive: true });
  mkdirSync(logRoot, { recursive: true });
  mkdirSync(runRoot, { recursive: true });

  writeFileSync(routerBinary, '#!/usr/bin/env sh\nexit 0\n', 'utf8');
  writeFileSync(path.join(bootstrapDataRoot, 'channels', 'default.json'), '{}\n', 'utf8');
  writeFileSync(path.join(bootstrapDataRoot, 'providers', 'default.json'), '{}\n', 'utf8');
  writeFileSync(path.join(bootstrapDataRoot, 'routing', 'default.json'), '{}\n', 'utf8');
  writeFileSync(releasePayloadManifest, '{}\n', 'utf8');
  writeFileSync(releasePayloadReadmeFile, 'release readme\n', 'utf8');
  writeFileSync(configFile, 'router: {}\n', 'utf8');
  writeFileSync(path.join(adminSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');
  writeFileSync(path.join(portalSiteDistDir, 'index.html'), '<html></html>\n', 'utf8');
  writeFileSync(
    path.join(currentRoot, 'release-manifest.json'),
    `${JSON.stringify({
      runtime: 'sdkwork-api-router',
      layoutVersion: 2,
      installMode: 'portable',
      productRoot: runtimeRoot,
      controlRoot: currentRoot,
      releaseVersion: '1.2.3',
      releasesRoot: path.join(runtimeRoot, 'releases'),
      releaseRoot,
      target: process.platform === 'win32' ? 'x86_64-pc-windows-msvc' : 'x86_64-unknown-linux-gnu',
      installedBinaries: ['router-product-service'],
      bootstrapDataRoot,
      deploymentAssetRoot,
      releasePayloadManifest,
      releasePayloadReadmeFile,
      adminSiteDistDir,
      portalSiteDistDir,
      routerBinary,
      configRoot,
      configFile,
      mutableDataRoot,
      logRoot,
      runRoot,
      installedAt: '2026-04-20T00:00:00.000Z',
    }, null, 2)}\n`,
    'utf8',
  );

  mkdirSync(path.join(backupRoot, 'bundle-state', 'control'), { recursive: true });
  mkdirSync(path.join(backupRoot, 'bundle-state', 'config-snapshot'), { recursive: true });
  mkdirSync(path.join(backupRoot, 'bundle-state', 'data-snapshot'), { recursive: true });
  writeFileSync(
    path.join(backupRoot, 'backup-manifest.json'),
    `${JSON.stringify({
      formatVersion: 2,
      createdAt: '2026-04-20T00:00:00Z',
      runtimeHome: '/opt/sdkwork-api-router/current',
      configRoot: '/etc/sdkwork-api-router',
      mutableDataRoot: '/var/lib/sdkwork-api-router',
      logRoot: '/var/log/sdkwork-api-router',
      runRoot: '/run/sdkwork-api-router',
      bundle: {
        controlManifestFile: 'bundle-state/control/runtime-manifest.json',
        configSnapshotRoot: 'bundle-state/config-snapshot',
        mutableDataSnapshotRoot: 'bundle-state/data-snapshot',
      },
      database: {
        kind: 'sqlite',
        strategy: 'file-copy',
        dumpFile: null,
      },
    }, null, 2)}\n`,
    'utf8',
  );
  writeFileSync(
    path.join(backupRoot, 'bundle-state', 'control', 'runtime-manifest.json'),
    '{}\n',
    'utf8',
  );

  return {
    runtimeRoot,
    backupRoot,
    cleanup() {
      rmSync(runtimeRoot, { recursive: true, force: true });
      rmSync(backupRoot, { recursive: true, force: true });
    },
  };
}

function createSyntheticLinuxDockerComposeSmokeFallback({
  hostPlatform = process.platform,
} = {}) {
  const options = createLinuxDockerComposeSmokeOptions({
    repoRoot: rootDir,
    platform: 'linux',
    arch: 'x64',
    bundlePath: 'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
    evidencePath: 'artifacts/release-governance/docker-compose-smoke-linux-x64.json',
  });
  const plan = createLinuxDockerComposeSmokePlan({
    repoRoot: rootDir,
    hostPlatform,
    ...options,
  });
  const evidence = createLinuxDockerComposeSmokeEvidence({
    repoRoot: rootDir,
    plan,
    ok: true,
  });

  return {
    options,
    plan,
    evidence,
  };
}

function createSyntheticLinuxHelmRenderSmokeFallback() {
  const options = createLinuxHelmRenderSmokeOptions({
    repoRoot: rootDir,
    platform: 'linux',
    arch: 'x64',
    bundlePath: 'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
    evidencePath: 'artifacts/release-governance/helm-render-smoke-linux-x64.json',
  });
  const plan = createLinuxHelmRenderSmokePlan({
    repoRoot: rootDir,
    ...options,
  });
  const evidence = createLinuxHelmRenderSmokeEvidence({
    repoRoot: rootDir,
    plan,
    ok: true,
  });

  return {
    options,
    plan,
    evidence,
  };
}

function createSyntheticReleaseCatalogMaterializationFallback() {
  const releaseCatalogFixture = createSyntheticReleaseCatalogFixture({
    platform: 'linux',
    arch: 'x64',
  });
  const outputPath = path.join(
    releaseCatalogFixture.releaseOutputDir,
    'release-catalog.materialized.json',
  );

  try {
    const result = materializeReleaseCatalog({
      assetsRoot: releaseCatalogFixture.releaseOutputDir,
      releaseTag: 'release-governance-fallback',
      outputPath,
    });
    const catalog = readReleaseCatalogFile({
      releaseCatalogPath: outputPath,
    });
    const variant = findReleaseCatalogVariant(catalog, {
      productId: 'sdkwork-api-router-product-server',
      variantKind: 'server-archive',
      platform: 'linux',
      arch: 'x64',
    });

    return {
      result,
      catalog,
      variant,
    };
  } finally {
    releaseCatalogFixture.cleanup();
  }
}

function materializeLiveSloGovernanceEvidence({
  env = process.env,
} = {}) {
  let telemetrySnapshotPath = existsSync(defaultReleaseTelemetrySnapshotPath)
    ? defaultReleaseTelemetrySnapshotPath
    : '';

  if (!telemetrySnapshotPath) {
    try {
      telemetrySnapshotPath = materializeReleaseTelemetrySnapshot({
        env,
      }).outputPath;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      if (/missing release telemetry input/i.test(errorMessage)) {
        return {
          ok: false,
          blocked: true,
          reason: 'telemetry-input-missing',
          errorMessage,
          summary: null,
        };
      }

      return {
        ok: false,
        blocked: false,
        reason: 'telemetry-materialization-failed',
        errorMessage,
        summary: null,
      };
    }
  }

  try {
    materializeSloGovernanceEvidence({
      telemetrySnapshotPath,
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      blocked: false,
      reason: 'slo-evidence-materialization-failed',
      errorMessage,
      summary: null,
    };
  }

  return collectSloGovernanceResult();
}

export function resolveNodeRunner({
  platform = process.platform,
  nodeExecutable = process.execPath,
} = {}) {
  return {
    command: nodeExecutable,
    shell: false,
  };
}

export function listReleaseGovernanceCheckPlans({
  nodeExecutable = process.execPath,
  profile = 'release',
} = {}) {
  return createReleaseGovernanceCheckPlans({
    nodeExecutable,
    profile,
  });
}

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function parseJsonObject(value) {
  const text = String(value ?? '').trim();
  if (!text.startsWith('{')) {
    return null;
  }

  try {
    const payload = JSON.parse(text);
    return payload && typeof payload === 'object' ? payload : null;
  } catch {
    return null;
  }
}

function hasCommandExecBlockedReason(reasons = []) {
  return Array.isArray(reasons)
    && reasons.some((reason) => String(reason ?? '').trim() === 'command-exec-blocked');
}

function isReleaseGovernanceResultBlocked(result) {
  const errorText = [
    String(result?.errorMessage ?? ''),
    String(result?.stderr ?? ''),
    String(result?.stdout ?? ''),
  ].join('\n');
  if (/(eperm|eacces)/i.test(errorText) || /command-exec-blocked/i.test(errorText)) {
    return true;
  }

  const payload = parseJsonObject(result?.stdout);
  if (!payload) {
    return false;
  }

  if (payload.blocked === true || String(payload.reason ?? '').trim() === 'command-exec-blocked') {
    return true;
  }

  if (Array.isArray(payload.reports)) {
    return payload.reports.some((report) => hasCommandExecBlockedReason(report?.reasons));
  }

  return false;
}

function summarizeReleaseGovernanceResults(results = []) {
  const passingIds = [];
  const blockedIds = [];
  const failingIds = [];

  for (const result of results) {
    if (result?.ok === true) {
      passingIds.push(result.id);
      continue;
    }

    if (isReleaseGovernanceResultBlocked(result)) {
      blockedIds.push(result.id);
      continue;
    }

    failingIds.push(result.id);
  }

  return {
    blocked: blockedIds.length > 0,
    passingIds,
    blockedIds,
    failingIds,
  };
}

function isReleaseGovernanceTestPlan(plan) {
  return Array.isArray(plan?.args) && plan.args.includes('--test');
}

function resolveReleaseGovernanceChildEnv({
  plan,
  env = process.env,
} = {}) {
  if (!isReleaseGovernanceTestPlan(plan)) {
    return env;
  }

  const sanitizedEnv = { ...env };
  for (const key of Object.keys(sanitizedEnv)) {
    if (
      releaseGovernanceTestEnvKeys.has(key)
      || releaseGovernanceTestEnvPatterns.some((pattern) => pattern.test(key))
    ) {
      delete sanitizedEnv[key];
    }
  }

  return sanitizedEnv;
}

async function runFallbackReleaseGovernanceCheck({
  plan,
  env = process.env,
  fallbackSpawnSyncImpl = spawnSync,
} = {}) {
  if (plan.id === 'release-sync-audit-test') {
    await assertReleaseSyncAuditContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-workflow-test') {
    await assertReleaseWorkflowContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-workflow-publish-catalog-test') {
    const nativeOfficialAssetPaths = listReleaseWorkflowNativeOfficialAssetPaths();
    const publishOfficialAssetPaths = listReleaseWorkflowPublishOfficialAssetPaths();
    const expectedNativeOfficialAssetPaths = [
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.tar.gz',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.tar.gz.sha256.txt',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.manifest.json',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*',
    ];
    const expectedPublishOfficialAssetPaths = [
      'artifacts/release/**/sdkwork-api-router-product-server-*.tar.gz',
      'artifacts/release/**/sdkwork-api-router-product-server-*.tar.gz.sha256.txt',
      'artifacts/release/**/sdkwork-api-router-product-server-*.manifest.json',
      'artifacts/release/**/desktop/portal/sdkwork-router-portal-desktop-*',
      'artifacts/release/release-catalog.json',
    ];

    if (
      JSON.stringify(nativeOfficialAssetPaths)
      !== JSON.stringify(listReleaseWorkflowNativeOfficialAssetPathsByPaths(expectedNativeOfficialAssetPaths))
    ) {
      throw new Error('release workflow publish catalog fallback must preserve the governed native official asset paths');
    }

    if (
      JSON.stringify(publishOfficialAssetPaths)
      !== JSON.stringify(listReleaseWorkflowPublishOfficialAssetPathsByPaths(expectedPublishOfficialAssetPaths))
    ) {
      throw new Error('release workflow publish catalog fallback must preserve the governed publish asset globs');
    }

    const [governanceBundleArtifact, ghcrImagePublishArtifact, ghcrManifestPublishArtifact] = listReleaseWorkflowPublishMetadataArtifactsByIds([
      'governance-bundle',
      'ghcr-image-publish',
      'ghcr-manifest-publish',
    ]);

    if (JSON.stringify(findReleaseWorkflowPublishMetadataArtifact('governance-bundle')) !== JSON.stringify({
      id: 'governance-bundle',
      name: 'release-governance-bundle',
      path: 'artifacts/release-governance-bundle/**/*',
    })) {
      throw new Error('release workflow publish catalog fallback must preserve the governance bundle artifact contract');
    }

    if (JSON.stringify(ghcrImagePublishArtifact) !== JSON.stringify({
      id: 'ghcr-image-publish',
      artifactNameTemplate: 'release-governance-ghcr-image-publish-${{ matrix.platform }}-${{ matrix.arch }}',
      pathTemplate: 'artifacts/release-governance/ghcr-image-publish-${{ matrix.platform }}-${{ matrix.arch }}.json',
    })) {
      throw new Error('release workflow publish catalog fallback must preserve the GHCR image publish metadata contract');
    }

    if (JSON.stringify(ghcrManifestPublishArtifact) !== JSON.stringify({
      id: 'ghcr-manifest-publish',
      artifactName: 'release-governance-ghcr-image-manifest-publish',
      path: 'artifacts/release-governance/ghcr-image-manifest-publish.json',
    })) {
      throw new Error('release workflow publish catalog fallback must preserve the GHCR manifest publish metadata contract');
    }

    const nativePattern = createOrderedWorkflowLiteralPattern([
      'Upload official release assets',
      ...nativeOfficialAssetPaths,
    ]);
    if (!nativePattern.test(`
      - name: Upload official release assets
        with:
          path: |
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz.sha256.txt
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.manifest.json
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*
    `)) {
      throw new Error('release workflow publish catalog fallback must keep ordered workflow literal matching for native asset publication');
    }

    const publishPattern = createOrderedWorkflowLiteralPattern([
      'Publish release assets',
      ...publishOfficialAssetPaths,
    ]);
    if (!publishPattern.test(`
      - name: Publish release assets
        with:
          files: |
            artifacts/release/**/sdkwork-api-router-product-server-*.tar.gz
            artifacts/release/**/sdkwork-api-router-product-server-*.tar.gz.sha256.txt
            artifacts/release/**/sdkwork-api-router-product-server-*.manifest.json
            artifacts/release/**/desktop/portal/sdkwork-router-portal-desktop-*
            artifacts/release/release-catalog.json
    `)) {
      throw new Error('release workflow publish catalog fallback must keep ordered workflow literal matching for publish asset globs');
    }

    if (JSON.stringify(governanceBundleArtifact) !== JSON.stringify({
      id: 'governance-bundle',
      name: 'release-governance-bundle',
      path: 'artifacts/release-governance-bundle/**/*',
    })) {
      throw new Error('release workflow publish catalog fallback must preserve ordered metadata artifact lookup by ids');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-governance-workflow-test') {
    await assertReleaseGovernanceWorkflowContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-observability-test') {
    await assertObservabilityContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-attestation-verify-test') {
    await assertReleaseAttestationVerificationContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-runtime-tooling-test') {
    await assertRuntimeToolingContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-desktop-signing-test') {
    await assertDesktopReleaseSigningContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-unix-installed-runtime-smoke-test') {
    const { plan: smokePlan, evidence } = createSyntheticUnixInstalledRuntimeSmokeFallback();

    if (evidence.platform !== 'linux' || evidence.target !== 'x86_64-unknown-linux-gnu') {
      throw new Error('unix installed runtime smoke fallback must preserve the linux release target');
    }

    if (!Array.isArray(smokePlan.healthUrls) || smokePlan.healthUrls.length !== 3) {
      throw new Error('unix installed runtime smoke fallback must expose three health probe urls');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-windows-installed-runtime-smoke-test') {
    const { plan: smokePlan, evidence } = createSyntheticWindowsInstalledRuntimeSmokeFallback();

    if (evidence.platform !== 'windows' || evidence.target !== 'x86_64-pc-windows-msvc') {
      throw new Error('windows installed runtime smoke fallback must preserve the windows release target');
    }

    if (!Array.isArray(smokePlan.healthUrls) || smokePlan.healthUrls.length !== 3) {
      throw new Error('windows installed runtime smoke fallback must expose three health probe urls');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-installed-runtime-contract-test') {
    const fixture = createSyntheticInstalledRuntimeContractFallback();
    try {
      assertInstalledReleasePayloadContract(fixture.runtimeRoot);
      assertInstalledPackagedBootstrapData(fixture.runtimeRoot);
      assertInstalledRuntimeBackupBundle(fixture.backupRoot);
    } finally {
      fixture.cleanup();
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-linux-docker-compose-smoke-test') {
    const { plan: smokePlan, evidence } = createSyntheticLinuxDockerComposeSmokeFallback({
      hostPlatform: process.platform,
    });
    const expectedExecutionMode = resolveLinuxDockerComposeSmokeExecutionMode({
      hostPlatform: process.platform,
    });
    const expectedFallbackResources = createLinuxDockerRunFallbackResources({
      composeProjectName: smokePlan.composeProjectName,
    });

    if (evidence.platform !== 'linux' || evidence.arch !== 'x64') {
      throw new Error('linux docker compose smoke fallback must preserve the linux release target');
    }

    if (smokePlan.executionMode !== expectedExecutionMode) {
      throw new Error('linux docker compose smoke fallback must preserve host execution mode selection');
    }

    if (!Array.isArray(smokePlan.healthUrls) || smokePlan.healthUrls.length !== 3) {
      throw new Error('linux docker compose smoke fallback must expose three health probe urls');
    }

    if (JSON.stringify(smokePlan.fallbackResources) !== JSON.stringify(expectedFallbackResources)) {
      throw new Error('linux docker compose smoke fallback must preserve docker-run fallback resources');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-linux-helm-render-smoke-test') {
    const { plan: smokePlan, evidence } = createSyntheticLinuxHelmRenderSmokeFallback();

    if (evidence.platform !== 'linux' || evidence.arch !== 'x64') {
      throw new Error('linux helm render smoke fallback must preserve the linux release target');
    }

    if (
      !Array.isArray(smokePlan.requiredTemplateKinds)
      || smokePlan.requiredTemplateKinds.join(',') !== 'Secret,Service,Deployment,Ingress'
    ) {
      throw new Error('linux helm render smoke fallback must preserve required template kinds');
    }

    if (smokePlan.chartRelativePath !== 'deploy/helm/sdkwork-api-router') {
      throw new Error('linux helm render smoke fallback must preserve the packaged chart path');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-materialize-external-deps-test') {
    const specs = listExternalReleaseDependencySpecs();
    if (!Array.isArray(specs) || specs.length === 0) {
      throw new Error('external release dependency specs must be declared');
    }

    const clonePlan = buildExternalReleaseClonePlan({
      spec: specs[0],
      env,
    });
    if (!clonePlan || clonePlan.command !== 'git' || !Array.isArray(clonePlan.args)) {
      throw new Error('external release dependency clone plans must remain git-based');
    }

    const coverage = auditExternalReleaseDependencyCoverage();
    if (coverage.covered !== true) {
      const uncoveredDetails = (coverage.uncoveredReferences ?? [])
        .map((reference) => `${reference.sourceFile}:${reference.field}:${reference.name}`)
        .join(', ');
      throw new Error(`external release dependency coverage is incomplete: ${uncoveredDetails}`);
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-materialize-release-catalog-test') {
    const { result, catalog, variant } = createSyntheticReleaseCatalogMaterializationFallback();

    if (result.releaseTag !== 'release-governance-fallback') {
      throw new Error('release catalog fallback must preserve the governed release tag');
    }

    if (result.productCount !== 1 || result.variantCount !== 1) {
      throw new Error('release catalog fallback must produce a single governed product variant');
    }

    if (catalog.type !== 'sdkwork-release-catalog') {
      throw new Error('release catalog fallback must materialize the governed catalog document type');
    }

    if (!variant || variant.primaryFile !== 'sdkwork-api-router-product-server-linux-x64.tar.gz') {
      throw new Error('release catalog fallback must preserve the linux server archive variant');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-third-party-governance-test') {
    const sbom = createSyntheticThirdPartySbomArtifact();
    const notices = createSyntheticThirdPartyNoticesArtifact();
    const sbomValidation = validateThirdPartySbomArtifact(sbom);
    const noticesValidation = validateThirdPartyNoticesArtifact(notices);

    if (sbomValidation.spdxVersion !== 'SPDX-2.3') {
      throw new Error('third-party SBOM fallback must preserve SPDX-2.3');
    }

    if (noticesValidation.packageCount !== 2) {
      throw new Error('third-party notices fallback must preserve the governed package count');
    }

    if (!/react 19\.2\.4/i.test(notices.noticeText) || !/serde 1\.0\.218/i.test(notices.noticeText)) {
      throw new Error('third-party notices fallback must preserve the governed notice text inventory');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-publish-ghcr-image-test') {
    const parsed = parseGhcrImagePublishArgs([
      '--release-tag',
      'release-governance-fallback',
      '--platform',
      'linux',
      '--arch',
      'x64',
      '--bundle-path',
      'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
      '--metadata-path',
      'artifacts/release-governance/ghcr-image-publish-linux-x64.json',
    ]);
    const publishPlan = createGhcrImagePublishPlan({
      repoRoot: rootDir,
      releaseTag: parsed.releaseTag,
      platform: parsed.platform,
      arch: parsed.arch,
      bundlePath: parsed.bundlePath,
      metadataPath: parsed.metadataPath,
      env: {
        GITHUB_REPOSITORY_OWNER: 'Sdkwork-Cloud',
      },
    });
    const metadata = createImagePublishMetadata({
      releaseTag: publishPlan.releaseTag,
      platform: publishPlan.platform,
      arch: publishPlan.arch,
      bundlePath: publishPlan.bundlePath,
      imageRepository: publishPlan.imageRepository,
      imageTag: publishPlan.imageTag,
      imageRef: publishPlan.imageRef,
      digest: `sha256:${'d'.repeat(64)}`,
    });

    if (publishPlan.imageRef !== 'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-governance-fallback-linux-x64') {
      throw new Error('GHCR image publish fallback must preserve the governed target image ref');
    }

    if (!/sdkwork-api-router-product-server-linux-x64\.tar\.gz$/.test(publishPlan.bundlePath)) {
      throw new Error('GHCR image publish fallback must preserve the governed bundle path');
    }

    if (publishPlan.metadataPath !== path.join(rootDir, 'artifacts', 'release-governance', 'ghcr-image-publish-linux-x64.json')) {
      throw new Error('GHCR image publish fallback must preserve the governed metadata path');
    }

    if (metadata.type !== 'sdkwork-ghcr-image-publish' || metadata.arch !== 'x64') {
      throw new Error('GHCR image publish fallback must preserve governed metadata semantics');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-publish-ghcr-manifest-test') {
    const parsed = parseGhcrManifestPublishArgs([
      '--release-tag',
      'release-governance-fallback',
      '--metadata-path',
      'artifacts/release-governance/ghcr-image-manifest-publish.json',
      '--source-image-ref',
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-governance-fallback-linux-x64',
      '--source-image-ref',
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-governance-fallback-linux-arm64',
    ]);
    const publishPlan = createGhcrManifestPublishPlan({
      releaseTag: parsed.releaseTag,
      env: {
        GITHUB_REPOSITORY_OWNER: 'Sdkwork-Cloud',
      },
    });
    const metadata = createGhcrManifestPublishMetadata({
      releaseTag: publishPlan.releaseTag,
      imageRepository: publishPlan.imageRepository,
      targetImageTag: publishPlan.targetImageTag,
      targetImageRef: publishPlan.targetImageRef,
      sourceImageRefs: publishPlan.sourceImageRefs,
      inspectText: [
        `Name: ${publishPlan.targetImageRef}`,
        'MediaType: application/vnd.oci.image.index.v1+json',
        `Digest: sha256:${'a'.repeat(64)}`,
      ].join('\n'),
      rawManifestText: JSON.stringify({
        schemaVersion: 2,
        mediaType: 'application/vnd.oci.image.index.v1+json',
        manifests: [
          {
            mediaType: 'application/vnd.oci.image.manifest.v1+json',
            digest: `sha256:${'b'.repeat(64)}`,
            platform: {
              architecture: 'amd64',
              os: 'linux',
            },
          },
          {
            mediaType: 'application/vnd.oci.image.manifest.v1+json',
            digest: `sha256:${'c'.repeat(64)}`,
            platform: {
              architecture: 'arm64',
              os: 'linux',
            },
          },
        ],
      }),
    });

    if (parsed.metadataPath !== 'artifacts/release-governance/ghcr-image-manifest-publish.json') {
      throw new Error('GHCR manifest publish fallback must preserve the governed metadata path');
    }

    if (publishPlan.targetImageRef !== 'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-governance-fallback') {
      throw new Error('GHCR manifest publish fallback must preserve the governed target image ref');
    }

    if (publishPlan.sourceImageRefs.length !== 2) {
      throw new Error('GHCR manifest publish fallback must preserve both governed source image refs');
    }

    if (metadata.type !== 'sdkwork-ghcr-image-manifest-publish' || metadata.platformCount !== 2) {
      throw new Error('GHCR manifest publish fallback must preserve governed metadata semantics');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-slo-governance-contracts-test') {
    await assertSloGovernanceContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-slo-governance-test') {
    await assertSloGovernanceContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-slo-governance') {
    const currentResult = collectSloGovernanceResult();
    const result = currentResult.reason === 'evidence-missing'
      ? materializeLiveSloGovernanceEvidence({ env })
      : currentResult;
    const payload = result.summary
      ? {
        ok: result.summary.ok,
        blocked: result.summary.blocked,
        reason: result.summary.reason,
        summary: result.summary,
      }
      : result;
    return {
      id: plan.id,
      ok: result.ok,
      status: result.ok ? 0 : 1,
      stdout: `${JSON.stringify(payload, null, 2)}\n`,
      stderr: '',
      errorMessage: result.ok ? '' : String(result.errorMessage ?? ''),
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-window-snapshot-test') {
    await assertReleaseWindowSnapshotContracts({
      repoRoot: rootDir,
    });
    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-window-snapshot-materializer-test') {
    const artifact = createSyntheticReleaseWindowSnapshotArtifact();
    const validation = validateReleaseWindowSnapshotArtifact(artifact);

    if (validation.sourceKind !== 'release-window-snapshot-fixture') {
      throw new Error('release window snapshot materializer fallback must preserve the governed source kind');
    }

    if (artifact.snapshot.latestReleaseTag !== 'release-2026-03-28-8') {
      throw new Error('release window snapshot materializer fallback must preserve the governed release tag');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-sync-audit-materializer-test') {
    const artifact = createSyntheticReleaseSyncAuditArtifact();
    validateReleaseSyncAuditArtifact(artifact);

    if (String(artifact.source?.kind ?? '').trim() !== 'release-sync-audit-fixture') {
      throw new Error('release sync audit materializer fallback must preserve the governed source kind');
    }

    if (artifact.summary.releasable !== true) {
      throw new Error('release sync audit materializer fallback must preserve the releasable summary');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-governance-bundle-test') {
    const specs = listReleaseGovernanceBundleArtifactSpecs({
      repoRoot: rootDir,
    });
    if (!Array.isArray(specs) || specs.length !== 7) {
      throw new Error('release governance bundle specs must expose seven governed artifacts');
    }

    const manifest = createReleaseGovernanceBundleManifest({
      generatedAt: '2026-04-15T00:00:00.000Z',
      artifacts: specs.map((spec) => ({
        id: spec.id,
        relativePath: spec.relativePath,
        sourceRelativePath: spec.relativePath,
      })),
    });
    if (manifest.bundleEntryCount !== specs.length) {
      throw new Error('release governance bundle manifest must track every governed artifact');
    }
    if (!/restore-release-governance-latest\.mjs/.test(String(manifest.restore?.command ?? ''))) {
      throw new Error('release governance bundle manifest must expose the restore operator command');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'restore-release-governance-latest-test') {
    const specs = listReleaseGovernanceLatestArtifactSpecs();
    if (!Array.isArray(specs) || specs.length !== 7) {
      throw new Error('restore release governance helper must expose seven governed artifact specs');
    }

    const uniquePortablePaths = new Set(specs.map((spec) => spec.portableRelativePath));
    if (uniquePortablePaths.size !== specs.length) {
      throw new Error('restore release governance helper must expose unique artifact restore paths');
    }

    if (specs.some((spec) => !spec.id || !spec.optionKey || !spec.fileName)) {
      throw new Error('restore release governance helper specs must remain fully described');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-telemetry-export-test') {
    const exportBundle = createSyntheticReleaseTelemetryExportBundle();
    validateReleaseTelemetryExportShape({
      exportBundle,
    });

    if (String(exportBundle.source?.kind ?? '').trim() !== 'observability-control-plane') {
      throw new Error('release telemetry export fallback must keep the governed source kind');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-telemetry-snapshot-test') {
    const snapshot = createSyntheticReleaseTelemetrySnapshot();
    const validation = validateReleaseTelemetrySnapshotShape({
      snapshot,
    });

    if (validation.snapshotId !== 'release-telemetry-snapshot-v1') {
      throw new Error('release telemetry snapshot fallback must preserve the governed snapshot id');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-slo-evidence-materializer-test') {
    const evidence = deriveSloGovernanceEvidenceFromReleaseTelemetrySnapshot({
      snapshot: createSyntheticReleaseTelemetrySnapshot(),
    });
    const validation = validateSloGovernanceEvidenceShape({
      evidence,
    });

    if (validation.baselineId !== 'release-slo-governance-baseline-2026-04-08') {
      throw new Error('release slo evidence fallback must preserve the governed baseline id');
    }

    return {
      id: plan.id,
      ok: true,
      status: 0,
      stdout: '',
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-window-snapshot') {
    const hasGovernedWindowInput = [
      env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_PATH,
      env.SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON,
    ].some((value) => String(value ?? '').trim().length > 0);

    const replayResult = hasGovernedWindowInput
      ? collectReleaseWindowSnapshotResult({
        env,
        spawnSyncImpl: fallbackSpawnSyncImpl,
      })
      : existsSync(defaultReleaseWindowSnapshotPath)
        ? collectReleaseWindowSnapshotResult({
        snapshotPath: defaultReleaseWindowSnapshotPath,
        env: {},
      })
        : collectReleaseWindowSnapshotResult({
          env,
          spawnSyncImpl: fallbackSpawnSyncImpl,
        });
    return {
      id: plan.id,
      ok: replayResult.ok,
      status: replayResult.ok ? 0 : 1,
      stdout: `${JSON.stringify(replayResult, null, 2)}\n`,
      stderr: '',
      errorMessage: replayResult.ok ? '' : String(replayResult.errorMessage ?? ''),
      mode: 'fallback',
    };
  }

  if (plan.id === 'release-sync-audit') {
    const hasGovernedSyncInput = [
      env.SDKWORK_RELEASE_SYNC_AUDIT_PATH,
      env.SDKWORK_RELEASE_SYNC_AUDIT_JSON,
    ].some((value) => String(value ?? '').trim().length > 0);

    const replaySummary = hasGovernedSyncInput
      ? auditReleaseSyncRepositories({
        env,
        spawnSyncImpl: fallbackSpawnSyncImpl,
      })
        : existsSync(defaultReleaseSyncAuditPath)
        ? auditReleaseSyncRepositories({
        auditPath: defaultReleaseSyncAuditPath,
        env: {},
        spawnSyncImpl: fallbackSpawnSyncImpl,
      })
        : auditReleaseSyncRepositories({
          env,
          spawnSyncImpl: fallbackSpawnSyncImpl,
        });
    return {
      id: plan.id,
      ok: replaySummary.releasable,
      status: replaySummary.releasable ? 0 : 1,
      stdout: `${JSON.stringify(replaySummary, null, 2)}\n`,
      stderr: '',
      errorMessage: '',
      mode: 'fallback',
    };
  }

  return null;
}

export async function runReleaseGovernanceCheckPlan({
  plan,
  env = process.env,
  spawnSyncImpl = spawnSync,
  fallbackSpawnSyncImpl = spawnSync,
} = {}) {
  const runner = resolveNodeRunner({
    nodeExecutable: plan.command,
  });
  const childEnv = resolveReleaseGovernanceChildEnv({
    plan,
    env,
  });
  const result = spawnSyncImpl(runner.command, plan.args, {
    cwd: rootDir,
    encoding: 'utf8',
    env: childEnv,
    shell: runner.shell,
    stdio: 'pipe',
  });

  const stdout = String(result.stdout ?? '');
  const stderr = String(result.stderr ?? '');
  const errorMessage = result.error ? String(result.error.message ?? '') : '';
  const ok = !result.error && (result.status ?? 1) === 0;

  if (/(eperm|eacces)/i.test(errorMessage)) {
    const fallbackResult = await runFallbackReleaseGovernanceCheck({
      plan,
      env,
      fallbackSpawnSyncImpl,
    });
    if (fallbackResult) {
      return {
        ...fallbackResult,
        command: runner.command,
        args: [...plan.args],
        shell: runner.shell,
      };
    }
  }

  return {
    id: plan.id,
    command: runner.command,
    args: [...plan.args],
    shell: runner.shell,
    ok,
    status: result.status ?? (ok ? 0 : 1),
    stdout,
    stderr,
    errorMessage,
    mode: 'spawn',
  };
}

export async function runReleaseGovernanceChecks({
  plans,
  profile = 'release',
  env = process.env,
  spawnSyncImpl = spawnSync,
  fallbackSpawnSyncImpl = spawnSync,
} = {}) {
  const effectivePlans = plans ?? listReleaseGovernanceCheckPlans({ profile });
  const results = [];
  for (const plan of effectivePlans) {
    results.push(await runReleaseGovernanceCheckPlan({
      plan,
      env,
      spawnSyncImpl,
      fallbackSpawnSyncImpl,
    }));
  }

  const summary = summarizeReleaseGovernanceResults(results);

  return {
    ok: results.every((result) => result.ok === true),
    blocked: summary.blocked,
    passingIds: summary.passingIds,
    blockedIds: summary.blockedIds,
    failingIds: summary.failingIds,
    results,
  };
}

export function parseArgs(argv = process.argv.slice(2)) {
  let format = 'text';
  let profile = 'release';

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--format') {
      format = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    if (token === '--profile') {
      profile = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  assertSupportedReleaseCliFormat(format);

  assertSupportedReleaseGovernanceProfile(profile);

  return {
    format,
    profile,
  };
}

function printTextReport(summary) {
  for (const result of summary.results) {
    const state = result.ok
      ? 'PASS'
      : (summary.blockedIds ?? []).includes(result.id) ? 'BLOCK' : 'FAIL';
    const commandLine = [result.command, ...result.args].join(' ');
    console.error(`[release-governance] ${state} ${result.id}: ${commandLine}`);
    if (!result.ok) {
      if (result.errorMessage) {
        console.error(`  error: ${truncateText(result.errorMessage, 1000)}`);
      }
      if (result.stderr.trim()) {
        console.error(`  stderr: ${truncateText(result.stderr, 1000)}`);
      }
      if (result.stdout.trim()) {
        console.error(`  stdout: ${truncateText(result.stdout, 1000)}`);
      }
    }
  }
}

function main() {
  const { format, profile } = parseArgs();
  return runReleaseGovernanceChecks({ profile }).then((summary) => {
    if (format === 'json') {
      console.log(JSON.stringify(summary, null, 2));
    } else {
      printTextReport(summary);
    }

    if (!summary.ok) {
      process.exit(1);
    }
  });
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  });
}
