# Extension Runtime Reload Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an explicit extension runtime reload control plane that shuts down managed runtimes, invalidates the gateway extension-host cache, rebuilds discovered runtime state, and exposes the result through admin APIs plus the runtime console.

**Architecture:** Keep runtime reload process-local and admin-triggered. Add a gateway-owned reload orchestrator because the discovered extension-host cache already lives in `sdkwork-api-app-gateway`, then let the admin interface call that orchestrator and reuse existing runtime-status DTOs from `sdkwork-api-app-extension`.

**Tech Stack:** Rust, Axum, serde, tokio, existing extension host registries, TypeScript, React

---

## Chunk 1: Failing Runtime Reload Tests

### Task 1: Add failing coverage for explicit runtime reload

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Add a failing gateway reload test**

Add a serial native-dynamic test that prepares a signed mock package, calls a new gateway reload entry point twice, and expects the lifecycle log to contain `init`, `shutdown`, `init`.

- [ ] **Step 2: Run the focused gateway test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch configured_extension_host_reload_rebuilds_native_dynamic_runtimes -q`
Expected: FAIL because no explicit reload entry point exists yet.

- [ ] **Step 3: Add a failing admin reload route test**

Add an authenticated admin test that posts to `/admin/extensions/runtime-reloads`, expects a success payload with runtime counts and status details, and proves the runtime is active after reload.

- [ ] **Step 4: Run the focused admin test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes extension_runtime_reload_endpoint_rebuilds_runtime_state -q`
Expected: FAIL because the route does not exist yet.

## Chunk 2: Gateway Reload Orchestration

### Task 2: Implement explicit configured-host reload in the gateway layer

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Add a typed configured-host build report**

Track:

- total discovered package count
- loadable package count
- rebuilt `ExtensionHost`

- [ ] **Step 2: Add an explicit reload entry point**

Implement a public gateway function that:

1. shuts down connector runtimes
2. shuts down native-dynamic runtimes
3. clears the configured host cache
4. rebuilds the host from the current discovery policy
5. returns the discovery counts

- [ ] **Step 3: Reuse the same build path for cached and reload flows**

Avoid duplicate discovery logic between normal cache misses and explicit reload.

- [ ] **Step 4: Run the focused gateway reload test**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch configured_extension_host_reload_rebuilds_native_dynamic_runtimes -q`
Expected: PASS.

## Chunk 3: Admin API And Console Surface

### Task 3: Expose reload through the admin API and runtime console

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-runtime/src/index.tsx`

- [ ] **Step 1: Add admin reload response types and route wiring**

Expose `POST /admin/extensions/runtime-reloads` and return:

- `discovered_package_count`
- `loadable_package_count`
- `active_runtime_count`
- `reloaded_at_ms`
- `runtime_statuses`

- [ ] **Step 2: Extend shared console types and admin SDK**

Add runtime-status DTOs plus a `reloadExtensionRuntimes()` admin client helper.

- [ ] **Step 3: Upgrade the runtime page**

Render current managed runtime statuses, show the latest reload summary, and add a reload button that refreshes the page state.

- [ ] **Step 4: Run focused admin verification**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes extension_runtime_reload_endpoint_rebuilds_runtime_state -q`
Expected: PASS.

- [ ] **Step 5: Run console verification**

Run: `pnpm --dir console -r typecheck`
Expected: PASS.

## Chunk 4: Documentation And Final Verification

### Task 4: Document runtime reload as implemented and verify the workspace

**Files:**
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-implementation.md`

- [ ] **Step 1: Update architecture and compatibility docs**

Move explicit runtime reload from remaining gap to implemented capability, while keeping fully automatic watcher-driven hot reload as future work.

- [ ] **Step 2: Run targeted Rust verification**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS.

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
