# Portal Public Site CTA Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove dead public-site calls to action and make the top-level public modules behave like a coherent product journey.

**Architecture:** Keep the existing top-level Home / Models / Docs / Downloads split, but make the public modules route-aware. Docs should persist the active content group in the URL, downloads should hand users into real guides and product entrypoints instead of dead buttons, and top navigation labels should flow through shared portal i18n instead of component-local bilingual branching.

**Tech Stack:** React 19, React Router 7, TypeScript 5, Vite 7, Node test runner, `@sdkwork/ui-pc-react`

---

### Task 1: Lock Public-Site CTA Expectations With Failing Tests

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-public-modules.test.mjs`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-public-site-cta-polish.test.mjs`

- [ ] **Step 1: Extend the public-module expectations to require guided docs/download CTA copy instead of dead generic download buttons**
- [ ] **Step 2: Add a focused public-site CTA test that requires docs URL state, route-aware action handlers, and shared top-nav i18n labels**
- [ ] **Step 3: Run `node tests/portal-public-modules.test.mjs` and `node tests/portal-public-site-cta-polish.test.mjs` and verify they fail for the expected missing behavior**

### Task 2: Make The Top Navigation And Docs Module Route-Aware

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalTopNavigation.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\portalMessages.zh-CN.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-docs\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-docs\src\registry.ts`

- [ ] **Step 1: Replace component-local bilingual top-nav labels with shared translation keys**
- [ ] **Step 2: Move docs group selection into the URL via `useSearchParams` so refresh and deep links preserve the active group**
- [ ] **Step 3: Add real docs CTA actions that navigate into console, downloads, or related public modules based on the active content group**
- [ ] **Step 4: Re-run `node tests/portal-public-site-cta-polish.test.mjs` and verify the docs and top-nav expectations pass**

### Task 3: Turn Downloads Into Guided Product Entry Instead Of Dead Buttons

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-downloads\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\portalMessages.zh-CN.ts`

- [ ] **Step 1: Replace generic download buttons with explicit guided actions such as install guide, desktop mode, and server mode handoff**
- [ ] **Step 2: Surface executable product commands and route-aware next steps so the downloads page becomes a launch funnel instead of a dead asset index**
- [ ] **Step 3: Re-run `node tests/portal-public-modules.test.mjs` and `node tests/portal-public-site-cta-polish.test.mjs` and verify the downloads expectations pass**

### Task 4: Final Verification

**Files:**
- Modify: targeted source files above if verification reveals regressions

- [ ] **Step 1: Run `node tests/portal-public-modules.test.mjs`**
- [ ] **Step 2: Run `node tests/portal-public-site-cta-polish.test.mjs`**
- [ ] **Step 3: Run `node tests/portal-i18n-coverage.test.mjs`**
- [ ] **Step 4: Run `node tests/portal-zh-cn-direct-coverage.test.mjs`**
- [ ] **Step 5: Run `node tests/portal-shell-parity.test.mjs`**
- [ ] **Step 6: Run `pnpm.cmd typecheck`**
- [ ] **Step 7: Run `pnpm.cmd build`**
- [ ] **Step 8: Fix any CTA, routing, or i18n regressions and repeat until all checks pass**
