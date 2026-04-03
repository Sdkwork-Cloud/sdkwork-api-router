# Router Portal UI Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the portal-local UI system with `@sdkwork/ui-pc-react`, remove stale UI modules, and rebuild every route page on top of the shared framework.

**Architecture:** The portal becomes a direct consumer of the shared SDKWORK PC React UI framework. Portal-owned UI primitives, shell composition, and duplicated theme ownership are removed instead of preserved through compatibility layers. Route packages keep business data flow but render through framework primitives and patterns only.

**Tech Stack:** React 19, Vite 7, Tailwind CSS 4, Tauri 2, TypeScript 5, pnpm, `@sdkwork/ui-pc-react`

---

### Task 1: Wire The Shared UI Package Into The Portal Root

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\pnpm-workspace.yaml`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\src\App.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\src\main.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-admin-ui-foundation.test.mjs`

- [ ] **Step 1: Write the failing foundation test for shared UI package adoption**

Add assertions that require:

```js
assert.match(packageJson, /@sdkwork\/ui-pc-react/);
assert.match(mainFile, /@sdkwork\/ui-pc-react\/styles\.css/);
```

- [ ] **Step 2: Run the focused foundation test and confirm it fails**

Run:

```bash
node --test tests/portal-admin-ui-foundation.test.mjs
```

Expected: FAIL because the root app does not yet consume `@sdkwork/ui-pc-react`.

- [ ] **Step 3: Expand the workspace and add the shared package dependency**

Apply these changes:

```yaml
# pnpm-workspace.yaml
packages:
  - "packages/*"
  - "../../../sdkwork-ui/sdkwork-ui-pc-react"
```

```json
// package.json dependencies
"@sdkwork/ui-pc-react": "workspace:*"
```

- [ ] **Step 4: Mount framework CSS at the application entry**

Import the stylesheet before portal-local overrides:

```tsx
import '@sdkwork/ui-pc-react/styles.css';
import './theme.css';
```

- [ ] **Step 5: Re-run the focused foundation test**

Run:

```bash
node --test tests/portal-admin-ui-foundation.test.mjs
```

Expected: PASS for framework package and stylesheet adoption checks.

- [ ] **Step 6: Commit**

```bash
git add pnpm-workspace.yaml package.json src/App.tsx src/main.tsx tests/portal-admin-ui-foundation.test.mjs
git commit -m "refactor: wire router portal to shared sdkwork ui package"
```

### Task 2: Replace Portal Theme Ownership With SdkworkThemeProvider

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\providers\AppProviders.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\providers\ThemeManager.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\src\theme.css`
- Test: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-admin-ui-foundation.test.mjs`

- [ ] **Step 1: Extend the foundation test to require theme-provider ownership**

Add checks like:

```js
assert.match(appProviders, /SdkworkThemeProvider/);
assert.match(themeManager, /createSdkworkTheme|CLAW_LIGHT_THEME|CLAW_DARK_THEME/);
assert.doesNotMatch(themeCss, /portalx-button-primary/);
```

- [ ] **Step 2: Run the focused theme test and confirm it fails**

Run:

```bash
node --test tests/portal-admin-ui-foundation.test.mjs
```

Expected: FAIL because the portal still owns the primary theme contract.

- [ ] **Step 3: Mount `SdkworkThemeProvider` in `AppProviders.tsx`**

Wrap the router tree:

```tsx
<SdkworkThemeProvider defaultTheme="light" overrides={themeOverrides}>
  <PortalI18nProvider>
    <BrowserRouter basename="/portal">...</BrowserRouter>
  </PortalI18nProvider>
</SdkworkThemeProvider>
```

- [ ] **Step 4: Rewrite `ThemeManager.tsx` to produce framework theme overrides instead of raw portal token ownership**

Expose one hook or resolver that maps:

```ts
themeMode -> light | dark | system
themeColor -> framework brand override
```

and leaves only document-level `dark` class sync where still required by the framework.

