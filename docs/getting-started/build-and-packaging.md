# Build and Packaging

This page explains how to compile engineering outputs, prepare official release inputs, and package the two supported SDKWork products from a repository checkout.

For the exact release artifacts published by GitHub Releases, use [Release Builds](/getting-started/release-builds). For the GitHub-hosted release procedure itself, use [Online Release](/getting-started/online-release). For production installation and deployment, use [Production Deployment](/getting-started/production-deployment).

## Official Products

SDKWork API Router publishes exactly two official user-facing products:

- `sdkwork-api-router-product-server`
- `sdkwork-router-portal-desktop`

Everything else in this repository is either a build input, a developer-facing workspace output, or a validation artifact.

## Build Targets

| Target | Command | Output |
|---|---|---|
| product host runtime | `cargo build --release -p router-product-service` | `target/release/router-product-service` |
| standalone service binaries | `cargo build --release -p gateway-service -p admin-api-service -p portal-api-service -p router-web-service` | `target/release/` |
| admin browser app | `pnpm --dir apps/sdkwork-router-admin build` | `apps/sdkwork-router-admin/dist/` |
| portal browser app | `pnpm --dir apps/sdkwork-router-portal build` | `apps/sdkwork-router-portal/dist/` |
| portal desktop sidecar payload | `node scripts/prepare-router-portal-desktop-runtime.mjs` | `bin/portal-rt/router-product/` |
| official portal desktop bundle | `pnpm --dir apps/sdkwork-router-portal tauri:build` | Tauri platform bundle output |
| docs site | `pnpm --dir docs build` | `docs/.vitepress/dist/` |

## Build Server Product Inputs

Compile the server-facing Rust binaries:

```bash
cargo build --release -p router-product-service -p gateway-service -p admin-api-service -p portal-api-service -p router-web-service
```

Then build the browser assets bundled into the server product:

```bash
pnpm --dir apps/sdkwork-router-admin install
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal install
pnpm --dir apps/sdkwork-router-portal build
```

If you want the repository-managed packaging flow that matches the release workflow, use:

```bash
./bin/build.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

That managed build prepares the inputs used by `sdkwork-api-router-product-server` and stages the native release assets under `artifacts/release/`.

For the governed local release path, run `./bin/build.sh --verify-release` or `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease`. That mode always includes the docs site build because the docs site is treated as part of the public product surface for official release verification. It also runs the local release governance preflight (`node scripts/release/run-release-governance-checks.mjs --profile preflight`) so the official build path validates governance contracts in addition to packaged runtime smoke. `--skip-docs cannot be combined with --verify-release`.

The managed build output contract for the official server product is:

- `artifacts/release/native/<platform>/<arch>/bundles/sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
- `artifacts/release/native/<platform>/<arch>/bundles/sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
- `artifacts/release/native/<platform>/<arch>/bundles/sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`

The external server manifest describes the archive file, checksum file, the embedded bundle contract, the governed `releaseVersion`, and the bundle-root installer entrypoints.
The server archive itself expands into a product root that already includes `install.sh`, `install.ps1`, `bin/`, `sites/`, `data/`, `deploy/`, `README.txt`, and an embedded `release-manifest.json`.
That official server bundle also carries `control/bin/` and `control/bin/lib/`; bundle-root native install tooling materializes the installed `current/bin/` operator surface from that embedded control tree so production installs stay bound to the governed release artifact instead of to repository-local helper scripts.

When the managed build output tree contains a complete official asset set, it also materializes the release-level metadata index at `artifacts/release/release-catalog.json`. That catalog aggregates the external manifests for the two official products into one machine-readable release index, carries `generatedAt` plus per-variant `variantKind`, `primaryFileSizeBytes`, and `checksumAlgorithm` fields, and remains release metadata rather than a third installable product.

## Build The Official Portal Desktop Product

The official desktop product is portal-first. It packages `apps/sdkwork-router-portal` as a native shell and embeds a release-like `router-product-service` sidecar payload.

Stage the runtime payload first:

```bash
node scripts/prepare-router-portal-desktop-runtime.mjs
```

Run the desktop shell in development:

```bash
pnpm --dir apps/sdkwork-router-portal tauri:dev
```

If you want to enter the official product-mode development flows from the repository root, use:

```bash
pnpm tauri:dev
pnpm server:dev
```

`pnpm tauri:dev` launches the portal desktop product path through the shared root entrypoint.
`pnpm server:dev` launches the full server development workspace through the same root entrypoint.
That root-level server dev flow is development-only and starts backend APIs, the admin Vite server, the portal Vite server, and the unified Pingora web host together.

Use `pnpm --dir apps/sdkwork-router-portal server:start` when you need the standalone integrated `router-product-service` CLI or deployment-oriented server runtime flags.

Build the production desktop installer or bundle:

```bash
pnpm --dir apps/sdkwork-router-portal tauri:build
```

The managed build normalizes the official desktop product files to:

- `artifacts/release/native/<platform>/<arch>/desktop/portal/sdkwork-router-portal-desktop-<platform>-<arch>.<ext>`
- `artifacts/release/native/<platform>/<arch>/desktop/portal/sdkwork-router-portal-desktop-<platform>-<arch>.<ext>.sha256.txt`
- `artifacts/release/native/<platform>/<arch>/desktop/portal/sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json`

Raw Tauri bundle trees remain intermediate platform build output. They are not the public product contract for packaging, docs, or release publication.

The admin Tauri shell remains available as an explicit development path, but it is not part of the official release product set.

## Build The Standalone Browser Apps

Admin browser app:

```bash
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-admin preview
```

Portal browser app:

```bash
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir apps/sdkwork-router-portal preview
```

These are useful for local validation and source development, but they are not published as standalone GitHub release products.

## Build The Documentation Site

```bash
pnpm --dir docs install
pnpm --dir docs build
pnpm --dir docs preview
```

The docs site is optional for ad hoc engineering builds, but it becomes mandatory again when you run the managed local release-verification flow with `--verify-release`. The same flow also runs the release governance preflight to keep local official verification aligned with release-governance expectations.

## Recommended Verification Before Packaging

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
node scripts/prepare-router-portal-desktop-runtime.mjs
pnpm --dir docs build
```

If you are changing TypeScript or docs config as well:

```bash
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir docs typecheck
```

## Related Docs

- source workflows:
  - [Source Development](/getting-started/source-development)
- official release assets:
  - [Release Builds](/getting-started/release-builds)
- GitHub-hosted publication:
  - [Online Release](/getting-started/online-release)
- production install and deployment:
  - [Production Deployment](/getting-started/production-deployment)
- workspace structure:
  - [Repository Layout](/reference/repository-layout)
