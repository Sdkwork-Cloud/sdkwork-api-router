# Online Release

This is the canonical GitHub-hosted release runbook for SDKWork API Router.

Use this page when you are publishing an official GitHub release, preparing repository variables and secrets, wiring desktop signing, or validating the published release after the workflow finishes.

The public release contract stays intentionally narrow:

- installable products:
  - `sdkwork-api-router-product-server`
  - `sdkwork-router-portal-desktop`
- published release metadata:
  - `release-catalog.json`

Governance evidence, telemetry exports, release-window snapshots, sync audits, third-party SBOM and notice inventories, and smoke evidence remain workflow artifacts and attestations. They are not public release downloads.
The Linux server image is published separately to GHCR as `ghcr.io/<owner>/sdkwork-api-router:<release-tag>` and is not duplicated as a GitHub release attachment.
Per-architecture GHCR image publish metadata is also retained as workflow evidence at `artifacts/release-governance/ghcr-image-publish-<platform>-<arch>.json` so operators can audit the pushed digest without scraping runner logs.
The final multi-architecture GHCR manifest publish is also recorded as workflow evidence at `artifacts/release-governance/ghcr-image-manifest-publish.json`, including the release-tag image ref and manifest digest.
Treat both GHCR JSON artifacts as machine-readable operator contracts:

- `ghcr-image-publish-<platform>-<arch>.json` is emitted with `version`, `type` (`sdkwork-ghcr-image-publish`), `generatedAt`, `releaseTag`, `platform`, `arch`, `bundlePath`, `imageRepository`, `imageTag`, `imageRef`, and `digest`.
- `ghcr-image-manifest-publish.json` is emitted with `version`, `type` (`sdkwork-ghcr-image-manifest-publish`), `generatedAt`, `releaseTag`, `imageRepository`, `targetImageTag`, `targetImageRef`, `sourceImageRefs`, `digest`, `manifestMediaType`, and `platformCount`.

The assembled `release-governance-bundle` workflow artifact is also attested as a payload tree so the downloadable governance pack has the same provenance coverage as the individual governed JSON evidence files.

## Supported Release Triggers

The repository supports two official triggers:

- tag push:
  - any Git ref that matches `release-*`
- `workflow_dispatch`:
  - `release_tag` is required
  - `git_ref` is optional and defaults to `refs/tags/<release_tag>`

Recommended tag examples:

- `release-2026-04-19`
- `release-v1.0.0`

If you want the release to be reproducible and auditable, create the tag only after the release commit has passed the local governed verification path.

## Recommended Local Preflight

Before pushing a release tag or invoking `workflow_dispatch`, run the repository-owned verification path from the target commit:

```bash
./bin/build.sh --verify-release
node --test scripts/release/tests/release-workflow.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/check-router-docs-safety.test.mjs
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease
node --test scripts/release/tests/release-workflow.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/check-router-docs-safety.test.mjs
```

That verifies the official local build, packaged runtime smoke, docs governance, and the workflow contracts for the release, release-governance, product-verification, and rust-verification lanes before GitHub runners spend time building artifacts.

## Required Repository Variables

The workflow materializes referenced external release dependencies before verification and packaging. Configure these repository variables so the workflow resolves the exact dependency refs you intend to release:

- `SDKWORK_CORE_GIT_REF`
- `SDKWORK_UI_GIT_REF`
- `SDKWORK_APPBASE_GIT_REF`
- `SDKWORK_IM_SDK_GIT_REF`

Use branch names for rolling integration or immutable commit SHAs for stricter release provenance. If a variable is omitted, the workflow falls back to `main`.

## Governance Inputs And Overrides

The governance lane can consume either committed artifacts or explicit repository variables supplied by the control plane.

Primary governance override variables:

- `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
- `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
- `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`

Telemetry supplement variables:

