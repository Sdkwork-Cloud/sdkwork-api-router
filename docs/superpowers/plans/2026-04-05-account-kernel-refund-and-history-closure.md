# Account Kernel Refund And History Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the transactional billing loop by making hold/create/release/capture/refund write canonical account ledger history in the same transaction, and expose ledger history from admin and portal APIs.

**Architecture:** keep account balance truth in account lots and settlement records, but add transactional ledger/audit records as first-class side effects of hold mutations. Refund will be implemented as a canonical mutation on `RequestSettlementRecord` plus lot restoration and ledger history, with idempotent replay keyed by request settlement identity instead of layering ad hoc balance compensation outside the account kernel.

**Tech Stack:** Rust, sqlx, Axum, utoipa, cargo test

---

### Task 1: Add failing account-kernel closure tests

**Files:**
- Modify: `crates/sdkwork-api-app-billing/tests/account_kernel_commands.rs`
- Modify: `crates/sdkwork-api-app-billing/tests/account_kernel_service.rs`

- [ ] **Step 1: Add a failing test that refunds a captured settlement and restores lot balances**

```rust
#[tokio::test]
async fn refund_account_settlement_restores_captured_balance() {
    // arrange captured settlement
    // refund 20.0 credits
    // assert settlement.status == Refunded
    // assert settlement.refunded_amount == 20.0
    // assert lot.remaining_quantity increases by 20.0
}
```

- [ ] **Step 2: Add a failing idempotency test for refund replay**

```rust
#[tokio::test]
async fn refund_account_settlement_replays_existing_refund() {
    // run refund twice for same settlement
    // assert same settlement id returned
    // assert only one ledger refund entry exists
}
```

- [ ] **Step 3: Add failing tests that create/release/capture/refund all emit ledger history and allocations**

```rust
assert_eq!(ledger_entries.iter().map(|entry| entry.entry_type).collect::<Vec<_>>(), vec![
    AccountLedgerEntryType::HoldCreate,
    AccountLedgerEntryType::HoldRelease,
    AccountLedgerEntryType::SettlementCapture,
    AccountLedgerEntryType::Refund,
]);
```

- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-app-billing --test account_kernel_commands -- --nocapture` and confirm the new tests fail first**

### Task 2: Extend transactional storage interfaces for ledger writes

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Test: `crates/sdkwork-api-storage-sqlite/tests/account_kernel_roundtrip.rs`

- [ ] **Step 1: Add transactional account-ledger methods to `AccountKernelTransaction`**

```rust
async fn upsert_account_ledger_entry_record(
    &mut self,
    record: &AccountLedgerEntryRecord,
) -> Result<AccountLedgerEntryRecord>;

async fn upsert_account_ledger_allocation(
    &mut self,
    record: &AccountLedgerAllocationRecord,
) -> Result<AccountLedgerAllocationRecord>;
```

- [ ] **Step 2: Implement the new transaction methods in SQLite and Postgres transaction adapters using existing `ai_account_ledger_entry` and `ai_account_ledger_allocation` tables**

- [ ] **Step 3: Add a storage roundtrip test proving ledger entry/allocation upserts participate in the same transaction boundary as lot mutations**

- [ ] **Step 4: Run targeted storage tests**

Run: `cargo test --offline -p sdkwork-api-storage-sqlite --test account_kernel_roundtrip -- --nocapture`

### Task 3: Implement canonical refund and transactional ledger history in app billing

**Files:**
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/Cargo.toml`
- Test: `crates/sdkwork-api-app-billing/tests/account_kernel_commands.rs`

- [ ] **Step 1: Introduce refund input/result types**

```rust
pub struct RefundAccountSettlementInput {
    pub request_settlement_id: u64,
    pub refund_ledger_entry_id: u64,
    pub refund_ledger_allocation_start_id: u64,
    pub refunded_amount: f64,
    pub refunded_at_ms: u64,
}
```

- [ ] **Step 2: Add internal helpers that build account ledger entries and ledger allocations for hold create/release/capture/refund**

- [ ] **Step 3: Update `create_account_hold`, `release_account_hold`, and `capture_account_hold` so lot mutations and ledger writes happen inside the same account-kernel transaction**

- [ ] **Step 4: Implement `refund_account_settlement(...)`**

```rust
// load settlement
// reject non-captured / over-refund states
// restore remaining_quantity on consumed lots
// write refund ledger entry + allocations
// update settlement.refunded_amount and status
```

- [ ] **Step 5: Re-run `cargo test --offline -p sdkwork-api-app-billing --test account_kernel_commands -- --nocapture` until green**

### Task 4: Expose account ledger/history APIs in admin and portal

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/account_billing_routes.rs`

- [ ] **Step 1: Add failing admin route tests for `/admin/billing/accounts/:id/ledger`**

- [ ] **Step 2: Add failing portal route tests for `/portal/billing/account/ledger`**

- [ ] **Step 3: Implement route handlers that return account ledger entries with related allocations filtered to account/workspace scope**

- [ ] **Step 4: Run targeted interface tests**

Run: `cargo test --offline -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`

Run: `cargo test --offline -p sdkwork-api-interface-portal --test account_billing_routes -- --nocapture`

### Task 5: Full regression for the billing closure slice

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-billing -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-interface-portal --test account_billing_routes -- --nocapture`**
- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test runtime_execution -- --nocapture` to confirm billing-usage integration still holds**
