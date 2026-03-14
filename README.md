# sdkwork-api-server

[中文文档](./README.zh-CN.md)

SDKWork API Server is an Axum-based OpenAI-compatible gateway, control plane, extension host, and public self-service portal built with Rust, React, pnpm, and Tauri.

The repository is organized around four runtime surfaces:

- `gateway-service`
  - OpenAI-compatible `/v1/*` gateway
- `admin-api-service`
  - operator-only `/admin/*` control plane
- `portal-api-service`
  - public `/portal/*` registration, login, workspace inspection, and API key issuance
- `console/`
  - browser-accessible React shell that also runs inside the Tauri desktop host

## Current State

What is already live:

- OpenAI-compatible gateway routing for the current `/v1/*` surface documented in [docs/api/compatibility-matrix.md](./docs/api/compatibility-matrix.md)
- stateful and stateless execution paths
- admin APIs for tenants, projects, API keys, channels, proxy providers, credentials, models, routing, usage, billing, and extensions
- public portal APIs:
  - `POST /portal/auth/register`
  - `POST /portal/auth/login`
  - `GET /portal/auth/me`
  - `GET /portal/workspace`
  - `GET /portal/api-keys`
  - `POST /portal/api-keys`
- SQLite and PostgreSQL persistence through one shared storage contract
- encrypted secret persistence with:
  - `database_encrypted`
  - `local_encrypted_file`
  - `os_keyring`
- extension runtime support for:
  - `builtin`
  - `connector`
  - `native_dynamic`
- React console packages for:
  - portal SDK
  - portal auth
  - portal dashboard
  - admin workspace
  - channel management
  - routing
  - runtime inspection
  - usage and billing
- browser and Tauri-friendly hash routes:
  - `#/portal/register`
  - `#/portal/login`
  - `#/portal/dashboard`
  - `#/admin`

## Supported Platforms

This repository is intended to run on:

- Windows
- Linux
- macOS

The Rust services are cross-platform. The React console runs in any modern browser on all three platforms. The Tauri desktop shell is optional and uses the same frontend routes as the browser console.

## Prerequisites

Required:

- Rust stable with Cargo
- Node.js 20+
- pnpm 10+

Optional:

- PostgreSQL 15+ for PostgreSQL-backed deployments
- Tauri CLI for desktop development:

```bash
cargo install tauri-cli
```

## Repository Layout

