import assert from 'node:assert/strict';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const releaseWindowSnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-window-snapshot-latest.json',
);
const releaseSyncAuditPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-sync-audit-latest.json',
);
const releaseTelemetryExportPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-export-latest.json',
);
const releaseTelemetrySnapshotPath = path.join(
  repoRoot,
  'docs',
  'release',
  'release-telemetry-snapshot-latest.json',
);
const sloGovernanceEvidencePath = path.join(
  repoRoot,
  'docs',
  'release',
  'slo-governance-latest.json',
);
const thirdPartySbomPath = path.join(
  repoRoot,
  'docs',
  'release',
  'third-party-sbom-latest.spdx.json',
);
const thirdPartyNoticesPath = path.join(
  repoRoot,
  'docs',
  'release',
  'third-party-notices-latest.json',
);

const DIRECTLY_DERIVED_AVAILABILITY_TARGET_IDS = new Set([
  'gateway-availability',
  'admin-api-availability',
  'portal-api-availability',
]);

function createReleaseWindowSnapshotArtifactPayload() {
  return {
    version: 1,
    generatedAt: '2026-04-08T12:00:00Z',
    source: {
      kind: 'release-window-snapshot-fixture',
      provenance: 'synthetic-test',
    },
    snapshot: {
      latestReleaseTag: 'release-2026-03-28-8',
      commitsSinceLatestRelease: 16,
      workingTreeEntryCount: 627,
      hasReleaseBaseline: true,
    },
  };
}

