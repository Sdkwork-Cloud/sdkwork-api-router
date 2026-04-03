# Portal Header Menu Width Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the top header menu horizontal scrollbar and give the center navigation the dominant width between the brand block and trailing actions while upgrading the visual treatment to a more business-oriented presentation.

**Architecture:** Keep the shared `DesktopShellFrame` integration intact and fix the issue at the portal composition layer. Widen the title-bar center slot allocation in both the public site shell and console shell, then restyle `PortalTopNavigation` so it fills the center slot instead of self-scrolling.

**Tech Stack:** React 19, Vite, TypeScript, Tailwind CSS, Node test runner

---

### Task 1: Lock the header layout requirement with regression coverage

**Files:**
- Modify: `tests/portal-shell-parity.test.mjs`

- [ ] **Step 1: Write a failing test**

Add assertions that both portal shells provide explicit `centerMaxWidth` and `centerShell` slot props, and that the top navigation no longer uses `overflow-x-auto` / `min-w-max`.

- [ ] **Step 2: Run test to verify it fails**

Run: `node tests/portal-shell-parity.test.mjs`
Expected: FAIL because the current shell layouts do not widen the center slot and the navigation still self-scrolls.

### Task 2: Widen the header center slot and restyle the menu

**Files:**
- Modify: `packages/sdkwork-router-portal-core/src/application/layouts/PortalSiteLayout.tsx`
- Modify: `packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx`
- Modify: `packages/sdkwork-router-portal-core/src/components/PortalTopNavigation.tsx`

- [ ] **Step 1: Add shared center-slot sizing in both shell compositions**

Set `centerMaxWidth` and `slotProps.centerShell` so the menu gets most of the header width after accounting for the brand and trailing actions.

- [ ] **Step 2: Replace self-scrolling menu behavior**

Remove `overflow-x-auto` and `min-w-max`. Make the navigation consume full center width with evenly distributed items.

- [ ] **Step 3: Apply the visual polish**

Use subtler borders, stronger spacing discipline, and a denser typographic treatment so the menu reads as a product navigation strip rather than a casual chip list.

### Task 3: Verify the change

**Files:**
- Test: `tests/portal-shell-parity.test.mjs`

- [ ] **Step 1: Run targeted regression coverage**

Run: `node tests/portal-shell-parity.test.mjs`
Expected: PASS

- [ ] **Step 2: Run build-safety checks**

Run: `pnpm.cmd typecheck`
Expected: PASS

Run: `pnpm.cmd build`
Expected: PASS