- [ ] **Step 5: Shrink `src/theme.css` to override-only responsibility**

Delete duplicated primitive styling families such as:

```css
.portalx-button-*
.portalx-field input
.portalx-table*
```

Keep only portal-specific compatibility or app-host overrides that are still needed after framework CSS is mounted.

- [ ] **Step 6: Re-run the foundation test**

Run:

```bash
node --test tests/portal-admin-ui-foundation.test.mjs
```

Expected: PASS with framework theme ownership enforced.

- [ ] **Step 7: Commit**

```bash
git add packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx src/theme.css tests/portal-admin-ui-foundation.test.mjs
git commit -m "refactor: move router portal theme ownership to sdkwork theme provider"
```

### Task 3: Collapse `portal-commons` Into A Thin Non-Visual Utility Layer

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\index.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\framework.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-commons\src\clipboard.ts`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-admin-ui-foundation.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-table-polish.test.mjs`

- [ ] **Step 1: Replace old `commons` primitive tests with thin-adapter tests**

Require:

```js
assert.match(commons, /from '@sdkwork\\/ui-pc-react'/);
assert.doesNotMatch(commons, /CheckboxPrimitive\\.Root/);
assert.doesNotMatch(commons, /export const Textarea = forwardRef/);
```

- [ ] **Step 2: Run the focused commons tests and confirm failure**

Run:

```bash
node --test tests/portal-admin-ui-foundation.test.mjs tests/portal-table-polish.test.mjs
```

Expected: FAIL because `commons` still owns local primitive implementations.

- [ ] **Step 3: Move visual exports into `framework.tsx` and re-export framework-owned components**

Use direct imports such as:

```tsx
export {
  Button,
  DataTable,
  EmptyState,
  Input,
  Select,
  Textarea,
  Toolbar,
  ToolbarGroup,
  ToolbarSpacer,
  Modal,
  Dialog,
  DialogContent,
} from '@sdkwork/ui-pc-react';
```

- [ ] **Step 4: Keep only non-visual ownership in `commons`**

Leave portal-owned utilities only:

```ts
export { PortalI18nProvider, usePortalI18n, translatePortalText } from './i18n-core';
export { formatCurrency, formatDateTime, formatUnits } from './format-core';
export { copyText } from './clipboard';
```

- [ ] **Step 5: Delete dead primitive code from `src/index.tsx`**

Remove local ownership of:

```tsx
Button
Card
DataTable
Dialog
Input
Select
Textarea
Checkbox
Toolbar*
Surface
MetricCard
```

- [ ] **Step 6: Re-run the focused commons tests**

Run:

```bash
node --test tests/portal-admin-ui-foundation.test.mjs tests/portal-table-polish.test.mjs
```

Expected: PASS with `commons` acting as a thin adapter plus utility layer.

- [ ] **Step 7: Commit**

```bash
git add packages/sdkwork-router-portal-commons/package.json packages/sdkwork-router-portal-commons/src/index.tsx packages/sdkwork-router-portal-commons/src/framework.tsx packages/sdkwork-router-portal-commons/src/clipboard.ts tests/portal-admin-ui-foundation.test.mjs tests/portal-table-polish.test.mjs
git commit -m "refactor: remove portal local primitives from commons"
```

### Task 4: Replace The Portal Shell With Framework Desktop Shell Patterns

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\package.json`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\layouts\MainLayout.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\application\router\AppRoutes.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalDesktopShell.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalNavigationRail.tsx`
- Create: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\PortalSettingsCenter.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\AppHeader.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\Sidebar.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\SidebarProfileDock.tsx`
- Delete: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-core\src\components\ConfigCenter.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-shell-parity.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-navigation-polish.test.mjs`

- [ ] **Step 1: Rewrite shell tests to enforce framework desktop-shell adoption**

Require:

