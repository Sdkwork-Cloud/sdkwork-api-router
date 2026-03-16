# Admin Portal Separation Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split admin and portal into independent browser apps while adding real default users and password change flows for both.

**Architecture:** Keep backend admin and portal APIs as separate security domains, add persisted admin accounts, seed local-development default users on demand, and move frontend traffic onto distinct app entry points plus `/api/*` proxy prefixes.

**Tech Stack:** Rust, Axum, SQLx, React, Vite, TypeScript, pnpm

---

## Chunk 1: Backend Identity Model

### Task 1: Add admin account domain and store support

**Files:**
- Modify: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [ ] Add admin user record/profile types and store trait methods.
- [ ] Add SQLite and Postgres persistence for admin users.
- [ ] Add schema creation for `admin_users`.

### Task 2: Extend app identity flows

**Files:**
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Test: `crates/sdkwork-api-app-identity/tests/portal_identity.rs`
- Test: `crates/sdkwork-api-app-identity/tests/admin_identity.rs`

- [ ] Write failing tests for default admin login and admin password change.
- [ ] Write failing tests for default portal login and portal password change.
- [ ] Implement seeded default users, login, profile loading, and password change flows.

## Chunk 2: HTTP Interface

### Task 3: Upgrade admin auth API

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/auth_and_project_routes.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/support/mod.rs`

- [ ] Write failing tests for email/password login, `me`, and password change.
- [ ] Replace `subject` login with real admin credentials.
- [ ] Return authenticated admin profile from `me`.

### Task 4: Extend portal auth API

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_auth.rs`

- [ ] Write failing tests for seeded portal login and password change.
- [ ] Add `change-password` route and wire seeded default portal login.

## Chunk 3: Frontend Separation

### Task 5: Split browser entries

**Files:**
- Modify: `console/vite.config.ts`
- Modify: `console/index.html`
- Create: `console/admin/index.html`
- Create: `console/portal/index.html`
- Create: `console/src/admin/main.tsx`
- Create: `console/src/portal/main.tsx`
- Create: `console/src/admin/App.tsx`
- Create: `console/src/portal/App.tsx`

- [ ] Add a lightweight root landing page.
- [ ] Create independent admin and portal entry documents.
- [ ] Remove shared hash-based shell coupling.

### Task 6: Add frontend auth/session support

**Files:**
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-portal-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-portal-user/src/index.tsx`
- Modify: `console/packages/sdkwork-api-types/src/index.ts`

- [ ] Add admin token persistence helpers and auth APIs.
- [ ] Add password change APIs for admin and portal.
- [ ] Update portal UI to expose password change.

### Task 7: Add independent admin UI

**Files:**
- Modify: `console/packages/sdkwork-api-workspace/src/index.tsx`
- Create: `console/src/admin/AdminLoginPage.tsx`

- [ ] Create an admin login experience with default local credentials.
- [ ] Add authenticated admin dashboard and password change UI.

## Chunk 4: Docs and Verification

### Task 8: Update docs

**Files:**
- Modify: `docs/getting-started/quickstart.md`
- Modify: `docs/api-reference/overview.md`
- Modify: `docs/api-reference/admin-api.md`
- Modify: `docs/api-reference/portal-api.md`
- Modify: `docs/zh/getting-started/quickstart.md`
- Modify: `docs/zh/api-reference/overview.md`
- Modify: `docs/zh/api-reference/admin-api.md`
- Modify: `docs/zh/api-reference/portal-api.md`

- [ ] Document separate app URLs.
- [ ] Document default local credentials.
- [ ] Document password change endpoints.

### Task 9: Verify

**Files:**
- None

- [ ] Run targeted Rust tests for app identity, admin interface, and portal interface.
- [ ] Run `pnpm --dir console typecheck`.
- [ ] Start the workspace and verify `/admin/` and `/portal/` load independently.
