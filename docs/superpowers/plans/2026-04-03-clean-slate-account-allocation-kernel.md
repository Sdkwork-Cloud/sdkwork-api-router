# Clean-Slate Account Allocation Kernel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the canonical account kernel by adding lot-allocation evidence objects so holds and ledger mutations can be audited without relying on legacy quota or summary paths.

**Architecture:** Treat this repository as a clean-slate commercial gateway. Extend the billing domain with canonical allocation records, extend `AccountKernelStore` with explicit allocation CRUD, land the physical tables in SQLite and PostgreSQL, and prove the SQLite path with red-green roundtrip tests before any gateway or UI cutover work.

**Tech Stack:** Rust, sqlx, SQLite, PostgreSQL, async-trait, anyhow, cargo test

---

### Task 1: Add canonical allocation domain records and storage seam

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`

- [ ] **Step 1: Add failing tests that reference `AccountHoldAllocationRecord` and `AccountLedgerAllocationRecord` through the storage roundtrip tests**
- [ ] **Step 2: Run the focused SQLite storage test and verify compile or test failure**
- [ ] **Step 3: Add canonical billing domain records for hold allocations and ledger allocations**
- [ ] **Step 4: Extend `AccountKernelStore` with explicit allocation insert and list methods**
- [ ] **Step 5: Re-run the focused SQLite storage test and confirm it still fails only because storage implementation is missing**

### Task 2: Land clean-slate allocation schema in SQLite and PostgreSQL

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/account_schema.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Extend migration tests to require `ai_account_hold_allocation` and `ai_account_ledger_allocation`**
- [ ] **Step 2: Run the focused migration tests and verify they fail**
- [ ] **Step 3: Add SQLite tables and indexes for hold allocation and ledger allocation evidence**
- [ ] **Step 4: Mirror the same canonical allocation tables in PostgreSQL migrations**
- [ ] **Step 5: Re-run the focused migration tests and confirm green**

### Task 3: Implement SQLite allocation CRUD and roundtrip proof

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/account_kernel_roundtrip.rs`

- [ ] **Step 1: Extend the roundtrip test to insert and read allocation records**
- [ ] **Step 2: Run the focused roundtrip test and verify it fails**
- [ ] **Step 3: Implement SQLite insert or list behavior plus row decoders for the two allocation record types**
- [ ] **Step 4: Re-run the focused roundtrip test and confirm green**
- [ ] **Step 5: Run `cargo test -p sdkwork-api-storage-sqlite -- --nocapture` and confirm the crate stays green**
