# Release Builds

This page covers the official release products and the build commands that materialize them.

This page is for build and package generation only.

If you need production deployment, PostgreSQL initialization, service registration, or OS-standard server installation, use [Production Deployment](/getting-started/production-deployment).
If you need the GitHub-hosted release procedure, repository variables, signing hooks, or post-publish validation, use [Online Release](/getting-started/online-release).

## Official Products

SDKWork API Router now publishes exactly two official user-facing products:

- `sdkwork-api-router-product-server`
- `sdkwork-router-portal-desktop`

The repository may still generate intermediate outputs such as Rust binaries, browser build directories, and Tauri bundle roots, but those are build inputs, not official release products.

## What The Release Workflow Publishes

The GitHub release workflow publishes only:

- the server product archive set:
  - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
  - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
  - `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`
- the portal desktop installer set for the current platform:
  - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>`
  - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>.sha256.txt`
  - `sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json`
- one release-level asset index:
  - `release-catalog.json`

The workflow does not publish:

- standalone admin desktop installers
- standalone web release asset bundles

Governance evidence, telemetry exports, sync audits, and similar material remain workflow artifacts and attestations, not user-facing downloads.

`release-catalog.json` is materialized at `artifacts/release/release-catalog.json` whenever the repository has a complete official asset set. Local native packaging writes it for the current output tree, and the release workflow re-materializes it in the publish stage after the native assets are downloaded. It aggregates the official per-product manifests into one machine-readable release index for automation and operators. The catalog includes a top-level `generatedAt` timestamp and per-variant `variantKind`, `primaryFileSizeBytes`, and `checksumAlgorithm` fields so deployment automation can audit asset identity without re-parsing each manifest first.

## Managed Local Build

Linux or macOS:

```bash
./bin/build.sh
./bin/build.sh --verify-release
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease
```

The managed build prepares the inputs for the official products:

- Rust release service binaries
- admin and portal browser assets used by the server product
- the staged `router-product/` sidecar payload used by the portal desktop bundle
- the portal desktop installer bundle
- packaged native release assets under `artifacts/release/`
- a local `artifacts/release/release-catalog.json` when the output tree contains a complete official asset set

`--verify-release` is the managed local release-verification mode. It keeps the same official build and packaging steps, then runs the platform-native smoke checks against the packaged assets instead of against raw workspace outputs:

- Windows: packaged installed-runtime smoke
- macOS: packaged installed-runtime smoke
- Linux: packaged installed-runtime smoke, Docker Compose packaged smoke, and Helm render packaged smoke

`--verify-release` also always includes the governed docs site build (`pnpm --dir docs build`). Official local release verification treats the docs site as part of the public product surface, so docs-site governance stays enabled even if a normal engineering build would otherwise skip docs.

`--verify-release` also runs the local `release governance preflight` step (`node scripts/release/run-release-governance-checks.mjs --profile preflight`). That keeps the official local release path aligned with the repository's release-governance contract instead of proving only the packaged runtime smoke lanes.

`--skip-docs cannot be combined with --verify-release`. If you need to omit docs for a local engineering build, run the normal build mode instead of the official local release-verification mode.

## Official Asset Locations In A Local Build

After a successful local release build, the official assets are under:

- server product archive set:
  - `artifacts/release/native/<platform>/<arch>/bundles/`
  - contains:
    - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
    - `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
    - `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`
- portal desktop installer assets:
  - `artifacts/release/native/<platform>/<arch>/desktop/portal/`
  - contains only normalized official desktop release files:
    - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>`
    - `sdkwork-router-portal-desktop-<platform>-<arch>.<ext>.sha256.txt`
    - `sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json`
- portal desktop staged sidecar payload:
  - `bin/portal-rt/router-product/`
  - contains:
    - `router-product/bin/router-product-service`
    - `router-product/sites/admin/dist/`
    - `router-product/sites/portal/dist/`
    - `router-product/data/`
    - `router-product/release-manifest.json`
    - `router-product/README.txt`
- release-level asset index:
  - `artifacts/release/release-catalog.json`

## Portal Desktop Packaging Contract

The portal desktop bundle is a native shell plus a bundled release-like runtime payload.

