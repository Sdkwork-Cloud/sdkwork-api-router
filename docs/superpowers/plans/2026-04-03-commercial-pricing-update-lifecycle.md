# Commercial Pricing Update Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add first-class update lifecycle support for canonical pricing plans and pricing rates so admin operators can maintain existing commercial pricing definitions instead of only creating new ones.

**Architecture:** preserve the existing two-plane pricing model and extend the settlement-facing canonical pricing control plane with explicit update routes, client methods, and admin pricing workbench editing. Backend persistence should expose clear account-kernel upsert semantics rather than relying on UI-side duplication or implicit recreation.

**Tech Stack:** Rust, Axum, sqlx, SQLite, PostgreSQL, React, TypeScript, pnpm, cargo test

---

### Task 1: Add failing backend tests for pricing-plan and pricing-rate updates

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/account_billing_routes.rs`
- Modify: `apps/sdkwork-router-admin/tests/admin-pricing-api-surface.test.mjs`

- [x] **Step 1: Add a failing admin route test for `PUT /admin/billing/pricing-plans/{pricing_plan_id}`**
- [x] **Step 2: Run the targeted Rust route test and verify it fails for the missing update route**
- [x] **Step 3: Add a failing admin route test for `PUT /admin/billing/pricing-rates/{pricing_rate_id}`**
- [x] **Step 4: Run the targeted Rust route test and verify it fails for the missing update route**
- [x] **Step 5: Add a failing admin TypeScript API surface test for `updateCommercialPricingPlan` and `updateCommercialPricingRate`**
- [x] **Step 6: Run the targeted Node test and verify it fails for the missing client methods**

### Task 2: Audit storage upsert semantics for pricing-plan and pricing-rate updates

**Files:**
- Review: `crates/sdkwork-api-app-billing/src/lib.rs`
- Review: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Review: `crates/sdkwork-api-storage-postgres/src/lib.rs`

- [x] **Step 1: Review existing account-kernel pricing persistence semantics before expanding traits**
- [x] **Step 2: Confirm whether admin billing update routes can reuse existing storage methods safely**
- [x] **Step 3: Verify SQLite pricing writes already perform `ON CONFLICT ... DO UPDATE` behavior**
- [x] **Step 4: Verify PostgreSQL pricing writes already perform `ON CONFLICT ... DO UPDATE` behavior**
- [x] **Step 5: Record that no new storage-trait surface is required for the first update-lifecycle slice**

Progress note:
The existing pricing insert methods in both SQLite and PostgreSQL already act as upserts. That made a storage-trait expansion unnecessary for this lifecycle slice, so the work shifted to admin routing, client, and UI maintenance.

### Task 3: Add admin update routes and API reference entries

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `docs/api-reference/admin-api.md`

- [x] **Step 1: Add typed request DTOs and handlers for pricing-plan update**
- [x] **Step 2: Add typed request DTOs and handlers for pricing-rate update**
- [x] **Step 3: Wire `PUT /admin/billing/pricing-plans/{pricing_plan_id}` and `PUT /admin/billing/pricing-rates/{pricing_rate_id}` into the admin router and OpenAPI surface**
- [x] **Step 4: Document the new update routes in the admin API reference**
- [x] **Step 5: Re-run `cargo test -p sdkwork-api-interface-admin --test account_billing_routes -- --nocapture`**

### Task 4: Extend admin TypeScript API client and workbench actions

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbenchActions.ts`
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/workbench.tsx`

- [x] **Step 1: Add `updateCommercialPricingPlan` and `updateCommercialPricingRate` client methods**
- [x] **Step 2: Expose workbench actions for updating pricing plans and pricing rates**
- [x] **Step 3: Re-run the admin pricing API surface test**

### Task 5: Upgrade the admin pricing package from create-only to edit-and-maintain

**Files:**
- Modify: `apps/sdkwork-router-admin/packages/sdkwork-router-admin-pricing/src/index.tsx`
- Modify: `apps/sdkwork-router-admin/tests/admin-pricing-module.test.mjs`
- Modify: `apps/sdkwork-router-admin/tests/admin-pricing-workbench.test.mjs`

- [x] **Step 1: Add failing UI assertions for edit/update pricing flows**
- [x] **Step 2: Implement pricing-plan edit state and pricing-rate edit state in the pricing package**
- [x] **Step 3: Reuse the shared commercial pricing helpers so edit mode preserves the same billing semantics labels**
- [x] **Step 4: Re-run the targeted admin pricing tests and `pnpm.cmd typecheck`**

Progress note:
The pricing package now supports create and update flows from the same workbench, including reusable edit state for pricing plans and rates plus richer commercial input fields needed for multimodal pricing governance.
It also supports operator-driven version cloning so a live plan can be forked into a new draft version with all settlement-facing rates carried forward safely, controlled publishing so draft versions can be promoted while older active siblings are archived automatically, and explicit plan retirement so obsolete plan bundles can be archived without deleting history.

### Task 6: Verify the slice and update roadmap notes

**Files:**
- Modify: `docs/superpowers/plans/2026-04-03-commercial-pricing-management-program.md`
- Modify: `docs/superpowers/plans/2026-04-03-commercial-pricing-update-lifecycle.md`

- [x] **Step 1: Mark completed tasks in the pricing-management plan**
- [x] **Step 2: Record any remaining lifecycle gaps such as publish/clone/retire workflows**
- [x] **Step 3: Summarize the next highest-value commercial pricing gap after update support lands**

Remaining lifecycle gaps:
- finer-grained lifecycle controls such as rate-level retirement overrides, approval workflows, and background activation of planned pricing versions without relying on either explicit admin synchronization or pricing-read traffic

Next highest-value gap:
Add approval controls and background activation so commercial teams can review future-dated pricing changes, approve them explicitly, and let the router promote planned versions automatically even when no control-plane pricing read happens at the activation moment.
