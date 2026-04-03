# Portal API Key Group Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the remaining portal self-service gap by adding a workspace-scoped API key group management workbench and the routing profile discovery contract it needs for correct group editing.

**Architecture:** extend the portal Axum boundary with a workspace-filtered routing profile listing route, then thread that contract through the portal TypeScript SDK, repository layer, and API key page state. Keep the UI slice focused by adding a dedicated API key group dialog that reuses portal design-system components, loads routing profiles on demand, and refreshes the existing key inventory after group mutations.

**Tech Stack:** Rust, Axum, sqlx store traits, TypeScript, React, Vite, existing portal framework components, node:test, cargo test, pnpm typecheck

---

### Task 1: Lock routing profile discovery into failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_api_keys.rs`

- [ ] **Step 1: Write the failing test**

Add assertions that:

- portal types expose `RoutingProfileRecord`
- portal API SDK exposes `listPortalRoutingProfiles`
- the portal API interface serves `GET /portal/routing/profiles`
- the route returns only profiles inside the authenticated workspace scope

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
node apps/sdkwork-router-portal/tests/portal-architecture.test.mjs
cargo test -p sdkwork-api-interface-portal portal_routing_profiles -- --nocapture
```

Expected: FAIL because the portal frontend contract and backend route do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add the new route, handler, client method, and shared portal type.

- [ ] **Step 4: Run test to verify it passes**

Run the same commands and confirm both are green.

### Task 2: Lock the portal group workbench into failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-api-key-i18n.test.mjs`

- [ ] **Step 1: Write the failing test**

Add assertions that:

- the API key page exposes a `Manage groups` entrypoint
- a dedicated `PortalApiKeyGroupsDialog` component exists and is exported
- the page wires create, update, toggle, and delete handlers through repository calls
- the group dialog localizes routing profile and lifecycle copy through shared portal i18n

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
node apps/sdkwork-router-portal/tests/portal-architecture.test.mjs
node apps/sdkwork-router-portal/tests/portal-api-key-i18n.test.mjs
```

Expected: FAIL because the workbench dialog and UI wiring do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add a focused API key group dialog plus page state for loading routing profiles and refreshing workspace inventory after mutations.

- [ ] **Step 4: Run test to verify it passes**

Run the same commands and confirm both are green.

### Task 3: Verify the full slice end to end

**Files:**
- Modify: `docs/api-reference/portal-api.md`

- [ ] **Step 1: Run focused regression**

Run:

```bash
node apps/sdkwork-router-portal/tests/portal-architecture.test.mjs
node apps/sdkwork-router-portal/tests/portal-api-key-i18n.test.mjs
node apps/sdkwork-router-portal/tests/portal-api-key-preview-base.test.mjs
node apps/sdkwork-router-admin/tests/admin-architecture.test.mjs
node apps/sdkwork-router-admin/tests/admin-crud-ux.test.mjs
pnpm.cmd --dir apps/sdkwork-router-portal typecheck
pnpm.cmd --dir apps/sdkwork-router-admin typecheck
cargo test -p sdkwork-api-interface-portal portal_api_key -- --nocapture
```

Expected: all relevant portal and admin checks pass, and portal docs reflect the new routing profile discovery route.
