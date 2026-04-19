# Online Release

This is the canonical GitHub-hosted release runbook for SDKWork API Router.

Use this page when you are publishing an official GitHub release, preparing repository variables and secrets, wiring desktop signing, or validating the published release after the workflow finishes.

The public release contract stays intentionally narrow:

- installable products:
  - `sdkwork-api-router-product-server`
  - `sdkwork-router-portal-desktop`
- published release metadata:
  - `release-catalog.json`

Governance evidence, telemetry exports, release-window snapshots, sync audits, and smoke evidence remain workflow artifacts and attestations. They are not public release downloads.

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
- `SDKWORK_CRAW_CHAT_SDK_GIT_REF`

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
  - materializes governed evidence and publishes governance workflow artifacts
- `native-release`
  - builds the official server and portal desktop products for each supported platform and architecture
- `publish`
  - downloads the packaged assets, regenerates `release-catalog.json`, and creates the GitHub release

## Post-Publish Validation

After the workflow succeeds, validate the release at two levels.

### GitHub Release Assets

Confirm that the published GitHub release contains only:

- `sdkwork-api-router-product-server-*.tar.gz`
- `sdkwork-api-router-product-server-*.tar.gz.sha256.txt`
- `sdkwork-api-router-product-server-*.manifest.json`
- `sdkwork-router-portal-desktop-*`
- `release-catalog.json`

Confirm that it does not contain:

- governance bundles
- release-window snapshots
- telemetry exports
- sync audits
- raw Tauri bundle trees
- standalone web release assets

### Workflow Artifacts And Attestations

Confirm the workflow run still exposes the governance and smoke evidence as workflow artifacts:

- `release-governance-bundle`
- `release-governance-window-snapshot`
- `release-governance-sync-audit`
- `release-governance-telemetry-export`
- `release-governance-telemetry-snapshot`
- `release-governance-slo-evidence`
- per-platform desktop signing evidence
- installed-runtime smoke evidence
- Linux Docker Compose smoke evidence
- Linux Helm render smoke evidence

If artifact attestations are enabled for the repository, confirm attestations exist for:

- governed evidence artifacts
- desktop signing evidence
- smoke evidence
- packaged native release assets
- `release-catalog.json`

From a checkout of the released tag or published commit, run the repository-owned attestation verifier instead of relying only on the GitHub UI. This command requires `gh` on `PATH`:

```bash
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

```powershell
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

Use `--format json` when the validation needs to be archived or fed into automation.

### Catalog Validation

Inspect `release-catalog.json` and confirm:

- `generatedAt` is present
- only the official product ids are included
- each variant exposes `variantKind`
- each variant exposes `primaryFileSizeBytes`
- each variant exposes `checksumAlgorithm`
- each variant references the normalized published filenames

## Related Guides

- [Release Builds](/getting-started/release-builds)
- [Production Deployment](/getting-started/production-deployment)
- [Install Layout](/operations/install-layout)
- [Service Management](/operations/service-management)
