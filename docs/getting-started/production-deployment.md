# Production Deployment

This is the canonical production deployment guide for SDKWork API Router.

Use this page when you are publishing an online server deployment, preparing a native server install, using Docker Compose, or rolling out a Helm release.

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

For native installs, only the packaged server bundle is valid install input. The installer then materializes `bin/`, `sites/*/dist/`, `data/`, `deploy/`, `release-manifest.json`, and `README.txt` into `releases/<version>/`.
`release-catalog.json` is the release-level source of truth for selecting and resolving that bundle from a complete official asset set.

## Release Governance

The release workflow separates governance evidence from user-facing products:

- `governance-release` materializes release-window, sync-audit, telemetry, and SLO evidence
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
./bin/install.sh --mode system
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 -Mode system
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
- `conf.d/*.yaml`
  - optional domain-specific overlays
- `router.env`
  - discovery values and fallback values for fields the config file leaves unset

Recommended first edits:

- replace the PostgreSQL placeholder with a real database URL
- set JWT, credential, and metrics secrets
- review bind addresses and trusted network boundaries
- confirm admin and portal static site locations

## Validate Before Service Registration

From the installed runtime home, run:

```bash
./current/bin/validate-config.sh --home ./current
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home .\current
```

From a repository checkout, the managed fallback remains:

```bash
node bin/router-ops.mjs validate-config --mode system --home <install-root>
```

```powershell
node .\bin\router-ops.mjs validate-config --mode system --home <install-root>
```

Validation checks:

- config discovery and merge order
- config-file-over-environment precedence for business fields
- production security posture
- rejection of SQLite in `system` mode unless an explicit development override is enabled

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
cp deploy/docker/.env.example deploy/docker/.env
docker build -f deploy/docker/Dockerfile -t sdkwork-api-router:local .
docker compose -f deploy/docker/docker-compose.yml --env-file deploy/docker/.env up -d
```

## Helm Quick Deployment

```bash
helm upgrade --install sdkwork-api-router deploy/helm/sdkwork-api-router \
  --set image.repository=ghcr.io/your-org/sdkwork-api-router \
  --set image.tag=2026.04.18 \
  --set secrets.databaseUrl='postgresql://sdkwork:change-me@postgresql:5432/sdkwork_api_router'
```

## Initialization Checklist

- target platform release inputs built successfully
- PostgreSQL database created and reachable
- `router.yaml` reviewed
- `router.env` secrets replaced
- `validate-config` run successfully
- service registered through the OS-native manager
- health endpoints verified after first start