- `SDKWORK_RELEASE_TELEMETRY_GATEWAY_PROMETHEUS_TEXT`
- `SDKWORK_RELEASE_TELEMETRY_ADMIN_PROMETHEUS_TEXT`
- `SDKWORK_RELEASE_TELEMETRY_PORTAL_PROMETHEUS_TEXT`
- `SDKWORK_RELEASE_TELEMETRY_SUPPLEMENTAL_TARGETS_JSON`
- `SDKWORK_RELEASE_TELEMETRY_GENERATED_AT`
- `SDKWORK_RELEASE_TELEMETRY_SOURCE_KIND`
- `SDKWORK_RELEASE_TELEMETRY_SOURCE_PROVENANCE`
- `SDKWORK_RELEASE_TELEMETRY_SOURCE_FRESHNESS_MINUTES`

Attestation policy:

- `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED`
  - set to `true` when you want artifact attestations in private repositories
  - public repositories already emit attestations by default in the workflow contract

If you do not provide explicit JSON overrides, the workflow seeds governance materializers from the committed governed artifact paths inside the repository.
The governance lane also materializes third-party governance outputs directly from the governed release inputs:

- `docs/release/third-party-sbom-latest.spdx.json`
- `docs/release/third-party-notices-latest.json`

Those files are generated from the Rust dependency graph plus the governed admin and portal `node_modules/` trees after frozen installs complete in the governance lane.

## Desktop Signing Configuration

The desktop release lane now consumes signing configuration directly from GitHub repository variables and secrets in the `Run portal desktop signing hook` step. That means the online workflow can be configured without editing the workflow file.

Fail-closed policy:

- `SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED`
  - set to `true` when unsigned desktop output must fail the release

Hook sources:

- `SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK`
- `SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK`
- `SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK`
- `SDKWORK_RELEASE_DESKTOP_SIGN_HOOK`

Resolution rules:

- platform-specific hook variables take precedence over the generic hook
- the workflow resolves hook values as `secrets.*` first, then `vars.*`
- store sensitive signing commands, tokens, or certificate material in GitHub Secrets rather than plain repository variables

Hook placeholder expansion:

- `{app}`
- `{platform}`
- `{arch}`
- `{target}`
- `{file}`
- `{evidence}`

The hook is responsible for calling the platform-native signing or notarization toolchain. The repository contract guarantees installer discovery, hook execution, and evidence materialization at:

- `artifacts/release-governance/desktop-release-signing-<platform>-<arch>.json`

Treat that desktop signing artifact as a machine-readable operator contract:

- `desktop-release-signing-<platform>-<arch>.json` is emitted with `version`, `type` (`sdkwork-desktop-release-signing`), `appId`, `platform`, `arch`, `targetTriple`, `required`, `status`, `hook`, `bundleFiles`, `evidencePath`, and `commandCount`
- `hook` records the resolved `kind` and the selected configuration `envVar`
- `status` resolves to `skipped`, `signed`, or `failed`
- failed runs also materialize `failure.message` so downstream automation can archive the exact signing failure reason

## Trigger The Online Release

### Option 1: Push A Release Tag

```bash
git tag release-2026-04-19
git push origin release-2026-04-19
```

This is the preferred path for immutable releases because the tag itself becomes the release identity and the default build ref.

### Option 2: Run `workflow_dispatch`

Use this when:

- the tag already exists and you need to retry the workflow
- you want to rebuild from an explicit `git_ref`
- you are rehearsing a release from a protected tag or controlled ref

Required inputs:

- `release_tag`

Optional inputs:

- `git_ref`

If `git_ref` is omitted, the workflow builds `refs/tags/<release_tag>`.

## What The Workflow Produces

The online workflow runs four major stages:

- `rust-dependency-audit`
  - verifies Rust dependency health against the exact release ref
- `product-verification`
  - installs the product workspaces, builds the docs site, and runs repository-owned release product verification
