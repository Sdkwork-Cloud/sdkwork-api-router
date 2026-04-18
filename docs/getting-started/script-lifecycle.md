# Script Lifecycle

This page explains what each startup script does, when to use it, what state it writes, and how the full startup and shutdown lifecycle works.

Use this page together with:

- [Quickstart](/getting-started/quickstart) for the fastest first run
- [Source Development](/getting-started/source-development) for raw source workflows
- [Release Builds](/getting-started/release-builds) for production-oriented packaging and deployment

## Two Script Layers

SDKWork ships two different script layers. They solve different problems.

For repository ergonomics, root-level `start.*`, `build.*`, `install.*`, and `stop.*` scripts are thin compatibility wrappers. They exist so common entrypoint names stay available from the repository root, but the real lifecycle implementation remains owned by `bin/*`.

### `scripts/dev/*`

These are the raw source-oriented launchers.

Use them when:

- you are actively developing inside the repository
- you want fine-grained control over which surfaces start
- you are comfortable managing foreground processes in the current terminal

Characteristics:

- run directly from the source tree
- mostly stay in the foreground
- ideal for iterative debugging and narrow workflows
- do not maintain a managed runtime home with PID files and log rotation

### `bin/*`

These are the managed orchestration scripts.

Use them when:

- you want a predictable development or release lifecycle
- you want one command to prepare runtime directories, track the main PID, and print a formatted startup summary
- you want a script set that can be used consistently across Windows, Linux, and macOS

Characteristics:

- create and reuse runtime homes
- write PID files, logs, and environment files
- support dry-run, foreground, and service-manager usage
- print unified URLs, direct service URLs, and bootstrap identity guidance after successful startup

## Script Catalog

| Script | Scope | Primary purpose | Runtime state | How it stops |
|---|---|---|---|---|
| `bin/build.sh` / `bin/build.ps1` | release | build release binaries, browser assets, docs, and desktop bundles | `artifacts/release/` and Rust target output | exits when the build finishes |
| `bin/install.sh` / `bin/install.ps1` | release | install the built release runtime into a product root | `artifacts/install/sdkwork-api-router/` by default | exits when installation finishes |
| `bin/start.sh` / `bin/start.ps1` | release | start the installed `router-product-service` runtime | product-root `config/`, `log/`, `run/`, plus `current/release-manifest.json` | `bin/stop.sh` / `bin/stop.ps1`, or service manager stop |
| `bin/stop.sh` / `bin/stop.ps1` | release | stop the managed release runtime using the recorded PID | product-root `run/` PID file | exits after the process tree stops |
| `bin/start-dev.sh` / `bin/start-dev.ps1` | managed development | start a managed development runtime with writable local state | `artifacts/runtime/dev/` | `bin/stop-dev.sh` / `bin/stop-dev.ps1`, or `Ctrl+C` in foreground mode |
| `bin/stop-dev.sh` / `bin/stop-dev.ps1` | managed development | stop the managed development runtime | `artifacts/runtime/dev/run/` PID file | exits after the process tree stops |
| `node scripts/prepare-router-portal-desktop-runtime.mjs` | desktop packaging | stage the portal desktop sidecar payload | `bin/portal-rt/router-product/` | exits when staging completes |
| `scripts/dev/start-workspace.mjs` / `.ps1` | raw source development | start backend services plus browser or desktop surfaces | source tree only | `Ctrl+C` in the current terminal |
| `scripts/dev/start-stack.mjs` / `start-servers.ps1` | raw source development | start backend services only | source tree only | `Ctrl+C` in the current terminal |
| `scripts/dev/start-admin.mjs` | raw source development | start the admin browser app or Tauri shell | source tree only | `Ctrl+C` in the current terminal |
| `scripts/dev/start-portal.mjs` | raw source development | start the portal browser app | source tree only | `Ctrl+C` in the current terminal |
| `scripts/dev/start-web.mjs` | raw source development | build admin and portal static assets, then expose them through the Pingora web host | source tree only | `Ctrl+C` in the current terminal |

## Port Model

There are two important default-port sets in this repository.

### Managed script defaults

The managed script layer uses the `998x` range to avoid common local conflicts:

- gateway: `127.0.0.1:9980`
- admin: `127.0.0.1:9981`
- portal: `127.0.0.1:9982`
- unified web host: `127.0.0.1:9983` for `bin/start-dev.*` and `0.0.0.0:9983` for raw preview bindings

### Built-in binary defaults

The standalone service binaries still keep their built-in local defaults unless you override them:

- gateway: `127.0.0.1:8080`
- admin: `127.0.0.1:8081`
- portal: `127.0.0.1:8082`

This distinction matters:

- use `998x` when you are following `bin/*` workflows or the updated source helper defaults
- expect `808x` when you run raw binaries without helper-script overrides

## Development Lifecycle

The recommended development lifecycle is the managed source flow:

### 1. Start the managed development runtime

Linux or macOS:

```bash
./bin/start-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1
```

Default behavior:

- uses a writable SQLite database under `artifacts/runtime/dev/data/`
- defaults to preview mode, so the built-in web host becomes the primary browser entrypoint
- waits for backend and frontend health before reporting success
- prints a formatted startup summary

### 2. Read the startup summary

After startup succeeds, the scripts print:

- unified browser entrypoints:
  - `http://127.0.0.1:9983/admin/`
  - `http://127.0.0.1:9983/portal/`
- unified health entrypoint:
  - `http://127.0.0.1:9983/api/v1/health`
