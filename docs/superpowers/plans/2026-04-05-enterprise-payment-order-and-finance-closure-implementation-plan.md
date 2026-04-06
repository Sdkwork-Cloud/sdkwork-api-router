# Enterprise Payment, Order, and Finance Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** build a production-grade payment, order, refund, account-history, and finance-closure subsystem for `sdkwork-api-router` with Stripe, WeChat Pay, Alipay, QR checkout, callback-first settlement, reconciliation, and operator tooling.

**Architecture:** keep the existing commerce and billing kernels, add a new payment domain and orchestration layer, write immutable payment and finance evidence before fulfillment side effects, and bridge successful payment or refund outcomes into the account kernel through replay-safe outbox processing. Use Stripe as the overseas first gateway and WeChat Pay or Alipay as the domestic QR and web gateways under the same canonical state model.

**Tech Stack:** Rust, Axum, sqlx, utoipa, cargo test, React portal and admin packages, SQLite and PostgreSQL

---

### Task 1: Freeze canonical payment and finance contracts

**Files:**
- Create: `crates/sdkwork-api-domain-payment/Cargo.toml`
- Create: `crates/sdkwork-api-domain-payment/src/lib.rs`
- Modify: `Cargo.toml`
- Modify: `docs/superpowers/specs/2026-04-05-enterprise-payment-order-and-finance-closure-design.md`
- Test: `crates/sdkwork-api-domain-payment/src/lib.rs`

- [ ] **Step 1: Add the new domain crate to the workspace and define all payment, refund, dispute, callback, and reconciliation records**
- [ ] **Step 2: Add finance-facing enums and identifiers needed by payment orchestration without moving existing account-kernel types**
- [ ] **Step 3: Add unit tests for state transitions, identifier normalization, and status compatibility rules**
- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-domain-payment -- --nocapture`**

### Task 2: Add canonical storage schema and store interfaces

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/payment_schema.rs`

- [ ] **Step 1: Extend storage-core with payment, refund, finance-journal, and reconciliation store traits**
- [ ] **Step 2: Add SQLite tables, indexes, and row codecs for all new `ai_payment_*`, `ai_refund_*`, and `ai_finance_*` tables**
- [ ] **Step 3: Add PostgreSQL parity for the same schema and indexes**
- [ ] **Step 4: Add migration tests proving the new schema exists in both SQLite and PostgreSQL**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-storage-sqlite --test sqlite_migrations -- --nocapture`**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-storage-postgres --test integration_postgres -- --nocapture`**

### Task 3: Implement payment orchestration and compatibility cutover

**Files:**
- Create: `crates/sdkwork-api-app-payment/Cargo.toml`
- Create: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-commerce/src/lib.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_order_service.rs`
- Test: `crates/sdkwork-api-app-commerce/tests/commerce_checkout_bridge.rs`

- [ ] **Step 1: Add the app-payment crate and implement payment order, attempt, and session services**
- [ ] **Step 2: Replace the current `manual_lab` checkout assembly with a compatibility bridge that delegates to app-payment**
- [ ] **Step 3: Preserve existing zero-pay and coupon-fulfillment behavior without introducing external payment for non-payable orders**
- [ ] **Step 4: Add tests proving the new payment order model still supports current commerce order creation and settlement flows**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-app-commerce -- --nocapture`**

### Task 4: Build callback-first payment ingestion and outbox recovery

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Test: `crates/sdkwork-api-app-payment/tests/payment_callback_processing.rs`
- Test: `crates/sdkwork-api-interface-http/tests/payment_callbacks.rs`

- [ ] **Step 1: Add callback intake services that persist raw provider events before any mutation**
- [ ] **Step 2: Implement dedupe, signature verification hooks, and provider-query fallback for ambiguous outcomes**
- [ ] **Step 3: Add durable outbox events for account grant, entitlement activation, refund reversal, and timeline projection**
- [ ] **Step 4: Add retry and dead-letter behavior for failed callback processing**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-app-payment --test payment_callback_processing -- --nocapture`**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-interface-http --test payment_callbacks -- --nocapture`**

