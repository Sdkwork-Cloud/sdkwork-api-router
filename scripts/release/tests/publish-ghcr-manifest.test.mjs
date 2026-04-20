import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('GHCR manifest publish plan derives canonical target and source image refs', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-manifest.mjs'),
    ).href,
  );

  const plan = module.createGhcrManifestPublishPlan({
    releaseTag: 'release-2026-04-19',
    env: {
      GITHUB_REPOSITORY_OWNER: 'Sdkwork-Cloud',
    },
  });

  assert.equal(plan.imageRepository, 'ghcr.io/sdkwork-cloud/sdkwork-api-router');
  assert.equal(plan.targetImageTag, 'release-2026-04-19');
  assert.equal(plan.targetImageRef, 'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19');
  assert.deepEqual(
    plan.sourceImageRefs,
    [
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-x64',
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-arm64',
    ],
  );
  assert.deepEqual(
    plan.createArgs,
    [
      'buildx',
      'imagetools',
      'create',
      '-t',
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19',
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-x64',
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-arm64',
    ],
  );
});

test('GHCR manifest publish CLI parser accepts metadata path and repeated source refs', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-manifest.mjs'),
    ).href,
  );

  const parsed = module.parseArgs([
    '--release-tag',
    'release-2026-04-19',
    '--metadata-path',
    'artifacts/release-governance/ghcr-image-manifest-publish.json',
    '--source-image-ref',
    'ghcr.io/example/sdkwork-api-router:release-2026-04-19-linux-x64',
    '--source-image-ref',
    'ghcr.io/example/sdkwork-api-router:release-2026-04-19-linux-arm64',
  ]);

  assert.equal(parsed.releaseTag, 'release-2026-04-19');
  assert.equal(parsed.metadataPath, 'artifacts/release-governance/ghcr-image-manifest-publish.json');
  assert.deepEqual(
    parsed.sourceImageRefs,
    [
      'ghcr.io/example/sdkwork-api-router:release-2026-04-19-linux-x64',
      'ghcr.io/example/sdkwork-api-router:release-2026-04-19-linux-arm64',
    ],
  );
});

test('GHCR manifest publisher writes governed metadata from docker create and inspect commands', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-manifest.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-ghcr-manifest-'));
  try {
    const metadataPath = path.join(
      fixtureRoot,
      'artifacts',
      'release-governance',
      'ghcr-image-manifest-publish.json',
    );

    const summaryText = [
      'Name: ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19',
      'MediaType: application/vnd.oci.image.index.v1+json',
      `Digest: sha256:${'a'.repeat(64)}`,
    ].join('\n');
    const rawManifestText = JSON.stringify({
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
    });

    const calls = [];
    const metadata = module.publishGhcrManifest({
      repoRoot: fixtureRoot,
      releaseTag: 'release-2026-04-19',
      metadataPath,
      env: {
        GITHUB_REPOSITORY_OWNER: 'Sdkwork-Cloud',
      },
      spawnSyncImpl(command, args) {
        calls.push([command, args]);

        if (args.includes('create')) {
          return {
            status: 0,
            stdout: 'created',
            stderr: '',
          };
        }

        if (args.includes('--raw')) {
          return {
            status: 0,
            stdout: rawManifestText,
            stderr: '',
          };
        }

        return {
          status: 0,
          stdout: summaryText,
          stderr: '',
        };
      },
    });

    assert.equal(calls.length, 3);
    assert.equal(metadata.targetImageRef, 'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19');
    assert.equal(metadata.digest, `sha256:${'a'.repeat(64)}`);
    assert.equal(metadata.manifestMediaType, 'application/vnd.oci.image.index.v1+json');
    assert.equal(metadata.platformCount, 2);

    const writtenMetadata = JSON.parse(readFileSync(metadataPath, 'utf8'));
    assert.equal(writtenMetadata.type, 'sdkwork-ghcr-image-manifest-publish');
    assert.equal(writtenMetadata.targetImageRef, metadata.targetImageRef);
    assert.equal(writtenMetadata.digest, metadata.digest);
    assert.deepEqual(
      writtenMetadata.sourceImageRefs,
      [
        'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-x64',
        'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-arm64',
      ],
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