```js
assert.match(layout, /DesktopShellFrame/);
assert.match(layout, /PortalNavigationRail/);
assert.match(layout, /PortalSettingsCenter/);
assert.doesNotMatch(layout, /<Sidebar/);
assert.doesNotMatch(layout, /<AppHeader/);
```

- [ ] **Step 2: Run the focused shell tests and confirm failure**

Run:

```bash
node --test tests/portal-shell-parity.test.mjs tests/portal-navigation-polish.test.mjs
```

Expected: FAIL because the old shell is still present.

- [ ] **Step 3: Build a thin portal-owned shell adapter over framework patterns**

Use:

```tsx
<DesktopShellFrame
  title="SDKWork Router"
  navigation={<PortalNavigationRail ... />}
  content={children}
  windowControls={<WindowControls />}
/>
```

- [ ] **Step 4: Move config and navigation composition into explicit portal adapters**

`PortalNavigationRail.tsx` owns route rendering and workspace actions.

`PortalSettingsCenter.tsx` owns portal-specific settings content but renders through framework settings patterns.

- [ ] **Step 5: Delete obsolete shell modules**

Delete:

```text
AppHeader.tsx
Sidebar.tsx
SidebarProfileDock.tsx
ConfigCenter.tsx
```

- [ ] **Step 6: Re-run focused shell tests**

Run:

```bash
node --test tests/portal-shell-parity.test.mjs tests/portal-navigation-polish.test.mjs
```

Expected: PASS with framework shell composition as the only shell authority.

- [ ] **Step 7: Commit**

```bash
git add packages/sdkwork-router-portal-core/package.json packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx tests/portal-shell-parity.test.mjs tests/portal-navigation-polish.test.mjs
git rm packages/sdkwork-router-portal-core/src/components/AppHeader.tsx packages/sdkwork-router-portal-core/src/components/Sidebar.tsx packages/sdkwork-router-portal-core/src/components/SidebarProfileDock.tsx packages/sdkwork-router-portal-core/src/components/ConfigCenter.tsx
git commit -m "refactor: replace portal shell with sdkwork desktop shell patterns"
```

### Task 5: Rebuild Dashboard On Framework Workspace Patterns

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-dashboard\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-dashboard\src\components\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-dashboard-analytics.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-navigation-polish.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-table-polish.test.mjs`

- [ ] **Step 1: Update dashboard tests to require framework page patterns**

Require:

```js
assert.match(dashboardPage, /SectionHeader|WorkspacePanel|ManagementWorkbench/);
assert.doesNotMatch(dashboardPage, /surfaceClass/);
```

- [ ] **Step 2: Run dashboard-focused tests and confirm failure**

Run:

```bash
node --test tests/portal-dashboard-analytics.test.mjs tests/portal-table-polish.test.mjs tests/portal-navigation-polish.test.mjs
```

Expected: FAIL because dashboard still uses portal-local surface grammar.

- [ ] **Step 3: Replace portal-local surface wrappers with framework surfaces**

Refactor summary sections and tables to use:

```tsx
<SectionHeader ... />
<WorkspacePanel ... />
<DataTable ... />
```

- [ ] **Step 4: Remove local dashboard-only layout glue that duplicates framework patterns**

Delete local constants like:

```ts
const surfaceClass = ...
const chartFrameClass = ...
```

when equivalent framework surface ownership replaces them.

- [ ] **Step 5: Re-run dashboard-focused tests**

Run:

```bash
node --test tests/portal-dashboard-analytics.test.mjs tests/portal-table-polish.test.mjs tests/portal-navigation-polish.test.mjs
```

Expected: PASS with dashboard on framework workspace patterns.

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-portal-dashboard/src/pages/index.tsx packages/sdkwork-router-portal-dashboard/src/components/index.tsx tests/portal-dashboard-analytics.test.mjs tests/portal-navigation-polish.test.mjs tests/portal-table-polish.test.mjs
git commit -m "refactor: rebuild dashboard on shared workspace patterns"
```

