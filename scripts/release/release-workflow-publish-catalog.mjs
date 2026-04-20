import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';
import {
  findNativeDesktopReleaseProductSpecByAppId,
  findNativeReleaseProductSpec,
} from './native-release-product-catalog.mjs';

const RELEASE_WORKFLOW_MATRIX_PLATFORM = '${{ matrix.platform }}';
const RELEASE_WORKFLOW_MATRIX_ARCH = '${{ matrix.arch }}';
const productServerReleaseProductSpec = findNativeReleaseProductSpec('product-server');
const portalDesktopReleaseProductSpec = findNativeDesktopReleaseProductSpecByAppId('portal');
const productServerBaseNameTemplate = `${productServerReleaseProductSpec.baseNamePrefix}-${RELEASE_WORKFLOW_MATRIX_PLATFORM}-${RELEASE_WORKFLOW_MATRIX_ARCH}`;
const portalDesktopBaseNameGlob = `${portalDesktopReleaseProductSpec.baseNamePrefix}-*`;

const releaseWorkflowNativeOfficialAssetPathCatalog = createStrictKeyedCatalog({
  entries: [
    `artifacts/release/native/${RELEASE_WORKFLOW_MATRIX_PLATFORM}/${RELEASE_WORKFLOW_MATRIX_ARCH}/${productServerReleaseProductSpec.outputPathSegments.join('/')}/${productServerBaseNameTemplate}${productServerReleaseProductSpec.archiveFileExtension}`,
    `artifacts/release/native/${RELEASE_WORKFLOW_MATRIX_PLATFORM}/${RELEASE_WORKFLOW_MATRIX_ARCH}/${productServerReleaseProductSpec.outputPathSegments.join('/')}/${productServerBaseNameTemplate}${productServerReleaseProductSpec.archiveFileExtension}.sha256.txt`,
    `artifacts/release/native/${RELEASE_WORKFLOW_MATRIX_PLATFORM}/${RELEASE_WORKFLOW_MATRIX_ARCH}/${productServerReleaseProductSpec.outputPathSegments.join('/')}/${productServerBaseNameTemplate}.manifest.json`,
    `artifacts/release/native/${RELEASE_WORKFLOW_MATRIX_PLATFORM}/${RELEASE_WORKFLOW_MATRIX_ARCH}/${portalDesktopReleaseProductSpec.outputPathSegments.join('/')}/${portalDesktopBaseNameGlob}`,
  ],
  getKey: (assetPath) => assetPath,
  duplicateKeyMessagePrefix: 'duplicate release workflow native official asset path',
  missingKeyMessagePrefix: 'missing release workflow native official asset path',
});

const releaseWorkflowPublishOfficialAssetPathCatalog = createStrictKeyedCatalog({
  entries: [
    `artifacts/release/**/${productServerReleaseProductSpec.baseNamePrefix}-*${productServerReleaseProductSpec.archiveFileExtension}`,
    `artifacts/release/**/${productServerReleaseProductSpec.baseNamePrefix}-*${productServerReleaseProductSpec.archiveFileExtension}.sha256.txt`,
    `artifacts/release/**/${productServerReleaseProductSpec.baseNamePrefix}-*.manifest.json`,
    `artifacts/release/**/${portalDesktopReleaseProductSpec.outputPathSegments.join('/')}/${portalDesktopBaseNameGlob}`,
    'artifacts/release/release-catalog.json',
  ],
  getKey: (assetPath) => assetPath,
  duplicateKeyMessagePrefix: 'duplicate release workflow publish official asset path',
  missingKeyMessagePrefix: 'missing release workflow publish official asset path',
});

function clonePublishMetadataArtifact(artifact) {
  return {
    ...artifact,
  };
}

