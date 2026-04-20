import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('GHCR image publish plan derives canonical bundle, image ref, and metadata path', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-image.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-ghcr-image-plan-'));
  try {
    const plan = module.createGhcrImagePublishPlan({
      repoRoot: fixtureRoot,
      releaseTag: 'release-2026-04-19',
      platform: 'linux',
      arch: 'arm64',
      metadataPath: 'artifacts/release-governance/ghcr-image-publish-linux-arm64.json',
      env: {
        GITHUB_REPOSITORY_OWNER: 'Sdkwork-Cloud',
      },
    });

    assert.equal(plan.releaseTag, 'release-2026-04-19');
    assert.equal(plan.platform, 'linux');
    assert.equal(plan.arch, 'arm64');
    assert.equal(
      plan.bundlePath,
      path.join(
        fixtureRoot,
        'artifacts',
        'release',
        'native',
        'linux',
        'arm64',
        'bundles',
        'sdkwork-api-router-product-server-linux-arm64.tar.gz',
      ),
    );
    assert.equal(plan.imageRepository, 'ghcr.io/sdkwork-cloud/sdkwork-api-router');
    assert.equal(plan.imageTag, 'release-2026-04-19-linux-arm64');
    assert.equal(
      plan.imageRef,
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-arm64',
    );
    assert.equal(
      plan.metadataPath,
      path.join(
        fixtureRoot,
        'artifacts',
        'release-governance',
        'ghcr-image-publish-linux-arm64.json',
      ),
    );
    assert.deepEqual(
      plan.buildArgs,
      [
        'build',
        '-f',
        'deploy/docker/Dockerfile',
        '-t',
        'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-arm64',
        '.',
      ],
    );
    assert.deepEqual(
      plan.pushArgs,
      [
        'push',
        'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-arm64',
      ],
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('GHCR image publish CLI parser accepts governed metadata path and linux bundle inputs', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-image.mjs'),
    ).href,
  );

  const parsed = module.parseArgs([
    '--release-tag',
    'release-2026-04-19',
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--bundle-path',
    'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
    '--metadata-path',
    'artifacts/release-governance/ghcr-image-publish-linux-x64.json',
  ]);

  assert.equal(parsed.releaseTag, 'release-2026-04-19');
  assert.equal(parsed.platform, 'linux');
  assert.equal(parsed.arch, 'x64');
  assert.equal(
    parsed.bundlePath,
    'artifacts/release/native/linux/x64/bundles/sdkwork-api-router-product-server-linux-x64.tar.gz',
  );
  assert.equal(
    parsed.metadataPath,
    'artifacts/release-governance/ghcr-image-publish-linux-x64.json',
  );
});

test('GHCR image publisher extracts the bundle, builds and pushes the image, and writes governed metadata', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'publish-ghcr-image.mjs'),
    ).href,
  );

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-ghcr-image-publish-'));
  try {
    const bundlePath = path.join(
      fixtureRoot,
      'artifacts',
      'release',
      'native',
      'linux',
      'x64',
      'bundles',
      'sdkwork-api-router-product-server-linux-x64.tar.gz',
    );
    mkdirSync(path.dirname(bundlePath), { recursive: true });
    writeFileSync(bundlePath, 'synthetic bundle\n', 'utf8');

    const metadataPath = path.join(
      fixtureRoot,
      'artifacts',
      'release-governance',
      'ghcr-image-publish-linux-x64.json',
    );

    const calls = [];
    const digest = `sha256:${'d'.repeat(64)}`;
    const metadata = module.publishGhcrImage({
      repoRoot: fixtureRoot,
      releaseTag: 'release-2026-04-19',
      platform: 'linux',
      arch: 'x64',
      bundlePath,
      metadataPath,
      env: {
        GITHUB_REPOSITORY_OWNER: 'Sdkwork-Cloud',
      },
      spawnSyncImpl(command, args, options) {
        calls.push([command, args, options.cwd]);

        if (command === 'tar') {
          const extractRoot = args[3];
          const extractedBundleRoot = path.join(
            extractRoot,
            'sdkwork-api-router-product-server-linux-x64',
          );
          mkdirSync(path.join(extractedBundleRoot, 'deploy', 'docker'), { recursive: true });
          writeFileSync(
            path.join(extractedBundleRoot, 'deploy', 'docker', 'Dockerfile'),
            'FROM scratch\n',
            'utf8',
          );
          return {
            status: 0,
            stdout: '',
            stderr: '',
          };
        }

        if (command === 'docker' && args[0] === 'build') {
          return {
            status: 0,
            stdout: 'built',
            stderr: '',
          };
        }

        if (command === 'docker' && args[0] === 'push') {
          return {
            status: 0,
            stdout: `latest: digest: ${digest} size: 1234`,
            stderr: '',
          };
        }

        throw new Error(`unexpected command: ${command} ${args.join(' ')}`);
      },
    });

    assert.equal(calls.length, 3);
    assert.deepEqual(
      calls.map(([command, args]) => [command, args[0]]),
      [
        ['tar', '-xzf'],
        ['docker', 'build'],
        ['docker', 'push'],
      ],
    );
    assert.equal(
      metadata.imageRef,
      'ghcr.io/sdkwork-cloud/sdkwork-api-router:release-2026-04-19-linux-x64',
    );
    assert.equal(metadata.digest, digest);

    const writtenMetadata = JSON.parse(readFileSync(metadataPath, 'utf8'));
    assert.equal(writtenMetadata.type, 'sdkwork-ghcr-image-publish');
    assert.equal(writtenMetadata.imageRef, metadata.imageRef);
    assert.equal(writtenMetadata.digest, metadata.digest);
    assert.match(
      writtenMetadata.bundlePath,
      /artifacts\/release\/native\/linux\/x64\/bundles\/sdkwork-api-router-product-server-linux-x64\.tar\.gz$/,
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
