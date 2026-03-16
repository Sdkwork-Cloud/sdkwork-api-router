# Geo-Affinity Routing Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add request-scoped `geo_affinity` routing that prefers region-matching provider instances, persists requested-region decision evidence, and exposes the behavior through admin simulation plus the routing console.

**Architecture:** Extend the routing domain with a `geo_affinity` strategy and region metadata on assessments and decision logs. Reuse existing extension instance config as the provider-region source, propagate an optional `x-sdkwork-region` hint through request-scoped gateway context, and keep the selection fallback aligned with the current health-aware ranking when no regional match exists.

**Tech Stack:** Rust, Axum, serde, sqlx, tokio task-local context, TypeScript, React

---

## Chunk 1: Design Surface And Routing Tests

### Task 1: Add failing routing tests for geo affinity

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Write a failing test for region-match preference**

Add a test that creates two providers for the same model, gives the higher-ranked provider region `eu-west`, gives the lower-ranked provider region `us-east`, uses `RoutingStrategy::GeoAffinity`, simulates with `requested_region = "us-east"`, and expects the `us-east` provider to win.

- [ ] **Step 2: Run the targeted routing test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-routing --test simulate_route route_simulation_geo_affinity_prefers_matching_region -q`
Expected: FAIL because `geo_affinity` does not exist yet.

- [ ] **Step 3: Add a failing fallback test**

Add a second test proving the strategy degrades to the top-ranked healthy candidate when no region matches.

- [ ] **Step 4: Add a failing health-precedence test**

Add a third test proving an unhealthy matching candidate loses to a healthy non-matching candidate when a healthy option exists.

## Chunk 2: Domain And Routing Implementation

### Task 2: Add geo-affinity strategy and region-aware selection

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`

- [ ] **Step 1: Extend the routing domain**

Add:

- `RoutingStrategy::GeoAffinity`
- `region` and `region_match` on `RoutingCandidateAssessment`
- `requested_region` on `RoutingDecision`
- `requested_region` on `RoutingDecisionLog`

- [ ] **Step 2: Add routing-region aware selection plumbing**

Introduce internal helpers that accept `requested_region: Option<&str>` without breaking existing public wrappers that omit it.

- [ ] **Step 3: Implement geo-affinity selection**

Use existing ranked assessments, exclude unavailable candidates, exclude unhealthy candidates when any healthy candidate exists, prefer exact region matches, and degrade safely with clear reasons when no match exists.

- [ ] **Step 4: Resolve region hints from instance config**

Use `config.routing.region` first, then `config.region`.

- [ ] **Step 5: Run focused routing tests to confirm GREEN**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-routing --test simulate_route -q`
Expected: PASS.

## Chunk 3: Admin Simulation And Persistence

### Task 3: Add requested-region support to admin routing APIs and decision-log persistence

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Write failing admin API tests**

Add tests proving `/admin/routing/simulations` accepts `requested_region`, returns it in the simulation result, and persists it in `/admin/routing/decision-logs`.

- [ ] **Step 2: Run the targeted admin tests to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes routing_simulation_accepts_requested_region_and_persists_logs -q`
Expected: FAIL because the request and log schema do not yet support `requested_region`.

- [ ] **Step 3: Add the decision-log storage field**

Persist `requested_region` in both SQLite and PostgreSQL decision-log storage and round-trip it through list APIs.

- [ ] **Step 4: Wire admin simulation through the new routing context**

Pass `requested_region` into the routing selection entry point and return it in the simulation response.

- [ ] **Step 5: Run focused storage and admin tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-storage-sqlite --test routing_policies -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS.

## Chunk 4: Real Gateway Request Context

### Task 4: Propagate `x-sdkwork-region` into stateful gateway routing

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/routing_policy_dispatch.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`

- [ ] **Step 1: Write a failing gateway propagation test**

Add a stateful chat-route test that mounts two providers for the same model, sends `x-sdkwork-region: us-east`, and expects the `us-east` provider to receive the upstream request.

- [ ] **Step 2: Run the focused gateway test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-http --test chat_route stateful_chat_route_uses_requested_region_for_geo_affinity -q`
Expected: FAIL because the header is not routed into selection.

- [ ] **Step 3: Add request-scoped routing-region context**

Expose a gateway helper that scopes an optional region value for the lifetime of the request. Read `x-sdkwork-region` in gateway HTTP middleware and apply that scope around request handling.

- [ ] **Step 4: Include region in gateway selection caching**

Prevent routing-decision cache reuse across different requested regions.

- [ ] **Step 5: Verify targeted gateway tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test routing_policy_dispatch -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-http --test chat_route -q`

Expected: PASS.

## Chunk 5: Console And Type Surface

### Task 5: Expose region-aware routing fields to the console

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-routing/src/index.tsx`

- [ ] **Step 1: Extend admin client and shared types**

Add `geo_affinity`, `requested_region`, `region`, and `region_match` to the shared routing DTOs.

- [ ] **Step 2: Update routing console rendering**

Show requested region on the simulation summary and decision-log entries, plus candidate region-match evidence where present.

- [ ] **Step 3: Run console typecheck**

Run: `pnpm --dir console -r typecheck`
Expected: PASS.

## Chunk 6: Final Verification

### Task 6: Run full verification

**Files:**
- Modify: `docs/plans/2026-03-15-geo-affinity-routing-design.md`
- Modify: `docs/plans/2026-03-15-geo-affinity-routing-implementation.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`

- [ ] **Step 1: Update docs to move geo affinity from future work to implemented**

Adjust the compatibility and architecture docs to reflect the new routing capability and the operator-facing request-region contract.

- [ ] **Step 2: Run full Rust verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`

Expected: PASS.

- [ ] **Step 3: Run frontend verification**

Run: `pnpm --dir console -r typecheck`
Expected: PASS.
