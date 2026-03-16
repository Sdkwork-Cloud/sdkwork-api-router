# SDKWork Router Portal Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver `apps/sdkwork-router-portal` as a standalone portal product with independent architecture, real portal dashboard and usage reads, migrated auth and API key flows, and seam-backed commerce modules.

**Architecture:** Build a new pnpm workspace that follows `ARCHITECT.md`, extend the current portal backend with project-scoped dashboard, usage, and billing endpoints, and split frontend data access between live portal APIs and local repository seams for commerce capabilities that do not yet have backend support.

**Tech Stack:** Rust, Axum, React 19, TypeScript, Vite, pnpm workspace, Node test runner

---

## Chunk 1: Structural Guardrails

### Task 1: Add failing architecture tests for the standalone portal app

**Files:**
- Create: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`

- [ ] Write a Node test that asserts `apps/sdkwork-router-portal/package.json`, `src/App.tsx`, and `src/theme.css` exist.
- [ ] Assert the required package set exists under `apps/sdkwork-router-portal/packages`.
- [ ] Assert the shell route manifest includes `dashboard`, `api-keys`, `usage`, `credits`, `billing`, and `account`.
- [ ] Assert the app root imports `./theme.css` and does not import from `console/`.
- [ ] Run `cd apps/sdkwork-router-portal && node --test tests/portal-architecture.test.mjs` and confirm it fails before scaffolding.

## Chunk 2: Portal Backend Read Models

### Task 2: Add request-level usage details to persisted usage records

**Files:**
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-app-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-app-usage/tests/record_usage.rs`
- Modify: `crates/sdkwork-api-app-usage/tests/usage_summary.rs`

- [ ] Add a failing test that proves usage records carry `units`, `amount`, and `created_at_ms`.
- [ ] Update the usage domain model and persistence APIs.
- [ ] Add SQLite and PostgreSQL schema expansion for the new columns.
- [ ] Re-run the targeted usage tests and confirm they pass.

### Task 3: Expose project-scoped portal dashboard, usage, and billing endpoints

**Files:**
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_api_keys.rs`
- Create: `crates/sdkwork-api-interface-portal/tests/portal_dashboard.rs`

- [ ] Write failing portal route tests for `GET /portal/dashboard`, `GET /portal/usage/records`, `GET /portal/usage/summary`, `GET /portal/billing/summary`, and `GET /portal/billing/ledger`.
- [ ] Add project-scoped DTOs and app helpers for portal dashboard reads.
- [ ] Wire the new authenticated portal routes.
- [ ] Re-run the targeted portal interface tests and confirm they pass.

## Chunk 3: Standalone Portal Workspace

### Task 4: Scaffold `apps/sdkwork-router-portal`

**Files:**
- Create: `apps/sdkwork-router-portal/package.json`
- Create: `apps/sdkwork-router-portal/pnpm-workspace.yaml`
- Create: `apps/sdkwork-router-portal/tsconfig.json`
- Create: `apps/sdkwork-router-portal/turbo.json`
- Create: `apps/sdkwork-router-portal/vite.config.ts`
- Create: `apps/sdkwork-router-portal/index.html`
- Create: `apps/sdkwork-router-portal/src/main.tsx`
- Create: `apps/sdkwork-router-portal/src/App.tsx`
- Create: `apps/sdkwork-router-portal/src/theme.css`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/...`

- [ ] Create the standalone workspace root, scripts, and Vite entry.
- [ ] Create the foundation packages and route manifest.
- [ ] Re-run `node --test tests/portal-architecture.test.mjs` and confirm the remaining failures are only missing product modules if any.

## Chunk 4: Migrate Existing Live Portal Flows

### Task 5: Migrate auth and API key capabilities out of `console/`

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-auth/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/...`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/...`
- Modify: `apps/sdkwork-router-portal/src/App.tsx`

- [ ] Write failing component or structural tests where practical for auth and key workflows.
- [ ] Port the current portal session, login, register, password change, and API key create/list flows into app-local packages.
- [ ] Keep root `src/` as shell composition only.

## Chunk 5: Product Modules

### Task 6: Build dashboard, usage, credits, billing, and account modules

**Files:**
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/...`
- Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-account/...`

- [ ] Add dashboard cards for points, token units, request count, key posture, and recent API requests using live portal data.
- [ ] Add a usage workbench with request rows and model/provider summaries.
- [ ] Add a credits module that converts billing/quota data into clear points posture and redemption entry.
- [ ] Add a billing module using repository-backed subscription, recharge, and coupon data.
- [ ] Add an account module for profile and password rotation.

## Chunk 6: Product Polish

### Task 7: Deliver portal-grade shell, interactions, and seeded commerce seams

**Files:**
- Modify: `apps/sdkwork-router-portal/src/theme.css`
- Modify: portal business packages as needed

- [ ] Add a polished shell with sidebar navigation, hero treatment, status chips, and clear empty states.
- [ ] Add commerce repository seams and seeded plan, recharge, and coupon data.
- [ ] Ensure copy, hierarchy, and interaction states feel like a product instead of a demo screen.

## Chunk 7: Verification

### Task 8: Prove the independent portal app and backend endpoints work

**Files:**
- None

- [ ] Run `cd apps/sdkwork-router-portal && node --test tests/portal-architecture.test.mjs`.
- [ ] Run targeted Rust tests for portal routes and usage persistence.
- [ ] Run `pnpm --dir apps/sdkwork-router-portal install`.
- [ ] Run `pnpm --dir apps/sdkwork-router-portal typecheck`.
- [ ] Run `pnpm --dir apps/sdkwork-router-portal build`.