- `governance-release`
  - installs the governed admin and portal workspaces, materializes governed evidence, and publishes governance workflow artifacts
  - emits `third-party-sbom-latest.spdx.json` and `third-party-notices-latest.json` beside the release-window, sync-audit, telemetry, and SLO artifacts
- `native-release`
  - builds the official server and portal desktop products for each supported platform and architecture
  - publishes per-architecture Linux OCI images to GHCR as `:<release-tag>-linux-x64` and `:<release-tag>-linux-arm64`
- `publish`
  - downloads the packaged assets, regenerates `release-catalog.json`, creates the GitHub release, and assembles the multi-arch GHCR manifest tag `ghcr.io/<owner>/sdkwork-api-router:<release-tag>`

## Post-Publish Validation

After the workflow succeeds, validate the release at two levels.

### GitHub Release Assets

Confirm that the published GitHub release contains only:

- `sdkwork-api-router-product-server-*.tar.gz`
- `sdkwork-api-router-product-server-*.tar.gz.sha256.txt`
- `sdkwork-api-router-product-server-*.manifest.json`
- `sdkwork-router-portal-desktop-*`
- `release-catalog.json`

Before installing any downloaded asset, validate it against the sibling `.sha256.txt` file:

```bash
sha256sum -c sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt
sha256sum -c sdkwork-router-portal-desktop-linux-x64.AppImage.sha256.txt
```

```powershell
$expected = (Get-Content .\sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt).Split()[0]
$actual = (Get-FileHash -Algorithm SHA256 .\sdkwork-api-router-product-server-linux-x64.tar.gz).Hash.ToLowerInvariant()
if ($actual -ne $expected.ToLowerInvariant()) { throw 'checksum mismatch' }
```

Treat the published `release-catalog.json` as the machine-readable release index for the official SKU set:

- top-level fields include `version`, `type` (`sdkwork-release-catalog`), `releaseTag`, `generatedAt`, `productCount`, `variantCount`, and `products`
- each `products[]` entry records the official `productId` and its `variants[]`
- each `variants[]` entry records `platform`, `arch`, `outputDirectory`, `variantKind`, `primaryFile`, `primaryFileSizeBytes`, `checksumFile`, `checksumAlgorithm`, `manifestFile`, `sha256`, and the parsed external `manifest`

Treat the published external asset manifests as machine-readable install contracts too:

- `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json` emits `type` (`product-server-archive`), `productId`, `platform`, `arch`, `target`, `releaseVersion`, `archiveFile`, `checksumFile`, `embeddedManifestFile`, `installers`, `services`, `sites`, `bootstrapDataRoots`, and `deploymentAssetRoots`
- `sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json` emits `type` (`portal-desktop-installer`), `productId`, `appId`, `platform`, `arch`, `target`, `artifactKind`, `installerFile`, `checksumFile`, `sourceBundlePath`, and `embeddedRuntime`

Confirm that it does not contain:

- governance bundles
- release-window snapshots
- telemetry exports
- sync audits
- raw Tauri bundle trees
- standalone web release assets

### GHCR Images

Confirm that GHCR exposes:

- `ghcr.io/<owner>/sdkwork-api-router:<release-tag>`
- `ghcr.io/<owner>/sdkwork-api-router:<release-tag>-linux-x64`
- `ghcr.io/<owner>/sdkwork-api-router:<release-tag>-linux-arm64`

The release tag must resolve as a multi-architecture manifest that points to the per-architecture Linux images built from the packaged server bundle.
Confirm that the workflow artifacts also contain GHCR image publish metadata files such as `ghcr-image-publish-linux-x64.json` and `ghcr-image-publish-linux-arm64.json`, each recording the final `imageRef` and pushed digest.
Confirm that the workflow artifacts also contain `ghcr-image-manifest-publish.json`, recording the release-tag manifest `targetImageRef` and published digest for the assembled multi-architecture image.
Use those JSON artifacts as the post-publish source of truth rather than relying only on the GitHub web UI:

