# Source Development

This page documents the recommended source-based startup flows for Windows, Linux, and macOS.

For most contributors and evaluators, this is the primary entry point after installation.

## Default Ports

| Surface | Default Bind | Purpose |
|---|---|---|
| gateway | `127.0.0.1:8080` | OpenAI-compatible `/v1/*` traffic |
| admin | `127.0.0.1:8081` | operator control plane |
| portal | `127.0.0.1:8082` | public auth, dashboard, usage, billing, and API key lifecycle |
| web host | `0.0.0.0:3001` | Pingora public admin and portal delivery |
| admin web app | `127.0.0.1:5173` | standalone browser admin dev server |
| portal web app | `127.0.0.1:5174` | standalone browser portal dev server |

## Local Config Root

The standalone services read config from the local SDKWork config root:

- Linux and macOS: `~/.sdkwork/router/`
- Windows: `%USERPROFILE%\\.sdkwork\\router\\`

If the directory is empty, the services still start with built-in defaults.

## Fastest End-to-End Startup

### Windows

Browser mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Desktop mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

### Linux or macOS

Browser mode:

```bash
node scripts/dev/start-workspace.mjs
```

Desktop mode:

```bash
node scripts/dev/start-workspace.mjs --tauri
```

If you want the Pingora public host on a specific external bind while using preview or desktop mode:

```bash
node scripts/dev/start-workspace.mjs --preview --web-bind 0.0.0.0:3001
```

In desktop mode, the admin app stays inside Tauri while the shared Pingora web host exposes both admin and portal for external browser access.

After startup, the most useful local URLs are:

- gateway: `http://127.0.0.1:8080`
- admin: `http://127.0.0.1:8081/admin/health`
- portal: `http://127.0.0.1:8082/portal/health`
- admin app: `http://127.0.0.1:5173/admin/`
- portal app: `http://127.0.0.1:5174/portal/`
- Pingora portal: `http://127.0.0.1:3001/portal/`
- Pingora admin: `http://127.0.0.1:3001/admin/`

For LAN access, use the same `/admin/` and `/portal/` paths on the machine's local IP, for example:

- Pingora admin: `http://192.168.1.10:3001/admin/`
- Pingora portal: `http://192.168.1.10:3001/portal/`

## Partial Startup

Backend services only:

```bash
node scripts/dev/start-stack.mjs
```

Admin app only:

```bash
node scripts/dev/start-admin.mjs
```

Desktop admin only:

```bash
node scripts/dev/start-admin.mjs --tauri
```

Portal app only:

```bash
node scripts/dev/start-portal.mjs
```

Public web host only:

```bash
node scripts/dev/start-web.mjs
```

Public web host on a specific external bind:

```bash
node scripts/dev/start-web.mjs --bind 0.0.0.0:3001
```

Windows PowerShell wrappers are also available:

- `scripts/dev/start-servers.ps1`
- `scripts/dev/start-workspace.ps1`

## Storage Choices

### SQLite development

SQLite is the default local database.

When you use the helper scripts without `--database-url`, the services use the local config root defaults:

- Linux and macOS: `~/.sdkwork/router/sdkwork-api-server.db`
- Windows: `%USERPROFILE%\\.sdkwork\\router\\sdkwork-api-server.db`

No extra setup is required. Database creation and migrations happen on startup.

### PostgreSQL development

Use a shared PostgreSQL connection string across admin, gateway, and portal:

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server"
```

## Raw Source Commands

If you want to run the surfaces individually instead of using the helper scripts, use the following commands.

Run the Rust services directly:

```bash
cargo run -p admin-api-service
```

```bash
cargo run -p gateway-service
```

```bash
cargo run -p portal-api-service
```

If you want to override the local default explicitly:

```bash
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-server.db"
```

Run the admin app:

```bash
pnpm --dir apps/sdkwork-router-admin dev
```

Run Tauri from source:

```bash
pnpm --dir apps/sdkwork-router-admin tauri:dev
```

Run the standalone portal app:

```bash
pnpm --dir apps/sdkwork-router-portal dev
```

Run the Pingora public web host:

```bash
SDKWORK_WEB_BIND=0.0.0.0:3001 cargo run -p router-web-service
```

## Recommended Verification

Before or after local startup, the standard checks are:

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

## Next Steps

- compiling artifacts:
  - [Build and Packaging](/getting-started/build-and-packaging)
- deployment-oriented binaries:
  - [Release Builds](/getting-started/release-builds)
- architecture deep dive:
  - [Software Architecture](/architecture/software-architecture)
