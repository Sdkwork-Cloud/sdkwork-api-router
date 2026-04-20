import { createStrictContractCatalog } from '../strict-contract-catalog.mjs';

import {
  RELEASE_WORKFLOW_GHCR_IMAGE_PUBLISH_METADATA,
  RELEASE_WORKFLOW_GHCR_MANIFEST_PUBLISH_METADATA,
  RELEASE_WORKFLOW_GOVERNANCE_BUNDLE_ARTIFACT,
  createOrderedWorkflowLiteralPattern,
  listReleaseWorkflowNativeOfficialAssetPaths,
  listReleaseWorkflowPublishOfficialAssetPaths,
} from './release-workflow-publish-catalog.mjs';

function createWorkflowMatchContract(id, patternSource, message) {
  return {
    id,
    target: 'workflow',
    assertion: 'match',
    patternSource,
    message,
  };
}

function createWorkflowDoesNotMatchContract(id, patternSource, message) {
  return {
    id,
    target: 'workflow',
    assertion: 'doesNotMatch',
    patternSource,
    message,
  };
}

function createJobMatchContract(id, jobName, patternSource, message) {
  return {
    id,
    target: 'job',
    jobName,
    assertion: 'match',
    patternSource,
    message,
  };
}

const nativeOfficialAssetPaths = listReleaseWorkflowNativeOfficialAssetPaths();
const publishOfficialAssetPaths = listReleaseWorkflowPublishOfficialAssetPaths();