function createReleaseSyncAuditArtifactPayload(head = 'abc123') {
  return {
    version: 1,
    generatedAt: '2026-04-08T13:00:00Z',
    source: {
      kind: 'release-sync-audit-fixture',
      provenance: 'synthetic-test',
    },
    summary: {
      releasable: true,
      reports: [
        {
          id: 'sdkwork-api-router',
          targetDir: repoRoot,
          expectedGitRoot: repoRoot,
          topLevel: repoRoot,
          remoteUrl: 'https://github.com/Sdkwork-Cloud/sdkwork-api-router.git',
          localHead: head,
          remoteHead: head,
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
  };
}

function createCurrentRepoReleaseSyncGitFallback(specOrSpecs, {
  localHead = 'fed456',
  remoteHead = localHead,
  statusText = '## main...origin/main\n',
} = {}) {
  const specs = Array.isArray(specOrSpecs) ? specOrSpecs : [specOrSpecs];
  let gitSpawnCount = 0;

  return {
    getCount() {
      return gitSpawnCount;
    },
    fallbackSpawnSyncImpl(command, args, options = {}) {
      gitSpawnCount += 1;
      assert.equal(command, process.platform === 'win32' ? 'git.exe' : 'git');
      const spec = specs.find((candidate) => candidate.targetDir === options.cwd);
      assert.ok(spec, `unexpected cwd: ${options.cwd}`);

      const key = args.join('\u0000');
      if (key === 'rev-parse\u0000--show-toplevel') {
        return {
          status: 0,
          stdout: `${spec.expectedGitRoot}\n`,
          stderr: '',
        };
      }

      if (key === 'status\u0000--short\u0000--branch') {
        return {
          status: 0,
          stdout: statusText,
          stderr: '',
        };
      }

      if (key === 'remote\u0000get-url\u0000origin') {
        return {
          status: 0,
          stdout: `${spec.expectedRemoteUrl}\n`,
          stderr: '',
        };
      }

      if (key === 'rev-parse\u0000HEAD') {
        return {
          status: 0,
          stdout: `${localHead}\n`,
          stderr: '',
        };
      }

      if (key === 'ls-remote\u0000origin\u0000main') {
        return {
          status: 0,
          stdout: `${remoteHead}\trefs/heads/main\n`,
          stderr: '',
        };
      }

      throw new Error(`unexpected live git replay: ${command} ${args.join(' ')}`);
    },
  };
}

function createTelemetrySnapshotPayload() {
  return {
    version: 1,
    snapshotId: 'release-telemetry-snapshot-v1',
    generatedAt: '2026-04-08T10:00:00Z',
    source: {
      kind: 'release-telemetry-export',
      exportKind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
      directTargetIds: [
        'admin-api-availability',
        'gateway-availability',
        'portal-api-availability',
      ],
      supplementalTargetIds: [
        'account-hold-creation-success-rate',
        'api-key-issuance-success-rate',
        'billing-event-write-success-rate',
        'gateway-fallback-success-rate',
        'gateway-non-streaming-success-rate',
        'gateway-provider-timeout-budget',
        'gateway-streaming-completion-success-rate',
        'pricing-lifecycle-synchronize-success-rate',
        'request-settlement-finalize-success-rate',
        'routing-simulation-p95-latency',
        'runtime-rollout-success-rate',
      ],
    },
    targets: {
      'gateway-availability': { ratio: 0.9997, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-non-streaming-success-rate': { ratio: 0.997, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'gateway-streaming-completion-success-rate': { ratio: 0.996, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'gateway-fallback-success-rate': { ratio: 0.985, burnRates: { '1h': 0.7, '6h': 0.4 } },
      'gateway-provider-timeout-budget': { ratio: 0.004, burnRates: { '1h': 0.5, '6h': 0.3 } },
      'admin-api-availability': { ratio: 0.9994, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'portal-api-availability': { ratio: 0.9993, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'routing-simulation-p95-latency': { value: 420, burnRates: { '1h': 0.9, '6h': 0.5 } },
      'api-key-issuance-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'runtime-rollout-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'billing-event-write-success-rate': { ratio: 0.9995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'account-hold-creation-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'request-settlement-finalize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
      'pricing-lifecycle-synchronize-success-rate': { ratio: 0.995, burnRates: { '1h': 0.8, '6h': 0.4 } },
    },
  };
}

function createPrometheusHttpCounterSamples({
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

function createTelemetryExportPayload() {
  const snapshotPayload = createTelemetrySnapshotPayload();
  const supplementalTargets = Object.fromEntries(
    Object.entries(snapshotPayload.targets).filter(
      ([targetId]) => !DIRECTLY_DERIVED_AVAILABILITY_TARGET_IDS.has(targetId),
    ),
  );

  return {
    version: 1,
    generatedAt: snapshotPayload.generatedAt,
    source: {
      kind: 'observability-control-plane',
      freshnessMinutes: 5,
      provenance: 'synthetic-test',
    },
    prometheus: {
      gateway: createPrometheusHttpCounterSamples({
        service: 'gateway-service',
        healthyCount: 9997,
        unhealthyCount: 3,
      }),
      admin: createPrometheusHttpCounterSamples({
        service: 'admin-api-service',
        healthyCount: 4997,
        unhealthyCount: 3,
      }),
      portal: createPrometheusHttpCounterSamples({
        service: 'portal-api-service',
        healthyCount: 9993,
        unhealthyCount: 7,
      }),
    },
    supplemental: {
      targets: supplementalTargets,
    },
  };
}

function createSloGovernancePayload() {
  return {
    version: 1,
    baselineId: 'release-slo-governance-baseline-2026-04-08',
    baselineDate: '2026-04-08',
    generatedAt: '2026-04-08T10:00:00Z',
    targets: createTelemetrySnapshotPayload().targets,
  };
}

function createThirdPartySbomPayload() {
  return {
    spdxVersion: 'SPDX-2.3',
    SPDXID: 'SPDXRef-DOCUMENT',
    name: 'sdkwork-api-router-third-party-sbom',
    dataLicense: 'CC0-1.0',
    documentNamespace: 'https://sdkwork.example.test/spdx/2026-04-08',
    creationInfo: {
      created: '2026-04-08T14:00:00Z',
      creators: ['Tool: sdkwork-third-party-governance'],
    },
    packages: [],
    relationships: [],
  };
}

function createThirdPartyNoticesPayload() {
  return {
    version: 1,
    generatedAt: '2026-04-08T14:00:00Z',
    packageCount: 0,
    noticeText: 'Synthetic third-party notices\n',
    packages: [],
  };
}

function cleanupGovernedReleaseArtifacts() {
  for (const artifactPath of [
    releaseWindowSnapshotPath,
    releaseSyncAuditPath,
    releaseTelemetryExportPath,
    releaseTelemetrySnapshotPath,
    sloGovernanceEvidencePath,
    thirdPartySbomPath,
    thirdPartyNoticesPath,
  ]) {
    if (existsSync(artifactPath)) {
      rmSync(artifactPath, { force: true });
    }
  }
}

async function withCleanedGovernedReleaseArtifacts(callback) {
  const originals = [
    releaseWindowSnapshotPath,
    releaseSyncAuditPath,
    releaseTelemetryExportPath,
    releaseTelemetrySnapshotPath,
    sloGovernanceEvidencePath,
    thirdPartySbomPath,
    thirdPartyNoticesPath,
  ].map((filePath) => ({
    filePath,
    hadOriginalFile: existsSync(filePath),
    originalContent: existsSync(filePath) ? readFileSync(filePath, 'utf8') : null,
  }));

  cleanupGovernedReleaseArtifacts();

  try {
    return await callback();
  } finally {
    cleanupGovernedReleaseArtifacts();
    for (const entry of originals) {
      if (entry.hadOriginalFile) {
        writeFileSync(entry.filePath, entry.originalContent, 'utf8');
      }
    }
  }
}

function writeArtifact(root, directoryName, relativePath, payload) {
  const targetPath = path.join(root, directoryName, relativePath);
  mkdirSync(path.dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
}

test('restore release governance latest materializer restores required governance artifacts from a downloaded artifact directory', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
    ).href,
  );

  assert.equal(typeof module.listReleaseGovernanceLatestArtifactSpecs, 'function');
  assert.equal(typeof module.resolveReleaseGovernanceLatestArtifactSources, 'function');
  assert.equal(typeof module.restoreReleaseGovernanceLatestArtifacts, 'function');

  const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));
  const targetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-target-'));

  writeArtifact(
    artifactRoot,
    'release-governance-window-snapshot-web',
    path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    createReleaseWindowSnapshotArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-sync-audit-web',
    path.join('docs', 'release', 'release-sync-audit-latest.json'),
    createReleaseSyncAuditArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-export-web',
    path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    createTelemetryExportPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-snapshot-web',
    path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    createTelemetrySnapshotPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-slo-evidence-web',
    path.join('docs', 'release', 'slo-governance-latest.json'),
    createSloGovernancePayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-third-party-sbom-web',
    path.join('docs', 'release', 'third-party-sbom-latest.spdx.json'),
    createThirdPartySbomPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-third-party-notices-web',
    path.join('docs', 'release', 'third-party-notices-latest.json'),
    createThirdPartyNoticesPayload(),
  );

  const result = module.restoreReleaseGovernanceLatestArtifacts({
    artifactDir: artifactRoot,
    repoRoot: targetRoot,
  });

  assert.equal(result.restored.length, 7);
  assert.equal(
    existsSync(path.join(targetRoot, 'docs', 'release', 'release-window-snapshot-latest.json')),
    true,
  );
  assert.equal(
    JSON.parse(
      readFileSync(
        path.join(targetRoot, 'docs', 'release', 'release-sync-audit-latest.json'),
        'utf8',
      ),
    ).summary.releasable,
    true,
  );
});

test('restore release governance latest materializer tolerates duplicate identical artifacts across lanes', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
    ).href,
  );

  const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));
  const targetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-target-'));
  const syncAuditPayload = createReleaseSyncAuditArtifactPayload();

  for (const directoryName of ['release-governance-sync-audit-web', 'release-governance-sync-audit-linux-x64']) {
    writeArtifact(
      artifactRoot,
      directoryName,
      path.join('docs', 'release', 'release-sync-audit-latest.json'),
      syncAuditPayload,
    );
  }

  writeArtifact(
    artifactRoot,
    'release-governance-window-snapshot-web',
    path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    createReleaseWindowSnapshotArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-export-web',
    path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    createTelemetryExportPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-snapshot-web',
    path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    createTelemetrySnapshotPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-slo-evidence-web',
    path.join('docs', 'release', 'slo-governance-latest.json'),
    createSloGovernancePayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-third-party-sbom-web',
    path.join('docs', 'release', 'third-party-sbom-latest.spdx.json'),
    createThirdPartySbomPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-third-party-notices-web',
    path.join('docs', 'release', 'third-party-notices-latest.json'),
    createThirdPartyNoticesPayload(),
  );

  const result = module.restoreReleaseGovernanceLatestArtifacts({
    artifactDir: artifactRoot,
    repoRoot: targetRoot,
  });

  assert.equal(result.restored.length, 7);
  assert.match(
    result.restored.find((item) => item.id === 'release-sync-audit')?.sourcePath ?? '',
    /release-governance-sync-audit-/,
  );
});