- each per-architecture `sdkwork-ghcr-image-publish` record must point to the packaged server `bundlePath`, the pushed `imageRepository`, the resolved `imageTag`, the final `imageRef`, and the published `digest`
- the `sdkwork-ghcr-image-manifest-publish` record must capture the assembled `targetImageRef`, the contributing `sourceImageRefs`, the resolved `manifestMediaType`, and the final `platformCount`

### Workflow Artifacts And Attestations

Confirm the workflow run still exposes the governance and smoke evidence as workflow artifacts:

- `release-governance-bundle`
- `release-governance-window-snapshot`
- `release-governance-sync-audit`
- `release-governance-telemetry-export`
- `release-governance-telemetry-snapshot`
- `release-governance-slo-evidence`
- `release-governance-third-party-sbom`
- `release-governance-third-party-notices`
- per-platform desktop signing evidence
- installed-runtime smoke evidence
- Linux Docker Compose smoke evidence
- Linux Helm render smoke evidence
- per-architecture GHCR image publish metadata
- GHCR multi-architecture manifest publish metadata

Treat the downloadable `release-governance-bundle` as a machine-readable restore package, not just a convenience zip:

- it must include `release-governance-bundle-manifest.json`
- that manifest is emitted with `version`, `generatedAt`, `bundleEntryCount`, and `artifacts`
- each `artifacts[]` entry records `id`, `relativePath`, and `sourceRelativePath`
- `restore.command` is the repository-owned restore entrypoint operators should use when replaying the downloaded governance bundle
- running `node scripts/release/restore-release-governance-latest.mjs --artifact-dir <downloaded-dir>` returns JSON with `repoRoot` and `restored`, and each `restored[]` entry records `id`, `sourcePath`, `outputPath`, and `duplicateCount`

Treat the smoke evidence payloads as machine-readable operator contracts as well:

- `unix-installed-runtime-smoke-<platform>-<arch>.json` and `windows-installed-runtime-smoke-<platform>-<arch>.json` emit `generatedAt`, `ok`, `platform`, `arch`, `target`, `runtimeHome`, `evidencePath`, `backupBundlePath`, `backupRestoreVerified`, and `healthUrls`
- installed-runtime smoke evidence can also include `logs.stdout`, `logs.stderr`, and `failure.message` when startup, backup, restore, or health validation fails
- `docker-compose-smoke-<platform>-<arch>.json` emits `generatedAt`, `ok`, `platform`, `arch`, `executionMode`, `bundlePath`, `evidencePath`, `healthUrls`, `siteUrls`, `browserSmokeTargets`, and `databaseAssertions`
- Docker Compose smoke evidence can also include `browserSmokeResults`, `composePs`, `logs.router`, `logs.postgres`, `diagnostics`, and `failure.message`
- `helm-render-smoke-<platform>-<arch>.json` emits `generatedAt`, `ok`, `platform`, `arch`, `bundlePath`, `evidencePath`, `renderedManifestPath`, and `renderedKinds`
- Helm render smoke evidence can also include `kubeconformSummary` and `failure.message` when render or schema validation fails

Treat the governed release evidence payloads as machine-readable operator contracts too:

- `release-window-snapshot-latest.json` emits `generatedAt`, `source`, and `snapshot`; `snapshot` records `latestReleaseTag`, `commitsSinceLatestRelease`, `workingTreeEntryCount`, and `hasReleaseBaseline`
- `release-sync-audit-latest.json` emits `generatedAt`, `source`, and `summary`; `summary` records `releasable` and `reports`, and each `reports[]` entry includes `id`, `targetDir`, `expectedGitRoot`, `topLevel`, `remoteUrl`, `localHead`, `remoteHead`, `expectedRef`, `branch`, `upstream`, `ahead`, `behind`, `isDirty`, `reasons`, and `releasable`
- `release-telemetry-export-latest.json` emits `version`, `generatedAt`, `source`, `prometheus`, and `supplemental`; `source` can include `kind`, `provenance`, and `freshnessMinutes`, `prometheus` must carry `gateway`, `admin`, and `portal`, and `supplemental.targets` carries the non-directly-derived release targets
- `release-telemetry-snapshot-latest.json` emits `version`, `snapshotId`, `generatedAt`, `source`, and `targets`; `source` can include `kind`, `exportKind`, `provenance`, `freshnessMinutes`, and `supplementalTargetIds`
- `slo-governance-latest.json` emits `version`, `baselineId`, `baselineDate`, `generatedAt`, and `targets`