```text
.
|-- crates/                      # domain, app, interface, provider, runtime, storage crates
|-- services/
|   |-- admin-api-service/       # standalone admin HTTP service
|   |-- gateway-service/         # standalone OpenAI-compatible gateway HTTP service
|   `-- portal-api-service/      # standalone public portal HTTP service
|-- console/                     # React + pnpm workspace + optional Tauri desktop shell
|-- scripts/
|   `-- dev/                     # cross-platform startup helpers
|-- docs/                        # architecture notes, plans, compatibility docs
`-- README.zh-CN.md              # Chinese operational guide
```

## Default Ports

| Surface | Default Bind | Purpose |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI-compatible `/v1/*` traffic |
| admin | `127.0.0.1:8081` | operator control plane |
| portal | `127.0.0.1:8082` | public auth, workspace, and API key lifecycle |
| console | `127.0.0.1:5173` | browser and Tauri frontend dev server |

## Recommended Startup Commands

Use the unified workspace launcher first. Keep the lower-level helpers for partial startup or debugging.

| Workflow | Windows | Linux / macOS |
|---|---|---|
| full stack in browser mode | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1` | `node scripts/dev/start-workspace.mjs` |
| full stack in desktop mode and keep browser access | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri` | `node scripts/dev/start-workspace.mjs --tauri` |
| full stack dry run | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun` | `node scripts/dev/start-workspace.mjs --dry-run` |
| backend services only | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1` | `node scripts/dev/start-stack.mjs` |
| browser console only | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1` | `node scripts/dev/start-console.mjs` |
| Tauri only | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Tauri` | `node scripts/dev/start-console.mjs --tauri` |
| preview production console build | `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1 -Preview` | `node scripts/dev/start-console.mjs --preview` |

Notes:

- `start-workspace` launches backend services and the console together
- `start-workspace --tauri` keeps the browser UI reachable at `http://127.0.0.1:5173`
- `start-servers.ps1` still opens separate PowerShell windows on Windows for backend-only workflows
- the Node-based launchers are the portable path for Windows, Linux, and macOS

## Quick Start With SQLite

This is the fastest end-to-end local setup.

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

### Linux or macOS

```bash
node scripts/dev/start-workspace.mjs
```

Open:

- `http://127.0.0.1:5173/#/portal/register`
- `http://127.0.0.1:5173/#/portal/login`
- `http://127.0.0.1:5173/#/portal/dashboard`
- `http://127.0.0.1:5173/#/admin`

If you prefer separate terminals or windows for backend and frontend, use the lower-level `start-stack`, `start-servers`, and `start-console` helpers instead.

## Desktop Mode With Browser Access

If you want the Tauri host and a normal browser open at the same time, use the unified launcher in desktop mode.

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

### Linux or macOS

```bash
node scripts/dev/start-workspace.mjs --tauri
```

Why this works:

- `tauri dev` uses the Vite dev server as the frontend source
- the same Vite URL remains accessible from a normal browser
- portal registration, login, dashboard, and admin routes are identical in browser and Tauri

## Public Portal Walkthrough

Once the backend services and the console are running:

1. open `http://127.0.0.1:5173/#/portal/register`
2. register a portal account
3. land on `#/portal/dashboard`
4. create a gateway API key for `live`, `test`, or `staging`
5. copy the plaintext key immediately
6. call the gateway with that key

Example:

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer skw_live_your_key_here"
```

The portal list endpoint intentionally does not return plaintext keys again. Plaintext values are returned only at creation time.

## PostgreSQL Startup

To use PostgreSQL, point the full stack at the same database URL.

### Windows

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

### Linux or macOS

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

SQLite and PostgreSQL migrations are applied automatically at startup.

## Partial Startup Helpers

Use these when you want more control than the unified launcher provides.

### Backend Services Only

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-servers.ps1
```

Linux or macOS:

```bash
node scripts/dev/start-stack.mjs
```

### Console Only

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-console.ps1
```

Linux or macOS:

```bash
node scripts/dev/start-console.mjs
```

### Raw Command Fallback

If you prefer not to use helper scripts, these are the direct commands.

Windows PowerShell:

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

```powershell
$env:SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

```powershell
pnpm --dir console install
pnpm --dir console dev
```

```powershell
pnpm --dir console tauri:dev
```

Linux or macOS:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p admin-api-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p gateway-service
```

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
cargo run -p portal-api-service
```

```bash
pnpm --dir console install
pnpm --dir console dev
```

```bash
pnpm --dir console tauri:dev
```

## Browser Console Workspace

Install frontend dependencies:

```bash
pnpm --dir console install
```

Run the browser dev server:

```bash
pnpm --dir console dev
```

Typecheck all packages:

```bash
pnpm --dir console -r typecheck
```

Build production assets:

```bash
pnpm --dir console build
```

Preview the production build:

```bash
pnpm --dir console preview
```

The dev server proxies these paths by default:

- `/admin` -> `http://127.0.0.1:8081`
- `/portal` -> `http://127.0.0.1:8082`
- `/v1` -> `http://127.0.0.1:8080`

Override them if needed:

- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_GATEWAY_PROXY_TARGET`

## Health and Metrics

Health endpoints:

- gateway: `http://127.0.0.1:8080/health`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`

Metrics endpoints:

- gateway: `http://127.0.0.1:8080/metrics`
- admin: `http://127.0.0.1:8081/metrics`
- portal: `http://127.0.0.1:8082/metrics`

Example:

```bash
curl http://127.0.0.1:8082/portal/health
curl http://127.0.0.1:8082/metrics
```

## Runtime Configuration

Important environment variables:

- `SDKWORK_GATEWAY_BIND`
- `SDKWORK_ADMIN_BIND`
- `SDKWORK_PORTAL_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_ADMIN_JWT_SIGNING_SECRET`
- `SDKWORK_PORTAL_JWT_SIGNING_SECRET`
- `SDKWORK_SECRET_BACKEND`
- `SDKWORK_CREDENTIAL_MASTER_KEY`
- `SDKWORK_SECRET_LOCAL_FILE`
- `SDKWORK_SECRET_KEYRING_SERVICE`
- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`
- `SDKWORK_EXTENSION_PATHS`
- `SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS`
- `SDKWORK_EXTENSION_TRUSTED_SIGNERS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS`
- `SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS`

Supported secret backends:

- `database_encrypted`
- `local_encrypted_file`
- `os_keyring`

## Extension and Package Conventions

The extension architecture distinguishes three names:

- runtime ID
  - `sdkwork.provider.openrouter`
  - `sdkwork.channel.openai`
- distribution package name
  - `sdkwork-provider-openrouter`
  - `sdkwork-channel-openai`
- Rust crate name
  - `sdkwork-api-ext-provider-openrouter`
  - `sdkwork-api-ext-channel-openai`

This keeps channel and proxy-provider concerns explicit while allowing config-driven loading, runtime discovery, and future external packaging.

## Architecture Summary

Backend layering:

- interface or controller crates under `crates/sdkwork-api-interface-*`
- app or service crates under `crates/sdkwork-api-app-*`
- repository or storage crates under `crates/sdkwork-api-storage-*`

Frontend layering:

- root shell composition in `console/src/`
- reusable business packages in `console/packages/`
- public portal split into:
  - `sdkwork-api-portal-sdk`
  - `sdkwork-api-portal-auth`
  - `sdkwork-api-portal-user`

## Intentionally Missing For Now

The current system is usable end-to-end, but these areas remain roadmap work:

- multi-user portal workspaces and invitations
- password reset and email delivery
- OAuth or SSO
- standalone MySQL or libsql deployment flows
- hot reload for discovered external extensions

## Reference Docs

Operational and architectural detail:

- [docs/api/compatibility-matrix.md](./docs/api/compatibility-matrix.md)
- [docs/architecture/runtime-modes.md](./docs/architecture/runtime-modes.md)
- [docs/plans/2026-03-14-public-portal-cross-platform-design.md](./docs/plans/2026-03-14-public-portal-cross-platform-design.md)
- [docs/plans/2026-03-14-unified-workspace-launch-design.md](./docs/plans/2026-03-14-unified-workspace-launch-design.md)

## Verification Commands

Fresh verification baseline:

```bash
node --check scripts/dev/workspace-launch-lib.mjs
node --check scripts/dev/start-workspace.mjs
node --test scripts/dev/tests/start-workspace.test.mjs
node scripts/dev/start-workspace.mjs --dry-run
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
```