### Task 6: Rebuild Gateway On `ManagementWorkbench`

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-gateway\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-gateway\src\components\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-gateway-command-center.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-table-polish.test.mjs`

- [ ] **Step 1: Change gateway tests to enforce `ManagementWorkbench` adoption**

Add checks such as:

```js
assert.match(gatewayPage, /ManagementWorkbench/);
assert.doesNotMatch(gatewayPage, /<Surface/);
```

- [ ] **Step 2: Run gateway-focused tests and confirm failure**

Run:

```bash
node --test tests/portal-gateway-command-center.test.mjs tests/portal-table-polish.test.mjs
```

Expected: FAIL because gateway still uses local surface composition.

- [ ] **Step 3: Rebuild the page frame around `ManagementWorkbench`**

Use:

```tsx
<ManagementWorkbench
  eyebrow={...}
  title={...}
  description={...}
  filters={...}
  main={{ title: ..., children: ... }}
  detail={{ title: ..., children: ... }}
/>
```

- [ ] **Step 4: Delete local gateway surface ownership**

Remove framework-duplicating helpers like:

```tsx
Surface
portal-local section shells
```

- [ ] **Step 5: Re-run gateway-focused tests**

Run:

```bash
node --test tests/portal-gateway-command-center.test.mjs tests/portal-table-polish.test.mjs
```

Expected: PASS with gateway centered on `ManagementWorkbench`.

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-portal-gateway/src/pages/index.tsx packages/sdkwork-router-portal-gateway/src/components/index.tsx tests/portal-gateway-command-center.test.mjs tests/portal-table-polish.test.mjs
git commit -m "refactor: rebuild gateway on management workbench"
```

### Task 7: Rebuild Routing On `ManagementWorkbench` And Framework Forms

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-routing\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-routing\src\components\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-routing-polish.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-table-polish.test.mjs`

- [ ] **Step 1: Update routing tests to require framework workbench and dialog ownership**

Require:

```js
assert.match(routingPage, /ManagementWorkbench/);
assert.match(routingPage, /Dialog|Modal/);
assert.doesNotMatch(routingPage, /<Surface/);
```

- [ ] **Step 2: Run routing-focused tests and confirm failure**

Run:

```bash
node --test tests/portal-routing-polish.test.mjs tests/portal-table-polish.test.mjs
```

Expected: FAIL because routing still owns local surface and workbench composition.

- [ ] **Step 3: Re-author the routing page around framework workbench patterns**

Use a structure like:

```tsx
<ManagementWorkbench
  filters={<Toolbar ... />}
  main={{ title: t('Routing workbench'), children: <DataTable ... /> }}
  detail={{ title: t('Preview outcome'), children: <RoutingPreviewPanel /> }}
/>
```

- [ ] **Step 4: Keep only routing-specific business widgets local**

Local code should retain:

```text
provider ordering logic
preview request mapping
policy/evidence rendering
```

and should stop owning generic workbench layout primitives.

- [ ] **Step 5: Re-run routing-focused tests**

Run:

```bash
node --test tests/portal-routing-polish.test.mjs tests/portal-table-polish.test.mjs
```

Expected: PASS with routing rebuilt on shared workbench and form patterns.

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-portal-routing/src/pages/index.tsx packages/sdkwork-router-portal-routing/src/components/index.tsx tests/portal-routing-polish.test.mjs tests/portal-table-polish.test.mjs
git commit -m "refactor: rebuild routing on shared workbench patterns"
```

### Task 8: Rebuild API Keys On `CrudWorkbench`

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-api-keys\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-api-keys\src\components\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-api-keys\src\components\PortalApiKeyTable.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-api-key-preview-base.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-table-polish.test.mjs`

- [ ] **Step 1: Update API-key tests to require `CrudWorkbench`**

Require:

```js
assert.match(apiKeyPage, /CrudWorkbench/);
assert.doesNotMatch(apiKeyPage, /PortalApiKeyManagerToolbar/);
```

- [ ] **Step 2: Run the focused API-key tests and confirm failure**

Run:

```bash
node --test tests/portal-api-key-preview-base.test.mjs tests/portal-table-polish.test.mjs
```

Expected: FAIL because API keys still uses portal-local page composition.

- [ ] **Step 3: Move the page to framework CRUD patterns**

Use:

```tsx
<CrudWorkbench
  title={t('API Keys')}
  filters={...}
  table={...}
  createDialog={...}
  detailDrawer={...}
