# Clean-Slate Account Billing Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add the first real `AccountKernelStore`-backed billing service APIs so the app layer can project balances and plan holds from canonical account lots instead of relying on legacy quota summaries.

**Architecture:** keep existing compatibility helpers compiling, but introduce clean-slate account billing types and functions in `sdkwork-api-app-billing`. Drive behavior from SQLite-backed tests first, implement deterministic lot selection and balance math second, then use the new APIs as the future gateway and UI cutover seam.

**Tech Stack:** Rust, anyhow, async-trait, sqlx, SQLite, cargo test

---

### Task 1: Add failing tests for account balance projection and hold planning

**Files:**
- Create: `crates/sdkwork-api-app-billing/tests/account_kernel_service.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`

- [ ] **Step 1: Write a failing SQLite-backed test that projects a balance snapshot from canonical account lots**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-app-billing --test account_kernel_service -- --nocapture` and verify it fails**
- [ ] **Step 3: Add a failing SQLite-backed test that plans a hold allocation across multiple lots in canonical spend order**
- [ ] **Step 4: Re-run the focused test command and confirm it still fails for missing service APIs**

### Task 2: Implement clean-slate account read models and planning logic

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`

- [ ] **Step 1: Add `AccountBalanceSnapshot`, `AccountLotBalanceSnapshot`, `PlannedHoldAllocation`, and `AccountHoldPlan`**
- [ ] **Step 2: Implement lot eligibility helpers and deterministic spend-priority ordering**
- [ ] **Step 3: Implement `summarize_account_balance(store, account_id, now_ms)`**
- [ ] **Step 4: Implement `plan_account_hold(store, account_id, requested_quantity, now_ms)`**
- [ ] **Step 5: Re-run `cargo test -p sdkwork-api-app-billing --test account_kernel_service -- --nocapture` and confirm green**

### Task 3: Protect the existing crate behavior and confirm full crate health

**Files:**
- Modify: `crates/sdkwork-api-app-billing/tests/billing_summary.rs`
- Modify: `crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs`

- [ ] **Step 1: Run `cargo test -p sdkwork-api-app-billing -- --nocapture` after the new service is green**
- [ ] **Step 2: If legacy tests or exports break, adapt the crate surface without re-centering it on quota-first logic**
- [ ] **Step 3: Re-run `cargo test -p sdkwork-api-app-billing -- --nocapture` and confirm the crate stays green**

### Task 4: Create a stable checkpoint for gateway cutover

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-router-implementation-audit-and-upgrade-plan.md`

- [ ] **Step 1: Update the audit doc to record that account balance projection and hold planning now exist in the app layer**
- [ ] **Step 2: Review `git diff` for account-billing changes and keep the slice focused on read-model and planning behavior only**
- [ ] **Step 3: Commit with a message like `feat: add clean-slate account billing service planning`**
