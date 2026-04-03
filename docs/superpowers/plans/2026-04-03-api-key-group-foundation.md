# API Key Group Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add first-class API key groups to the backend so admin and portal workflows can create, manage, and assign grouped gateway keys without breaking existing ungrouped keys.

**Architecture:** introduce a new identity-domain record plus SQLite storage, then extend the app-identity workflows and admin or portal HTTP boundaries to validate and persist optional group membership. Keep backward compatibility by making key group membership nullable and by enforcing workspace and environment consistency in application logic.

**Tech Stack:** Rust, Axum, sqlx SQLite, serde, existing `sdkwork-api-*` crates, cargo tests

---

### Task 1: Define identity-domain records for API key groups

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Test: `crates/sdkwork-api-app-identity/tests/create_api_key.rs`

- [ ] **Step 1: Write the failing test**

Add an app-identity test that expects a key-group-aware workflow to return the created group's id and reject invalid group attachment inputs.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-identity create_api_key -- --nocapture`
Expected: FAIL because the group record and validation path do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:

- `ApiKeyGroupRecord`
- constructors and metadata helpers
- optional `api_key_group_id` on `GatewayApiKeyRecord`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-identity create_api_key -- --nocapture`
Expected: the domain shapes compile and the new failing points move into storage or app logic.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-domain-identity/src/lib.rs crates/sdkwork-api-app-identity/tests/create_api_key.rs
git commit -m "feat: add api key group identity records"
```

### Task 2: Extend storage contracts and SQLite schema

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Test: `crates/sdkwork-api-storage-sqlite/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add SQLite tests that:

- insert and list API key groups
- persist an API key with `api_key_group_id`
- preserve null group ids for legacy keys

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-storage-sqlite api_key_group -- --nocapture`
Expected: FAIL because the table, store methods, and decode paths do not exist.

- [ ] **Step 3: Write minimal implementation**

Add:

- `ai_app_api_key_groups` table creation and migrations
- storage-core methods for create/list/find/update/delete group records
- `api_key_group_id` column support in `ai_app_api_keys`
- decode and encode changes for key records

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-storage-sqlite api_key_group -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs
git commit -m "feat: persist api key groups in sqlite"
```

### Task 3: Implement app-identity group workflows and validation

**Files:**
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/create_api_key.rs`

- [ ] **Step 1: Write the failing test**

Add tests for:

- create and list group workflows
- key creation with a matching group
- rejection when group tenant or project mismatch
- rejection when group environment mismatch
- rejection when group is inactive

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-identity -- --nocapture`
Expected: FAIL because the workflow functions and validation rules are missing.

- [ ] **Step 3: Write minimal implementation**

Add app functions for:

- create/list/update/delete groups
- set group active state
- validate group assignment during admin and portal key creation or update

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-identity -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-app-identity/src/lib.rs crates/sdkwork-api-app-identity/tests/create_api_key.rs
git commit -m "feat: validate api key group assignment"
```

### Task 4: Extend admin HTTP APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `docs/api-reference/admin-api.md`

- [ ] **Step 1: Write the failing test**

Add admin route tests that:

- create and list API key groups
- update group metadata and active state
- reject invalid key creation with mismatched groups
- accept valid key creation with `api_key_group_id`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture`
Expected: FAIL because the routes and request models do not exist.

- [ ] **Step 3: Write minimal implementation**

Add:

- request and response types
- `/admin/api-key-groups` handlers
- `api_key_group_id` support in existing key create and update handlers
- API reference updates

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs docs/api-reference/admin-api.md
git commit -m "feat: add admin api key group routes"
```

### Task 5: Extend portal HTTP APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/sqlite_portal_routes.rs`
- Modify: `docs/api-reference/portal-api.md`

- [ ] **Step 1: Write the failing test**

Add portal route tests that:

- create and list workspace-scoped API key groups
- reject cross-workspace access
- create a key bound to a valid group

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-portal -- --nocapture`
Expected: FAIL because group routes and request fields are missing.

- [ ] **Step 3: Write minimal implementation**

Add:

- `/portal/api-key-groups` handlers
- `api_key_group_id` support in portal key create
- portal API reference updates

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-portal -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-portal/src/lib.rs crates/sdkwork-api-interface-portal/tests/sqlite_portal_routes.rs docs/api-reference/portal-api.md
git commit -m "feat: add portal api key group routes"
```

### Task 6: Run focused regression verification

**Files:**
- Modify: none unless regressions are found

- [ ] **Step 1: Run focused backend suites**

Run:

```bash
cargo test -p sdkwork-api-app-identity -- --nocapture
cargo test -p sdkwork-api-storage-sqlite -- --nocapture
cargo test -p sdkwork-api-interface-admin sqlite_admin_routes -- --nocapture
cargo test -p sdkwork-api-interface-portal -- --nocapture
```

Expected: PASS

- [ ] **Step 2: Run a final grep sanity check**

Run:

```bash
rg -n "api_key_group" crates docs
```

Expected: all new code paths and docs reference the same field names consistently.

- [ ] **Step 3: Commit**

```bash
git add crates docs
git commit -m "feat: add api key group foundation"
```

### Task 7: Prepare follow-on architecture handoff

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md`

- [ ] **Step 1: Update phase notes after implementation**

Document any deviations, follow-up work, or constraints discovered during the implementation.

- [ ] **Step 2: Record next phase entry criteria**

Document the conditions required to start routing profiles and billing 2.0 work.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md
git commit -m "docs: capture api key group follow-up notes"
```