export const RELEASE_WORKFLOW_STEP_CONTRACTS = [
  createWorkflowMatchContract(
    'workflow-dispatch-inputs',
    String.raw`workflow_dispatch:\s*[\s\S]*?inputs:\s*[\s\S]*?release_tag:\s*[\s\S]*?description:\s*Existing release tag to publish[\s\S]*?required:\s*true[\s\S]*?type:\s*string[\s\S]*?git_ref:\s*[\s\S]*?description:\s*Git ref to build; defaults to refs\/tags\/<release_tag>[\s\S]*?required:\s*false[\s\S]*?type:\s*string`,
    'release workflow must expose workflow_dispatch inputs for a required release_tag plus an optional git_ref that defaults to the release tag ref',
  ),
  createWorkflowMatchContract(
    'release-tag-push-trigger',
    String.raw`push:\s*[\s\S]*tags:\s*[\s\S]*release-\*`,
    'release workflow must trigger from pushed release-* tags for governed publication lanes',
  ),
  createWorkflowMatchContract(
    'resolve-release-target',
    String.raw`Resolve release target[\s\S]*?if \[\[ \"\$\{GITHUB_EVENT_NAME\}\" == \"push\" \]\]; then[\s\S]*?release_tag=\"\$\{GITHUB_REF_NAME\}\"[\s\S]*?git_ref=\"\$\{GITHUB_REF\}\"[\s\S]*?else[\s\S]*?release_tag=\"\$\{\{\s*github\.event\.inputs\.release_tag\s*\}\}\"[\s\S]*?git_ref=\"\$\{\{\s*github\.event\.inputs\.git_ref\s*\}\}\"[\s\S]*?if \[\[ -z \"\$git_ref\" \]\]; then[\s\S]*?git_ref=\"refs\/tags\/\$release_tag\"`,
    'release workflow must resolve release_tag and git_ref from either a pushed release-* tag or workflow_dispatch inputs, defaulting manual git_ref back to refs/tags/<release_tag>',
  ),
  createWorkflowMatchContract(
    'permissions-baseline',
    String.raw`permissions:\s*[\s\S]*contents:\s*write[\s\S]*id-token:\s*write[\s\S]*attestations:\s*write[\s\S]*artifact-metadata:\s*write[\s\S]*packages:\s*write`,
    'release workflow must request the governed release permissions for contents, id-token, attestations, artifact metadata, and packages',
  ),
  createWorkflowMatchContract(
    'force-javascript-actions-node24',
    String.raw`permissions:\s*[\s\S]*artifact-metadata:\s*write[\s\S]*?env:\s*[\s\S]*?FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*'true'[\s\S]*?jobs:`,
    'release workflow must opt GitHub JavaScript actions into the Node 24 runtime to avoid Node 20 deprecation drift on hosted runners',
  ),
  createWorkflowMatchContract(
    'release-workflow-concurrency',
    String.raw`concurrency:\s*[\s\S]*?group:\s*release-\$\{\{\s*github\.workflow\s*\}\}-\$\{\{\s*github\.event\.inputs\.release_tag\s*\|\|\s*github\.ref_name\s*\|\|\s*github\.run_id\s*\}\}[\s\S]*?cancel-in-progress:\s*false`,
    'release workflow must serialize runs for the same release tag so retries or duplicate tag pushes cannot race and overwrite release assets',
  ),
  createWorkflowMatchContract(
    'rust-dependency-audit-gate',
    String.raw`rust-dependency-audit:[\s\S]*?runs-on:\s*ubuntu-latest[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?dtolnay\/rust-toolchain@stable[\s\S]*?Swatinem\/rust-cache@v2[\s\S]*?taiki-e\/install-action@cargo-audit[\s\S]*?node scripts\/check-rust-dependency-audit\.mjs`,
    'release workflow must execute a dedicated Rust dependency audit gate against the exact release ref before any assets are built',
  ),
  createJobMatchContract(
    'rust-dependency-audit-cache-disabled',
    'rust-dependency-audit',
    String.raw`actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?package-manager-cache:\s*false`,
    'rust dependency audit must disable setup-node package-manager auto-cache because the job does not install pnpm before actions/setup-node@v5 runs',
  ),
  createWorkflowMatchContract(
    'product-verification-materialize-external-deps',
    String.raw`product-verification:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile`,
    'release workflow product verification must materialize only referenced external release dependencies before frozen installs so workspace-linked packages resolve on GitHub runners without cloning unrelated governance-only repositories',
  ),
  createWorkflowMatchContract(
    'product-verification-strict-gate',
    String.raw`product-verification:[\s\S]*?runs-on:\s*ubuntu-latest[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}[\s\S]*?pnpm\/action-setup@v5[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?dtolnay\/rust-toolchain@stable[\s\S]*?Swatinem\/rust-cache@v2[\s\S]*?taiki-e\/install-action@cargo-audit[\s\S]*?Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?Run release product verification[\s\S]*?SDKWORK_STRICT_FRONTEND_INSTALLS:\s*'1'[\s\S]*?node scripts\/check-router-product\.mjs`,
    'release workflow must execute product verification with frozen installs and strict frontend install mode before any assets are built',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-console-lockfile',
    String.raw`console\/pnpm-lock\.yaml`,
    'release workflow should not cache the legacy console lockfile when that workspace is not part of the official release pipeline',
  ),
  createWorkflowMatchContract(
    'include-docs-lockfile',
    String.raw`docs\/pnpm-lock\.yaml`,
    'release workflow must cache the docs lockfile because the public docs site is a governed product surface during release verification',
  ),
  createWorkflowMatchContract(
    'docs-frozen-install',
    String.raw`Install product verification workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?pnpm --dir docs install --frozen-lockfile`,
    'release workflow must use an explicit frozen install for the docs workspace before building the public docs site',
  ),
  createWorkflowMatchContract(
    'docs-build-before-product-verification',
    String.raw`Build docs site[\s\S]*?pnpm --dir docs build`,
    'release workflow must build the docs site before running release product verification',
  ),
  createWorkflowMatchContract(
    'governance-release-needs',
    String.raw`governance-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit[\r\n]+\s*-\s*product-verification`,
    'governance release job must wait for prepare, Rust dependency audit, and product verification gates',
  ),
  createWorkflowMatchContract(
    'governance-release-setup-pnpm',
    String.raw`governance-release:[\s\S]*?Setup pnpm[\s\S]*?uses:\s*pnpm\/action-setup@v5`,
    'governance release job must install pnpm before actions/setup-node evaluates the repository packageManager field',
  ),
  createJobMatchContract(
    'governance-release-setup-node-pnpm-cache',
    'governance-release',
    String.raw`actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?cache:\s*pnpm[\s\S]*?cache-dependency-path:[^\S\r\n]*\|[^\S\r\n]*(?:\r?\n)+[^\S\r\n]*apps\/sdkwork-router-admin\/pnpm-lock\.yaml(?:\r?\n)+[^\S\r\n]*apps\/sdkwork-router-portal\/pnpm-lock\.yaml`,
    'governance release must use pnpm-backed setup-node caching for the governed admin and portal workspaces',
  ),
  createWorkflowMatchContract(
    'governance-release-install-workspace-dependencies',
    String.raw`governance-release:[\s\S]*?Install governance workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile`,
    'governance release job must install the governed admin and portal workspaces before third-party governance artifacts are materialized',
  ),
  createWorkflowMatchContract(
    'governance-release-sync-audit-seeded',
    String.raw`governance-release:[\s\S]*?Materialize release sync audit[\s\S]*?SDKWORK_RELEASE_SYNC_AUDIT_PATH:\s*docs\/release\/release-sync-audit-latest\.json[\s\S]*?node scripts\/release\/materialize-release-sync-audit\.mjs`,
    'governance release job must seed the sync-audit materializer from the committed governed artifact path unless an explicit JSON override is supplied',
  ),
  createWorkflowMatchContract(
    'governance-release-telemetry-export-seeded',
    String.raw`governance-release:[\s\S]*?Materialize release telemetry export[\s\S]*?SDKWORK_RELEASE_TELEMETRY_EXPORT_PATH:\s*docs\/release\/release-telemetry-export-latest\.json[\s\S]*?node scripts\/release\/materialize-release-telemetry-export\.mjs`,
    'governance release job must seed the telemetry-export materializer from the committed governed artifact path unless an explicit control-plane handoff overrides it',
  ),
  createWorkflowMatchContract(
    'governance-release-materialize-third-party-governance',
    String.raw`governance-release:[\s\S]*?Materialize third-party governance[\s\S]*?node scripts\/release\/materialize-third-party-governance\.mjs`,
    'governance release job must materialize governed third-party SBOM and notice inventory artifacts before the bundle is assembled',
  ),
  createWorkflowMatchContract(
    'governance-release-upload-third-party-artifacts',
    createOrderedWorkflowLiteralPattern([
      'governance-release:',
      'Upload third-party SBOM governance artifact',
      'actions/upload-artifact@v6',
      'release-governance-third-party-sbom',
      'docs/release/third-party-sbom-latest.spdx.json',
      'Upload third-party notices governance artifact',
      'actions/upload-artifact@v6',
      'release-governance-third-party-notices',
      'docs/release/third-party-notices-latest.json',
    ]).source,
    'governance release job must upload the third-party SBOM and notices as standalone workflow artifacts',
  ),
  createWorkflowMatchContract(
    'governance-release-evidence-attestation-subjects',
    String.raw`governance-release:[\s\S]*?Generate governance evidence attestation[\s\S]*?subject-path:\s*\|[\s\S]*?docs\/release\/release-window-snapshot-latest\.json[\s\S]*?docs\/release\/release-sync-audit-latest\.json[\s\S]*?docs\/release\/release-telemetry-export-latest\.json[\s\S]*?docs\/release\/release-telemetry-snapshot-latest\.json[\s\S]*?docs\/release\/slo-governance-latest\.json[\s\S]*?docs\/release\/third-party-sbom-latest\.spdx\.json[\s\S]*?docs\/release\/third-party-notices-latest\.json`,
    'governance release attestation must cover every governed evidence document, including third-party SBOM and notices artifacts',
  ),
  createWorkflowMatchContract(
    'governance-release-materialization-sequence',
    String.raw`governance-release:[\s\S]*?Materialize external release dependencies[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs[\s\S]*?Install governance workspace dependencies[\s\S]*?pnpm --dir apps\/sdkwork-router-admin install --frozen-lockfile[\s\S]*?pnpm --dir apps\/sdkwork-router-portal install --frozen-lockfile[\s\S]*?Materialize third-party governance[\s\S]*?node scripts\/release\/materialize-third-party-governance\.mjs[\s\S]*?Materialize release window snapshot[\s\S]*?node scripts\/release\/materialize-release-window-snapshot\.mjs[\s\S]*?Materialize release sync audit[\s\S]*?node scripts\/release\/materialize-release-sync-audit\.mjs[\s\S]*?Materialize release telemetry export[\s\S]*?node scripts\/release\/materialize-release-telemetry-export\.mjs[\s\S]*?Materialize release telemetry snapshot[\s\S]*?node scripts\/release\/materialize-release-telemetry-snapshot\.mjs[\s\S]*?Materialize SLO governance evidence[\s\S]*?node scripts\/release\/materialize-slo-governance-evidence\.mjs[\s\S]*?Materialize release governance bundle[\s\S]*?node scripts\/release\/materialize-release-governance-bundle\.mjs[\s\S]*?Run release governance gate[\s\S]*?node scripts\/release\/run-release-governance-checks\.mjs --format json`,
    'governance release job must materialize governed evidence, assemble the governance bundle, and execute the governance gate',
  ),
  createWorkflowMatchContract(
    'governance-release-upload-bundle-artifact',
    createOrderedWorkflowLiteralPattern([
      'governance-release:',
      'Upload release governance bundle artifact',
      'actions/upload-artifact@v6',
      RELEASE_WORKFLOW_GOVERNANCE_BUNDLE_ARTIFACT.name,
      RELEASE_WORKFLOW_GOVERNANCE_BUNDLE_ARTIFACT.path,
    ]).source,
    'governance release job must upload the governance bundle as a workflow artifact',
  ),
  createWorkflowMatchContract(
    'governance-release-bundle-attestation-order',
    createOrderedWorkflowLiteralPattern([
      'governance-release:',
      'Upload release governance bundle artifact',
      'Generate governance bundle attestation',
      'actions/attest-build-provenance@v3',
      RELEASE_WORKFLOW_GOVERNANCE_BUNDLE_ARTIFACT.path,
      'Generate governance evidence attestation',
    ]).source,
    'governance release job must attest the uploaded governance bundle payloads before the broader governance evidence attestation runs',
  ),
  createWorkflowMatchContract(
    'native-release-needs',
    String.raw`native-release:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*rust-dependency-audit[\r\n]+\s*-\s*product-verification[\r\n]+\s*-\s*governance-release`,
    'native release job must wait for governance release completion before official assets are built and published',
  ),
  createWorkflowMatchContract(
    'native-release-build-portal-desktop',
    String.raw`native-release:[\s\S]*?Build portal desktop release[\s\S]*?node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{\s*matrix\.target\s*\}\}`,
    'native release job must build the portal desktop product',
  ),
  createWorkflowMatchContract(
    'native-release-materialize-external-deps',
    String.raw`native-release:[\s\S]*?Materialize external release dependencies[\s\S]*?SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_SCOPE:\s*referenced[\s\S]*?node scripts\/release\/materialize-external-deps\.mjs`,
    'native release job must materialize referenced external release dependencies before downstream runtime verification and publication steps',
  ),
  createWorkflowMatchContract(
    'native-release-sign-before-packaging-and-smokes',
    String.raw`native-release:[\s\S]*?Build portal desktop release[\s\S]*?Run portal desktop signing hook[\s\S]*?node scripts\/release\/run-desktop-release-signing\.mjs --app portal --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --target \$\{\{\s*matrix\.target\s*\}\} --evidence-path artifacts\/release-governance\/desktop-release-signing-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json[\s\S]*?Collect native release assets[\s\S]*?node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --target \$\{\{\s*matrix\.target\s*\}\} --output-dir artifacts\/release[\s\S]*?Run installed native runtime smoke on Windows[\s\S]*?Run installed native runtime smoke on Unix`,
    'native release job must execute the desktop signing hook before packaging official assets and before installed-runtime smoke verifies the packaged server bundle',
  ),
  createWorkflowMatchContract(
    'native-release-signing-env',
    String.raw`native-release:[\s\S]*?Run portal desktop signing hook[\s\S]*?env:[\s\S]*?SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED:\s*\$\{\{\s*vars\.SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED\s*\|\|\s*''\s*\}\}[\s\S]*?SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK:\s*\$\{\{\s*secrets\.SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK\s*\|\|\s*vars\.SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK\s*\|\|\s*''\s*\}\}[\s\S]*?SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK:\s*\$\{\{\s*secrets\.SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK\s*\|\|\s*vars\.SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK\s*\|\|\s*''\s*\}\}[\s\S]*?SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK:\s*\$\{\{\s*secrets\.SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK\s*\|\|\s*vars\.SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK\s*\|\|\s*''\s*\}\}[\s\S]*?SDKWORK_RELEASE_DESKTOP_SIGN_HOOK:\s*\$\{\{\s*secrets\.SDKWORK_RELEASE_DESKTOP_SIGN_HOOK\s*\|\|\s*vars\.SDKWORK_RELEASE_DESKTOP_SIGN_HOOK\s*\|\|\s*''\s*\}\}`,
    'native release job must pass signing-required policy plus platform and generic desktop signing hook configuration into the signing step',
  ),
  createWorkflowMatchContract(
    'native-release-upload-desktop-signing-evidence',
    String.raw`native-release:[\s\S]*?Upload desktop signing evidence[\s\S]*?uses:\s*actions\/upload-artifact@v6[\s\S]*?name:\s*release-governance-desktop-signing-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/desktop-release-signing-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must publish desktop signing evidence for every release lane',
  ),
  createWorkflowMatchContract(
    'native-release-attest-desktop-signing-evidence',
    String.raw`native-release:[\s\S]*?Generate desktop signing evidence attestation[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/desktop-release-signing-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must attest the desktop signing evidence artifact',
  ),
  createJobMatchContract(
    'native-release-upload-windows-installed-runtime-smoke-evidence',
    'native-release',
    String.raw`Upload Windows installed runtime smoke evidence[\s\S]*?if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform == 'windows'[\s\S]*?hashFiles\(format\('artifacts\/release-governance\/windows-installed-runtime-smoke-\{0\}-\{1\}\.json', matrix\.platform, matrix\.arch\)\)\s*!=\s*''\s*\}\}[\s\S]*?uses:\s*actions\/upload-artifact@v6[\s\S]*?name:\s*release-governance-windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must upload Windows installed-runtime smoke evidence when the governed evidence artifact exists',
  ),
  createJobMatchContract(
    'native-release-attest-windows-installed-runtime-smoke-evidence',
    'native-release',
    String.raw`Generate Windows smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(!github\.event\.repository\.private \|\| vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED == 'true'\)\s*&&\s*matrix\.platform == 'windows'[\s\S]*?hashFiles\(format\('artifacts\/release-governance\/windows-installed-runtime-smoke-\{0\}-\{1\}\.json', matrix\.platform, matrix\.arch\)\)\s*!=\s*''\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/windows-installed-runtime-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must attest Windows installed-runtime smoke evidence when attestations are enabled and the evidence artifact exists',
  ),
  createJobMatchContract(
    'native-release-upload-linux-helm-render-smoke-evidence',
    'native-release',
    String.raw`Upload Linux Helm render smoke evidence[\s\S]*?if:\s*\$\{\{\s*always\(\)\s*&&\s*matrix\.platform == 'linux'[\s\S]*?hashFiles\(format\('artifacts\/release-governance\/helm-render-smoke-\{0\}-\{1\}\.json', matrix\.platform, matrix\.arch\)\)\s*!=\s*''\s*\}\}[\s\S]*?uses:\s*actions\/upload-artifact@v6[\s\S]*?name:\s*release-governance-helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}[\s\S]*?path:\s*artifacts\/release-governance\/helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must upload Linux Helm render smoke evidence when the governed evidence artifact exists',
  ),
  createJobMatchContract(
    'native-release-attest-linux-helm-render-smoke-evidence',
    'native-release',
    String.raw`Generate Linux Helm render smoke evidence attestation[\s\S]*?if:\s*\$\{\{\s*\(!github\.event\.repository\.private \|\| vars\.SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED == 'true'\)\s*&&\s*matrix\.platform == 'linux'[\s\S]*?hashFiles\(format\('artifacts\/release-governance\/helm-render-smoke-\{0\}-\{1\}\.json', matrix\.platform, matrix\.arch\)\)\s*!=\s*''\s*\}\}[\s\S]*?uses:\s*actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release-governance\/helm-render-smoke-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must attest Linux Helm render smoke evidence when attestations are enabled and the evidence artifact exists',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-admin-desktop',
    String.raw`Build admin desktop release|run-desktop-release-build\.mjs --app admin`,
    'native release job must not publish or build a standalone admin desktop product',
  ),
  createWorkflowMatchContract(
    'native-release-upload-official-assets',
    createOrderedWorkflowLiteralPattern([
      'native-release:',
      'Upload official release assets',
      'actions/upload-artifact@v6',
      'release-assets-native-${{ matrix.platform }}-${{ matrix.arch }}',
      ...nativeOfficialAssetPaths,
    ]).source,
    'native release upload step must publish only official server and portal desktop assets',
  ),
  createWorkflowMatchContract(
    'native-release-publish-linux-ghcr-image',
    String.raw`native-release:[\s\S]*?Publish Linux container image[\s\S]*?if:\s*matrix\.platform == 'linux'[\s\S]*?docker\/login-action@v3[\s\S]*?node scripts\/release\/publish-ghcr-image\.mjs --release-tag \$\{\{\s*needs\.prepare\.outputs\.release_tag\s*\}\} --platform \$\{\{\s*matrix\.platform\s*\}\} --arch \$\{\{\s*matrix\.arch\s*\}\} --bundle-path artifacts\/release\/native\/\$\{\{\s*matrix\.platform\s*\}\}\/\$\{\{\s*matrix\.arch\s*\}\}\/bundles\/sdkwork-api-router-product-server-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.tar\.gz --metadata-path artifacts\/release-governance\/ghcr-image-publish-\$\{\{\s*matrix\.platform\s*\}\}-\$\{\{\s*matrix\.arch\s*\}\}\.json`,
    'native release job must publish per-architecture Linux container images from the packaged server bundle',
  ),
  createWorkflowMatchContract(
    'native-release-upload-ghcr-image-publish-metadata',
    createOrderedWorkflowLiteralPattern([
      'native-release:',
      'Upload GHCR image publish metadata',
      'actions/upload-artifact@v6',
      RELEASE_WORKFLOW_GHCR_IMAGE_PUBLISH_METADATA.artifactNameTemplate,
      RELEASE_WORKFLOW_GHCR_IMAGE_PUBLISH_METADATA.pathTemplate,
    ]).source,
    'native release job must upload GHCR image publish metadata as workflow evidence',
  ),
  createWorkflowMatchContract(
    'native-release-attest-ghcr-image-publish-metadata',
    createOrderedWorkflowLiteralPattern([
      'native-release:',
      'Generate GHCR image publish metadata attestation',
      'actions/attest-build-provenance@v3',
      RELEASE_WORKFLOW_GHCR_IMAGE_PUBLISH_METADATA.pathTemplate,
    ]).source,
    'native release job must attest the GHCR image publish metadata artifact',
  ),
  createWorkflowMatchContract(
    'native-release-attest-official-assets',
    createOrderedWorkflowLiteralPattern([
      'native-release:',
      'Generate native release assets attestation',
      ...nativeOfficialAssetPaths,
    ]).source,
    'native release attestation must cover only the official server and portal desktop assets',
  ),
  createWorkflowMatchContract(
    'publish-needs',
    String.raw`publish:[\s\S]*?needs:\s*[\r\n]+\s*-\s*prepare[\r\n]+\s*-\s*governance-release[\r\n]+\s*-\s*native-release`,
    'publish job must wait for governance release and native release jobs',
  ),
  createWorkflowMatchContract(
    'publish-checkout-release-ref',
    String.raw`publish:[\s\S]*?Checkout release ref[\s\S]*?actions\/checkout@v5[\s\S]*?ref:\s*\$\{\{\s*needs\.prepare\.outputs\.git_ref\s*\}\}`,
    'publish job must check out the exact release ref before invoking repository-owned publish scripts',
  ),
  createWorkflowMatchContract(
    'publish-setup-node',
    String.raw`publish:[\s\S]*?actions\/setup-node@v5[\s\S]*?node-version:\s*22`,
    'publish job must pin the Node runtime before generating repository-owned release metadata',
  ),
  createJobMatchContract(
    'publish-setup-node-cache-disabled',
    'publish',
    String.raw`actions\/setup-node@v5[\s\S]*?node-version:\s*22[\s\S]*?package-manager-cache:\s*false`,
    'publish job must disable setup-node package-manager auto-cache because it does not install pnpm before setup-node evaluates the repository packageManager field',
  ),
  createWorkflowMatchContract(
    'publish-generate-release-catalog',
    String.raw`publish:[\s\S]*?Download packaged release assets[\s\S]*?path:\s*artifacts\/release[\s\S]*?Generate release catalog[\s\S]*?node scripts\/release\/materialize-release-catalog\.mjs --release-tag \$\{\{\s*needs\.prepare\.outputs\.release_tag\s*\}\} --assets-root artifacts\/release --output artifacts\/release\/release-catalog\.json`,
    'publish job must materialize a single release-catalog.json into the canonical artifacts/release tree before publishing',
  ),
  createWorkflowMatchContract(
    'publish-attest-release-catalog',
    String.raw`publish:[\s\S]*?Generate release catalog[\s\S]*?Generate release catalog attestation[\s\S]*?actions\/attest-build-provenance@v3[\s\S]*?subject-path:\s*artifacts\/release\/release-catalog\.json`,
    'publish job must generate a dedicated attestation for the release-catalog asset after it is materialized',
  ),
  createWorkflowMatchContract(
    'publish-attach-official-assets',
    createOrderedWorkflowLiteralPattern([
      'publish:',
      'Download packaged release assets',
      'actions/download-artifact@v8',
      'release-assets-native-*',
      'Publish release assets',
      'softprops/action-gh-release@v3',
      ...publishOfficialAssetPaths,
    ]).source,
    'publish job must attach the explicit official server and portal desktop asset globs plus the unified release catalog',
  ),
  createWorkflowMatchContract(
    'publish-multiarch-ghcr-manifest',
    String.raw`publish:[\s\S]*?Publish multi-arch container image manifest[\s\S]*?node scripts\/release\/publish-ghcr-manifest\.mjs --release-tag \$\{\{\s*needs\.prepare\.outputs\.release_tag\s*\}\} --metadata-path artifacts\/release-governance\/ghcr-image-manifest-publish\.json`,
    'publish job must assemble the multi-architecture GHCR image manifest for the release tag through the repository-owned publish script so the release tag manifest stays reproducible and governed',
  ),
  createWorkflowMatchContract(
    'publish-upload-ghcr-manifest-metadata',
    createOrderedWorkflowLiteralPattern([
      'publish:',
      'Upload GHCR image manifest publish metadata',
      'actions/upload-artifact@v6',
      RELEASE_WORKFLOW_GHCR_MANIFEST_PUBLISH_METADATA.artifactName,
      RELEASE_WORKFLOW_GHCR_MANIFEST_PUBLISH_METADATA.path,
    ]).source,
    'publish job must upload GHCR multi-architecture manifest publish metadata as workflow evidence',
  ),
  createWorkflowMatchContract(
    'publish-attest-ghcr-manifest-metadata',
    createOrderedWorkflowLiteralPattern([
      'publish:',
      'Generate GHCR image manifest publish metadata attestation',
      'actions/attest-build-provenance@v3',
      RELEASE_WORKFLOW_GHCR_MANIFEST_PUBLISH_METADATA.path,
    ]).source,
    'publish job must attest the GHCR multi-architecture manifest publish metadata artifact',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-raw-desktop-tree',
    String.raw`desktop\/portal\/\*\*\/*`,
    'desktop release publication must not expose raw bundle trees; only normalized official desktop asset names are allowed',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-catch-all-artifacts-release-glob',
    String.raw`files:\s*\|\s*[\s\S]*?artifacts\/release\/\*\*\/\*`,
    'publish job must not use a catch-all artifacts/release glob that could leak non-product files',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-web-release-job',
    String.raw`web-release:`,
    'web release job must not exist in the official release workflow',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-web-packaging',
    String.raw`package-release-assets\.mjs web`,
    'web release packaging must not be part of the official workflow',
  ),
  createWorkflowDoesNotMatchContract(
    'exclude-web-release-assets',
    String.raw`release-assets-web`,
    'official release workflow must not upload web release assets',
  ),
];
const releaseWorkflowStepContractCatalog = createStrictContractCatalog({
  contracts: RELEASE_WORKFLOW_STEP_CONTRACTS,
  duplicateIdMessagePrefix: 'duplicate release workflow step contract id',
  missingIdMessagePrefix: 'missing release workflow step contract',
});

export function listReleaseWorkflowStepContracts() {
  return releaseWorkflowStepContractCatalog.list();
}

export function findReleaseWorkflowStepContract(contractId) {
  return releaseWorkflowStepContractCatalog.find(contractId);
}

export function listReleaseWorkflowStepContractsByIds(contractIds = []) {
  return releaseWorkflowStepContractCatalog.listByIds(contractIds);
}