- direct backend URLs:
  - `http://127.0.0.1:9980/health`
  - `http://127.0.0.1:9981/admin/health`
  - `http://127.0.0.1:9982/portal/health`
- development identity bootstrap guidance:
  - identities come from the active bootstrap profile
  - review `data/identities/dev.json` before sharing a local environment

### 3. Optional: switch back to standalone browser dev servers

If you explicitly want the Vite admin and portal dev servers instead of the unified Pingora host:

Linux or macOS:

```bash
./bin/start-dev.sh --browser
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -Browser
```

That mode exposes:

- admin browser app: `http://127.0.0.1:5173/admin/`
- portal browser app: `http://127.0.0.1:5174/portal/`
- direct backend ports on `9980`, `9981`, and `9982`

### 4. Stop the managed development runtime

Linux or macOS:

```bash
./bin/stop-dev.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop-dev.ps1
```

The stop scripts read the managed PID file, stop the owned process tree, and clean up stale runtime state when possible.

## Release Lifecycle

The recommended release lifecycle uses the managed release scripts.

### 1. Build release artifacts

Linux or macOS:

```bash
./bin/build.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1
```

This stage compiles and prepares:

- Rust release binaries
- admin and portal static assets for the server product
- optional docs site assets for documentation validation
- the staged portal desktop `router-product/` sidecar payload
- the official portal desktop bundle
- the native release assets under `artifacts/release/`

### 2. Install the runtime home

Linux or macOS:

```bash
./bin/install.sh
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1
```

This creates a product root, by default:

- `artifacts/install/sdkwork-api-router/`

Key directories:

- `current/`
- `releases/<version>/`
- `config/router.env`
- `config/router.yaml`
- `data/`
- `log/`
- `run/`
- `current/service/systemd/`
- `current/service/launchd/`
- `current/service/windows-service/`

### 3. Review or override runtime configuration

Before starting production, review:

- `config/router.env`
- `config/router.yaml`
- `current/release-manifest.json`

Use `router.yaml` as the canonical runtime configuration file for:

- bind addresses
- database location
- proxy targets

Use `router.env` only for config discovery and fallback values that the config file leaves unset.

`current/release-manifest.json` is generated metadata. It points `current/` at the active immutable payload under `releases/<version>/`.

### 4. Start the release runtime

Linux or macOS:

```bash
./bin/start.sh --home artifacts/install/sdkwork-api-router/current
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -Home .\artifacts\install\sdkwork-api-router\current
```

The release startup scripts:

- start `router-product-service`
- resolve the active binary and static sites from `current/release-manifest.json`
- use the portable product-root SQLite database by default
- wait for unified health checks to pass
- print the same formatted startup summary style as the dev scripts

### 5. Stop the release runtime

Linux or macOS:

```bash
./bin/stop.sh --home artifacts/install/sdkwork-api-router/current
```

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\stop.ps1 -Home .\artifacts\install\sdkwork-api-router\current
```

### 6. Optional: register a service manager entry

From the product root:

- Linux / systemd:
  - `./current/service/systemd/install-service.sh`
  - `./current/service/systemd/uninstall-service.sh`
- macOS / launchd:
  - `./current/service/launchd/install-service.sh`
  - `./current/service/launchd/uninstall-service.sh`
- Windows / Service Control Manager:
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\install-service.ps1`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File .\current\service\windows-service\uninstall-service.ps1`

For service-manager scenarios, use foreground mode:

- `bin/start.sh --foreground --home <product-root>/current`
- `bin/start.ps1 -Foreground -Home <product-root>\current`

## Portal Desktop Lifecycle

The portal desktop product has its own packaging and runtime lifecycle. It does not use `bin/start.sh` or OS service registration for the bundled user-facing app.

Build inputs:

- `pnpm --dir apps/sdkwork-router-admin build`
- `pnpm --dir apps/sdkwork-router-portal build`
- `cargo build --release -p router-product-service`
- `node scripts/prepare-router-portal-desktop-runtime.mjs`

Runtime contract:

- Tauri shell starts the bundled `router-product-service` sidecar
- fixed local shell base URL: `http://127.0.0.1:3001`
- access mode controls the public bind:
  - local-only: `127.0.0.1:3001`
  - shared network: `0.0.0.0:3001`
- mutable desktop runtime state lives in OS-standard app config, data, and log directories
- `desktop-runtime.json` persists shell access mode
- `router.yaml` is the canonical sidecar config file

## Dry-Run Lifecycle

Every managed script supports dry-run. Use it before changing a machine:

- `./bin/build.sh --dry-run`
- `./bin/install.sh --dry-run`
- `./bin/start-dev.sh --dry-run`
- `./bin/start.sh --dry-run`

Windows equivalents:

- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 --dry-run`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\install.ps1 --dry-run`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start-dev.ps1 -DryRun`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\start.ps1 -DryRun`

## Operational Notes

- `bin/start-dev.*` is a source-tree helper. It does not require `bin/build.*` or `bin/install.*`.
- `bin/start.*` is a release-runtime helper. It assumes the install home already exists.
- `bin/stop-dev.*` and `bin/stop.*` only manage processes started through the corresponding managed runtime home and PID file.
- the gateway does not use a seeded username and password. Its primary user-facing auth is a portal-issued API key.
- if you need the source Vite dev servers for frontend iteration, use raw `scripts/dev/*` or `bin/start-dev.* --browser`.
- if you need one browser-accessible entrypoint for demos, QA, or release-style validation, prefer preview mode or the release runtime.
