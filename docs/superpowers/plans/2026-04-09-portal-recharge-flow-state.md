# Portal Recharge Flow-State Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add a dynamic flow-state tracker to the recharge quote area so operators can read the current recharge-to-settlement step at a glance.

**Architecture:** Extend the presentation helper layer with a pure flow-state builder that maps current page state into three visual stages, then render a focused flow tracker component in the quote card. Keep all logic front-end only and preserve existing CTA, handoff, and pending-settlement behavior.

**Tech Stack:** React 19, TypeScript, Node test runner, existing portal commons UI

---

### Task 1: Lock the Flow-State Contract

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-recharge-presentation.test.mjs`

- [ ] **Step 1: Add failing page-contract assertions**

Add assertions for:

- `data-slot="portal-recharge-flow-tracker"`
- `Funding flow`
- `Choose amount`
- `Create order`
- `Complete payment in billing`

- [ ] **Step 2: Add a failing presentation-helper test**

Add a pure helper test covering:

- pre-selection state
- selection-ready state
- post-order handoff state
- pending-settlement attention state

- [ ] **Step 3: Run targeted tests to verify they fail**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs tests/portal-recharge-presentation.test.mjs`

Expected: FAIL because the flow tracker helper and page slot do not exist yet.

### Task 2: Implement Flow-State Derivation

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/presentation.ts`

- [ ] **Step 1: Add a pure flow-state builder**

Return a structured tracker model with:

- tracker title
- ordered steps
- step status
- short detail copy

- [ ] **Step 2: Keep logic narrow**

Derive the tracker only from current page state:

- active selection
- quote availability
- handoff mode
- pending settlement queue

- [ ] **Step 3: Re-run the presentation test**

Run: `node --test tests/portal-recharge-presentation.test.mjs`

Expected: PASS.

### Task 3: Render the Flow Tracker

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`

- [ ] **Step 1: Read the tracker state from the presentation layer**

Compute the tracker model next to the existing CTA and handoff state.

- [ ] **Step 2: Add a focused quote-card component**

Render the new `portal-recharge-flow-tracker` section above the quote hero with responsive three-stage cards or connectors.

- [ ] **Step 3: Preserve existing hierarchy**

Do not change existing order creation, handoff, mobile CTA, or pending-settlement behavior.

### Task 4: Verify the Full Portal Surface

**Files:**
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx`
- Verify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/presentation.ts`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-presentation.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-recharge-finance-projection.test.mjs`
- Verify: `apps/sdkwork-router-portal/tests/portal-runtime-dependency-governance.test.mjs`

- [ ] **Step 1: Run portal tests**

Run: `node --test tests/portal-recharge-center.test.mjs tests/portal-recharge-workflow-polish.test.mjs tests/portal-recharge-presentation.test.mjs tests/portal-recharge-finance-projection.test.mjs tests/portal-runtime-dependency-governance.test.mjs`

Expected: PASS.

- [ ] **Step 2: Run typecheck**

Run: `pnpm typecheck`

Expected: PASS.

- [ ] **Step 3: Run production build**

Run: `pnpm build`

Expected: PASS.

- [ ] **Step 4: Commit and push**

Run:

```bash
git add docs/superpowers/specs/2026-04-09-portal-recharge-flow-state-design.md docs/superpowers/plans/2026-04-09-portal-recharge-flow-state.md apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/presentation.ts apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge/src/pages/index.tsx apps/sdkwork-router-portal/tests/portal-recharge-center.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-workflow-polish.test.mjs apps/sdkwork-router-portal/tests/portal-recharge-presentation.test.mjs
git commit -m "feat(portal): add recharge flow tracker"
git push origin main
```

Expected: local `main` clean and synced with `origin/main`.