/>
```

- [ ] **Step 4: Keep business-specific quick-setup and reveal logic only**

Retain:

```text
key issuance
plaintext reveal handling
quick-setup plan logic
instance application flow
```

and delete local generic CRUD shell wrappers.

- [ ] **Step 5: Re-run API-key-focused tests**

Run:

```bash
node --test tests/portal-api-key-preview-base.test.mjs tests/portal-table-polish.test.mjs
```

Expected: PASS with API keys rebuilt on `CrudWorkbench`.

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-portal-api-keys/src/pages/index.tsx packages/sdkwork-router-portal-api-keys/src/components/index.tsx packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx tests/portal-api-key-preview-base.test.mjs tests/portal-table-polish.test.mjs
git commit -m "refactor: rebuild api keys on shared crud workbench"
```

### Task 9: Rebuild The Remaining Pages And Delete Old UI Residue

**Files:**
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-auth\src\pages\AuthPage.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-usage\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-billing\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-credits\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-account\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\packages\sdkwork-router-portal-user\src\pages\index.tsx`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-auth-parity.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-usage-workbench.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-product-polish.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-theme-config.test.mjs`
- Modify: `D:\javasource\spring-ai-plus\spring-ai-plus-business\apps\sdkwork-api-router\apps\sdkwork-router-portal\tests\portal-form-ux-polish.test.mjs`

- [ ] **Step 1: Update each page test to require framework patterns and forbid old wrappers**

For example:

```js
assert.match(authPage, /@sdkwork\/ui-pc-react|WorkspacePanel|SectionHeader/);
assert.doesNotMatch(authPage, /portalx-auth-/);
```

- [ ] **Step 2: Run the page-specific test set and confirm failure**

Run:

```bash
node --test tests/portal-auth-parity.test.mjs tests/portal-usage-workbench.test.mjs tests/portal-product-polish.test.mjs tests/portal-theme-config.test.mjs tests/portal-form-ux-polish.test.mjs
```

Expected: FAIL because remaining pages still depend on old page-local UI grammar.

- [ ] **Step 3: Rebuild each remaining page against framework primitives and patterns**

Apply the following bias:

```text
auth -> framework form + panel patterns
usage -> management workbench
billing/credits/account -> workspace panels + shared tables
user -> shared forms + workspace panels
```

- [ ] **Step 4: Delete obsolete CSS and stale exports after all page migrations land**

Remove:

```text
dead portalx-* CSS blocks
obsolete portal-local surface helpers
stale commons UI exports
```

- [ ] **Step 5: Run full verification**

Run:

```bash
node --test tests/*.test.mjs
pnpm typecheck
pnpm build
```

Expected:

```text
all portal tests PASS
typecheck PASS
build PASS
```

- [ ] **Step 6: Commit**

```bash
git add packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx packages/sdkwork-router-portal-usage/src/pages/index.tsx packages/sdkwork-router-portal-billing/src/pages/index.tsx packages/sdkwork-router-portal-credits/src/pages/index.tsx packages/sdkwork-router-portal-account/src/pages/index.tsx packages/sdkwork-router-portal-user/src/pages/index.tsx src/theme.css packages/sdkwork-router-portal-commons/src/index.tsx tests/portal-auth-parity.test.mjs tests/portal-usage-workbench.test.mjs tests/portal-product-polish.test.mjs tests/portal-theme-config.test.mjs tests/portal-form-ux-polish.test.mjs
git commit -m "refactor: finish router portal ui unification on shared sdkwork framework"
```
