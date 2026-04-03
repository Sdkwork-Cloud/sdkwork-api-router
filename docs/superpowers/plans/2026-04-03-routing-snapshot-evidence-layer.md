# Routing Snapshot And Evidence Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add durable compiled routing snapshots and stronger routing evidence contracts so admin diagnostics can inspect the effective route state, selected candidate, rejected candidates, and fallback posture without introducing a separate routing semantics model.

**Architecture:** reuse the current route-selection pipeline as the canonical source of truth and extract a compile step that derives an effective routing snapshot from global policy, project preferences, group-bound routing profile, and catalog-backed provider candidates. Persist that compiled view through the existing storage seam, then enrich routing decisions and admin simulation responses with explicit evidence fields that point back to the compiled snapshot and explain fallback behavior in a durable schema.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, serde, existing `sdkwork-api-*` crates, cargo tests

---

### Task 1: Define routing snapshot and evidence domain contracts

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Test: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

- [ ] **Step 1: Write the failing test**

Add domain tests that expect:

- `CompiledRoutingSnapshotRecord` to retain scope, derivation sources, effective strategy, ranked provider order, thresholds, and preferred region
- `RoutingDecision` and `RoutingDecisionLog` to expose `compiled_routing_snapshot_id` and `fallback_reason`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-domain-routing -- --nocapture`
Expected: FAIL because compiled snapshot and evidence fields do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:

- `CompiledRoutingSnapshotRecord`
- optional `compiled_routing_snapshot_id` on `RoutingDecision`
- optional `compiled_routing_snapshot_id` on `RoutingDecisionLog`
- optional `fallback_reason` on `RoutingDecision`
- optional `fallback_reason` on `RoutingDecisionLog`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-domain-routing -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-domain-routing/src/lib.rs crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs
git commit -m "feat: add routing snapshot and evidence domain contracts"
```

### Task 2: Expand routing storage seams and backend persistence

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`

- [ ] **Step 1: Write the failing test**

Add storage tests that expect:

- compiled routing snapshots round-trip through SQLite
- decision logs round-trip `compiled_routing_snapshot_id` and `fallback_reason`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -- --nocapture`
Expected: FAIL because snapshot persistence and new evidence columns do not exist.

- [ ] **Step 3: Write minimal implementation**

Add:

- routing store methods for insert and list compiled routing snapshots
- `ai_compiled_routing_snapshots`
- decision-log persistence for `compiled_routing_snapshot_id` and `fallback_reason`
- immediate PostgreSQL parity with the same schema and row codecs

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs
git commit -m "feat: persist compiled routing snapshots"
```

### Task 3: Compile and persist effective routing snapshots during route selection

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Test: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Write the failing test**

Add route-selection tests that expect:

- route compilation persists a snapshot keyed to tenant, project, group, capability, and route key
- snapshots record matched policy, project preference scope, routing profile, effective thresholds, and effective preferred region
- decisions expose `compiled_routing_snapshot_id` and explicit `fallback_reason` when selection degrades or falls back

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture`
Expected: FAIL because route selection does not yet compile or persist snapshots and does not expose explicit fallback reasons.

- [ ] **Step 3: Write minimal implementation**

Implement:

- a compile step that derives the effective routing policy from canonical sources
- candidate ordering derived from catalog entries plus the effective policy overlay
- snapshot upsert before final candidate assessment
- explicit fallback-reason propagation for geo-affinity, weighted, SLO-degraded, and static fallback outcomes

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-app-routing/src/lib.rs crates/sdkwork-api-app-routing/tests/simulate_route.rs
git commit -m "feat: compile routing snapshots during selection"
```

### Task 4: Expose snapshot inspection and richer admin observability

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `docs/api-reference/admin-api.md`

- [ ] **Step 1: Write the failing test**

Add admin route tests that expect:

- `GET /admin/routing/snapshots`
- routing simulation response includes `compiled_routing_snapshot_id`
- routing simulation response includes `selected_candidate`, `rejected_candidates`, and `fallback_reason`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
Expected: FAIL because the route and enriched response contract do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:

- list handler for compiled routing snapshots
- enriched admin simulation response derived from persisted decision evidence
- admin API reference updates for the new response fields and snapshot endpoint

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs docs/api-reference/admin-api.md
git commit -m "feat: expose routing snapshots and enriched diagnostics"
```

### Task 5: Capture phase notes and run focused regressions

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md`

- [ ] **Step 1: Update phase notes**

Document:

- compiled routing snapshots as the canonical derived routing state
- explicit fallback evidence fields
- remaining Billing 2.0 handoff assumptions

- [ ] **Step 2: Run focused verification**

Run:

```bash
cargo test -p sdkwork-api-domain-routing -- --nocapture
cargo test -p sdkwork-api-storage-sqlite --test routing_policies -- --nocapture
cargo test -p sdkwork-api-app-routing --test simulate_route -- --nocapture
cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture
cargo test -p sdkwork-api-storage-postgres --no-run
```

Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add docs crates
git commit -m "feat: add compiled routing snapshots and routing evidence"
```
