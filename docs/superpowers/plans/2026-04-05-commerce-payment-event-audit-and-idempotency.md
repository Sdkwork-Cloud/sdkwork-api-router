# Commerce Payment Event Audit And Idempotency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make commerce payment events auditable, idempotent, and safe to replay without duplicating order side effects.

**Architecture:** add a commerce-native payment event record in the domain and storage layers, then route `/portal/commerce/orders/{order_id}/payment-events` through a small processor that persists the event, deduplicates by provider event identity, and only applies order state mutations once. Keep the current manual portal settlement flow working by treating missing provider metadata as a local/manual event with a synthetic dedupe boundary.

**Tech Stack:** Rust, Axum, sqlx, utoipa, cargo test

---

### Task 1: Add failing portal payment event audit tests

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Add a failing test proving payment events are listed with audit fields**
- [ ] **Step 2: Add a failing test proving the same `provider_event_id` is idempotent and does not duplicate quota side effects**
- [ ] **Step 3: Add a failing test proving conflicting reuse of the same `provider_event_id` on another order is rejected**
- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce payment_event -- --nocapture` and confirm the new tests fail first**

### Task 2: Add commerce payment event domain and store support

**Files:**
- Modify: `crates/sdkwork-api-domain-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [ ] **Step 1: Add `CommercePaymentEventRecord` and a processing status enum in the commerce domain**
- [ ] **Step 2: Extend `AdminStore` with insert/find/list payment event methods**
- [ ] **Step 3: Add SQLite/Postgres schema, indexes, and row decoders for commerce payment events**
- [ ] **Step 4: Add migration/backfill safety where needed**

### Task 3: Implement audited and idempotent payment event processing

**Files:**
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`

- [ ] **Step 1: Extend `PortalCommercePaymentEventRequest` with optional provider metadata and dedupe identity**
- [ ] **Step 2: Persist payment events before applying order mutations**
- [ ] **Step 3: Deduplicate exact replays and return the current order state without replaying side effects**
- [ ] **Step 4: Reject reuse of a provider event identity across different orders**
- [ ] **Step 5: Expose order payment event history through app-commerce helpers**

### Task 4: Expose portal audit API and keep OpenAPI aligned

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Add a `GET /portal/commerce/orders/{order_id}/payment-events` route**
- [ ] **Step 2: Ensure existing POST route accepts the extended request schema**
- [ ] **Step 3: Re-run portal commerce tests until green**

### Task 5: Regression for the payment event slice

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-domain-commerce -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-commerce -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-storage-sqlite --test admin_store_trait -- --nocapture`**
- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-storage-postgres --lib -- --nocapture`**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`**
