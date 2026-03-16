# Admin Credential Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver full admin-side credential lifecycle management for proxy providers so the standalone `sdkwork-router-admin` can list, create, update, and delete provider credentials end to end.

**Architecture:** Extend the admin control plane with a true credential delete contract, implement secret-store cleanup across supported backends, and surface the resulting lifecycle inside the standalone catalog workbench without weakening the current `admin` and `portal` app separation.

**Tech Stack:** Rust, Axum, sqlx, React 19, TypeScript, Vite, pnpm workspace, Node test runner

---

## Chunk 1: Backend Contract

### Task 1: Prove the admin API is missing credential delete support

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] Add a failing test that creates a credential, deletes it through the admin API, then verifies the credential list is empty and secret resolution fails.
- [ ] Run `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes credential -- --nocapture` and confirm the new test fails for the missing route or delete behavior.

### Task 2: Add credential delete support through the control plane

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-app-credential/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] Add store-level credential delete behavior.
- [ ] Ensure secret payload cleanup is performed for database, local-file, and keyring-backed credentials.
- [ ] Expose `DELETE /admin/credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}`.
- [ ] Re-run the targeted Rust test and confirm it passes.

## Chunk 2: Standalone Admin Product Surface

### Task 3: Add credential inventory to the standalone admin app

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`

- [ ] Add frontend types and API calls for credential list/save/delete.
- [ ] Load credentials into the admin workspace snapshot.
- [ ] Expand Catalog into a credential operations console with lifecycle controls and coverage visibility.
- [ ] Update architecture assertions so the new capability stays protected.

## Chunk 3: Verification

### Task 4: Prove the feature works across backend and frontend

**Files:**
- None

- [ ] Run `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`.
- [ ] Run `node --test apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin typecheck`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin build`.
- [ ] Run `git diff --check -- crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-app-credential/src/lib.rs crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/src/index.tsx apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`.
