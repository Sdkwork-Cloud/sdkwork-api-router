# Provider Health Snapshots Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Persist provider health snapshots, supervise runtime health in the background, expose recent health history through admin APIs, and let routing fall back to persisted health evidence when live runtime status is unavailable.

**Architecture:** Extend the routing domain and admin stores with a provider-health snapshot aggregate, add app-layer capture logic on top of existing runtime status normalization, and start a lightweight periodic capture task from standalone services using runtime config. Reuse the existing provider-to-instance binding semantics so persisted health stays consistent with live gateway dispatch.

**Tech Stack:** Rust, Axum, Tokio, sqlx, React, TypeScript

---

### Task 1: Add failing domain and storage tests for health snapshots

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- `ProviderHealthSnapshot` can be constructed in the routing domain
- SQLite can persist and list snapshots ordered by newest first
- PostgreSQL can persist and list snapshots when the integration URL is available

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`

Expected: FAIL because the snapshot domain model and storage methods do not exist yet.

### Task 2: Add failing app-extension, routing, config, and admin tests

**Files:**
- Modify: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- runtime status capture persists provider health snapshots
- routing falls back to the latest persisted snapshot when live status is unavailable
- admin APIs list recent snapshots
- runtime config parses the new snapshot interval setting

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-config -q`

Expected: FAIL because no snapshot capture, config field, or admin endpoint exists yet.

### Task 3: Implement the routing-domain snapshot model

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

**Step 1: Add `ProviderHealthSnapshot`**

Use a provider-centric record with `observed_at_ms` and optional message.

**Step 2: Run focused test**

Run: `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`

Expected: PASS

### Task 4: Extend store contracts and persistence

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

**Step 1: Add store methods**

Support inserting one snapshot and listing snapshots ordered newest first.

**Step 2: Add migrations**

Create `routing_provider_health` with the fields required by the domain model.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`

Expected: PASS

### Task 5: Implement app-layer snapshot capture

**Files:**
- Modify: `crates/sdkwork-api-app-extension/src/lib.rs`
- Modify: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`

**Step 1: Build provider snapshot capture**

Join:

- providers
- extension instances
- live runtime statuses

into provider health snapshots using the same matching semantics already used by routing.

**Step 2: Add persistence helper**

Persist the captured snapshots through the admin store.

**Step 3: Run focused test**

Run: `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`

Expected: PASS

### Task 6: Add runtime config and background supervision

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/gateway-service/src/main.rs`

**Step 1: Add interval config**

Parse `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS` with default `0`.

**Step 2: Start periodic capture**

Spawn a Tokio task that does:

- immediate capture on startup
- `tokio::time::interval` loop when enabled

**Step 3: Run config tests**

Run: `cargo test -p sdkwork-api-config -q`

Expected: PASS

### Task 7: Expose snapshots through admin APIs and routing fallback

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

**Step 1: Add admin endpoint**

Expose `GET /admin/routing/health-snapshots`.

**Step 2: Add routing fallback**

When live runtime status is missing for a candidate, use the latest persisted provider health snapshot before treating health as unknown.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 8: Update console runtime page and docs

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-runtime/src/index.tsx`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Extend console types and SDK**

Add health snapshot types and admin client call.

**Step 2: Upgrade runtime page**

Render recent snapshot history and health state.

**Step 3: Refresh docs**

Move persisted health snapshots from "gap" to implemented, while keeping official upstream probes, quota, and SLO routing in the remaining-gap list.

**Step 4: Run console typecheck**

Run: `pnpm --dir console -r typecheck`

Expected: PASS

### Task 9: Run full verification and commit

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Format and verify**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`
- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-config -q`
- `pnpm --dir console -r typecheck`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-domain-routing -p sdkwork-api-storage-core -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres -p sdkwork-api-app-extension -p sdkwork-api-app-routing -p sdkwork-api-interface-admin -p sdkwork-api-config --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-provider-health-snapshots-design.md docs/plans/2026-03-14-provider-health-snapshots-implementation.md crates/sdkwork-api-domain-routing crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-app-extension crates/sdkwork-api-app-routing crates/sdkwork-api-interface-admin crates/sdkwork-api-config services/admin-api-service services/gateway-service console/packages/sdkwork-api-types console/packages/sdkwork-api-admin-sdk console/packages/sdkwork-api-runtime README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md
git commit -m "feat: add provider health snapshot supervision"
```