The public release contract is intentionally narrower than the raw Tauri bundle tree. The release packager copies exactly one platform-native installer into the official desktop release directory, renames it to the canonical product filename, writes a SHA-256 checksum file, and emits a manifest that records the embedded sidecar payload contract. That installer manifest exposes the embedded runtime paths through `embeddedRuntime.routerBinary`, `embeddedRuntime.adminSiteDir`, `embeddedRuntime.portalSiteDir`, `embeddedRuntime.bootstrapDataDir`, `embeddedRuntime.releaseManifestFile`, and `embeddedRuntime.readmeFile`.

The staged payload contains:

- `router-product/bin/router-product-service`
- `router-product/sites/admin/dist/`
- `router-product/sites/portal/dist/`
- `router-product/data/`
- `router-product/release-manifest.json`
- `router-product/README.txt`

Desktop runtime contract:

- fixed local shell base URL: `http://127.0.0.1:3001`
- access-mode bind switch:
  - local-only: `127.0.0.1:3001`
  - shared network: `0.0.0.0:3001`
- the shared-network bind is a desktop-only access-mode override; the native server product still defaults to `127.0.0.1:3001` unless config, environment, or CLI changes it
- mutable runtime state lives in OS-standard app config, data, and log directories
- the shell persists access mode in `desktop-runtime.json` and synthesizes canonical sidecar `router.yaml`
- the public release directory does not expose raw Tauri bundle trees; it exposes only canonical `sdkwork-router-portal-desktop-*` product assets

### Desktop Signing Hooks

The official desktop release flow supports an explicit signing stage before the normalized installer assets are collected.

- set `SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED=true` when a release must fail closed if no signing hook is configured
- use one of these hook variables:
  - `SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK`
  - `SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK`
  - `SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK`
  - `SDKWORK_RELEASE_DESKTOP_SIGN_HOOK` as the generic fallback
- hook commands receive placeholder expansion for `{app}`, `{platform}`, `{arch}`, `{target}`, `{file}`, and `{evidence}`
- the release workflow writes signing evidence to `artifacts/release-governance/desktop-release-signing-<platform>-<arch>.json`

The hook is responsible for invoking the platform-native signing or notarization toolchain. The repository contract only guarantees installer discovery, hook execution, and evidence emission.

## Server Packaging Contract

The server product is published as a normalized archive set instead of a raw build directory. The release packager emits:

- one canonical `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
- one SHA-256 checksum file for that archive
- one external `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`

The external manifest records the archive identity plus the embedded bundle contract. Inside the archive, `release-manifest.json` continues to describe the unpacked product payload used by install and runtime tooling.

## Release Catalog Contract

`release-catalog.json` is the release-level index for the official SKU set. It does not replace the per-asset manifests. It aggregates them and is the only published metadata asset that sits beside the two installable products.

The catalog records:

- the release tag
- the `generatedAt` timestamp for the catalog snapshot
- the official product ids
- per-platform and per-arch variants, including `variantKind`
- the primary asset filename, `primaryFileSizeBytes`, checksum filename, `checksumAlgorithm`, manifest filename, and parsed SHA-256 value
- the parsed external manifest payload for each official asset

## Native Server Install Generation

Repository-driven native install generation still uses the managed install entrypoints:

Portable install:

```bash
./bin/install.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

Production-oriented system install:

```bash
./bin/install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
```

Custom system install rooted at `<product-root>`:

```bash
./bin/install.sh --mode system --home <product-root>
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -Home <product-root>
```

Use `--home` or `-Home` when you want repository-driven install generation to materialize a product tree under a specific `<product-root>`. Validate and operate that generated install from the installed product root as described in [Production Deployment](/getting-started/production-deployment).

`system` mode is the canonical server install path and defaults to PostgreSQL.

## Dry Run

```bash
./bin/build.sh --dry-run
./bin/install.sh --dry-run
./bin/install.sh --mode system --dry-run
./bin/install.sh --mode system --home <product-root> --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system -Home <product-root> -DryRun
```

## Verification

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/release-workflow.test.mjs scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/release-flow-contract.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs apps/sdkwork-router-portal/tests/portal-desktop-api-base.test.mjs apps/sdkwork-router-portal/tests/portal-desktop-sidecar-runtime.test.mjs
```

## Next Steps

- [Production Deployment](/getting-started/production-deployment)
- [Online Release](/getting-started/online-release)
- [Install Layout](/operations/install-layout)
- [Service Management](/operations/service-management)
