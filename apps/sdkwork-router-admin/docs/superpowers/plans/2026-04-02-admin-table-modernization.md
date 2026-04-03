# Admin Table Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move coupon and gateway table-centric admin pages onto the same top-form, paginated-table, drawer-detail pattern already used by users and tenants.

**Architecture:** Keep each feature package responsible for its own filtered table state, dialogs, and drawer state. Remove legacy `ManagementWorkbench` wrappers from CRUD-style surfaces, preserve existing business logic and dialogs, and standardize registry sections around paginated `DataTable` plus explicit drawer modules.

**Tech Stack:** React, TypeScript, `@sdkwork/ui-pc-react`, node:test assertion baselines, Vite build, pnpm typecheck.

---

### Task 1: Lock the new UX baseline in tests

**Files:**
- Modify: `tests/admin-page-patterns.test.mjs`
- Modify: `tests/admin-crud-ux.test.mjs`
- Modify: `tests/admin-product-experience.test.mjs`

- [ ] Add failing assertions for `coupons` and gateway CRUD pages to require top-level `<form>`, `Drawer`, `Pagination`, and the absence of `ManagementWorkbench` / `FilterBar` shells on those surfaces.
- [ ] Run the targeted node:test commands and verify the new assertions fail for the current legacy implementations.

### Task 2: Refactor coupons to the users pattern

**Files:**
- Modify: `packages/sdkwork-router-admin-coupons/src/index.tsx`
- Modify: `packages/sdkwork-router-admin-coupons/src/page/CouponsRegistrySection.tsx`
- Modify: `packages/sdkwork-router-admin-coupons/src/page/CouponsDetailPanel.tsx`
- Modify: `packages/sdkwork-router-admin-coupons/src/page/shared.tsx`
- Create: `packages/sdkwork-router-admin-coupons/src/page/CouponsDetailDrawer.tsx`

- [ ] Replace the filter/workbench shell with a top search form, inline filters, and action buttons inside `Card` surfaces.
- [ ] Add explicit coupon drawer state and close/reset behavior that mirrors users and tenants.
- [ ] Convert the registry to a paginated `DataTable` with summary strip, selected row styling, and action buttons aligned to the shared admin pattern.
- [ ] Re-run the new coupon tests and verify they pass before moving on.

### Task 3: Refactor gateway access to the users pattern

**Files:**
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailPanel.tsx`
- Create: `packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailDrawer.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/shared.tsx`

- [ ] Replace the workbench shell with top-form controls and explicit drawer state while preserving create/edit/route/usage/delete dialogs.
- [ ] Upgrade the registry to paginated table behavior and move detail actions into the drawer footer.
- [ ] Re-run targeted gateway access tests and verify they pass.

### Task 4: Refactor gateway routes, model mappings, and usage pages to the same shell

**Files:**
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/GatewayRoutesPage.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/GatewayModelMappingsPage.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesRegistrySection.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsRegistrySection.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageRegistrySection.tsx`
- Create: `packages/sdkwork-router-admin-apirouter/src/pages/routes/GatewayRoutesDetailDrawer.tsx`
- Create: `packages/sdkwork-router-admin-apirouter/src/pages/mappings/GatewayModelMappingsDetailDrawer.tsx`
- Create: `packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageDetailDrawer.tsx`

- [ ] Standardize each page around top-form filters, action buttons, paginated table registries, and drawer-driven detail inspection.
- [ ] Preserve page-specific dense-data behavior such as CSV export, refresh, and overlay mutation flows while removing legacy shell wrappers.
- [ ] Re-run targeted tests for routes, mappings, and usage pages and verify they pass.

### Task 5: Full verification

**Files:**
- Verify only

- [ ] Run the broader admin UI test batch covering framework adoption, page patterns, product experience, shell parity, CRUD UX, table polish, and architecture.
- [ ] Run `pnpm.cmd typecheck`.
- [ ] Run `pnpm.cmd build`.
- [ ] Summarize changed files, behavior shifts, and any remaining follow-up candidates.
