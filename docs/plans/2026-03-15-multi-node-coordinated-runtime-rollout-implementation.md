# Multi-Node Coordinated Runtime Rollout Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add shared-store coordinated extension-runtime rollout so one admin node can trigger scoped runtime reload across all active gateway and admin nodes.

**Architecture:** Persist node heartbeats and rollout participant records in the admin store, derive rollout status from participant progress, and let each standalone node poll for and execute rollout work addressed to itself. Keep the existing process-local reload endpoint intact and add separate cluster rollout endpoints.

**Tech Stack:** Rust, Axum, Tokio, shared `AdminStore` trait with SQLite and PostgreSQL backends, existing extension-runtime reload orchestration

---

## Chunk 1: RED Rollout Coordination Tests

### Task 1: Add failing coverage for cluster rollout creation and execution

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add a failing admin API rollout creation test**

Write a test that:

1. records active gateway and admin nodes in the store
2. calls `POST /admin/extensions/runtime-rollouts`
3. expects participant rows for both nodes and aggregate status `pending`

- [ ] **Step 2: Add a failing runtime worker completion test**

Write a test that:

1. seeds a shared store with two active nodes and one pending rollout
2. starts two rollout workers with different node ids
3. expects both participants to become `succeeded`
4. expects aggregate rollout status `succeeded`

- [ ] **Step 3: Add a failing timeout aggregation test**

Write a test that seeds a stale pending participant past the deadline and expects aggregate rollout status `timed_out`.

- [ ] **Step 4: Run the focused RED tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_rollout_creation_snapshots_active_nodes -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_runtime_rollout_workers_complete_shared_rollout -q`

Expected: FAIL because there is no shared rollout schema, no admin API, and no node worker yet.

## Chunk 2: Store Schema And Rollout Records

### Task 2: Persist node heartbeats and rollout participant state

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/admin_store_trait.rs`

- [ ] **Step 1: Add rollout record structs and store trait methods**

Define:

- service node record
- rollout record
- rollout participant record

And add `AdminStore` methods for:

- upserting a node heartbeat
- listing active nodes
- inserting rollout operations
- inserting rollout participants
- listing rollout operations
- listing rollout participants for one rollout
- claiming or updating one participant for one node

- [ ] **Step 2: Extend SQLite schema and query mappings**

Add tables for nodes, rollouts, and rollout participants with the required indexes and upsert behavior.

- [ ] **Step 3: Extend PostgreSQL schema and query mappings**

Mirror the SQLite support with `CREATE TABLE IF NOT EXISTS` plus `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`.

- [ ] **Step 4: Add a small store trait regression test**

Assert the SQLite store can round-trip a heartbeat plus rollout records.

- [ ] **Step 5: Run focused storage tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-storage-sqlite --test admin_store_trait -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_rollout_creation_snapshots_active_nodes -q`

Expected: admin test still FAILS, storage test PASSES.

## Chunk 3: Rollout Aggregation And Admin API

### Task 3: Add cluster rollout app logic and admin routes

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Add rollout aggregation helpers**

Implement helpers that:

- resolve active rollout nodes
- create rollout ids and deadlines
- compute aggregate rollout status from participants

- [ ] **Step 2: Add admin cluster rollout endpoints**

Implement:

- `POST /admin/extensions/runtime-rollouts`
- `GET /admin/extensions/runtime-rollouts`
- `GET /admin/extensions/runtime-rollouts/:rollout_id`

Preserve the existing local `runtime-reloads` endpoint unchanged.

- [ ] **Step 3: Verify the admin RED test turns green**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_rollout_creation_snapshots_active_nodes -q`

Expected: PASS.

## Chunk 4: Background Rollout Worker

### Task 4: Teach gateway and admin services to process shared rollout work

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`

- [ ] **Step 1: Add node heartbeat and pending-rollout polling to runtime supervision**

Use the live store handle so the worker survives store replacement.

- [ ] **Step 2: Add per-node rollout processing**

For gateway and admin nodes:

1. mark one pending participant as applying
2. run the existing local scoped reload
3. persist succeeded or failed participant state

Portal should not participate.

- [ ] **Step 3: Generate or read a stable-enough node id**

Use `SDKWORK_SERVICE_INSTANCE_ID` when present, otherwise synthesize one from service kind plus process startup identity.

- [ ] **Step 4: Wire gateway and admin services to pass their node id into runtime supervision**

- [ ] **Step 5: Verify the runtime RED test turns green**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_runtime_rollout_workers_complete_shared_rollout -q`

Expected: PASS.

## Chunk 5: Docs And Full Verification

### Task 5: Update docs to close the remaining rollout gap and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-multi-node-coordinated-runtime-rollout-design.md`
- Create: `docs/plans/2026-03-15-multi-node-coordinated-runtime-rollout-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/operations/health-and-metrics.md`
- Modify: `docs/plans/2026-03-15-targeted-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`

- [ ] **Step 1: Document multi-node rollout as implemented for explicit extension-runtime control**

- [ ] **Step 2: Update operations docs for node identity, heartbeats, and rollout endpoints**

- [ ] **Step 3: Re-run focused rollout tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_rollout_creation_snapshots_active_nodes -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_runtime_rollout_workers_complete_shared_rollout -q`

- [ ] **Step 4: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
