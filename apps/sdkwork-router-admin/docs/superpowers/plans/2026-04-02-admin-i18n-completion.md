# Admin I18n Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete internationalization across the admin app so every page and user-facing component switches automatically from the central locale configuration.

**Architecture:** Keep `sdkwork-router-admin-core` as the single i18n owner. Add a real locale dictionary and reuse the existing provider so pages call `t()` and shared utilities use global admin formatting helpers instead of package-local `Intl` instances.

**Tech Stack:** React, TypeScript, `@sdkwork/ui-pc-react`, Node test runner, existing admin core i18n provider.

---

### Task 1: Lock The i18n Contract In Tests

**Files:**
- Modify: `tests/admin-i18n-coverage.test.mjs`

- [ ] Add a failing test that requires a real translation dictionary in `packages/sdkwork-router-admin-core/src/i18n.tsx`.
- [ ] Add a failing test that requires the major admin business pages to use `useAdminI18n` or `translateAdminText`.
- [ ] Run the focused i18n test and confirm it fails for the expected missing coverage.

### Task 2: Upgrade Core i18n

**Files:**
- Modify: `packages/sdkwork-router-admin-core/src/i18n.tsx`
- Modify: `packages/sdkwork-router-admin-core/src/index.tsx`

- [ ] Add locale dictionaries for `en-US` and `zh-CN`.
- [ ] Make `t()` and `translateAdminText()` resolve translated copy by locale with interpolation support.
- [ ] Keep number, currency, and date formatting centralized in admin core.
- [ ] Run the focused i18n test and confirm the core contract passes while page coverage still fails.

### Task 3: Internationalize Shared Shell And Overview

**Files:**
- Modify: `packages/sdkwork-router-admin-overview/src/index.tsx`
- Modify: shell files that still expose user-visible text without full translation wiring

- [ ] Replace hard-coded user-facing strings with `t(...)`.
- [ ] Replace package-local date/currency formatting with admin core formatting helpers where needed.
- [ ] Run focused tests for i18n coverage and architecture assertions.

### Task 4: Internationalize CRUD Admin Modules

**Files:**
- Modify: `packages/sdkwork-router-admin-users/src/**/*.tsx`
- Modify: `packages/sdkwork-router-admin-tenants/src/**/*.tsx`
- Modify: `packages/sdkwork-router-admin-coupons/src/**/*.tsx`
- Modify: `packages/sdkwork-router-admin-apirouter/src/pages/**/*.tsx`
- Modify: `packages/sdkwork-router-admin-catalog/src/**/*.tsx`

- [ ] Wire page-level components to `useAdminI18n`.
- [ ] Replace shared dialog/button/status copy with `t(...)` or `translateAdminText(...)`.
- [ ] Replace package-local number/date/currency formatting with admin core formatters.
- [ ] Run focused i18n and CRUD/page-pattern tests after each package batch.

### Task 5: Internationalize Dense Data Modules

**Files:**
- Modify: `packages/sdkwork-router-admin-traffic/src/index.tsx`
- Modify: `packages/sdkwork-router-admin-operations/src/index.tsx`

- [ ] Route all visible copy through i18n.
- [ ] Replace local `Intl` usage with admin core formatting helpers.
- [ ] Run focused i18n/product-polish tests.

### Task 6: Full Verification

**Files:**
- Verify only

- [ ] Run `node -e "Promise.all([import('./tests/admin-ui-framework-adoption.test.mjs'), import('./tests/admin-page-patterns.test.mjs'), import('./tests/admin-product-experience.test.mjs'), import('./tests/admin-shell-parity.test.mjs'), import('./tests/admin-crud-ux.test.mjs'), import('./tests/admin-table-polish.test.mjs'), import('./tests/admin-architecture.test.mjs'), import('./tests/admin-claw-foundation-parity.test.mjs'), import('./tests/admin-i18n-coverage.test.mjs'), import('./tests/admin-overview-runtime.test.mjs'), import('./tests/admin-desktop-api-base.test.mjs')]).catch((error) => { console.error(error); process.exit(1); })"`
- [ ] Run `pnpm.cmd typecheck`
- [ ] Run `pnpm.cmd build`
