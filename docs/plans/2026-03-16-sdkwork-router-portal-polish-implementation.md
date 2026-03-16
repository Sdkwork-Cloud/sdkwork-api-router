# SDKWork Router Portal Polish Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Raise `apps/sdkwork-router-portal` from a functional standalone portal to a more product-complete, architecture-aligned experience with stronger package boundaries, richer usage and billing interactions, and better key-management ergonomics.

**Architecture:** Keep the root app shell thin while pushing portal business modules closer to the `ARCHITECT.md` standard. Business packages should expose a page entry from `pages/`, pull external data through package-local `repository/` helpers, centralize derived product logic in `services/`, and define local view contracts in `types/`. Product polish should stay additive on top of the existing live `/portal/*` read models and seed-backed commerce seam.

**Tech Stack:** React 19, TypeScript, Vite, pnpm workspace, Node test runner

---

## Chunk 1: Architecture Alignment

### Task 1: Enforce `ARCHITECT.md`-style business package structure

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify/Create: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-{dashboard,usage,api-keys,credits,billing,account,auth}/src/**`

- [ ] Extend the portal architecture test so each portal business package must contain `types/`, `components/`, `repository/`, `services/`, and `pages/`.
- [ ] Refactor the selected package entrypoints so `src/index.tsx` becomes a re-export surface rather than the full implementation file.
- [ ] Move live data access and derived product logic out of page bodies into package-local helpers.
- [ ] Re-run `node --test apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`.

## Chunk 2: Product Interaction Polish

### Task 2: Upgrade dashboard, usage, and API key workflows

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/**`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-usage/src/**`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/**`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/src/theme.css`

- [ ] Add dashboard insight cards and recommended next-action logic derived from live usage, billing, and API key posture.
- [ ] Add usage filtering and spotlight summaries so heavy requests, model selection, and provider mix are easier to understand.
- [ ] Add API key environment summaries, plaintext copy ergonomics, and a stronger first-call integration handoff.
- [ ] Add any shared UI primitives needed for the improved product flows.

### Task 3: Upgrade credits and billing decision support

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-credits/src/**`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing/src/**`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commerce/src/index.ts`
- Modify: `apps/sdkwork-router-portal/src/theme.css`

- [ ] Add redemption-impact previews so coupon entry feels like a real growth motion rather than a raw input box.
- [ ] Add billing recommendations using live quota posture and seed-backed plans or packs.
- [ ] Add clearer subscription comparison and recharge decision support for launch readiness.

## Chunk 3: Product Documentation

### Task 4: Refresh portal product docs and roadmap language

**Files:**
- Modify: `apps/sdkwork-router-portal/README.md`
- Modify: `docs/plans/2026-03-16-sdkwork-router-control-plane-product-roadmap.md`

- [ ] Update the portal README so the package structure and product capabilities reflect the polished module boundaries.
- [ ] Expand the control-plane roadmap language so the portal and super-admin surfaces read like a coherent product system.

## Chunk 4: Verification

### Task 5: Prove the polish pass still builds and documents correctly

**Files:**
- None

- [ ] Run `node --test apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`.
- [ ] Run `pnpm --dir apps/sdkwork-router-portal typecheck`.
- [ ] Run `pnpm --dir apps/sdkwork-router-portal build`.
- [ ] Run `pnpm --dir docs typecheck`.
- [ ] Run `pnpm --dir docs build`.
