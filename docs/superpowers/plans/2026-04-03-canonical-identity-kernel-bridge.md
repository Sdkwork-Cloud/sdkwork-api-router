# Canonical Identity Kernel Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add the first real canonical identity kernel and API-key-to-`GatewayAuthSubject` resolution path so the commercial account kernel has a valid request principal to bill.

**Architecture:** keep legacy workspace-string identity flows alive, but add a parallel canonical identity kernel with dedicated domain records, storage seam, SQLite CRUD, PostgreSQL schema mirror, and app-identity resolution for API keys. This creates the clean subject boundary needed before account lookup and hold-settle gateway cutover.

**Tech Stack:** Rust, anyhow, async-trait, sqlx, SQLite, PostgreSQL, cargo test

---

### Task 1: Add failing tests for canonical identity records and API-key auth-subject resolution

**Files:**
- Create: `crates/sdkwork-api-domain-identity/tests/identity_kernel.rs`
- Create: `crates/sdkwork-api-storage-sqlite/tests/identity_kernel_roundtrip.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/create_api_key.rs`

- [ ] **Step 1: Write failing domain tests for canonical identity records**
- [ ] **Step 2: Write a failing SQLite roundtrip test for `ai_user`, `ai_api_key`, and `ai_identity_binding`**
- [ ] **Step 3: Write a failing app-identity test that resolves `GatewayAuthSubject` from a canonical API key**
- [ ] **Step 4: Run the focused test commands and verify failure**

### Task 2: Add canonical identity domain records and storage seam

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`

- [ ] **Step 1: Add `IdentityUserRecord`, `CanonicalApiKeyRecord`, and `IdentityBindingRecord`**
- [ ] **Step 2: Add `IdentityKernelStore` with explicit unsupported defaults**
- [ ] **Step 3: Re-run the focused tests and confirm they still fail only because storage or app logic is missing**

### Task 3: Land SQLite and PostgreSQL schema for the canonical identity kernel

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Extend migration tests to require `ai_user`, `ai_api_key`, and `ai_identity_binding`**
- [ ] **Step 2: Run focused migration tests and verify failure**
- [ ] **Step 3: Add the SQLite tables and indexes**
- [ ] **Step 4: Mirror the same tables and indexes in PostgreSQL migrations**
- [ ] **Step 5: Re-run migration tests and confirm green**

### Task 4: Implement SQLite CRUD and app-identity canonical API-key resolution

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/create_api_key.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/identity_kernel_roundtrip.rs`

- [ ] **Step 1: Implement SQLite CRUD and row decoders for canonical user, API key, and identity binding records**
- [ ] **Step 2: Add `resolve_gateway_auth_subject_from_api_key(store, plaintext_key)`**
- [ ] **Step 3: Run focused tests for domain, storage, and app identity; confirm green**
- [ ] **Step 4: Run package verification for app-identity and storage-sqlite**

### Task 5: Update the audit baseline and checkpoint the slice

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-router-implementation-audit-and-upgrade-plan.md`

- [ ] **Step 1: Record that canonical identity storage and API-key auth-subject resolution now exist**
- [ ] **Step 2: Note that account lookup and transactional hold-settle mutation still remain**
- [ ] **Step 3: Commit with a message like `feat: add canonical identity kernel bridge`**
