# Coordinated Standalone Config Rollout Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add shared-store coordinated standalone-config reload rollout so one admin node can trigger and observe on-demand config reload across active gateway, admin, and portal nodes.

**Architecture:** Persist standalone-config rollout operations and per-node participants in the admin store, derive aggregate status from participant rows, and teach standalone runtime supervision to process rollout work by reusing its existing local config reload pass. Keep config ownership local to each node; only the trigger and progress ledger are shared.

**Tech Stack:** Rust, Axum, Tokio, shared `AdminStore` trait with SQLite and PostgreSQL backends, existing standalone runtime supervision and live-handle reload path

---

## Chunk 1: RED Coverage

### Task 1: Add failing tests for standalone-config rollout creation and execution

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add a failing admin API rollout creation test**

Write a test that:

1. seeds active gateway, admin, portal, and stale nodes
2. calls `POST /admin/runtime-config/rollouts`
3. optionally filters by `service_kind`
4. expects the rollout snapshot to contain only active matching participants

- [ ] **Step 2: Add a failing runtime-supervision rollout execution test**

Write a test that:

1. creates two standalone services with local config files and live handles
2. changes their local config on disk
3. creates one shared standalone-config rollout
4. expects both participants to become `succeeded`
5. expects the live handles to reflect the reloaded config

- [ ] **Step 3: Add a failing timeout aggregation test**

Write a test that seeds an expired standalone-config rollout with a stale pending participant and expects aggregate status `timed_out`.

- [ ] **Step 4: Run the focused RED tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_config_rollout_creation_snapshots_active_nodes -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_standalone_config_rollout_workers_apply_shared_reload -q`

Expected: FAIL because there is no standalone-config rollout schema, no admin API, and no runtime-supervision worker path.

## Chunk 2: Store Schema

### Task 2: Persist standalone-config rollout operations

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/admin_store_trait.rs`

- [ ] **Step 1: Add rollout record structs and store trait methods**

Define:

- standalone-config rollout record
- standalone-config rollout participant record

And add `AdminStore` methods for:

- insert or find or list standalone-config rollouts
- insert or list participants
- list pending participants for one node
- transition one participant status

- [ ] **Step 2: Extend SQLite schema and mappings**

Add tables and indexes for standalone-config rollouts and participants.

- [ ] **Step 3: Extend PostgreSQL schema and mappings**

Mirror SQLite support using `CREATE TABLE IF NOT EXISTS`.

- [ ] **Step 4: Add a small store regression test**

Assert the SQLite store can round-trip standalone-config rollout rows and participants.

- [ ] **Step 5: Run focused storage verification**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-storage-sqlite --test admin_store_trait -q`

Expected: PASS.

## Chunk 3: Runtime Supervision And Aggregation

### Task 3: Reuse the local config reload pass for shared rollout work

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`

- [ ] **Step 1: Extract the existing config reload pass into a reusable helper**

The helper should:

1. load the next config snapshot
2. compute which live-reloadable changes apply
3. rebuild or swap stores, JWTs, listeners, and secret managers when needed
4. return a small outcome describing whether anything changed

- [ ] **Step 2: Add node identity to standalone runtime supervision**

Allow runtime supervision to know its node id when coordinated rollout participation is enabled.

- [ ] **Step 3: Add heartbeat and pending-rollout processing inside runtime supervision**

For the configured node id:

1. heartbeat the shared node record
2. claim one pending standalone-config participant for the current node
3. run one forced reload pass
4. persist `succeeded` or `failed`

- [ ] **Step 4: Add standalone-config rollout app helpers**

Implement:

- create rollout
- list rollouts
- find rollout
- aggregate status computation

- [ ] **Step 5: Wire node ids into all standalone services**

Gateway, admin, and portal should all participate in standalone-config rollout.

- [ ] **Step 6: Verify the runtime RED tests turn green**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_standalone_config_rollout_workers_apply_shared_reload -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_standalone_config_rollout_times_out_when_participant_stays_pending -q`

Expected: PASS.

## Chunk 4: Admin API

### Task 4: Add standalone-config rollout endpoints

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Add create, list, and get endpoints**

Implement:

- `POST /admin/runtime-config/rollouts`
- `GET /admin/runtime-config/rollouts`
- `GET /admin/runtime-config/rollouts/{rollout_id}`

- [ ] **Step 2: Map JWT subject into `created_by`**

- [ ] **Step 3: Return rollout snapshots with per-node participants**

- [ ] **Step 4: Verify the admin RED test turns green**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_config_rollout_creation_snapshots_active_nodes -q`

Expected: PASS.

## Chunk 5: Docs And Full Verification

### Task 5: Update docs and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-coordinated-standalone-config-rollout-design.md`
- Create: `docs/plans/2026-03-15-coordinated-standalone-config-rollout-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/operations/health-and-metrics.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-multi-node-coordinated-runtime-rollout-design.md`

- [ ] **Step 1: Document standalone-config rollout as implemented**

- [ ] **Step 2: Update operations docs for node identity and rollout endpoints**

- [ ] **Step 3: Re-run focused rollout tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes cluster_runtime_config_rollout_creation_snapshots_active_nodes -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision cluster_standalone_config_rollout_workers_apply_shared_reload -q`

- [ ] **Step 4: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
