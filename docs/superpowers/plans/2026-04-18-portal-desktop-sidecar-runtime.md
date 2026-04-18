# Portal Desktop Sidecar Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Converge `sdkwork-router-portal-desktop` onto a real sidecar-based product runtime with a fixed public port `3001`, config-file-first runtime settings, and release packaging that bundles `router-product-service` plus portal/admin site assets.

**Architecture:** The desktop app stops booting the router product in-process and instead supervises a bundled `router-product-service` sidecar. Desktop runtime settings live in an OS-standard app config directory, public bind selection (`127.0.0.1:3001` vs `0.0.0.0:3001`) is persisted in a desktop runtime config file, and Tauri release resources embed a release-like `router-product/` payload so the sidecar and web assets share the same contract as the server product.

**Tech Stack:** Rust, Tauri 2, Node.js release/build tooling, pnpm workspace apps, Zustand, Markdown docs

---

### Task 1: Extend The Runtime Config Contract To Cover Public Web Bind

**Files:**
- Modify: `crates/sdkwork-api-config/src/env_keys.rs`
- Modify: `crates/sdkwork-api-config/src/types.rs`
- Modify: `crates/sdkwork-api-config/src/loader.rs`
- Modify: `crates/sdkwork-api-config/src/standalone_config.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `services/router-product-service/src/main.rs`

- [ ] **Step 1: Write failing config tests for `SDKWORK_WEB_BIND` and config-file precedence**
- [ ] **Step 2: Run the focused Rust tests and confirm the new assertions fail**
- [ ] **Step 3: Add `public_web_bind` to the shared standalone config model and file overlays**
- [ ] **Step 4: Make `router-product-service` dry-run, runtime startup, and security validation use the effective config-file-first public bind**
- [ ] **Step 5: Re-run the focused Rust tests and confirm they pass**

### Task 2: Freeze The Desktop Sidecar Packaging Contract In Tests

**Files:**
- Modify: `scripts/release-flow-contract.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-shell-parity.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-product-polish.test.mjs`
- Modify: `apps/sdkwork-router-portal/tests/portal-desktop-api-base.test.mjs`

- [ ] **Step 1: Add failing tests that require portal desktop resources to bundle a release-like `router-product/` payload**
- [ ] **Step 2: Add failing tests that require a fixed desktop public port contract on `3001` and an explicit local/shared access mode**
- [ ] **Step 3: Add failing tests for settings-center runtime access controls and Tauri IPC commands for reading/updating desktop runtime settings**
- [ ] **Step 4: Run the targeted Node tests and confirm the new assertions fail**

### Task 3: Implement Desktop Sidecar Resource Staging And Release Build Inputs

**Files:**
- Modify: `scripts/build-router-desktop-assets.mjs`
- Modify: `apps/sdkwork-router-portal/src-tauri/tauri.conf.json`
- Modify: `scripts/release/run-desktop-release-build.mjs`
- Modify: `scripts/release/package-release-assets.mjs` (only if resource staging helpers must be exported or validated)

- [ ] **Step 1: Write the minimal build helper changes needed to stage `router-product-service` and site assets into a stable desktop resource root**
- [ ] **Step 2: Keep the Tauri bundle resource map release-like: `router-product/bin/` and `router-product/sites/{admin,portal}/dist/`**
- [ ] **Step 3: Re-run the packaging contract tests and ensure resource expectations pass**

### Task 4: Replace In-Process Desktop Runtime Boot With Sidecar Supervision

**Files:**
- Modify: `apps/sdkwork-router-portal/src-tauri/src/main.rs`
- Modify: `apps/sdkwork-router-portal/src-tauri/Cargo.toml`
- Create: `apps/sdkwork-router-portal/src-tauri/src/desktop_runtime.rs`
- Create: `apps/sdkwork-router-portal/src-tauri/src/desktop_runtime_config.rs`

- [ ] **Step 1: Add a failing Rust test slice for desktop runtime config resolution and access-mode defaults**
- [ ] **Step 2: Add a failing Rust test slice for release-resource fallback versus workspace fallback**
- [ ] **Step 3: Implement desktop runtime config discovery under OS-standard app config/data/log directories**
- [ ] **Step 4: Implement sidecar process launch, health stabilization, snapshot reporting, restart, and shutdown**
- [ ] **Step 5: Keep the portal shell commands stable while switching the backend from in-process runtime boot to bundled `router-product-service` supervision**
- [ ] **Step 6: Re-run the focused Rust and Node tests to confirm the sidecar contract is green**

### Task 5: Wire The Portal Settings Center To Live Desktop Runtime Access Controls

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-types/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/types/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/services/index.ts`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-gateway/src/components/index.tsx` (only if runtime controls must reflect access-mode state)

- [ ] **Step 1: Add desktop runtime settings types and IPC-backed API helpers**
- [ ] **Step 2: Surface a settings-center control for local-only vs shared access with immediate restart/apply messaging**
- [ ] **Step 3: Update gateway command-center runtime posture to describe the fixed public `3001` contract and current access mode**
- [ ] **Step 4: Re-run the targeted frontend tests and confirm the settings/runtime control contract passes**

### Task 6: Update Desktop Packaging, Release, And Operations Documentation

**Files:**
- Modify: `apps/sdkwork-router-portal/README.md`
- Modify: `docs/getting-started/release-builds.md`
- Modify: `docs/reference/build-and-tooling.md`
- Modify: `docs/getting-started/production-deployment.md`
- Modify: `docs/getting-started/script-lifecycle.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/operations/service-management.md`
- Modify: `docs/zh/getting-started/release-builds.md`
- Modify: `docs/zh/reference/build-and-tooling.md`
- Modify: `docs/zh/getting-started/production-deployment.md`
- Modify: `docs/zh/getting-started/script-lifecycle.md`
- Modify: `docs/zh/operations/configuration.md`
- Modify: `docs/zh/operations/service-management.md`

- [ ] **Step 1: Rewrite desktop docs so they describe the sidecar model, fixed public port, and OS-standard config/data directories**
- [ ] **Step 2: Document that config files override environment fallback values for both server and desktop runtime startup**
- [ ] **Step 3: Update build/release docs so they describe the portal desktop bundle as a shell plus bundled `router-product-service` payload**

### Task 7: Full Verification

**Files:**
- No new files

- [ ] **Step 1: Run the targeted Node desktop/release contract suite**
- [ ] **Step 2: Run the focused Rust config and product-service tests**
- [ ] **Step 3: Run the broader packaging verification subset**
- [ ] **Step 4: Check the implementation against the packaging design spec before closing**