### Task 5: Land Stripe production-grade overseas payments first

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-secret-core/src/lib.rs`
- Modify: `crates/sdkwork-api-secret-local/src/lib.rs`
- Modify: `crates/sdkwork-api-secret-keyring/src/lib.rs`
- Test: `crates/sdkwork-api-app-payment/tests/stripe_checkout.rs`

- [ ] **Step 1: Add Stripe gateway-account configuration and secret wiring**
- [ ] **Step 2: Implement Stripe Checkout session creation, webhook verification, success normalization, and refund execution**
- [ ] **Step 3: Add dispute ingestion as canonical payment evidence even if the first release only exposes read models**
- [ ] **Step 4: Add tests covering idempotent checkout creation, webhook replay safety, refund replay safety, and failure transitions**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-app-payment --test stripe_checkout -- --nocapture`**

### Task 6: Land WeChat Pay and Alipay domestic QR and web flows

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commerce/src`
- Test: `crates/sdkwork-api-app-payment/tests/wechatpay_checkout.rs`
- Test: `crates/sdkwork-api-app-payment/tests/alipay_checkout.rs`

- [ ] **Step 1: Implement WeChat Native or H5 checkout normalization with QR session output**
- [ ] **Step 2: Implement Alipay Precreate or page-payment normalization with QR or redirect session output**
- [ ] **Step 3: Add provider callback verification, refund query handling, and session expiry behavior**
- [ ] **Step 4: Add portal QR rendering and session-refresh UX for domestic checkout**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-app-payment --test wechatpay_checkout -- --nocapture`**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-app-payment --test alipay_checkout -- --nocapture`**

### Task 7: Close refund, finance journal, and account-history lineage

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Test: `crates/sdkwork-api-app-billing/tests/payment_refund_account_closure.rs`
- Test: `crates/sdkwork-api-interface-admin/tests/payment_finance_routes.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/payment_history_routes.rs`

- [ ] **Step 1: Extend account-ledger records with business references to order, payment, refund, and finance journal evidence**
- [ ] **Step 2: Implement finance journal writing for payment success, usage settlement consumption, and refund reversal**
- [ ] **Step 3: Implement safe refundable-amount calculation for recharge, custom recharge, and subscription orders**
- [ ] **Step 4: Add admin and portal views for payment history, refund history, and linked account history**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-app-billing --test payment_refund_account_closure -- --nocapture`**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-interface-admin --test payment_finance_routes -- --nocapture`**
- [ ] **Step 7: Run `cargo test --offline -p sdkwork-api-interface-portal --test payment_history_routes -- --nocapture`**

### Task 8: Add reconciliation, monitoring, traffic control, and launch gates

**Files:**
- Modify: `crates/sdkwork-api-app-payment/src/lib.rs`
- Modify: `crates/sdkwork-api-app-rate-limit/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/src`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src`
- Test: `crates/sdkwork-api-app-payment/tests/payment_reconciliation.rs`

- [ ] **Step 1: Implement reconciliation batch import, matching, and mismatch-resolution state**
- [ ] **Step 2: Add provider and subject rate limits for checkout, refunds, and callback processing**
- [ ] **Step 3: Add circuit breaker state, provider health scoring, and controlled failover for new checkout sessions**
- [ ] **Step 4: Add observability metrics, drift alarms, and dead-letter counters**
- [ ] **Step 5: Add admin operator queues for callback retry, fulfillment drift, refund retry, and reconciliation mismatch**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-app-payment --test payment_reconciliation -- --nocapture`**

### Task 9: Full-system verification and launch review

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-domain-payment -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-payment -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-app-billing -- --nocapture`**
- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-app-commerce -- --nocapture`**
- [ ] **Step 5: Run `cargo test --offline -p sdkwork-api-interface-portal -- --nocapture`**
- [ ] **Step 6: Run `cargo test --offline -p sdkwork-api-interface-admin -- --nocapture`**
- [ ] **Step 7: Run `cargo test --offline -p sdkwork-api-interface-http -- --nocapture`**
- [ ] **Step 8: Run the portal and admin package test suites that cover recharge, payments, refunds, and history views**
- [ ] **Step 9: Verify launch gates for callback replay, refund replay, reconciliation drift, and provider failover before production cutover**
