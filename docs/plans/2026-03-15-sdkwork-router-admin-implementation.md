# SDKWork Router Admin Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a standalone `apps/sdkwork-router-admin` super-admin application with a real package architecture, a product-grade admin shell, and live integration with the current admin backend where available.

**Architecture:** Build the admin app as an independent pnpm workspace following `ARCHITECT.md`, keep business logic inside app-local packages, and split data access between live admin API adapters and repository-backed mock domains for capabilities that are not server-side yet.

**Tech Stack:** React 19, TypeScript, Vite, pnpm workspace, Node test runner

---

## Chunk 1: Design Freeze and Structural Tests

### Task 1: Add failing structure tests for the new independent admin app

**Files:**
- Create: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`

- [ ] Write a Node test that asserts the app root exists.
- [ ] Assert the required package set exists under `apps/sdkwork-router-admin/packages`.
- [ ] Assert the shell route manifest includes the management sections.
- [ ] Assert the app root imports its own theme and does not import from `console/`.
- [ ] Run `cd apps/sdkwork-router-admin && node --test tests/admin-architecture.test.mjs` and confirm it fails.

## Chunk 2: Standalone App Scaffold

### Task 2: Create the independent workspace and foundation packages

**Files:**
- Create: `apps/sdkwork-router-admin/package.json`
- Create: `apps/sdkwork-router-admin/pnpm-workspace.yaml`
- Create: `apps/sdkwork-router-admin/tsconfig.json`
- Create: `apps/sdkwork-router-admin/turbo.json`
- Create: `apps/sdkwork-router-admin/vite.config.ts`
- Create: `apps/sdkwork-router-admin/index.html`
- Create: `apps/sdkwork-router-admin/src/main.tsx`
- Create: `apps/sdkwork-router-admin/src/App.tsx`
- Create: `apps/sdkwork-router-admin/src/theme.css`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commons/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/...`

- [ ] Create the workspace root and scripts.
- [ ] Create foundation packages matching `ARCHITECT.md`.
- [ ] Re-run the structure test and confirm it still fails only for missing routes/modules if needed.

## Chunk 3: Product Modules

### Task 3: Implement business modules and shell composition

**Files:**
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-auth/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-overview/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-users/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-tenants/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-coupons/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-catalog/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-traffic/...`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations/...`
- Modify: `apps/sdkwork-router-admin/src/App.tsx`

- [ ] Add shell navigation, command bar, and dashboard layout.
- [ ] Add management workbenches for each major admin module.
- [ ] Keep root `src/` as composition only.

## Chunk 4: Live Admin Data

### Task 4: Wire current admin backend into the new app

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/...`
- Modify: module packages as needed

- [ ] Add login/session handling.
- [ ] Add live fetchers for tenants, projects, channels, providers, models, usage, billing, routing, and runtime data.
- [ ] Present unavailable domains through repository seams instead of leaking fetch logic into pages.

## Chunk 5: Minimal Backend Extension

### Task 5: Expose live admin and portal user lists for the new app

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] Write failing tests for new user listing endpoints.
- [ ] Add store list methods for operator and portal users.
- [ ] Expose read-only admin endpoints for those lists.
- [ ] Verify the new tests pass.

## Chunk 6: Verification

### Task 6: Prove the new admin app builds independently

**Files:**
- None

- [ ] Run `cd apps/sdkwork-router-admin && node --test tests/admin-architecture.test.mjs`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin install`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin typecheck`.
- [ ] Run `pnpm --dir apps/sdkwork-router-admin build`.
- [ ] Run targeted Rust admin interface tests for the new user list endpoints.
