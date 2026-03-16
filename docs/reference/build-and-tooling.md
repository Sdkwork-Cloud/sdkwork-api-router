# Build and Tooling

This page summarizes the toolchains, commands, and helper scripts used across the repository.

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
| `pnpm --dir console build` | frontend | admin console production build |
| `pnpm --dir console -r typecheck` | frontend | admin console TypeScript verification |
| `pnpm --dir apps/sdkwork-router-portal build` | frontend | standalone portal production build |
| `pnpm --dir apps/sdkwork-router-portal typecheck` | frontend | standalone portal TypeScript verification |
| `pnpm --dir console tauri:build` | desktop | package Tauri app |
| `pnpm --dir docs build` | docs | build docs site |
| `pnpm --dir docs typecheck` | docs | VitePress config typecheck |

## Developer Startup Scripts

| Script | Purpose |
|---|---|
| `node scripts/dev/start-workspace.mjs` | start services plus admin console and portal app |
| `node scripts/dev/start-workspace.mjs --tauri` | start services plus Tauri admin shell and browser portal |
| `node scripts/dev/start-stack.mjs` | start backend services only |
| `node scripts/dev/start-console.mjs` | start admin console only |
| `node scripts/dev/start-portal.mjs` | start portal app only |
| `node scripts/dev/start-web.mjs --bind 0.0.0.0:3001` | build admin and portal static assets, then expose them through the Pingora public host |
| `scripts/dev/start-workspace.ps1` | Windows PowerShell wrapper for full workspace startup |
| `scripts/dev/start-servers.ps1` | Windows PowerShell wrapper for backend-only startup |
| `scripts/dev/start-console.ps1` | Windows PowerShell wrapper for admin-console-only startup |

## Artifact Locations

| Artifact | Path |
|---|---|
| debug Rust binaries | `target/debug/` |
| release Rust binaries | `target/release/` |
| admin console assets | `console/dist/` |
| portal web assets | `apps/sdkwork-router-portal/dist/` |
| docs site build | `docs/.vitepress/dist/` |
| Tauri bundles | Tauri platform-specific output under `console/src-tauri/target/` |

## Recommended Verification Sets

### Documentation-only changes

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
```

### Frontend and docs changes

```bash
pnpm --dir console -r typecheck
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

### Full repository confidence pass

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```
