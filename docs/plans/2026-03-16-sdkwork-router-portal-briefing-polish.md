# SDKWork Router Portal Briefing Polish Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a shared briefing layer to `sdkwork-router-portal` so the standalone portal can surface a daily brief in the shell and focus/risk guidance in the dashboard without introducing parallel product logic.

**Architecture:** Extend the existing dashboard view model in `sdkwork-router-portal-dashboard` with briefing-oriented structures, then consume those structures from both the dashboard page and the core shell. This keeps product guidance derived from one live dashboard snapshot and aligned with `ARCHITECT.md` package boundaries.

**Tech Stack:** React, TypeScript, pnpm workspace, Vite, Node test runner

---

## Chunk 1: Red-Green Product Surface

### Task 1: Add failing product-semantic coverage

**Files:**
- Create: `apps/sdkwork-router-portal/tests/portal-briefing-polish.test.mjs`
- Test: `apps/sdkwork-router-portal/tests/portal-briefing-polish.test.mjs`

- [ ] **Step 1: Write the failing test**

```js
test('portal shell exposes a daily brief', () => {
  assert.match(core, /Daily brief/);
  assert.match(core, /Top focus/);
  assert.match(core, /Risk watch/);
});

test('dashboard exposes focus board and risk watchlist', () => {
  assert.match(dashboardPage, /Focus board/);
  assert.match(dashboardPage, /Risk watchlist/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test apps/sdkwork-router-portal/tests/portal-briefing-polish.test.mjs`
Expected: FAIL because the new product surfaces are not rendered yet.

### Task 2: Implement shared briefing view model and UI

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/types/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-dashboard/src/pages/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/index.tsx`

- [ ] **Step 3: Extend dashboard view model types**

Add focused interfaces for a briefing summary, focus board items, and risk watch items to keep the new product language explicit and typed.

- [ ] **Step 4: Build minimal shared briefing logic**

Derive the new structures from the existing live dashboard snapshot and existing launch/growth/recovery heuristics, without inventing new backend data.

- [ ] **Step 5: Render the dashboard surfaces**

Add `Focus board` and `Risk watchlist` surfaces to the dashboard page using the shared view model.

- [ ] **Step 6: Render the shell surface**

Add a `Daily brief` card to the shell that reuses the shared dashboard briefing signals instead of duplicating route-level product logic.

- [ ] **Step 7: Run the new test to verify it passes**

Run: `node --test apps/sdkwork-router-portal/tests/portal-briefing-polish.test.mjs`
Expected: PASS

## Chunk 2: Product Docs And Verification

### Task 3: Update the product planning docs

**Files:**
- Modify: `docs/plans/2026-03-16-sdkwork-router-portal-entry-polish.md`
- Modify: `docs/plans/2026-03-16-sdkwork-router-control-plane-product-roadmap.md`
- Modify: `docs/plans/2026-03-16-sdkwork-router-super-admin-product-spec.md`

- [ ] **Step 8: Document the new briefing and risk posture**

Record that the portal now needs a daily brief, a focus board, and a risk watchlist, and that admin should observe adoption/risk patterns for those same surfaces.

### Task 4: Run full verification

**Files:**
- Test: `apps/sdkwork-router-portal/tests/*.test.mjs`

- [ ] **Step 9: Run targeted portal tests**

Run:
- `node --test apps/sdkwork-router-portal/tests/portal-briefing-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-operations-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-journey-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-guidance-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-evidence-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-mode-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-navigation-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-playbook-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`

- [ ] **Step 10: Run workspace verification**

Run:
- `pnpm --dir apps/sdkwork-router-portal typecheck`
- `pnpm --dir apps/sdkwork-router-portal build`
- `pnpm --dir docs build`
- `git diff --check -- apps/sdkwork-router-portal docs/plans`
