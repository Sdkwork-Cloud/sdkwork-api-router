# Health-Aware Routing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make routing decisions and admin simulations health-aware, cost-aware, and explainable by combining current catalog, extension state, and runtime status data.

**Architecture:** Keep routing deterministic and schema-light by reusing existing provider, model, installation, instance, and runtime-status state. Add candidate assessment metadata to the routing domain, compute assessment in `sdkwork-api-app-routing`, expose it through the admin API, and render it in the console routing module.

**Tech Stack:** Rust, serde, Axum, React, TypeScript

---

### Task 1: Add failing tests for enriched routing decisions

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- routing decisions can carry strategy, reason, and candidate assessments
- unhealthy runtime-backed providers are demoted behind healthy ones
- lower-cost healthy providers win when policy order alone is insufficient
- admin route simulation returns the richer explanation payload

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`
- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: FAIL because routing decisions do not yet include assessment metadata and routing still ignores runtime health and config hints.

### Task 2: Extend the routing domain model

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

**Step 1: Add candidate assessment structures**

Introduce a normalized assessment record with:

- provider ID
- availability
- health
- policy rank
- optional weight
- optional cost
- optional latency
- reasons

**Step 2: Extend `RoutingDecision`**

Add:

- `strategy`
- `selection_reason`
- `assessments`

while preserving current fields for compatibility.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-domain-routing --test routing_decision_rules -q`

Expected: PASS

### Task 3: Implement runtime-aware candidate assessment in app routing

**Files:**
- Modify: `crates/sdkwork-api-app-routing/Cargo.toml`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

**Step 1: Load candidate state from current sources**

Use:

- model catalog
- providers
- extension installations
- extension instances
- runtime statuses

**Step 2: Parse routing hints from instance config**

Support:

- root `weight`, `cost`, `latency_ms`
- nested `routing.weight`, `routing.cost`, `routing.latency_ms`

**Step 3: Rank candidates deterministically**

Sort by:

- availability
- health
- lower cost
- lower latency
- higher weight
- policy rank
- provider ID

**Step 4: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-routing --test simulate_route -q`

Expected: PASS

### Task 4: Expose richer simulation payload through admin APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Serialize the richer decision structure**

Return:

- selected provider
- matched policy ID
- strategy
- selection reason
- candidate IDs
- candidate assessments

**Step 2: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 5: Update console types and routing page

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-routing/src/index.tsx`

**Step 1: Expand routing simulation result types**

Add typed assessment fields matching the admin payload.

**Step 2: Render the decision explanation**

Show:

- strategy
- selection reason
- per-candidate health and availability
- cost, latency, and weight hints
- reasons list

**Step 3: Run focused verification**

Run:

- `pnpm --dir console -r typecheck`

Expected: PASS

### Task 6: Update docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-14-health-aware-routing-design.md`
- Modify: `docs/plans/2026-03-14-health-aware-routing-implementation.md`

**Step 1: Update docs**

Reflect that routing now supports:

- deterministic health-aware failover
- cost and latency hints from instance config
- explainable route simulation

while weighted random balancing, SLO routing, and persisted telemetry remain future work.

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-domain-routing -p sdkwork-api-app-routing -p sdkwork-api-interface-admin --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS

**Step 3: Commit**

```bash
git add README.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md docs/plans/2026-03-14-health-aware-routing-design.md docs/plans/2026-03-14-health-aware-routing-implementation.md crates/sdkwork-api-domain-routing crates/sdkwork-api-app-routing crates/sdkwork-api-interface-admin console/packages/sdkwork-api-types console/packages/sdkwork-api-admin-sdk console/packages/sdkwork-api-routing
git commit -m "feat: add health-aware routing decisions"
git push
```
