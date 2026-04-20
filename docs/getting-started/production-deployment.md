# Production Deployment

This is the canonical production deployment guide for SDKWork API Router.

Use this page when you are publishing an online server deployment, preparing a native server install, using Docker Compose, or rolling out a Helm release.

If you need the GitHub Actions release procedure itself, repository variables and secrets, desktop signing hooks, or post-publish GitHub validation, use [Online Release](/getting-started/online-release).

## Product Contract

- the official server-side product is `sdkwork-api-router-product-server`
- the official desktop product is `sdkwork-router-portal-desktop`
- public GitHub releases publish only those two products
- `release-catalog.json` is published alongside them as release metadata, not as a third product
- `system` install mode is the native production standard
- PostgreSQL is the default database contract for `system` installs
- config files are the primary source of truth
- environment variables are discovery inputs and fallback values
- service supervision belongs to `systemd`, `launchd`, or Windows Service Control Manager

The desktop product is not the online server deployment path. It is a per-user Tauri shell that supervises a bundled `router-product-service` sidecar with the same public web and API surface on a fixed desktop port `3001`.

## Server Product Contents

The server product archive is built around `router-product-service` and includes:

- release service binaries
- admin static assets
- portal static assets
- bootstrap data
- deploy assets for Docker and Helm

That bundle is the canonical deployment input for:

- native server installs
- Docker image builds
- Docker Compose
- Helm

The release workflow also runs `installed-runtime smoke` against that same packaged server bundle before publish, so the native install path is verified from the exact packaged artifact operators deploy.
Native install tooling selects the canonical server archive from `release-catalog.json`, then rejects any archive, checksum, or external manifest that does not match that published catalog entry before unpacking it.

## Choose A Deployment Path

### Docker Compose

Use this when you want the fastest single-host rollout with PostgreSQL included.

Primary assets:

- `deploy/docker/Dockerfile`
- `deploy/docker/docker-compose.yml`
- `deploy/docker/.env.example`

### Helm

Use this when you want Kubernetes deployment with externally managed PostgreSQL.

Primary assets:

- `deploy/helm/sdkwork-api-router/`

### Native System Install

Use this when you need an OS-standard installation with service-managed startup.

## Build Official Release Inputs

Linux or macOS:

