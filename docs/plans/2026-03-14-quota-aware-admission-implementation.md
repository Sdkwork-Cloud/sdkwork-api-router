# Quota-Aware Admission Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add persistent project-level quota policies, admin management APIs, and synchronous quota rejection for the core stateful create routes.

**Architecture:** Extend the billing domain and admin stores with a `QuotaPolicy` aggregate, evaluate quota against existing ledger totals in the billing application layer, and enforce quota in the gateway HTTP layer before expensive upstream dispatch. Keep the first version project-scoped and hard-limit based so it remains small, testable, and compatible with later rolling-window or provider-aware policy extensions.

**Tech Stack:** Rust, Axum, Tokio, sqlx, React, TypeScript

---

### Task 1: Add failing quota domain and storage tests

**Files:**
- Create: `crates/sdkwork-api-domain-billing/tests/quota_policy.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/quota_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- `QuotaPolicy` can be constructed in the billing domain
- SQLite can persist and list quota policies
- PostgreSQL can persist and list quota policies when the integration URL is available

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-domain-billing --test quota_policy -q`
- `cargo test -p sdkwork-api-storage-sqlite --test quota_policies -q`

Expected: FAIL because the quota policy domain model and storage methods do not exist yet.

### Task 2: Add failing app-billing, admin, and HTTP quota tests

**Files:**
- Modify: `crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Create: `crates/sdkwork-api-interface-http/tests/quota_enforcement.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- billing evaluates quota against persisted ledger usage
- admin APIs can create and list quota policies
- stateful chat requests return `429` when the project is over quota

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-app-billing --test quota_and_ledger -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-http --test quota_enforcement -q`

Expected: FAIL because quota persistence, quota admin APIs, and gateway enforcement do not exist yet.

### Task 3: Implement the billing-domain quota model

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Create: `crates/sdkwork-api-domain-billing/tests/quota_policy.rs`

**Step 1: Add `QuotaPolicy`**

Use a project-scoped aggregate with `policy_id`, `project_id`, `max_units`, and `enabled`.

**Step 2: Run focused test**

Run: `cargo test -p sdkwork-api-domain-billing --test quota_policy -q`

Expected: PASS

### Task 4: Extend store contracts and persistence

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/quota_policies.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

**Step 1: Add store methods**

Support inserting and listing quota policies.

**Step 2: Add migrations**

Create `billing_quota_policies`.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-storage-sqlite --test quota_policies -q`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`

Expected: PASS

### Task 5: Implement billing quota evaluation

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs`

**Step 1: Add quota evaluation**

Evaluate project quota using persisted ledger totals and the strictest enabled policy.

**Step 2: Run focused test**

Run: `cargo test -p sdkwork-api-app-billing --test quota_and_ledger -q`

Expected: PASS

### Task 6: Add admin quota policy APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Add create and list endpoints**

Expose:

- `GET /admin/billing/quota-policies`
- `POST /admin/billing/quota-policies`

**Step 2: Run focused test**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: PASS

### Task 7: Enforce quota in core stateful HTTP create routes

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/tests/quota_enforcement.rs`

**Step 1: Add preflight quota enforcement**

Reject over-quota requests before dispatch for:

- `/v1/chat/completions`
- `/v1/completions`
- `/v1/responses`
- `/v1/embeddings`

**Step 2: Return OpenAI-compatible errors**

Use HTTP `429` with an `insufficient_quota` envelope and `quota_exceeded` code.

**Step 3: Run focused test**

Run: `cargo test -p sdkwork-api-interface-http --test quota_enforcement -q`

Expected: PASS

### Task 8: Update console and docs

**Files:**
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-usage/src/index.tsx`
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Extend console types and SDK**

Add quota policy types and billing admin client calls.

**Step 2: Upgrade usage page**

Render configured quota policies alongside usage and ledger state.

**Step 3: Refresh docs**

Move quota-aware admission from remaining gap to implemented, while keeping SLO routing, geo affinity, and hot reload in the remaining-gap list.

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
- `cargo test -p sdkwork-api-domain-billing --test quota_policy -q`
- `cargo test -p sdkwork-api-storage-sqlite --test quota_policies -q`
- `cargo test -p sdkwork-api-storage-postgres --test integration_postgres -q`
- `cargo test -p sdkwork-api-app-billing --test quota_and_ledger -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-http --test quota_enforcement -q`
- `pnpm --dir console -r typecheck`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-domain-billing -p sdkwork-api-storage-core -p sdkwork-api-storage-sqlite -p sdkwork-api-storage-postgres -p sdkwork-api-app-billing -p sdkwork-api-interface-admin -p sdkwork-api-interface-http --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add docs/plans/2026-03-14-quota-aware-admission-design.md docs/plans/2026-03-14-quota-aware-admission-implementation.md crates/sdkwork-api-domain-billing crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres crates/sdkwork-api-app-billing crates/sdkwork-api-interface-admin crates/sdkwork-api-interface-http console/packages/sdkwork-api-types console/packages/sdkwork-api-admin-sdk console/packages/sdkwork-api-usage README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md
git commit -m "feat: add quota aware admission control"
```
