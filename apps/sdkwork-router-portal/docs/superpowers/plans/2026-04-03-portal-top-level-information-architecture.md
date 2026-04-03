# Portal Top-Level Information Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-architect the portal into a unified app with top-level navigation for Home, Console, Models, Docs, and Downloads, while moving the current business workspace under `/console`.

**Architecture:** Keep a single Vite/Tauri app with one router and one shared auth/theme/i18n stack, but split top-level modules into independent workspace packages. Public-facing modules render through a shared site shell, while the current operational workspace becomes a dedicated console shell under `/console/*`.

**Tech Stack:** React 19, React Router 7, TypeScript 5, Vite 7, Tailwind CSS 4, Tauri 2, `@sdkwork/ui-pc-react`

---

### Task 1: Define Top-Level Route Taxonomy

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-types\src\index.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\router\routePaths.ts`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-route-architecture.test.mjs`

- [ ] **Step 1: Write the failing route-architecture test**
- [ ] **Step 2: Run `node tests/portal-route-architecture.test.mjs` and verify it fails**
- [ ] **Step 3: Add top-level route keys for `home`, `console`, `models`, `docs`, `downloads`, and `auth` paths**
- [ ] **Step 4: Move current operational pages under `/console/*` route paths**
- [ ] **Step 5: Re-run `node tests/portal-route-architecture.test.mjs` and verify it passes**

### Task 2: Create Independent Top-Level Packages

**Files:**
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-home\package.json`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-home\src\index.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-models\package.json`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-models\src\index.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-docs\package.json`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-docs\src\index.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-downloads\package.json`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-downloads\src\index.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-console\package.json`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-console\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tsconfig.json`

- [ ] **Step 1: Add failing route-import tests for the new packages**
- [ ] **Step 2: Run the focused route tests and verify they fail**
- [ ] **Step 3: Scaffold the five new packages with typed entry exports**
- [ ] **Step 4: Add TypeScript path aliases for the new package entry points**
- [ ] **Step 5: Re-run the route tests and verify they pass**

### Task 3: Split Public Shell And Console Shell

**Files:**
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\layouts\PortalSiteLayout.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\layouts\MainLayout.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalDesktopShell.tsx`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-shell-parity.test.mjs`

- [ ] **Step 1: Update shell tests to require a public site shell and console shell separation**
- [ ] **Step 2: Run the shell tests and verify they fail**
- [ ] **Step 3: Keep the existing desktop shell for `/console/*` only**
- [ ] **Step 4: Add a public-facing site shell for Home, Models, Docs, and Downloads**
- [ ] **Step 5: Re-run the shell tests and verify they pass**

### Task 4: Add Shared Top Navigation

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalDesktopShell.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalTopNavigation.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-theme-config.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-shell-parity.test.mjs`

- [ ] **Step 1: Write failing tests for `首页 / 控制台 / 模型 / 文档 / 下载` top navigation**
- [ ] **Step 2: Run focused shell tests and verify they fail**
- [ ] **Step 3: Add a shared top navigation component and mount it into the header center region**
- [ ] **Step 4: Preserve `logo + application name` on the left while the top nav sits in the center**
- [ ] **Step 5: Re-run focused shell tests and verify they pass**

### Task 5: Route Public Modules Through Lazy Packages

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\router\AppRoutes.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\router\routeManifest.ts`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-route-architecture.test.mjs`

- [ ] **Step 1: Extend route tests to require lazy imports for home/models/docs/downloads/console**
- [ ] **Step 2: Run the route tests and verify they fail**
- [ ] **Step 3: Lazy-load the new top-level packages and wire the route tree**
- [ ] **Step 4: Ensure `/portal/` lands on Home, not Console**
- [ ] **Step 5: Re-run route tests and verify they pass**

### Task 6: Build The Home Landing Package

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-home\src\index.tsx`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-public-modules.test.mjs`

- [ ] **Step 1: Write failing tests for a real landing page with primary CTAs**
- [ ] **Step 2: Run `node tests/portal-public-modules.test.mjs` and verify it fails**
- [ ] **Step 3: Build the landing page with hero, capability sections, and calls to Console/Models/Docs/Downloads**
- [ ] **Step 4: Re-run `node tests/portal-public-modules.test.mjs` and verify the home checks pass**

### Task 7: Build The Models Center Package

**Files:**
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-models\src\catalog.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-models\src\index.tsx`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-models-center.test.mjs`

- [ ] **Step 1: Write failing tests for a searchable and filterable multimodal model center**
- [ ] **Step 2: Run `node tests/portal-models-center.test.mjs` and verify it fails**
- [ ] **Step 3: Add a typed model catalog with providers, modalities, capabilities, context, and pricing metadata**
- [ ] **Step 4: Build a first-pass models center with search, modality filters, provider filters, and a rich model table**
- [ ] **Step 5: Re-run `node tests/portal-models-center.test.mjs` and verify it passes**

### Task 8: Build The Docs And Downloads Packages

**Files:**
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-docs\src\registry.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-docs\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-downloads\src\index.tsx`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-public-modules.test.mjs`

- [ ] **Step 1: Extend public-module tests to require real Docs and Downloads modules**
- [ ] **Step 2: Run the public-module tests and verify they fail**
- [ ] **Step 3: Build the docs center with grouped navigation and starter content**
- [ ] **Step 4: Build the downloads center with desktop download targets, environment requirements, and installation guidance**
- [ ] **Step 5: Re-run the public-module tests and verify they pass**

### Task 9: Create The Console Aggregator Package

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-console\src\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\router\AppRoutes.tsx`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-route-architecture.test.mjs`

- [ ] **Step 1: Write failing tests that require the current business routes to live under `/console/*`**
- [ ] **Step 2: Run the route tests and verify they fail**
- [ ] **Step 3: Create a console aggregator export that renders the existing console route pages**
- [ ] **Step 4: Route authenticated business pages only under the console boundary**
- [ ] **Step 5: Re-run the route tests and verify they pass**

### Task 10: Run Full Verification And Polish

**Files:**
- Modify: targeted source files based on failed checks

- [ ] **Step 1: Run `node tests/portal-route-architecture.test.mjs`**
- [ ] **Step 2: Run `node tests/portal-public-modules.test.mjs`**
- [ ] **Step 3: Run `node tests/portal-models-center.test.mjs`**
- [ ] **Step 4: Run `node tests/portal-shell-parity.test.mjs`**
- [ ] **Step 5: Run `node tests/portal-theme-config.test.mjs`**
- [ ] **Step 6: Run `pnpm.cmd typecheck`**
- [ ] **Step 7: Run `pnpm.cmd build`**
- [ ] **Step 8: Fix any route, visual, or typing regressions and repeat until all checks pass**