```bash
./bin/build.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

This prepares the same server product inputs used by the release workflow:

- Rust release service binaries
- admin and portal browser assets
- the staged portal desktop `router-product/` payload
- the packaged server product archive

If you want the local repository run to prove the same governed contract as the official release path, use `./bin/build.sh --verify-release` or `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease`. That mode keeps the same build inputs, then also requires the docs site build, packaged runtime smoke, and the local `release governance preflight`.

For native installs, only the packaged server bundle is valid install input. After extracting the archive, the governed bundle-root `install.sh` and `install.ps1` entrypoints materialize `bin/`, `sites/*/dist/`, `data/`, `deploy/`, `release-manifest.json`, and `README.txt` into `releases/<version>/`.
That bundle also carries the governed `control/bin/` control tree used to materialize installed `current/bin/` entrypoints, keeping production operations pinned to the official release artifact.
`release-catalog.json` is the release-level source of truth for selecting and resolving that bundle from a complete official asset set.

## Release Governance

The release workflow separates governance evidence from user-facing products:

- `governance-release` materializes release-window, sync-audit, telemetry, SLO evidence, and the third-party governance artifacts `docs/release/third-party-sbom-latest.spdx.json` plus `docs/release/third-party-notices-latest.json`
- `native-release` builds the official server and portal desktop products
- governance artifacts stay as workflow artifacts and attestations
- installable public products stay limited to the server archive set and portal desktop installer set
- `release-catalog.json` is generated at `artifacts/release/release-catalog.json`, attested, and published as the machine-readable release index for the official asset set
- that catalog carries `generatedAt` plus per-variant `variantKind`, `primaryFileSizeBytes`, and `checksumAlgorithm` metadata for audit and deployment tooling

For local governance validation from a repository checkout:

Linux or macOS:

```bash
export SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT="$PWD/artifacts/external-release-deps"
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

Windows:

```powershell
$env:SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT = (Join-Path (Get-Location) 'artifacts\external-release-deps')
node scripts/release/materialize-external-deps.mjs
node scripts/release/verify-release-sync.mjs --format text --live
node scripts/release/run-release-governance-checks.mjs
```

## Generate A Native Server Install

Linux or macOS:

```bash
./install.sh --mode system
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Mode system
```

Generated production assets include:

- canonical `router.yaml`
- `conf.d/` overlay directory
- `router.env`
- `router.env.example`
- service descriptors for `systemd`, `launchd`, and Windows Service

## Initialize Production Configuration

Edit the generated runtime config before first start:

- `router.yaml`
  - canonical runtime config
- `conf.d/*.{yaml,yml,json}`
  - optional domain-specific overlays
- `router.env`
  - discovery values and fallback values for fields the config file leaves unset

Recommended first edits:

- replace the PostgreSQL placeholder with a real database URL
- set JWT, credential, and metrics secrets
- review bind addresses and trusted network boundaries
- confirm admin and portal static site locations

## Validate Before Service Registration

From the installed product root, run:

```bash
./current/bin/validate-config.sh --home <product-root>
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home <product-root>
```

From a repository checkout, the managed fallback remains:

```bash
node bin/router-ops.mjs validate-config --mode system --home <product-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <product-root>
```

Validation checks:

- config discovery and merge order
- config-file-over-environment precedence for business fields
- production security posture
- rejection of placeholder database URLs and secrets during startup and `validate-config`
- rejection of SQLite in `system` mode unless an explicit development override is enabled

## Backup And Restore

Run installed backup and restore operations from the installed product root:

```bash
./current/bin/backup.sh --home <product-root> --output ./backups/2026-04-19 --force
./current/bin/restore.sh --home <product-root> --source ./backups/2026-04-19 --force
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\backup.ps1 -Home <product-root> -OutputPath .\backups\2026-04-19 -Force
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\restore.ps1 -Home <product-root> -SourcePath .\backups\2026-04-19 -Force
```

Dry-run planning is also available:

```bash
./current/bin/backup.sh --home <product-root> --output ./backups/2026-04-19 --dry-run
./current/bin/restore.sh --home <product-root> --source ./backups/2026-04-19 --force --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\backup.ps1 -Home <product-root> -OutputPath .\backups\2026-04-19 -DryRun
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\restore.ps1 -Home <product-root> -SourcePath .\backups\2026-04-19 -Force -DryRun
```

Operational contract:

- stop the managed runtime before backup and before restore
- the backup bundle contains `control/release-manifest.json`, a full config snapshot, a mutable data snapshot, and a PostgreSQL dump when the installed database URL is PostgreSQL
- `backup-manifest.json` is the machine-readable backup contract; the current `formatVersion` is `2`, and its `bundle.controlManifestFile`, `bundle.configSnapshotRoot`, and `bundle.mutableDataSnapshotRoot` fields declare the exported control manifest, config snapshot, and mutable-data snapshot paths
- restore replaces the installed config and mutable data roots from that bundle, then replays the PostgreSQL dump against the database configured by the restored runtime config
- `log/` and `run/` are operational state and are not restored from the backup bundle
- PostgreSQL backups require `pg_dump` on `PATH`; PostgreSQL restores require `pg_restore` on `PATH`

## Support Bundle

Run installed support-bundle exports from the installed product root:

```bash
./current/bin/support-bundle.sh --home <product-root> --output ./support/2026-04-19 --force
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\support-bundle.ps1 -Home <product-root> -OutputPath .\support\2026-04-19 -Force
```

Dry-run planning is also available:

```bash
./current/bin/support-bundle.sh --home <product-root> --output ./support/2026-04-19 --dry-run
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\support-bundle.ps1 -Home <product-root> -OutputPath .\support\2026-04-19 -DryRun
```

Operational contract:

- support-bundle is safe to run against an installed runtime without replacing mutable state
- the bundle contains `control/release-manifest.json`, redacted config snapshots, log inventories and redacted text captures when available, runtime-state inventory, and managed process-state metadata
- `support-bundle-manifest.json` is the machine-readable support export contract; the current `formatVersion` is `2`, and its `paths.controlManifestFile`, `paths.configSnapshotRoot`, `paths.configInventoryFile`, `paths.logsSnapshotRoot`, `paths.logsInventoryFile`, `paths.runtimeSnapshotRoot`, `paths.runtimeInventoryFile`, and `paths.processStateFile` fields declare the produced artifact paths
- known secret-bearing config values are redacted before export; binary credential stores and key material are omitted from the support bundle
- use support bundles for operator escalation and release-audit capture, not for disaster recovery; backup and restore remain the state-migration surfaces

## Register And Start Services

Use foreground entrypoints under a service manager:

- Linux: `./current/service/systemd/install-service.sh`
- macOS: `./current/service/launchd/install-service.sh`
- Windows: `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1`

Reference guides:

- [Install Layout](/operations/install-layout)
- [Service Management](/operations/service-management)

## Docker Compose Quick Deployment

```bash
tar -xzf sdkwork-api-router-product-server-linux-x64.tar.gz
cd sdkwork-api-router-product-server-linux-x64
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

Before the first `docker compose up`, replace every `replace-with-*` value in `deploy/docker/.env`. The container entrypoint and `validate-config` both fail closed when placeholder database credentials, JWT secrets, credential keys, or metrics tokens remain configured.

## Helm Quick Deployment

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/<owner>/sdkwork-api-router \
  --set image.tag=<release-tag> \
  --set secrets.databaseUrl='postgresql://sdkwork:replace-with-db-password@postgresql:5432/sdkwork_api_router' \
  --set secrets.adminJwtSigningSecret='replace-with-admin-jwt-secret' \
  --set secrets.portalJwtSigningSecret='replace-with-portal-jwt-secret' \
  --set secrets.credentialMasterKey='replace-with-credential-master-key' \
  --set secrets.metricsBearerToken='replace-with-metrics-token'
```

Official GitHub releases also publish the multi-architecture Linux OCI image `ghcr.io/<owner>/sdkwork-api-router:<release-tag>`. The workflow first publishes `:<release-tag>-linux-x64` and `:<release-tag>-linux-arm64`, then assembles the public release tag as a multi-arch manifest in GHCR.

## Initialization Checklist

- target platform release inputs built successfully
- PostgreSQL database created and reachable
- `router.yaml` reviewed
- `router.env` secrets replaced
- `validate-config` run successfully
- service registered through the OS-native manager
- health endpoints verified after first start
