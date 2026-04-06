# Commercial Pricing Management Program Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** turn canonical pricing into a professional commercial billing surface with standardized charge units, pricing methods, admin write APIs, and a dedicated admin pricing management module.

**Architecture:** preserve the existing two-plane pricing model. Keep catalog pricing as the provider and market reference layer while upgrading canonical pricing plans and rates into the settlement-facing commercial source of truth. Land backend schema and API changes first, then wire the admin package and snapshot integration, then extend tenant-facing display.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, React, TypeScript, pnpm, cargo test

---

### Task 1: Enrich canonical pricing records in the billing domain

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/tests/account_kernel.rs`

- [x] **Step 1: Write a failing billing-domain test that expects pricing rates to retain charge units, pricing methods, rounding metadata, minimums, notes, and status**
- [x] **Step 2: Run `cargo test -p sdkwork-api-domain-billing --test account_kernel -- --nocapture` and verify the new test fails for the missing fields**
- [x] **Step 3: Extend `PricingRateRecord` with the new commercial pricing fields and builder helpers**
- [x] **Step 4: Re-run `cargo test -p sdkwork-api-domain-billing --test account_kernel -- --nocapture` and verify it passes**

### Task 2: Upgrade SQLite and PostgreSQL canonical pricing persistence

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/account_kernel_roundtrip.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [x] **Step 1: Write failing storage tests that round-trip enriched pricing plan and rate records**
- [ ] **Step 2: Run the targeted SQLite and PostgreSQL pricing tests and verify they fail for missing schema columns or decode logic**
- [ ] **Step 3: Add the new canonical pricing columns and CRUD bindings for both storage backends**
- [ ] **Step 4: Re-run the targeted storage tests and verify the richer pricing metadata survives persistence**

Progress note:
SQLite round-trip coverage now passes with the richer canonical pricing shape. PostgreSQL schema columns were aligned, but PostgreSQL account-kernel CRUD parity still needs to be finished before this task can be marked complete.

### Task 3: Add admin pricing management write routes

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`
- Modify: `docs/api-reference/admin-api.md`

- [x] **Step 1: Write failing admin route tests for creating canonical pricing plans and canonical pricing rates with token, request, image, audio, video, and music examples**
- [x] **Step 2: Run `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture` and verify the new routes fail**
- [x] **Step 3: Add typed admin request DTOs plus `POST /admin/billing/pricing-plans` and `POST /admin/billing/pricing-rates` handlers**
- [x] **Step 4: Re-run `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture` and verify the route suite passes**
- [x] **Step 5: Update the admin API reference to document canonical pricing management**

### Task 4: Extend admin shared types and API client for canonical pricing management

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Create: `apps/sdkwork-router-admin/tests/admin-pricing-api-surface.test.mjs`

- [x] **Step 1: Write a failing admin API surface test that expects rich commercial pricing record types plus create methods for plans and rates**
- [x] **Step 2: Run `node tests/admin-pricing-api-surface.test.mjs` and verify it fails**
- [x] **Step 3: Add TypeScript interfaces, enums or unions, and API client methods for canonical pricing management**
- [x] **Step 4: Re-run `node tests/admin-pricing-api-surface.test.mjs` and verify it passes**

### Task 5: Add a dedicated admin pricing management package

**Files:**
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-pricing/package.json`
- Create: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-pricing/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/package.json`
- Modify: `apps/sdkwork-router-admin/tsconfig.json`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-types/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routePaths.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routes.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-shell/src/application/router/routePrefetch.ts`
- Create: `apps/sdkwork-router-admin/tests/admin-pricing-module.test.mjs`

- [x] **Step 1: Write a failing module architecture test for a first-class admin pricing package and route wiring**
- [x] **Step 2: Run `node tests/admin-pricing-module.test.mjs` and verify it fails**
- [x] **Step 3: Implement the dedicated admin pricing package with plan and rate governance views, charge-unit framing, and billing-method framing**
- [x] **Step 4: Re-run `node tests/admin-pricing-module.test.mjs` and verify it passes**

### Task 6: Sync pricing data into the admin workbench snapshot and operator surfaces

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbench.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx`
- Create: `apps/sdkwork-router-admin/tests/admin-pricing-workbench.test.mjs`

- [x] **Step 1: Write a failing admin workbench test that expects pricing governance to surface richer charge units and billing methods**
- [x] **Step 2: Run `node tests/admin-pricing-workbench.test.mjs` and verify it fails**
- [x] **Step 3: Thread the richer pricing records through the snapshot and update the commercial and gateway pages to display them coherently**
- [x] **Step 4: Re-run `node tests/admin-pricing-workbench.test.mjs` and the existing commercial package tests**

Progress note:
Admin workbench surfaces now expose commercial pricing display units, billing methods, and primary settlement-rate semantics across the pricing package, commercial workspace, and gateway access and usage pages.

### Task 7: Extend portal display after admin authoring stabilizes

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/pages/index.tsx`
- Create: `apps/sdkwork-router-portal/tests/portal-pricing-display.test.mjs`
- Modify: `docs/api-reference/portal-api.md`

- [x] **Step 1: Write a failing portal test for active-plan and modality-aware pricing display**
- [x] **Step 2: Run `node tests/portal-pricing-display.test.mjs` and verify it fails**
- [x] **Step 3: Extend portal types, API client, and billing pages to present canonical pricing clearly**
- [x] **Step 4: Re-run the portal pricing test and targeted portal package verification**

Progress note:
Portal account and settlement modules now render operator-authored pricing semantics with human-readable billing methods, charge units, and display units instead of exposing only raw metric codes.
The lifecycle surface now also supports scheduling future-dated pricing plans into a `planned` state, giving commercial operators a first-class staging step between draft authoring and active publication.
Admin and portal pricing reads now auto-promote the latest due `planned` version into `active` status and archive replaced active siblings, which closes the main control-plane lifecycle gap before a background activator is introduced.
Admin now also exposes an explicit pricing-lifecycle synchronization operation so operators and automation can force due staged versions to converge without depending on a pricing-read request path.

### Task 8: Verify the slice and record remaining pricing roadmap gaps

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-commercial-pricing-architecture-design.md`
- Modify: `docs/superpowers/plans/2026-04-03-commercial-pricing-management-program.md`

- [x] **Step 1: Run the targeted Rust tests for billing domain, storage, and admin routes**
- [x] **Step 2: Run the targeted Node tests for admin pricing API, module, and workbench coverage**
- [x] **Step 3: Run `pnpm.cmd --dir apps/sdkwork-router-admin typecheck`**
- [x] **Step 4: Update the spec and plan notes with any gaps discovered during verification**
