# Build and Tooling

This page summarizes the toolchains, commands, and helper scripts used across the repository.

For the complete startup and shutdown lifecycle, see [Script Lifecycle](/getting-started/script-lifecycle).

## Required Toolchains

| Tool | Used for |
|---|---|
| Rust + Cargo | all backend crates and standalone services |
| Node.js 20+ | docs runtime, frontend runtimes, developer scripts |
| pnpm 10+ | admin app, portal app, and docs dependency management |
| Tauri CLI | optional desktop development and packaging |

## Primary Commands

| Command | Scope | Purpose |
|---|---|---|
| `cargo build -p gateway-service` | backend | compile one service |
| `node scripts/release/run-service-release-build.mjs --target <triple>` | official local release | build the governed release binary set with managed target and temp paths |
| `cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service` | backend | compile standalone release binaries |
| `cargo build --release -p router-product-service` | backend | compile the integrated product host runtime |
| `cargo test --workspace -q -j 1` | backend | workspace regression suite |
| `cargo fmt --all --check` | backend | formatting verification |
| `pnpm --dir apps/sdkwork-router-admin build` | frontend | admin production build |
| `pnpm --dir apps/sdkwork-router-admin typecheck` | frontend | admin TypeScript verification |
| `pnpm --dir apps/sdkwork-router-portal build` | frontend | standalone portal production build |
| `pnpm --dir apps/sdkwork-router-portal typecheck` | frontend | standalone portal TypeScript verification |
| `node scripts/check-router-product.mjs` | product verification | run the governed product gate, including portal/admin checks, browser smoke, Tauri capability audit, desktop release-like payload staging, and a loopback-safe server dry-run plan |
| `node scripts/check-tauri-capabilities.mjs` | desktop | verify that every desktop capability includes the generated invoke permissions, the window permissions required by the approved desktop bridge, and that Tauri globals plus `@tauri-apps/api/window` stay centralized in the governed bridge files |
| `node scripts/check-browser-storage-governance.mjs` | frontend governance | verify that browser `localStorage` and `sessionStorage` access stays centralized in the approved governed store modules |
| `./bin/build.sh --verify-release` / `.\bin\build.ps1 -VerifyRelease` | official local release verification | run the governed local release path: docs build, packaged runtime smoke, and release governance preflight |
| `node scripts/prepare-router-portal-desktop-runtime.mjs` | desktop | stage the portal desktop sidecar payload under `bin/portal-rt/router-product/` |
| `pnpm --dir apps/sdkwork-router-portal tauri:build` | desktop | package the official portal desktop installer |
| `pnpm --dir docs build` | docs | build docs site |
| `pnpm --dir docs typecheck` | docs | VitePress config typecheck |

## Frontend Governance

- Portal and admin desktop capabilities must stay behind the approved bridge modules. `node scripts/check-tauri-capabilities.mjs` enforces that product rule across the repository.
- Portal authentication is session-only by contract. `readPortalSessionToken`, `persistPortalSessionToken`, and `clearPortalSessionToken` delegate to the canonical user-center session store, while `usePortalAuthStore` stays in memory and restores identity through `hydrate()`.
- Sensitive browser persistence is session-scoped by contract. Session tokens, one-time plaintext API-key reveals, and user-center local preference drafts that contain personal contact bindings must use governed session-storage modules or volatile in-memory fallbacks only. Legacy `localStorage` copies may exist only as migration sources and must be cleared once the governed session store takes ownership.
- Non-sensitive browser preferences may persist across sessions only through dedicated governed store modules. Locale and similar shell preferences must not read or write `localStorage` inline from i18n or feature entrypoints.
- `node scripts/check-browser-storage-governance.mjs` is the static enforcement point for that browser-persistence contract. If a feature needs browser storage, it must land in an approved governed store module first instead of inlining storage access inside the feature surface.
- Any frontend package that source-links TypeScript files with explicit `.ts` extensions from workspace-standard packages must enable `"allowImportingTsExtensions": true` in its local `tsconfig.json`. This is part of the governed typecheck contract, not an optional convenience flag.

## Official Local Release Verification

`./bin/build.sh --verify-release` and `.\bin\build.ps1 -VerifyRelease` are the canonical repository entrypoints for local official release validation.

That governed path always includes:

- the docs site build because the docs surface is part of the public release contract
- the packaged installed-runtime and platform smoke lanes driven from the release asset tree
- the local `release governance preflight` step via `node scripts/release/run-release-governance-checks.mjs --profile preflight`

Do not combine `--skip-docs` with `--verify-release`. If you need an ad hoc engineering build without docs governance, use the normal build mode instead of the official local release-verification mode.

## Startup Script Matrix

| Script | Layer | Lifecycle role | Notes |
|---|---|---|---|
| `./bin/start-dev.sh` / `.\bin\start-dev.ps1` | managed dev | start a managed development runtime | defaults to preview mode and the unified `9983` browser entrypoint |
| `./bin/stop-dev.sh` / `.\bin\stop-dev.ps1` | managed dev | stop the managed development runtime | uses the managed PID file under `artifacts/runtime/dev/` |
| `./bin/build.sh` / `.\bin\build.ps1` | managed release | build releasable artifacts | writes release output and Rust build artifacts |
| `./bin/install.sh` / `.\bin\install.ps1` | managed release | create the product root | stages `current/`, `releases/<version>/`, `config/`, `data/`, `log/`, and `run/` |
| `./bin/start.sh` / `.\bin\start.ps1` | managed release | start the installed release runtime | starts `router-product-service` from the active `current/release-manifest.json` payload and prints unified plus direct URLs |
| `./bin/stop.sh` / `.\bin\stop.ps1` | managed release | stop the installed release runtime | uses the product-root `run/` PID file |
| `node scripts/dev/start-workspace.mjs` | raw source dev | start backend services plus browser surfaces | defaults to browser mode with direct frontend dev servers |
| `node scripts/dev/start-workspace.mjs --preview` | raw source dev | start backend services plus unified Pingora host | uses the `9983` unified host |
| `node scripts/dev/start-workspace.mjs --tauri` | raw source dev | start backend services plus the portal desktop shell | still exposes browser access through the unified host |
| `node scripts/dev/start-stack.mjs` | raw source dev | start backend services only | backend ports default to `9980`, `9981`, and `9982` |
| `node scripts/dev/start-admin.mjs` | raw source dev | start the admin app only | browser or Tauri |
| `node scripts/dev/start-portal.mjs` | raw source dev | start the portal app only | browser or Tauri |
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
| raw release Rust binaries | `target/release/` |
| managed official release Rust binaries | `$CARGO_TARGET_DIR/<triple>/release/` |
| admin browser assets | `apps/sdkwork-router-admin/dist/` |
| portal web assets | `apps/sdkwork-router-portal/dist/` |
| staged portal desktop sidecar payload | `bin/portal-rt/router-product/` |
| docs site build | `docs/.vitepress/dist/` |
| managed dev runtime | `artifacts/runtime/dev/` |
| managed install root | `artifacts/install/sdkwork-api-router/` |

For official release service builds, prefer `node scripts/release/run-service-release-build.mjs --target <triple>` over raw `cargo build --release`.
That runner reuses the governed release build plan, prints the resolved release directory, and automatically keeps Windows on managed short target and temp roots instead of the repository-local default path depth.

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

### Official local release verification

```bash
./bin/build.sh --verify-release
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
