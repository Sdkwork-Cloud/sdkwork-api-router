# Admin Portal Product Differentiation Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the already-separated admin and portal URLs into clearly different products with isolated design systems and information architecture.

**Architecture:** Keep backend separation unchanged, add source-level independence checks, replace the shared browser theme with app-specific stylesheets, and redesign each app shell around its own user role.

**Tech Stack:** React 19, TypeScript, Vite, Node test runner, pnpm

---

## Chunk 1: Structural Guardrails

### Task 1: Add a failing independence test

**Files:**
- Create: `console/tests/independent-apps.test.mjs`

- [ ] Assert that admin, portal, and landing use dedicated stylesheet entry points.
- [ ] Assert that admin and portal expose distinct product root classes.
- [ ] Assert that admin and portal SDKs keep distinct storage keys and API prefixes.
- [ ] Assert that HTML entry points remain separate.
- [ ] Run `cd console && node --test tests/independent-apps.test.mjs` and confirm it fails before implementation.

## Chunk 2: App-Specific Styling

### Task 2: Replace the shared theme entry point

**Files:**
- Modify: `console/src/LandingApp.tsx`
- Modify: `console/src/admin/App.tsx`
- Modify: `console/src/portal/App.tsx`
- Create: `console/src/landing.css`
- Create: `console/src/admin/admin.css`
- Create: `console/src/portal/portal.css`
- Delete: `console/src/App.css`

- [ ] Move landing onto its own stylesheet.
- [ ] Move admin onto an admin-only stylesheet.
- [ ] Move portal onto a portal-only stylesheet.
- [ ] Ensure neither app imports the old shared theme.

## Chunk 3: Admin Product Redesign

### Task 3: Reframe admin around operator workflows

**Files:**
- Modify: `console/src/admin/App.tsx`
- Modify: `console/packages/sdkwork-api-workspace/src/index.tsx`
- Modify: `console/packages/sdkwork-api-channel/src/index.tsx`
- Modify: `console/packages/sdkwork-api-usage/src/index.tsx`
- Modify: `console/packages/sdkwork-api-runtime/src/index.tsx`

- [ ] Add an admin command-center shell with summary band and section navigation.
- [ ] Tighten copy so the app reads like an operator control plane.
- [ ] Style admin modules with a denser telemetry-first presentation.

## Chunk 4: Portal Product Redesign

### Task 4: Reframe portal around self-service workflows

**Files:**
- Modify: `console/src/portal/App.tsx`
- Modify: `console/packages/sdkwork-api-portal-auth/src/index.tsx`
- Modify: `console/packages/sdkwork-api-portal-user/src/index.tsx`

- [ ] Add a portal shell with onboarding-oriented copy and action framing.
- [ ] Make portal cards and forms feel product-led rather than operator-led.
- [ ] Emphasize workspace access and API key issuance as the primary user journey.

## Chunk 5: Verification

### Task 5: Prove the redesign keeps independence intact

**Files:**
- None

- [ ] Run `cd console && node --test tests/independent-apps.test.mjs`.
- [ ] Run `pnpm --dir console typecheck`.
- [ ] Run `pnpm --dir console build`.
- [ ] Verify `/admin/` and `/portal/` still load distinct entry documents at runtime.
