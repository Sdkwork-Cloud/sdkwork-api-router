import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-workflow-publish-catalog.mjs'),
    ).href,
  );
}

test('release workflow publish catalog exposes the governed official asset and metadata paths', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listReleaseWorkflowNativeOfficialAssetPaths, 'function');
  assert.equal(typeof module.listReleaseWorkflowPublishOfficialAssetPaths, 'function');
  assert.equal(typeof module.createOrderedWorkflowLiteralPattern, 'function');

  assert.deepEqual(
    module.listReleaseWorkflowNativeOfficialAssetPaths(),
    [
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.tar.gz',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.tar.gz.sha256.txt',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.manifest.json',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*',
    ],
  );
  assert.deepEqual(
    module.listReleaseWorkflowPublishOfficialAssetPaths(),
    [
      'artifacts/release/**/sdkwork-api-router-product-server-*.tar.gz',
      'artifacts/release/**/sdkwork-api-router-product-server-*.tar.gz.sha256.txt',
      'artifacts/release/**/sdkwork-api-router-product-server-*.manifest.json',
      'artifacts/release/**/desktop/portal/sdkwork-router-portal-desktop-*',
      'artifacts/release/release-catalog.json',
    ],
  );
  assert.deepEqual(
    module.RELEASE_WORKFLOW_GOVERNANCE_BUNDLE_ARTIFACT,
    {
      name: 'release-governance-bundle',
      path: 'artifacts/release-governance-bundle/**/*',
    },
  );
  assert.deepEqual(
    module.RELEASE_WORKFLOW_GHCR_IMAGE_PUBLISH_METADATA,
    {
      artifactNameTemplate: 'release-governance-ghcr-image-publish-${{ matrix.platform }}-${{ matrix.arch }}',
      pathTemplate: 'artifacts/release-governance/ghcr-image-publish-${{ matrix.platform }}-${{ matrix.arch }}.json',
    },
  );
  assert.deepEqual(
    module.RELEASE_WORKFLOW_GHCR_MANIFEST_PUBLISH_METADATA,
    {
      artifactName: 'release-governance-ghcr-image-manifest-publish',
      path: 'artifacts/release-governance/ghcr-image-manifest-publish.json',
    },
  );
});

test('release workflow publish catalog creates ordered literal regex patterns for workflow assertions', async () => {
  const module = await loadModule();

  const pattern = module.createOrderedWorkflowLiteralPattern([
    'Upload official release assets',
    ...module.listReleaseWorkflowNativeOfficialAssetPaths(),
  ]);

  assert.match(
    `
      - name: Upload official release assets
        with:
          path: |
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.tar.gz.sha256.txt
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-\${{ matrix.platform }}-\${{ matrix.arch }}.manifest.json
            artifacts/release/native/\${{ matrix.platform }}/\${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*
    `,
    pattern,
  );
});

test('release workflow publish catalog exposes strict asset-path and metadata lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findReleaseWorkflowNativeOfficialAssetPath, 'function');
  assert.equal(typeof module.listReleaseWorkflowNativeOfficialAssetPathsByPaths, 'function');
  assert.equal(typeof module.findReleaseWorkflowPublishOfficialAssetPath, 'function');
  assert.equal(typeof module.listReleaseWorkflowPublishOfficialAssetPathsByPaths, 'function');
  assert.equal(typeof module.findReleaseWorkflowPublishMetadataArtifact, 'function');
  assert.equal(typeof module.listReleaseWorkflowPublishMetadataArtifactsByIds, 'function');

  assert.equal(
    module.findReleaseWorkflowNativeOfficialAssetPath(
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*',
    ),
    'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*',
  );
  assert.deepEqual(
    module.listReleaseWorkflowNativeOfficialAssetPathsByPaths([
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.tar.gz',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*',
    ]),
    [
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/bundles/sdkwork-api-router-product-server-${{ matrix.platform }}-${{ matrix.arch }}.tar.gz',
      'artifacts/release/native/${{ matrix.platform }}/${{ matrix.arch }}/desktop/portal/sdkwork-router-portal-desktop-*',
    ],
  );

  assert.equal(
    module.findReleaseWorkflowPublishOfficialAssetPath('artifacts/release/release-catalog.json'),
    'artifacts/release/release-catalog.json',
  );
  assert.deepEqual(
    module.listReleaseWorkflowPublishOfficialAssetPathsByPaths([
      'artifacts/release/**/sdkwork-api-router-product-server-*.manifest.json',
      'artifacts/release/release-catalog.json',
    ]),
    [
      'artifacts/release/**/sdkwork-api-router-product-server-*.manifest.json',
      'artifacts/release/release-catalog.json',
    ],
  );

  const governanceBundleArtifact = module.findReleaseWorkflowPublishMetadataArtifact('governance-bundle');
  assert.deepEqual(governanceBundleArtifact, {
    id: 'governance-bundle',
    name: 'release-governance-bundle',
    path: 'artifacts/release-governance-bundle/**/*',
  });

  governanceBundleArtifact.name = 'mutated-locally';
  assert.equal(
    module.findReleaseWorkflowPublishMetadataArtifact('governance-bundle').name,
    'release-governance-bundle',
  );

  assert.deepEqual(
    module.listReleaseWorkflowPublishMetadataArtifactsByIds([
      'ghcr-image-publish',
      'ghcr-manifest-publish',
    ]).map(({ id }) => id),
    [
      'ghcr-image-publish',
      'ghcr-manifest-publish',
    ],
  );

  assert.throws(
    () => module.findReleaseWorkflowNativeOfficialAssetPath('artifacts/release/native/missing-asset'),
    /missing release workflow native official asset path.*missing-asset/i,
  );
  assert.throws(
    () => module.findReleaseWorkflowPublishOfficialAssetPath('artifacts/release/missing-publish-asset'),
    /missing release workflow publish official asset path.*missing-publish-asset/i,
  );
  assert.throws(
    () => module.findReleaseWorkflowPublishMetadataArtifact('missing-publish-metadata-artifact'),
    /missing release workflow publish metadata artifact.*missing-publish-metadata-artifact/i,
  );
});

test('release workflow contracts and workflow tests consume publish-path governance through the step contract catalog', () => {
  assert.match(
    read('scripts/release/release-workflow-step-contract-catalog.mjs'),
    /release-workflow-publish-catalog\.mjs/,
  );
  assert.match(
    read('scripts/release/release-workflow-contracts.mjs'),
    /release-workflow-step-contract-catalog\.mjs/,
  );
  assert.match(
    read('scripts/release/tests/release-workflow.test.mjs'),
    /release-workflow-step-contract-catalog\.mjs/,
  );
  assert.doesNotMatch(
    read('scripts/release/release-workflow-contracts.mjs'),
    /release-workflow-publish-catalog\.mjs/,
  );
  assert.doesNotMatch(
    read('scripts/release/tests/release-workflow.test.mjs'),
    /release-workflow-publish-catalog\.mjs/,
  );
});