const releaseWorkflowPublishMetadataArtifactCatalog = createStrictKeyedCatalog({
  entries: [
    {
      id: 'governance-bundle',
      name: 'release-governance-bundle',
      path: 'artifacts/release-governance-bundle/**/*',
    },
    {
      id: 'ghcr-image-publish',
      artifactNameTemplate: 'release-governance-ghcr-image-publish-${{ matrix.platform }}-${{ matrix.arch }}',
      pathTemplate: 'artifacts/release-governance/ghcr-image-publish-${{ matrix.platform }}-${{ matrix.arch }}.json',
    },
    {
      id: 'ghcr-manifest-publish',
      artifactName: 'release-governance-ghcr-image-manifest-publish',
      path: 'artifacts/release-governance/ghcr-image-manifest-publish.json',
    },
  ],
  getKey: ({ id }) => id,
  clone: clonePublishMetadataArtifact,
  duplicateKeyMessagePrefix: 'duplicate release workflow publish metadata artifact id',
  missingKeyMessagePrefix: 'missing release workflow publish metadata artifact',
});

export const RELEASE_WORKFLOW_NATIVE_OFFICIAL_ASSET_PATHS = releaseWorkflowNativeOfficialAssetPathCatalog.list();

export const RELEASE_WORKFLOW_PUBLISH_OFFICIAL_ASSET_PATHS = releaseWorkflowPublishOfficialAssetPathCatalog.list();

const governanceBundleArtifact = releaseWorkflowPublishMetadataArtifactCatalog.find('governance-bundle');
const ghcrImagePublishMetadataArtifact = releaseWorkflowPublishMetadataArtifactCatalog.find('ghcr-image-publish');
const ghcrManifestPublishMetadataArtifact = releaseWorkflowPublishMetadataArtifactCatalog.find('ghcr-manifest-publish');

export const RELEASE_WORKFLOW_GOVERNANCE_BUNDLE_ARTIFACT = {
  name: governanceBundleArtifact.name,
  path: governanceBundleArtifact.path,
};

export const RELEASE_WORKFLOW_GHCR_IMAGE_PUBLISH_METADATA = {
  artifactNameTemplate: ghcrImagePublishMetadataArtifact.artifactNameTemplate,
  pathTemplate: ghcrImagePublishMetadataArtifact.pathTemplate,
};

export const RELEASE_WORKFLOW_GHCR_MANIFEST_PUBLISH_METADATA = {
  artifactName: ghcrManifestPublishMetadataArtifact.artifactName,
  path: ghcrManifestPublishMetadataArtifact.path,
};

export function listReleaseWorkflowNativeOfficialAssetPaths() {
  return releaseWorkflowNativeOfficialAssetPathCatalog.list();
}

export function findReleaseWorkflowNativeOfficialAssetPath(assetPath) {
  return releaseWorkflowNativeOfficialAssetPathCatalog.find(assetPath);
}

export function listReleaseWorkflowNativeOfficialAssetPathsByPaths(assetPaths = []) {
  return releaseWorkflowNativeOfficialAssetPathCatalog.listByKeys(assetPaths);
}

export function listReleaseWorkflowPublishOfficialAssetPaths() {
  return releaseWorkflowPublishOfficialAssetPathCatalog.list();
}

export function findReleaseWorkflowPublishOfficialAssetPath(assetPath) {
  return releaseWorkflowPublishOfficialAssetPathCatalog.find(assetPath);
}

export function listReleaseWorkflowPublishOfficialAssetPathsByPaths(assetPaths = []) {
  return releaseWorkflowPublishOfficialAssetPathCatalog.listByKeys(assetPaths);
}

export function listReleaseWorkflowPublishMetadataArtifacts() {
  return releaseWorkflowPublishMetadataArtifactCatalog.list();
}

export function findReleaseWorkflowPublishMetadataArtifact(artifactId) {
  return releaseWorkflowPublishMetadataArtifactCatalog.find(artifactId);
}

export function listReleaseWorkflowPublishMetadataArtifactsByIds(artifactIds = []) {
  return releaseWorkflowPublishMetadataArtifactCatalog.listByKeys(artifactIds);
}

function escapeRegexLiteral(value) {
  return String(value).replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
}

export function createOrderedWorkflowLiteralPattern(fragments, flags = '') {
  return new RegExp(
    fragments
      .map((fragment) => escapeRegexLiteral(fragment))
      .join('[\\s\\S]*?'),
    flags,
  );
}
