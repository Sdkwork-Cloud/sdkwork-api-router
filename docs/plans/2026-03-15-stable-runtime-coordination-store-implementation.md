# Stable Runtime Coordination Store Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep multi-node rollout coordination coherent across standalone `database_url` hot swaps by introducing a dedicated pinned coordination-store handle for rollout heartbeats, workers, and admin rollout APIs.

**Architecture:** Split request-serving live-store usage from rollout coordination-store usage. Keep the live store hot-swappable for ordinary runtime dependencies, but pin one coordination store from process startup and route all rollout state through it. Apply the same model to extension-runtime rollout, standalone-config rollout, admin API rollout handlers, and service wiring.

**Tech Stack:** Rust, Axum, Tokio, shared `AdminStore` trait with SQLite and PostgreSQL backends, existing runtime-supervision and rollout helpers

---

## Chunk 1: RED Coverage

### Task 1: Add failing regression tests for coordination-store split after live-store swap

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add a failing extension-runtime rollout admin test**

Write a test that:

1. builds an admin router with a reloadable live store
2. seeds active rollout-capable nodes only in the startup store
3. swaps the live store handle to a replacement database before the request
4. calls `POST /admin/extensions/runtime-rollouts`
5. expects the request to still succeed because rollout creation should use the pinned coordination store

- [ ] **Step 2: Add a failing standalone-config rollout admin test**

Write a test that repeats the same pattern for `POST /admin/runtime-config/rollouts`.

- [ ] **Step 3: Add a failing runtime worker test**

Write a test that:

1. starts an extension-runtime rollout worker with a reloadable live store
2. waits for initial heartbeats in the startup store
3. swaps the live store handle to a replacement database
4. creates an extension-runtime rollout in the startup store
5. expects the worker to keep consuming that rollout from the pinned coordination store

- [ ] **Step 4: Run focused RED verification**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes extension_runtime_rollout_creation_uses_coordination_store_after_live_store_swap -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes runtime_config_rollout_creation_uses_coordination_store_after_live_store_swap -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_runtime_rollout_worker_keeps_startup_coordination_store_after_live_store_swap -q`

Expected: FAIL because rollout handlers and extension-runtime workers still follow the live-store handle.

## Chunk 2: Runtime And Admin Wiring

### Task 2: Split coordination-store usage from live-store usage

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`

- [ ] **Step 1: Add explicit coordination-store support to extension-runtime rollout supervision**

Pass a dedicated `Arc<dyn AdminStore>` coordination store into the worker instead of re-snapshotting the live store on every tick.

- [ ] **Step 2: Add explicit coordination-store state to the admin API**

Keep ordinary handlers on the live request-serving store, but route rollout create/list/get handlers through the pinned coordination store.

- [ ] **Step 3: Wire startup coordination stores in all standalone services**

Derive one startup coordination store from the initial store and pass it explicitly into:

- admin API state
- extension-runtime rollout supervision
- standalone-config runtime supervision reload handles

- [ ] **Step 4: Re-run the focused tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes extension_runtime_rollout_creation_uses_coordination_store_after_live_store_swap -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes runtime_config_rollout_creation_uses_coordination_store_after_live_store_swap -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_runtime_rollout_worker_keeps_startup_coordination_store_after_live_store_swap -q`

Expected: PASS.

## Chunk 3: Documentation And Verification

### Task 3: Document pinned coordination-store semantics and verify the workspace

**Files:**
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/operations/health-and-metrics.md`
- Modify: `docs/plans/2026-03-15-coordinated-standalone-config-rollout-design.md`
- Modify: `docs/plans/2026-03-15-multi-node-coordinated-runtime-rollout-design.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`

- [ ] **Step 1: Clarify the coordination-store lifetime**

Document that distributed rollout coordination stays on the startup coordination store for the current process lifetime, even if the request-serving live store hot-swaps.

- [ ] **Step 2: Re-run focused rollout tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision -q`

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
