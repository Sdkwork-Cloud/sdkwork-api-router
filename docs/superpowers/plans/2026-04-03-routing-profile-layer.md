# Routing Profile Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add reusable routing profiles that can be bound to API key groups so gateway routing can inherit global policy, project defaults, and group-specific profile constraints without introducing a second routing semantics model.

**Architecture:** introduce a new routing-domain record for reusable profile bundles and persist it through the existing routing storage facet in both SQLite and PostgreSQL. Keep `RoutingPolicy` as the model/capability match layer, keep `ProjectRoutingPreferences` as the project-wide default layer, and add group-bound routing profiles as the most specific overlay applied at request time through the existing API key group context.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, serde, existing `sdkwork-api-*` crates, cargo tests

---

### Task 1: Define the routing profile domain contract

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Test: `crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs`

- [ ] **Step 1: Write the failing test**

Add domain tests that expect:

- `RoutingProfileRecord` to retain reusable provider-order, cost, latency, health, and preferred-region fields
- routing decisions and decision logs to carry the applied routing profile id when one is used

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-domain-routing -- --nocapture`
Expected: FAIL because `RoutingProfileRecord` and applied-profile fields do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:

- `RoutingProfileRecord`
- builder helpers aligned with `ProjectRoutingPreferences`
- optional `applied_routing_profile_id` on `RoutingDecision`
- optional `applied_routing_profile_id` on `RoutingDecisionLog`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-domain-routing -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-domain-routing/src/lib.rs crates/sdkwork-api-domain-routing/tests/routing_decision_rules.rs
git commit -m "feat: add routing profile domain contract"
```

### Task 2: Extend API key groups with a default routing profile binding

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Test: `crates/sdkwork-api-app-identity/tests/api_key_groups.rs`

- [ ] **Step 1: Write the failing test**

Add app-identity tests that expect:

- groups can store an optional `default_routing_profile_id`
- group create or update rejects a missing or cross-workspace routing profile id

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-identity api_key_groups -- --nocapture`
Expected: FAIL because groups do not yet persist or validate routing profile bindings.

- [ ] **Step 3: Write minimal implementation**

Add:

- `default_routing_profile_id` to `ApiKeyGroupRecord`
- `default_routing_profile_id` to `ApiKeyGroupInput` and portal-facing input adapters
- validation in `create_api_key_group` and `update_api_key_group`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-identity api_key_groups -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-domain-identity/src/lib.rs crates/sdkwork-api-app-identity/src/lib.rs crates/sdkwork-api-app-identity/tests/api_key_groups.rs
git commit -m "feat: bind api key groups to routing profiles"
```

### Task 3: Persist routing profiles and group bindings in both storage backends

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs`

- [ ] **Step 1: Write the failing test**

Add storage tests that expect:

- routing profiles round-trip through SQLite
- API key groups round-trip `default_routing_profile_id`
- applied routing profile ids round-trip in decision logs

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-storage-sqlite routing_policies -- --nocapture`
Expected: FAIL because routing profile tables and binding columns do not exist.

- [ ] **Step 3: Write minimal implementation**

Add:

- routing-store methods for insert, list, and find routing profiles
- `ai_routing_profiles` persistence
- `ai_app_api_key_groups.default_routing_profile_id`
- decision-log persistence for `applied_routing_profile_id`
- matching PostgreSQL parity immediately after SQLite

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-storage-sqlite routing_policies -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-storage-sqlite/tests/routing_policies.rs
git commit -m "feat: persist routing profiles and group bindings"
```

### Task 4: Apply routing profiles in route selection

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Test: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Write the failing test**

Add route-selection tests that expect:

- group-bound routing profiles override project defaults for provider order and preferred region
- cost and latency ceilings become the tighter of the inherited layers
- decisions and persisted logs include the applied routing profile id

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-routing simulate_route -- --nocapture`
Expected: FAIL because route selection does not yet load or merge group-bound profiles.

- [ ] **Step 3: Write minimal implementation**

Implement:

- effective routing merge order: matched policy -> project preferences -> group routing profile
- group lookup by `api_key_group_id`
- routing profile lookup by `default_routing_profile_id`
- propagation of `applied_routing_profile_id` into decisions and decision logs

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-routing simulate_route -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-app-routing/src/lib.rs crates/sdkwork-api-app-routing/tests/simulate_route.rs
git commit -m "feat: apply group routing profiles during route selection"
```

### Task 5: Expose admin routing profile management

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `docs/api-reference/admin-api.md`

- [ ] **Step 1: Write the failing test**

Add admin route tests that expect:

- `GET /admin/routing/profiles`
- `POST /admin/routing/profiles`
- API key group update accepts `default_routing_profile_id`
- routing simulations surface `applied_routing_profile_id`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture`
Expected: FAIL because the routes and new group field do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:

- request and response contracts for routing profiles
- admin handlers for create and list
- `default_routing_profile_id` support in API key group create and update handlers
- admin API reference updates

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs docs/api-reference/admin-api.md
git commit -m "feat: add admin routing profile management"
```

### Task 6: Run focused regression verification

**Files:**
- Modify: none unless regressions are found

- [ ] **Step 1: Run focused backend suites**

Run:

```bash
cargo test -p sdkwork-api-domain-routing -- --nocapture
cargo test -p sdkwork-api-app-identity api_key_groups -- --nocapture
cargo test -p sdkwork-api-app-routing simulate_route -- --nocapture
cargo test -p sdkwork-api-storage-sqlite routing_policies -- --nocapture
cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture
```

Expected: PASS

- [ ] **Step 2: Run storage parity checks**

Run:

```bash
cargo test -p sdkwork-api-storage-postgres -- --nocapture
rg -n "default_routing_profile_id|applied_routing_profile_id|RoutingProfileRecord" crates docs
```

Expected: PostgreSQL remains aligned with SQLite and all field names stay consistent.

- [ ] **Step 3: Commit**

```bash
git add crates docs
git commit -m "feat: add routing profile layer foundation"
```

### Task 7: Capture the next-phase observability and billing handoff

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md`

- [ ] **Step 1: Update phase notes after implementation**

Document:

- the final routing merge precedence
- the applied-profile observability contract
- remaining compiled snapshot and rejected-candidate evidence follow-up

- [ ] **Step 2: Record Billing 2.0 entry criteria**

Document that billing should build on:

- `api_key_group_id`
- `applied_routing_profile_id`
- richer route evidence already emitted by routing decisions

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md
git commit -m "docs: record routing profile follow-up notes"
```
