# Commerce Refund Headroom Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** block unsafe commerce refunds when recharge headroom has already been consumed after settlement, while preserving audit visibility for rejected refund events.

**Architecture:** keep the existing quota-based commerce model, but tighten refund validation so it uses project usage ledger evidence to compute current remaining headroom before allowing quota rollback. Drive the change from a portal regression test, then update the refund path and verify that payment-event audit records remain rejected when the refund is unsafe.

**Tech Stack:** Rust, Axum, sqlx, cargo test

---

### Task 1: Add refund-after-consumption regression coverage

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Add a failing test proving a settled recharge order cannot be refunded after later usage consumes the restored headroom**
- [ ] **Step 2: Assert the refund payment-event history remains visible with `processing_status = "rejected"` and a conflict message**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce refund_rejects_when_recharge_headroom_has_been_consumed -- --nocapture` and confirm it fails first**

### Task 2: Tighten refund safety against actual remaining headroom

**Files:**
- Modify: `crates/sdkwork-api-app-commerce/src/lib.rs`

- [ ] **Step 1: Read project usage ledger entries during recharge refund validation**
- [ ] **Step 2: Require `remaining_units >= refunded_units` before quota rollback**
- [ ] **Step 3: Return a clear conflict error when consumed headroom makes the refund unsafe**
- [ ] **Step 4: Re-run the targeted test until it passes**

### Task 3: Regress the commerce payment slice

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-commerce -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`**
