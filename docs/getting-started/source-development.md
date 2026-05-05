# Source Development

This page documents the recommended source-based startup flows for Windows, Linux, and macOS.

If you only need one page that explains what each script does and how the full lifecycle works, see [Script Lifecycle](/getting-started/script-lifecycle). This page stays focused on developer-facing source workflows.

## Port Sets You Need To Know

There are two distinct default-port sets in this repository.

### Managed source-script defaults

These are the defaults used by the updated source helper layer:

| Surface | Default bind | Purpose |
|---|---|---|
| gateway | `127.0.0.1:9980` | OpenAI-compatible `/v1/*` traffic |
| admin | `127.0.0.1:9981` | operator control plane |
| portal | `127.0.0.1:9982` | public auth, dashboard, usage, billing, and API key lifecycle |
| web host | `0.0.0.0:9983` | Pingora public admin and portal delivery |
| admin web app | `127.0.0.1:5173` | standalone browser admin dev server |
| portal web app | `127.0.0.1:5174` | standalone browser portal dev server |

### Built-in binary defaults

If you run the service binaries directly without helper-script overrides, the services still keep their built-in defaults:

- gateway: `127.0.0.1:8080`
- admin: `127.0.0.1:8081`
- portal: `127.0.0.1:8082`

## Local Config Root

The standalone services read config from the local SDKWork config root:

- Linux and macOS: `~/.sdkwork/router/`
- Windows: `%USERPROFILE%\\.sdkwork\\router\\`

If the directory is empty, the services still start with built-in defaults.

## Choose A Source Startup Path

### Option 1: Managed source startup

Use this when you want predictable runtime directories, PID tracking, a formatted startup summary, and a default unified browser entrypoint.

Linux or macOS:

```bash
./bin/start-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

If you want to enter the product-mode flows directly from the repository root, the workspace package also exposes:

```bash
pnpm tauri:dev
pnpm server:dev
```

`pnpm tauri:dev` launches the portal desktop product path through the shared root entrypoint.
`pnpm server:dev` launches the full server development workspace through the same root entrypoint.
That server workspace uses proxy-dev mode, so backend APIs, the admin Vite server, the portal Vite server, and the unified Pingora web host all start together.

Portal desktop source builds now wait longer before the supervised sidecar is marked unhealthy, which reduces false negatives on slower Windows development hosts. If you still need a larger warm-up budget, set `SDKWORK_ROUTER_RUNTIME_HEALTH_TIMEOUT_MS=<milliseconds>` before `pnpm tauri:dev`. When startup really fails, the desktop runtime error now prints the resolved router binary path, `router.yaml`, stdout/stderr log files, and the exact health probe URLs.

If you need the standalone integrated `router-product-service` CLI instead of the dev workspace contract, use `pnpm --dir apps/sdkwork-router-portal server:start`.

Characteristics:

- default mode is preview, so the built-in Pingora host becomes the primary browser entrypoint
- runtime state is written under `artifacts/runtime/dev/`
- startup logs print unified URLs, direct service URLs, bootstrap profile guidance, and log file paths
- stop with `./bin/stop-dev.sh` or `.\bin\stop-dev.ps1`

Primary URLs after startup:

- unified admin: `http://127.0.0.1:9983/admin/`
- unified portal: `http://127.0.0.1:9983/portal/`
- unified gateway health: `http://127.0.0.1:9983/api/v1/health`
- direct gateway health: `http://127.0.0.1:9980/health`
- direct admin health: `http://127.0.0.1:9981/admin/health`
- direct portal health: `http://127.0.0.1:9982/portal/health`

If you explicitly want the standalone Vite dev servers instead of the unified Pingora host:

Linux or macOS:

```bash
./bin/start-dev.sh --browser
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser
```

### Option 2: Raw source workspace startup

Use this when you want the original workspace launchers and foreground processes in the current terminal.

Windows:

Browser mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

Preview mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Preview
```

Desktop mode:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -Tauri
```

Linux or macOS:

Browser mode:

```bash
node scripts/dev/start-workspace.mjs
```

Preview mode:

```bash
node scripts/dev/start-workspace.mjs --preview
```

Desktop mode:

```bash
node scripts/dev/start-workspace.mjs --tauri
```

