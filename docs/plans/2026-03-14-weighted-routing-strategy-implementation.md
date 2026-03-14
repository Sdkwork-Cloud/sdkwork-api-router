# Weighted Routing Strategy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a first-class routing strategy model and implement seeded weighted-random provider selection without regressing the existing health-aware deterministic routing path.

**Architecture:** Extend `RoutingPolicy` with an explicit strategy enum persisted in the existing admin stores, then refactor `sdkwork-api-app-routing` to dispatch selection through strategy-specific logic. Keep candidate assessment shared across strategies so future quota-aware and SLO-aware routing can reuse the same assessment pipeline.

**Tech Stack:** Rust, serde, Axum, sqlx, React, TypeScript

---

### Task 1: Add failing routing-domain tests for strategy metadata

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

**Step 1: Write the failing test**

Add tests that prove:

- `RoutingPolicy` defaults to `deterministic_priority`
- `RoutingPolicy` can be constructed with `weighted_random`
- `RoutingDecision` can carry `selection_seed`

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`

Expected: FAIL because strategy and selection-seed support do not exist yet.

### Task 2: Add failing storage and admin API contract tests

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- SQLite routing policy persistence round-trips `strategy`
- PostgreSQL routing policy persistence round-trips `strategy` when a database URL is present
- admin routing policy creation accepts `strategy`
- admin route simulation accepts and returns `selection_seed`

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: FAIL because stores and admin payloads do not expose strategy or selection seed yet.

### Task 3: Add failing app-routing tests for seeded weighted selection

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- a `weighted_random` policy picks providers reproducibly from a supplied seed
- disabled candidates are excluded from weighted selection
- unhealthy candidates are excluded when healthy candidates remain
- weighted routing still falls back to deterministic ordering if only one candidate is eligible

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-routing --test simulate_route -q`

Expected: FAIL because the routing service only supports deterministic top-ranked selection.

### Task 4: Implement routing-domain strategy support

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

**Step 1: Add `RoutingStrategy` enum**

Serialize with `snake_case`, default to `deterministic_priority`, and provide a builder for `RoutingPolicy`.

**Step 2: Extend `RoutingDecision`**

Add optional `selection_seed` with a builder method.

**Step 3: Run focused test**

Run: `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`

Expected: PASS

### Task 5: Implement persistence and admin API plumbing

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Evolve the stores**

Persist `strategy` on `routing_policies` with safe defaults for existing rows.

**Step 2: Extend admin payloads**

Allow `CreateRoutingPolicyRequest.strategy` and `RoutingSimulationRequest.selection_seed`.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 6: Implement seeded weighted selection in app routing

**Files:**
- Modify: `crates/sdkwork-api-app-routing/Cargo.toml`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

**Step 1: Add selection dispatch**

Route assessed candidates through:

- deterministic priority selection
- weighted-random selection

based on the matched policy strategy.

**Step 2: Add seed handling**

Support an explicit seed helper for tests and admin simulation, while allowing runtime-generated seeds for the existing gateway call path.

**Step 3: Add eligibility rules**

Weighted selection should exclude:

- unavailable candidates
- unhealthy candidates when a healthier candidate is available

and should otherwise use resolved weights in stable ranked order.

**Step 4: Run focused test**

Run: `cargo test -p sdkwork-api-app-routing --test simulate_route -q`

Expected: PASS

### Task 7: Update console types and docs

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-routing/src/index.tsx`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Extend types**

Expose `selection_seed` in the routing simulation result.

**Step 2: Update routing page**

Render the seed when present so weighted decisions are reproducible from the UI.

**Step 3: Refresh docs**

Document weighted routing as implemented and move persisted health supervision plus quota or SLO routing back to the remaining-gap list.

**Step 4: Run console typecheck**

Run: `pnpm --dir console -r typecheck`

Expected: PASS

### Task 8: Run full verification and commit

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Format and verify**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `pnpm --dir console -r typecheck`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-weighted-routing-strategy-design.md docs/plans/2026-03-14-weighted-routing-strategy-implementation.md crates/sdkwork-api-domain-routing crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-app-routing crates/sdkwork-api-interface-admin console/packages/sdkwork-api-types console/packages/sdkwork-api-routing README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md Cargo.lock
git commit -m "feat: add weighted routing strategy support"
```
