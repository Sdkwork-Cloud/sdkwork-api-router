# SLO-Aware Routing And Decision Logs Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a first-class `slo_aware` routing strategy, persist routing decision logs, and expose recent routing decisions through admin and console surfaces.

**Architecture:** Extend the routing domain with small SLO and decision-log models, persist decision logs in the shared admin stores, and introduce one application-layer selection entry point that both computes and records routing decisions. Keep the first version deterministic and best-effort so the gateway prefers SLO-compliant candidates without rejecting traffic when no candidate qualifies.

**Tech Stack:** Rust, Axum, Tokio, sqlx, React, TypeScript

---

### Task 1: Add failing routing domain and storage tests

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- `RoutingStrategy` accepts `slo_aware`
- `RoutingCandidateAssessment` can carry `slo_eligible` and `slo_violations`
- a `RoutingDecisionLog` aggregate can be constructed
- SQLite and PostgreSQL can persist and list routing decision logs
- extended routing policies round-trip SLO fields

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`

Expected: FAIL because `slo_aware`, routing decision logs, and SLO fields do not exist yet.

### Task 2: Add failing app-routing, admin, and gateway tests

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- `slo_aware` prefers SLO-compliant candidates over cheaper-or-weighted but violating candidates
- `slo_aware` degrades gracefully when no candidate satisfies the threshold
- admin simulations persist routing decision logs
- real gateway dispatch writes a routing decision log that can be queried through admin APIs

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`

Expected: FAIL because SLO-aware routing and decision log persistence do not exist yet.

### Task 3: Implement routing domain support

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

**Step 1: Add domain types**

Implement:

- `RoutingStrategy::SloAware`
- optional SLO fields on `RoutingPolicy`
- SLO metadata on `RoutingCandidateAssessment`
- `RoutingDecisionSource`
- `RoutingDecisionLog`

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

Support:

- inserting and listing routing decision logs
- storing and reading routing policy SLO fields

**Step 2: Add migrations**

Create persistence for:

- `routing_decision_logs`
- policy SLO columns on `routing_policies`

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`

Expected: PASS

### Task 5: Implement SLO-aware selection and log persistence

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

**Step 1: Add SLO-aware selection**

Implement deterministic SLO filtering with best-effort fallback.

**Step 2: Add log persistence entry point**

Add one app-routing function that computes a decision and persists a `RoutingDecisionLog`.

**Step 3: Run focused test**

Run: `cargo test -p sdkwork-api-app-routing --test simulate_route -q`

Expected: PASS

### Task 6: Integrate real gateway dispatch and admin simulation logging

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`

**Step 1: Route gateway selection through the new logging entry point**

Replace direct routing simulation calls in real dispatch paths with the log-emitting selector.

**Step 2: Add admin decision log API**

Expose:

- `GET /admin/routing/decision-logs`

**Step 3: Persist admin simulation logs**

Use `decision_source = admin_simulation`.

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`

Expected: PASS

### Task 7: Update console and docs

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-routing/src/index.tsx`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Extend types and admin SDK**

Add decision log and SLO-aware strategy payload types.

**Step 2: Upgrade routing page**

Show recent decision logs and SLO-degraded vs compliant state.

**Step 3: Refresh docs**

Move SLO-aware routing and decision logging from remaining gap to implemented, while keeping geo affinity and hot reload in the remaining-gap list.

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
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-http --test chat_route -q`
- `cargo test -p sdkwork-api-interface-http -q`
- `pnpm --dir console -r typecheck`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-domain-routing -p sdkwork-api-storage-core -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres -p sdkwork-api-app-routing -p sdkwork-api-app-gateway -p sdkwork-api-interface-admin -p sdkwork-api-interface-http --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-slo-aware-routing-decision-logs-design.md docs/plans/2026-03-14-slo-aware-routing-decision-logs-implementation.md crates/sdkwork-api-domain-routing crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-app-routing crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-admin crates/sdkwork-api-interface-http console/packages/sdkwork-api-types console/packages/sdkwork-api-admin-sdk console/packages/sdkwork-api-routing README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md
git commit -m "feat: add slo aware routing decision logs"
```
