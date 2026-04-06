# Commerce Canonical Account Linkage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** make successful recharge settlement and refund write canonical account evidence so workspace account balance and ledger history reflect commerce transactions.

**Architecture:** add idempotent commerce credit issue/refund primitives to `app-billing`, keyed by a stable order-derived identity, then let portal commerce orchestrate them after order status transitions when a workspace commercial account is provisioned. Keep the existing quota model unchanged for now, but mirror recharge/refund into canonical account benefit lots and ledger entries for auditability and future convergence.

**Tech Stack:** Rust, Axum, sqlx, cargo test

---

### Task 1: Add failing canonical billing tests

**Files:**
- Modify: `crates/sdkwork-api-app-billing/tests/account_kernel_commands.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_commerce.rs`

- [ ] **Step 1: Add a failing app-billing test proving commerce recharge issue is idempotent and writes a grant-style ledger entry**
- [ ] **Step 2: Add a failing app-billing test proving commerce refund is idempotent and disables the issued lot with a refund ledger entry**
- [ ] **Step 3: Add a failing portal commerce test proving a workspace account balance and ledger reflect recharge settlement and refund**
- [ ] **Step 4: Run the focused test commands and confirm the new tests fail first**

### Task 2: Add canonical commerce credit issue/refund primitives

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`

- [ ] **Step 1: Add stable order-derived identity helpers and input/result structs for commerce account credit issue/refund**
- [ ] **Step 2: Implement idempotent credit issuance through the account kernel transaction executor**
- [ ] **Step 3: Implement idempotent credit refund/revocation through the account kernel transaction executor**
- [ ] **Step 4: Expose both operations on `CommercialBillingAdminKernel` so portal can call them via trait object**

### Task 3: Link portal commerce transitions to canonical account evidence

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: After successful recharge settlement, issue canonical account credits when a workspace commercial account exists**
- [ ] **Step 2: After successful recharge refund, revoke canonical account credits idempotently**
- [ ] **Step 3: Keep non-recharge orders and workspaces without canonical accounts as no-op paths**

### Task 4: Verify regressions

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-billing --test account_kernel_commands -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-portal --test portal_commerce -- --nocapture`**
