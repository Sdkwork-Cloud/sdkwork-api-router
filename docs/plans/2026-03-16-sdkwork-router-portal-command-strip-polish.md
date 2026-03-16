# SDKWork Router Portal Command Strip Polish Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a persistent command strip to the standalone portal shell so every authenticated route opens with a visible primary mission, immediate next move, lead risk, and operating mode.

**Architecture:** Reuse the existing shared dashboard view model as the source of truth and compose a shell-only command strip inside `sdkwork-router-portal-core`. This keeps business guidance centralized in the dashboard semantics while letting the shell present a stronger product-grade entry layer across all routes.

**Tech Stack:** React, TypeScript, pnpm workspace, Vite, Node test runner

---

## Chunk 1: Red-Green Shell Mission Layer

### Task 1: Add failing product-semantic coverage

**Files:**
- Create: `apps/sdkwork-router-portal/tests/portal-command-strip-polish.test.mjs`
- Test: `apps/sdkwork-router-portal/tests/portal-command-strip-polish.test.mjs`

- [ ] **Step 1: Write the failing test**

```js
test('portal shell exposes a mission strip across authenticated routes', () => {
  assert.match(core, /Mission strip/);
  assert.match(core, /Primary mission/);
  assert.match(core, /Immediate next move/);
  assert.match(core, /Lead risk/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test apps/sdkwork-router-portal/tests/portal-command-strip-polish.test.mjs`
Expected: FAIL because the command strip is not rendered yet.

### Task 2: Implement the command strip in the shell

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/src/theme.css`

- [ ] **Step 3: Compose a shell-level command strip model**

Add a focused helper in `sdkwork-router-portal-core` that derives command-strip cards from the existing dashboard view model without duplicating snapshot heuristics.

- [ ] **Step 4: Render the new shell surface**

Upgrade the command bar to include the new mission strip cards and preserve responsive behavior on desktop and mobile.

- [ ] **Step 5: Add matching shell styles**

Introduce only the CSS needed for the new strip and card layout, keeping it visually aligned with the existing portal command bar.

- [ ] **Step 6: Run the new test to verify it passes**

Run: `node --test apps/sdkwork-router-portal/tests/portal-command-strip-polish.test.mjs`
Expected: PASS

## Chunk 2: Product Docs And Verification

### Task 3: Update product planning docs

**Files:**
- Modify: `docs/plans/2026-03-16-sdkwork-router-portal-entry-polish.md`
- Modify: `docs/plans/2026-03-16-sdkwork-router-control-plane-product-roadmap.md`
- Modify: `docs/plans/2026-03-16-sdkwork-router-super-admin-product-spec.md`

- [ ] **Step 7: Capture the shell-level command strip standard**

Document that the portal shell should expose a mission strip and that super-admin should eventually observe where command-strip priorities and risks concentrate across the portal population.

### Task 4: Run full verification

**Files:**
- Test: `apps/sdkwork-router-portal/tests/*.test.mjs`

- [ ] **Step 8: Run targeted portal tests**

Run:
- `node --test apps/sdkwork-router-portal/tests/portal-command-strip-polish.test.mjs`
- `node --test apps/sdkwork-router-portal/tests/*.test.mjs`

- [ ] **Step 9: Run workspace verification**

Run:
- `pnpm --dir apps/sdkwork-router-portal typecheck`
- `pnpm --dir apps/sdkwork-router-portal build`
- `pnpm --dir docs build`
- `git diff --check -- apps/sdkwork-router-portal docs/plans`