test('restore release governance latest materializer rejects conflicting duplicate artifacts', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
    ).href,
  );

  const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));
  const targetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-target-'));

  writeArtifact(
    artifactRoot,
    'release-governance-sync-audit-web',
    path.join('docs', 'release', 'release-sync-audit-latest.json'),
    createReleaseSyncAuditArtifactPayload('abc123'),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-sync-audit-linux-x64',
    path.join('docs', 'release', 'release-sync-audit-latest.json'),
    createReleaseSyncAuditArtifactPayload('def456'),
  );

  writeArtifact(
    artifactRoot,
    'release-governance-window-snapshot-web',
    path.join('docs', 'release', 'release-window-snapshot-latest.json'),
    createReleaseWindowSnapshotArtifactPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-export-web',
    path.join('docs', 'release', 'release-telemetry-export-latest.json'),
    createTelemetryExportPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-telemetry-snapshot-web',
    path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
    createTelemetrySnapshotPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-slo-evidence-web',
    path.join('docs', 'release', 'slo-governance-latest.json'),
    createSloGovernancePayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-third-party-sbom-web',
    path.join('docs', 'release', 'third-party-sbom-latest.spdx.json'),
    createThirdPartySbomPayload(),
  );
  writeArtifact(
    artifactRoot,
    'release-governance-third-party-notices-web',
    path.join('docs', 'release', 'third-party-notices-latest.json'),
    createThirdPartyNoticesPayload(),
  );

  assert.throws(
    () => module.restoreReleaseGovernanceLatestArtifacts({
      artifactDir: artifactRoot,
      repoRoot: targetRoot,
    }),
    /conflicting duplicate governance artifact/i,
  );
});