Treat the third-party governance artifacts as machine-readable operator contracts too:

- `third-party-sbom-latest.spdx.json` is published as an `SPDX-2.3` document with `spdxVersion`, `documentNamespace`, `creationInfo.created`, `documentDescribes`, `packages`, and `relationships`
- `third-party-notices-latest.json` emits `version`, `generatedAt`, `packageCount`, `cargoPackageCount`, `npmPackageCount`, `packages`, and `noticeText`
- each third-party notices `packages[]` entry records at least `ecosystem`, `name`, `version`, `licenseDeclared`, `downloadLocation`, `sourcePath`, and `noticeFiles`

If artifact attestations are enabled for the repository, confirm attestations exist for:

- governance bundle payloads
- governed evidence artifacts, including `third-party-sbom-latest.spdx.json` and `third-party-notices-latest.json`
- desktop signing evidence
- smoke evidence
- GHCR image publish metadata
- GHCR multi-architecture manifest publish metadata
- packaged native release assets
- `release-catalog.json`

From a checkout of the released tag or published commit, run the repository-owned attestation verifier instead of relying only on the GitHub UI. This command requires `gh` on `PATH`:

```bash
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

```powershell
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

That verifier covers the governed release JSON artifacts, including `third-party-sbom-latest.spdx.json` and `third-party-notices-latest.json`, the `release-governance-bundle` payload tree, desktop signing evidence, installed-runtime smoke evidence, Linux Docker Compose and Helm smoke evidence, GHCR image publish metadata, GHCR multi-architecture manifest publish metadata, the published `release-catalog.json`, and the packaged native release asset tree.

For machine processing, run the same verifier with JSON output:

```bash
node scripts/release/verify-release-attestations.mjs --format json --repo Sdkwork-Cloud/sdkwork-api-router
```

```powershell
node scripts/release/verify-release-attestations.mjs --format json --repo Sdkwork-Cloud/sdkwork-api-router
```

That JSON payload is the machine-readable attestation verdict contract:

- top-level fields include `ok`, `blocked`, `reason`, `repoSlug`, `verifiedCount`, `blockedCount`, `failedCount`, `verifiedIds`, `blockedIds`, `failingIds`, and `reports`
- each `reports[]` entry includes `id`, `specId`, `description`, `ok`, `blocked`, `reason`, `relativeSubjectPath`, `expectedRelativePath`, `stdout`, `stderr`, and `errorMessage`

Use that `--format json` output when the validation needs to be archived or fed into automation.

### Catalog Validation

Inspect `release-catalog.json` and confirm:

- the top-level contract still resolves to `type` `sdkwork-release-catalog`
- `releaseTag`, `productCount`, `variantCount`, and `products` are present
- `generatedAt` is present
- only the official product ids are included
- each variant exposes `outputDirectory`, `variantKind`, `primaryFile`, `checksumFile`, `manifestFile`, `sha256`, and the parsed `manifest`
- each variant exposes `primaryFileSizeBytes`
- each variant exposes `checksumAlgorithm`
- each variant references the normalized published filenames

## Related Guides

- [Release Builds](/getting-started/release-builds)
- [Production Deployment](/getting-started/production-deployment)
- [Install Layout](/operations/install-layout)
- [Service Management](/operations/service-management)
