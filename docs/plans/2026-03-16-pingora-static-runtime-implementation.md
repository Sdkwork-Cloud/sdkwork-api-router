# Pingora Static Runtime Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the legacy `console` runtime path with Pingora-backed public delivery for the independent `sdkwork-router-admin` and `sdkwork-router-portal` applications.

**Architecture:** Introduce a shared Rust runtime-host layer that decides whether a request should be served from the admin site, portal site, or proxied to admin/portal/gateway APIs. Reuse that layer in a standalone web service for server mode and in the admin Tauri host so both runtime modes expose the same external URL contract.

**Tech Stack:** Rust, Pingora, Tauri, pnpm, Vite, Node test runner

---

## Chunk 1: Runtime host and static routing

### Task 1: Define the public routing contract

**Files:**
- Create: `crates/sdkwork-api-runtime-host/tests/web_runtime_routing.rs`
- Modify: `crates/sdkwork-api-runtime-host/src/lib.rs`

- [ ] Add tests that describe how `/`, `/portal/*`, `/admin/*`, `/api/portal/*`, `/api/admin/*`, and `/api/v1/*` should resolve.
- [ ] Make the tests fail before implementation.
- [ ] Implement pure routing and static file resolution helpers in `sdkwork-api-runtime-host`.
- [ ] Re-run the focused Rust tests until they pass.

### Task 2: Add Pingora-backed serving

**Files:**
- Modify: `crates/sdkwork-api-runtime-host/Cargo.toml`
- Modify: `crates/sdkwork-api-runtime-host/src/lib.rs`

- [ ] Build a Pingora service that serves admin and portal static files with SPA fallbacks.
- [ ] Proxy `/api/admin/*`, `/api/portal/*`, and `/api/v1/*` to the respective Rust services.
- [ ] Expose an embeddable startup API for Tauri and a fixed-bind startup API for standalone server mode.

## Chunk 2: Runtime entrypoints

### Task 3: Add standalone server-mode web entrypoint

**Files:**
- Create: `services/router-web-service/Cargo.toml`
- Create: `services/router-web-service/src/main.rs`
- Modify: `Cargo.toml`

- [ ] Add a new workspace service that starts the shared Pingora runtime host on a public bind address.
- [ ] Wire service configuration through environment variables with sensible local defaults.

### Task 4: Move Tauri ownership to the admin app

**Files:**
- Create: `apps/sdkwork-router-admin/src-tauri/Cargo.toml`
- Create: `apps/sdkwork-router-admin/src-tauri/build.rs`
- Create: `apps/sdkwork-router-admin/src-tauri/src/main.rs`
- Create: `apps/sdkwork-router-admin/src-tauri/tauri.conf.json`
- Modify: `apps/sdkwork-router-admin/package.json`

- [ ] Create an admin-owned Tauri host that starts the embedded runtime host on launch.
- [ ] Keep the admin UI independent while exposing the shared Pingora site externally.

## Chunk 3: Frontend builds and developer workflow

### Task 5: Make static builds mount under stable prefixes

**Files:**
- Modify: `apps/sdkwork-router-admin/vite.config.ts`
- Modify: `apps/sdkwork-router-admin/package.json`
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`
- Modify: `apps/sdkwork-router-portal/vite.config.ts`
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`

- [ ] Build admin assets under `/admin/`.
- [ ] Build portal assets under `/portal/`.
- [ ] Add architecture tests that lock those mount prefixes in place.

### Task 6: Replace console-centric startup scripts

**Files:**
- Create: `scripts/dev/start-admin.mjs`
- Create: `scripts/dev/start-web.mjs`
- Modify: `scripts/dev/start-workspace.mjs`
- Modify: `scripts/dev/workspace-launch-lib.mjs`
- Modify: `scripts/dev/start-workspace.ps1`
- Modify: `scripts/dev/tests/start-workspace.test.mjs`

- [ ] Add a standalone admin launcher.
- [ ] Add a Pingora web launcher for preview/server flows.
- [ ] Update the workspace launcher so browser mode starts admin plus portal, while preview and Tauri modes start the Pingora host instead of `console`.

## Chunk 4: Documentation

### Task 7: Remove `console` from the primary runtime story

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/index.md`
- Modify: `docs/reference/repository-layout.md`
- Modify: `docs/architecture/software-architecture.md`
- Modify: `docs/getting-started/source-development.md`
- Modify: `docs/getting-started/release-builds.md`

- [ ] Rewrite runtime and build docs around admin, portal, and the Pingora web service.
- [ ] Mark `console` as legacy/non-primary instead of a recommended entrypoint.