test('restore release governance latest materializer enables blocked-host governance replay when real latest artifacts are restored', async () => {
  await withCleanedGovernedReleaseArtifacts(async () => {
    const restoreModule = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'restore-release-governance-latest.mjs'),
      ).href,
    );
    const governanceModule = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'run-release-governance-checks.mjs'),
      ).href,
    );
    const verifyReleaseSyncModule = await import(
      pathToFileURL(
        path.join(repoRoot, 'scripts', 'release', 'verify-release-sync.mjs'),
      ).href,
    );

    const artifactRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-release-governance-restore-'));

    writeArtifact(
      artifactRoot,
      'release-governance-window-snapshot-web',
      path.join('docs', 'release', 'release-window-snapshot-latest.json'),
      createReleaseWindowSnapshotArtifactPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-sync-audit-web',
      path.join('docs', 'release', 'release-sync-audit-latest.json'),
      createReleaseSyncAuditArtifactPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-telemetry-export-web',
      path.join('docs', 'release', 'release-telemetry-export-latest.json'),
      createTelemetryExportPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-telemetry-snapshot-web',
      path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
      createTelemetrySnapshotPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-slo-evidence-web',
      path.join('docs', 'release', 'slo-governance-latest.json'),
      createSloGovernancePayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-third-party-sbom-web',
      path.join('docs', 'release', 'third-party-sbom-latest.spdx.json'),
      createThirdPartySbomPayload(),
    );
    writeArtifact(
      artifactRoot,
      'release-governance-third-party-notices-web',
      path.join('docs', 'release', 'third-party-notices-latest.json'),
      createThirdPartyNoticesPayload(),
    );

    restoreModule.restoreReleaseGovernanceLatestArtifacts({
      artifactDir: artifactRoot,
      repoRoot,
    });
    const specs = verifyReleaseSyncModule.listReleaseSyncRepositorySpecs();
    const liveGit = createCurrentRepoReleaseSyncGitFallback(specs, {
      localHead: 'fed456',
      remoteHead: 'fed456',
    });

    const summary = await governanceModule.runReleaseGovernanceChecks({
      env: {
        ...process.env,
        SDKWORK_API_ROUTER_GIT_REF: '',
        SDKWORK_CORE_GIT_REF: '',
        SDKWORK_UI_GIT_REF: '',
        SDKWORK_APPBASE_GIT_REF: '',
        SDKWORK_IM_SDK_GIT_REF: '',
      },
      spawnSyncImpl() {
        return {
          status: 1,
          stdout: '',
          stderr: '',
          error: new Error('spawnSync node EPERM'),
        };
      },
      fallbackSpawnSyncImpl: liveGit.fallbackSpawnSyncImpl,
    });

    assert.equal(summary.ok, true);
    assert.equal(summary.blocked, false);
    assert.deepEqual(summary.blockedIds, []);
    assert.equal(liveGit.getCount(), specs.length * 5);
  });
});
