# Build and Tooling

This page summarizes the toolchains, commands, and helper scripts used across the repository.

For the complete startup and shutdown lifecycle, see [Script Lifecycle](/getting-started/script-lifecycle).

## Required Toolchains

| Tool | Used for |
|---|---|
| Rust + Cargo | all backend crates and standalone services |
| Node.js 20+ | docs runtime, frontend runtimes, developer scripts |
| pnpm 10+ | admin console, portal app, and docs dependency management |
| Tauri CLI | optional desktop development and packaging |

## Primary Commands

| Command | Scope | Purpose |
|---|---|---|
| `cargo build -p gateway-service` | backend | compile one service |
| `cargo build --release -p admin-api-service -p gateway-service -p portal-api-service` | backend | compile release binaries |
| `cargo test --workspace -q -j 1` | backend | workspace regression suite |
| `cargo fmt --all --check` | backend | formatting verification |
| `pnpm --dir apps/sdkwork-router-admin build` | frontend | admin production build |
| `pnpm --dir apps/sdkwork-router-admin typecheck` | frontend | admin TypeScript verification |
| `pnpm --dir apps/sdkwork-router-portal build` | frontend | standalone portal production build |
| `pnpm --dir apps/sdkwork-router-portal typecheck` | frontend | standalone portal TypeScript verification |
| `pnpm --dir apps/sdkwork-router-admin tauri:build` | desktop | package the admin Tauri app |
| `pnpm --dir docs build` | docs | build docs site |
| `pnpm --dir docs typecheck` | docs | VitePress config typecheck |

## Startup Script Matrix

| Script | Layer | Lifecycle role | Notes |
|---|---|---|---|
| `./bin/start-dev.sh` / `.\bin\start-dev.ps1` | managed dev | start a managed development runtime | defaults to preview mode and the unified `9983` browser entrypoint |
| `./bin/stop-dev.sh` / `.\bin\stop-dev.ps1` | managed dev | stop the managed development runtime | uses the managed PID file under `artifacts/runtime/dev/` |
| `./bin/build.sh` / `.\bin\build.ps1` | managed release | build releasable artifacts | writes release output and Rust build artifacts |
| `./bin/install.sh` / `.\bin\install.ps1` | managed release | create the install home | stages `bin/`, `config/`, `sites/`, `service/`, and `var/` |
| `./bin/start.sh` / `.\bin\start.ps1` | managed release | start the installed release runtime | starts `router-product-service` and prints unified plus direct URLs |
| `./bin/stop.sh` / `.\bin\stop.ps1` | managed release | stop the installed release runtime | uses the install-home PID file |
| `node scripts/dev/start-workspace.mjs` | raw source dev | start backend services plus browser surfaces | defaults to browser mode with direct frontend dev servers |
| `node scripts/dev/start-workspace.mjs --preview` | raw source dev | start backend services plus unified Pingora host | uses the `9983` unified host |
| `node scripts/dev/start-workspace.mjs --tauri` | raw source dev | start backend services plus admin desktop shell | still exposes browser access through the unified host |
| `node scripts/dev/start-stack.mjs` | raw source dev | start backend services only | backend ports default to `9980`, `9981`, and `9982` |
| `node scripts/dev/start-admin.mjs` | raw source dev | start the admin app only | browser or Tauri |
| `node scripts/dev/start-portal.mjs` | raw source dev | start the portal app only | browser-only |
| `node scripts/dev/start-web.mjs --bind 0.0.0.0:9983` | raw source dev | build admin and portal static assets, then expose them through Pingora | useful for preview-style browser validation |
| `scripts/dev/start-workspace.ps1` | raw source dev | Windows PowerShell wrapper for workspace startup | forwards the same bind and mode options |
| `scripts/dev/start-servers.ps1` | raw source dev | Windows PowerShell wrapper for backend-only startup | backend only |

## Default Port Notes

- managed and helper-script startup flows use the `998x` range by default
- raw service binaries still keep their built-in `808x` defaults unless you override them
- admin and portal Vite dev servers stay on `5173` and `5174` in browser mode

## Artifact Locations

| Artifact | Path |
|---|---|
| debug Rust binaries | `target/debug/` |
| release Rust binaries | `target/release/` |
| admin browser assets | `apps/sdkwork-router-admin/dist/` |
| portal web assets | `apps/sdkwork-router-portal/dist/` |
| docs site build | `docs/.vitepress/dist/` |
| managed dev runtime | `artifacts/runtime/dev/` |
| managed install home | `artifacts/install/sdkwork-api-router/current/` |

## Recommended Verification Sets

### Documentation-only changes

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
```

### Frontend and docs changes

```bash
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

### Full repository confidence pass

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