Mode behavior:

- browser mode:
  - backend on `9980`, `9981`, `9982`
  - admin app on `http://127.0.0.1:5173/admin/`
  - portal app on `http://127.0.0.1:5174/portal/`
- preview mode:
  - backend on `9980`, `9981`, `9982`
  - unified web host on `http://127.0.0.1:9983/admin/` and `http://127.0.0.1:9983/portal/`
- tauri mode:
  - backend on `9980`, `9981`, `9982`
  - portal desktop shell plus unified Pingora browser access on `9983`

The raw workspace launcher now also prints a startup summary showing:

- mode
- frontend access
- direct service access
- active bootstrap profile guidance

## Partial Startup

Backend services only:

```bash
node scripts/dev/start-stack.mjs
```

Admin app only:

```bash
node scripts/dev/start-admin.mjs
```

Portal desktop only:

```bash
node scripts/dev/start-portal.mjs --tauri
```

Admin desktop only, for explicit admin-shell development:

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
node scripts/dev/start-web.mjs --bind 0.0.0.0:9983
```

Windows PowerShell wrappers are also available:

- `scripts/dev/start-servers.ps1`
- `scripts/dev/start-workspace.ps1`

## Storage Choices

### SQLite development

For raw helper scripts, when you do not pass `--database-url`, the services follow the local config-root behavior:

- Linux and macOS: `~/.sdkwork/router/sdkwork-api-router.db`
- Windows: `%USERPROFILE%\\.sdkwork\\router\\sdkwork-api-router.db`

For `bin/start-dev.*`, the managed dev runtime uses its own writable database path under:

- `artifacts/runtime/dev/data/sdkwork-api-router-dev.db`

### PostgreSQL development

Use a shared PostgreSQL connection string across admin, gateway, and portal:

```bash
node scripts/dev/start-workspace.mjs \
  --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 `
  -DatabaseUrl "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
```

Managed source startup also accepts database overrides:

```bash
./bin/start-dev.sh --database-url "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
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
export SDKWORK_DATABASE_URL="sqlite://sdkwork-api-router.db"
```

Run the admin app:

```bash
pnpm --dir apps/sdkwork-router-admin dev
```

Run the portal desktop shell from source:

```bash
pnpm --dir apps/sdkwork-router-portal tauri:dev
```

Run the admin desktop shell from source when you are explicitly developing the admin-owned Tauri host:

```bash
pnpm --dir apps/sdkwork-router-admin tauri:dev
```

Run the standalone portal app:

```bash
pnpm --dir apps/sdkwork-router-portal dev
```

Run the Pingora public web host:

```bash
SDKWORK_WEB_BIND=0.0.0.0:9983 cargo run -p router-web-service
```

## Development Identity Bootstrap

The local development flows do not rely on fixed built-in emails or passwords.

Instead:

- development identities come from the active bootstrap profile
- local `dev` profile data lives in `data/identities/dev.json`
- the default `prod` bootstrap profile does not seed development identities

The gateway itself does not use a seeded username and password. Use a portal-issued API key for authenticated gateway traffic.

## Recommended Verification

Before or after local startup, the standard checks are:

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm test:user-center-standard
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

`pnpm test:user-center-standard` is the canonical identity-governance lane for the repository. It executes the shared `sdkwork-appbase` user-center and independent validation-plugin contracts before the router portal bridge test so local-private deployments and external `sdkwork-cloud-app-api` handshakes stay aligned to the same AuthToken or AccessToken standard.

## Common Notes

- use `bin/start-dev.*` when you want a stable single-port entrypoint and managed runtime state
- use `scripts/dev/start-workspace.*` when you want source-native foreground control
- use browser mode for Vite iteration and preview mode when you want one external browser URL
- if a `998x` port is still occupied on your machine, override it explicitly with the corresponding bind flag or environment variable

## Next Steps

- full script responsibilities and lifecycle:
  - [Script Lifecycle](/getting-started/script-lifecycle)
- compiling artifacts:
  - [Build and Packaging](/getting-started/build-and-packaging)
- deployment-oriented binaries:
  - [Release Builds](/getting-started/release-builds)
- architecture deep dive:
  - [Software Architecture](/architecture/software-architecture)
