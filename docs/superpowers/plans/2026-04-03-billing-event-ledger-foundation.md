# Billing Event Ledger Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** introduce a canonical Billing 2.0 event ledger that can represent multimodal usage dimensions, routing evidence, API key group chargeback, and separate upstream-cost from customer-charge accounting without breaking the existing ledger and quota flows.

**Architecture:** keep `LedgerEntry` and `UsageRecord` as compatibility layers, then add a new `BillingEventRecord` that becomes the durable source for detailed accounting. Persist it through the existing storage seam, expose it through the admin control plane, and wire the gateway metering path to dual-write billing events so future pricing engines and plugin-based billing strategies can build on a stable event contract.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, serde, existing `sdkwork-api-*` crates, cargo tests

---

### Task 1: Define the Billing 2.0 event domain contract

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Test: `crates/sdkwork-api-domain-billing/tests/ledger_entry.rs`

- [ ] **Step 1: Write the failing test**

Add domain tests that expect:

- `BillingEventRecord` to capture tenant, project, API key group, capability, route key, usage model, provider, channel, modality, operation kind, routing evidence, and timestamps
- multimodal dimensions including request count, token counts, cached token deltas, image count, audio seconds, video seconds, and music seconds
- separate `upstream_cost` and `customer_charge`
- an explicit accounting mode for platform-credit, BYOK, and passthrough-style operation

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-domain-billing -- --nocapture`
Expected: FAIL because the billing event domain types do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:

- `BillingAccountingMode`
- `BillingEventRecord`
- small builder-style helpers for financials, meter dimensions, routing evidence, and request facts

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-domain-billing -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-domain-billing/src/lib.rs crates/sdkwork-api-domain-billing/tests/ledger_entry.rs
git commit -m "feat: add billing event domain contract"
```

### Task 2: Add application-layer billing event creation and aggregation

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Test: `crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs`
- Test: `crates/sdkwork-api-app-billing/tests/billing_summary.rs`

- [ ] **Step 1: Write the failing test**

Add app tests that expect:

- a billing event can be created and persisted with detailed dimensions
- billing events can be listed independently of legacy ledger entries
- a summary view aggregates by project, API key group, capability, and accounting mode

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-app-billing -- --nocapture`
Expected: FAIL because billing event helpers and summaries do not exist.

- [ ] **Step 3: Write minimal implementation**

Implement:

- `create_billing_event`
- `persist_billing_event`
- `list_billing_events`
- `summarize_billing_events`
- compatibility-preserving summary structs for group and capability aggregation

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-app-billing -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-app-billing/src/lib.rs crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs crates/sdkwork-api-app-billing/tests/billing_summary.rs
git commit -m "feat: add billing event application services"
```

### Task 3: Expand storage seams and persist billing events in SQLite and PostgreSQL

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/admin_store_trait.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Test: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Write the failing test**

Add storage tests that expect:

- `ai_billing_events` exists after migration
- billing events round-trip through SQLite with routing evidence and multimodal dimensions intact
- PostgreSQL compiles with matching SQL and row decoding

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-storage-sqlite --test admin_store_trait -- --nocapture`
Expected: FAIL because the storage seam and table do not exist.

- [ ] **Step 3: Write minimal implementation**

Add:

- `insert_billing_event` and `list_billing_events` to `AdminStore` and `BillingStore`
- `ai_billing_events` schema with indexes for project, group, capability, and created time
- SQLite and PostgreSQL codecs for the new record

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-storage-sqlite --test admin_store_trait -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-storage-sqlite/tests/admin_store_trait.rs crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs
git commit -m "feat: persist billing events across storage backends"
```

### Task 4: Expose billing event observability through the admin API

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `docs/api-reference/admin-api.md`

- [ ] **Step 1: Write the failing test**

Add admin API tests that expect:

- `GET /admin/billing/events`
- `GET /admin/billing/events/summary`
- summary output to expose group-level, capability-level, and accounting-mode aggregation

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
Expected: FAIL because billing event endpoints and response types do not exist.

- [ ] **Step 3: Write minimal implementation**

Add:

- billing event list handler
- billing event summary handler
- admin API documentation for the new routes and response fields

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs docs/api-reference/admin-api.md
git commit -m "feat: expose billing event ledger through admin api"
```

### Task 5: Dual-write billing events from gateway metering and capture phase notes

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/generation_billing_guardrails.rs`
- Modify: `docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md`

- [ ] **Step 1: Write the failing test**

Add gateway tests that expect:

- metered gateway requests still write legacy usage and ledger entries
- the same request also writes a billing event containing provider, route key, capability, tokens, request facts, API key group, and routing evidence when available

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test generation_billing_guardrails -- --nocapture`
Expected: FAIL because gateway metering does not yet emit billing events.

- [ ] **Step 3: Write minimal implementation**

Wire the existing request metering path to create billing events using:

- request context API key group
- planned execution provider and channel context
- token usage metrics
- routing evidence from the current request context when available

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test generation_billing_guardrails -- --nocapture`
Expected: PASS

- [ ] **Step 5: Run focused regression suite**

Run:

```bash
cargo test -p sdkwork-api-domain-billing -- --nocapture
cargo test -p sdkwork-api-app-billing -- --nocapture
cargo test -p sdkwork-api-storage-sqlite --test admin_store_trait -- --nocapture
cargo test -p sdkwork-api-storage-sqlite --test sqlite_migrations -- --nocapture
cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture
cargo test -p sdkwork-api-interface-http --test generation_billing_guardrails -- --nocapture
cargo test -p sdkwork-api-storage-postgres --no-run
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add docs crates
git commit -m "feat: add billing event ledger foundation"
```
