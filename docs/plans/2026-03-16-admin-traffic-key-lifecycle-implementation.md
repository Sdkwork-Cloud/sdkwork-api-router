# Admin Traffic Query and Gateway Key Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Elevate `sdkwork-router-admin` into a stronger daily operations console by adding true gateway API key deletion and a more capable traffic query surface with operator-grade filtering and export.

**Architecture:** Extend the control plane with an explicit gateway key delete contract while preserving revoke/restore semantics, and upgrade the standalone React traffic module with richer client-side query primitives over live usage and routing datasets already loaded into the workspace snapshot.

**Tech Stack:** Rust, Axum, sqlx, React 19, TypeScript, Vite, pnpm workspace, Node test runner

---

## Chunk 1: Backend Key Lifecycle

### Task 1: Prove gateway API keys are missing delete support

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] Add a failing route test that issues a gateway key, deletes it through the admin API, and verifies the key is absent from the inventory and cannot authenticate requests.
- [ ] Run the focused Rust test and confirm it fails before production code is touched.

### Task 2: Add gateway API key delete support end to end

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

- [ ] Add store-level delete behavior for hashed gateway keys.
- [ ] Expose a control-plane delete helper in the identity application layer.
- [ ] Add `DELETE /admin/api-keys/{hashed_key}`.
- [ ] Re-run the targeted Rust test and confirm it passes.

## Chunk 2: Traffic Query Productization

### Task 3: Protect the enhanced traffic workspace with a failing frontend architecture test

**Files:**
- Modify: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`

- [ ] Add assertions for advanced traffic filters and CSV export wording.
- [ ] Run the targeted Node test and confirm it fails before UI changes.

### Task 4: Implement the stronger traffic query surface and key-delete UI

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/README.md`

- [ ] Add frontend delete support for gateway keys.
- [ ] Add key inventory actions that keep revoke/restore and add explicit deletion.
- [ ] Add richer traffic filters for project, provider, model, user scope, and a recent-window selector.
- [ ] Add CSV export for the filtered usage/routing result sets.
- [ ] Update documentation so the admin feature map matches the product surface.

## Chunk 3: Verification

### Task 5: Prove the product works

**Files:**
- None

- [ ] Run `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`.
- [ ] Run `node --test apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin typecheck`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin build`.
- [ ] Run `git diff --check -- crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-sqlite/src/lib.rs crates/sdkwork-api-storage-postgres/src/lib.rs crates/sdkwork-api-app-identity/src/lib.rs crates/sdkwork-api-interface-admin/src/lib.rs crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/src/index.tsx apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/src/index.tsx apps/sdkwork-router-admin/tests/admin-architecture.test.mjs apps/sdkwork-router-admin/README.md docs/plans/2026-03-16-admin-traffic-key-lifecycle-implementation.md`.
