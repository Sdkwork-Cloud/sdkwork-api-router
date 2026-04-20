# SDKWork API Router

[中文文档](./README.zh-CN.md)

SDKWork API Router is a Rust-based OpenAI-compatible gateway, admin control plane, public portal, and product runtime. The repository ships both source-native development workflows and production-grade release/install tooling.

## Official Products

The repository publishes exactly two official user-facing release products:

- `sdkwork-api-router-product-server`
  - the canonical server bundle for native installs, Docker, Docker Compose, and Helm
- `sdkwork-router-portal-desktop`
  - the portal-first desktop shell with the bundled local product runtime

`release-catalog.json` is published beside those two products as machine-readable release metadata for automation and audit. It is not an installable product.

Everything else in this repository is either a source-development surface, an intermediate build output, or release-governance evidence.

## Production Entry Points

Use these pages first when you are planning an online deployment:

- [Production Deployment](./docs/getting-started/production-deployment.md)
- [Online Release](./docs/getting-started/online-release.md)
- [Install Layout](./docs/operations/install-layout.md)
- [Service Management](./docs/operations/service-management.md)
- [Docker And Helm Assets](./deploy/README.md)

For installed server products, the stable `current/bin/` operator surface is materialized from the official server bundle's embedded `control/bin/` tree rather than copied from a source checkout.

For local development only, use:

- [Quickstart](./docs/getting-started/quickstart.md)
- [Source Development](./docs/getting-started/source-development.md)

For repository ergonomics, root-level start/build/install/stop scripts are compatibility wrappers that delegate to `bin/*`.
The managed operator source of truth stays under `bin/*`, including installed `current/bin/backup.*`, `current/bin/restore.*`, `current/bin/support-bundle.*`, and `current/bin/validate-config.*` entrypoints.

## Runtime Surfaces

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-facing `/admin/*` control plane
- `portal-api-service`
  - developer-facing `/portal/*` self-service API
- `router-web-service`
  - Pingora-based public web host
- `router-product-service`
  - integrated production runtime serving `/admin/*`, `/portal/*`, and `/api/*`

## Configuration Contract

Primary config discovery order:

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

Effective precedence from lowest to highest:

- built-in defaults -> environment fallback -> config file -> CLI

Operational notes:

- `SDKWORK_CONFIG_DIR` and `SDKWORK_CONFIG_FILE` are discovery inputs.
- `conf.d/*.{yaml,yml,json}` overlays load after the primary file in lexical order.
- system installs default to PostgreSQL.
- SQLite remains supported for local development and explicit portable validation flows.

## Deployment Modes

The release/install tooling supports two modes:

- `portable`
  - single-directory local validation and CI-friendly installs
- `system`
  - OS-standard production layout with external config, data, log, and run directories

`system` mode is the production standard.

## Recommended Production Flow

Build release artifacts:

```bash
./bin/build.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

From an extracted official server bundle root, generate a production-grade native install:

```bash
./install.sh --mode system
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Mode system
```

From `<product-root>`, validate the generated production config before service registration:

```bash
./current/bin/validate-config.sh --home <product-root>
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\current\bin\validate-config.ps1 -Home <product-root>
```

After installation, the installed runtime also exposes:

- `<product-root>/current/bin/validate-config.sh`
- `<product-root>\current\bin\validate-config.ps1`
- `<product-root>/current/bin/backup.sh`
- `<product-root>\current\bin\backup.ps1`
- `<product-root>/current/bin/restore.sh`
- `<product-root>\current\bin\restore.ps1`
- `<product-root>/current/bin/support-bundle.sh`
- `<product-root>\current\bin\support-bundle.ps1`

Then continue with:

- [Production Deployment](./docs/getting-started/production-deployment.md) for Docker Compose, Helm, and native rollout guidance
- [Service Management](./docs/operations/service-management.md) for systemd, launchd, and Windows Service registration

## Local Development

Use the managed development entrypoints:

```bash
./bin/start-dev.sh
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

For product-mode development from the repository root, the workspace package also exposes:

```bash
pnpm tauri:dev
pnpm server:dev
```

`pnpm tauri:dev` launches the portal desktop product path through the packaged product entrypoint.
`pnpm server:dev` launches the full server development workspace: backend APIs, the admin dev server, the portal dev server, and the unified Pingora web host.

For portal desktop source builds, the supervised `router-product-service` sidecar now gets a longer debug warm-up window before startup is treated as unhealthy. If a slower Windows machine still needs more time, set `SDKWORK_ROUTER_RUNTIME_HEALTH_TIMEOUT_MS=<milliseconds>` before `pnpm tauri:dev`. Startup failures now report the router binary path, runtime config file, stdout/stderr log files, and the exact health probe URLs that were checked.

Use `pnpm --dir apps/sdkwork-router-portal server:start` when you need the standalone integrated `router-product-service` CLI for deployment-oriented server runtime flags.

## Desktop IPC Permissions

Desktop IPC permissions are defined explicitly per Tauri product:

- `apps/sdkwork-router-portal/src-tauri/capabilities/main.json`
- `apps/sdkwork-router-admin/src-tauri/capabilities/main.json`
- `console/src-tauri/capabilities/main.json`

Maintenance rules:

- every `#[tauri::command]` exposed in `src-tauri/src/main.rs` must also be listed in `src-tauri/build.rs` through `tauri_build::AppManifest::commands(...)`
- the main-window capability stays limited to `windows: ["main"]` and must not use `core:default`
- portal remote runtime windows such as admin and gateway stay outside the desktop IPC capability surface
- `pnpm server:dev` remains outside the Tauri permission model entirely

Use this contract test after changing desktop commands or window APIs:

```bash
node --test tests/tauri-permission-contract.test.mjs
```

On Windows, the managed Tauri entrypoints also pin `Visual Studio 17 2022` and use short managed target/temp directories to avoid CMake/MSBuild generator drift and deep path failures during development builds.

The local development contract is documented in:

- [Quickstart](./docs/getting-started/quickstart.md)
- [Script Lifecycle](./docs/getting-started/script-lifecycle.md)

## Release And Verification

Release build/package guidance:

- [Release Builds](./docs/getting-started/release-builds.md)
- [Online Release](./docs/getting-started/online-release.md)

Common verification baseline:

```bash
node --test scripts/check-router-docs-safety.test.mjs
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/release-workflow.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
cargo test -p sdkwork-api-config --test config_loading
cargo test -p router-product-service
pnpm --dir docs build
```
