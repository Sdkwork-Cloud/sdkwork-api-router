# Portal Routing Profile Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add a portal self-service routing profile workbench that lets workspace users review reusable profiles and save the current routing posture as a reusable profile for API key group binding.

**Architecture:** keep `admin` as the unrestricted control plane and add a narrower `portal` contract for workspace-scoped routing profile creation. The portal backend will generate profile ids and scope from authenticated workspace claims, while the portal routing page will lazily load routing profiles, show them in a dedicated dialog, and create new profiles from the current in-page routing posture instead of exposing full tenant/project control-plane editing.

**Tech Stack:** Rust, Axum, TypeScript, React, Vite, node:test, cargo test, pnpm typecheck

---

### Task 1: Lock the new portal routing profile contract into failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-architecture.test.mjs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_api_keys.rs`

- [ ] **Step 1: Write the failing test**

Add assertions that:

- the portal routing page exposes a `Manage routing profiles` entrypoint and a dedicated routing profile dialog component
- the portal routing repository exposes routing profile load/create helpers
- the portal API client exposes `createPortalRoutingProfile`
- the portal API interface serves `POST /portal/routing/profiles`
- the create route binds profiles to the authenticated workspace and does not require tenant/project/profile ids from the browser

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
node apps/sdkwork-router-portal/tests/portal-architecture.test.mjs
cargo test -p sdkwork-api-interface-portal portal_routing_profiles -- --nocapture
```

Expected: FAIL because the portal UI wiring and backend create route do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add the backend create route, portal API client call, routing repository methods, and routing page/dialog wiring.

- [ ] **Step 4: Run test to verify it passes**

Run the same commands and confirm both are green.

### Task 2: Lock the portal routing profile workbench copy and interaction model into failing tests

**Files:**
- Modify: `apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts`

- [ ] **Step 1: Write the failing test**

Add assertions that:

- the routing page localizes the new profile-management button and dialog copy
- the profile dialog explains that it saves the current routing posture as a reusable profile
- the dialog exposes creation metadata and existing profile inventory copy through shared portal i18n

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
node apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs
```

Expected: FAIL because the new localized routing profile workbench copy is missing.

- [ ] **Step 3: Write minimal implementation**

Add the shared portal i18n keys and wire them through the routing page/dialog.

- [ ] **Step 4: Run test to verify it passes**

Run the same command and confirm it is green.

### Task 3: Verify the vertical slice end to end

**Files:**
- Modify: `docs/api-reference/portal-api.md`

- [ ] **Step 1: Run focused regression**

Run:

```bash
node apps/sdkwork-router-portal/tests/portal-architecture.test.mjs
node apps/sdkwork-router-portal/tests/portal-routing-polish.test.mjs
node apps/sdkwork-router-portal/tests/portal-api-key-i18n.test.mjs
node apps/sdkwork-router-portal/tests/portal-api-key-preview-base.test.mjs
node apps/sdkwork-router-portal/tests/portal-i18n-coverage.test.mjs
node apps/sdkwork-router-portal/tests/portal-zh-cn-direct-coverage.test.mjs
pnpm.cmd --dir apps/sdkwork-router-portal typecheck
cargo test -p sdkwork-api-interface-portal portal_routing_profiles -- --nocapture
```

Expected: all portal checks pass and the portal API reference documents the new routing profile create contract.
