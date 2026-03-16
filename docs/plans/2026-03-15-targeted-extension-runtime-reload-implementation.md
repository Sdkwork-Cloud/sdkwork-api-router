# Targeted Extension Runtime Reload Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add targeted extension runtime reload control so the admin API and runtime console can reload one extension or one connector instance without forcing unrelated managed runtimes to churn.

**Architecture:** Keep the existing configured-host rebuild seam in `sdkwork-api-app-gateway`, but make runtime shutdown selective and native-dynamic rebuild reuse-aware. Resolve instance-targeted requests in the admin layer using persisted installation data, then expose the result through shared runtime DTOs and console actions.

**Tech Stack:** Rust, Tokio, Axum, existing extension host runtime registries, TypeScript, React

---

## Chunk 1: Failing Runtime Reload Tests

### Task 1: Add failing tests for targeted reload and native-dynamic reuse

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Add a failing gateway test for targeted reload of an unrelated scope**

Prove that an already loaded native-dynamic runtime should not append another `init` event when the reload target does not include that extension.

- [ ] **Step 2: Run the focused gateway test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch configured_extension_host_targeted_reload_does_not_reinitialize_unrelated_native_dynamic_runtime -q`
Expected: FAIL because only global reload exists and host rebuild does not yet reuse native-dynamic runtimes.

- [ ] **Step 3: Add failing admin API tests for targeted runtime reload**

Cover:

- `extension_id` request
- `instance_id` request resolving correctly
- invalid request when both identifiers are supplied

- [ ] **Step 4: Run the focused admin test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes extension_runtime_reload_endpoint_supports_targeted_scope -q`
Expected: FAIL because the endpoint does not yet accept or return scope metadata.

## Chunk 2: Selective Shutdown And Reuse

### Task 2: Implement selective extension runtime reload in Rust

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Add selective shutdown helpers in the extension host**

Implement:

- connector shutdown by extension ID
- native-dynamic shutdown by extension ID
- native-dynamic runtime reuse when the same entrypoint is already loaded

- [ ] **Step 2: Add targeted reload scope orchestration in the gateway layer**

Support:

- global
- extension
- connector instance

- [ ] **Step 3: Run focused gateway verification**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`
Expected: PASS.

## Chunk 3: Admin API And Console Surface

### Task 3: Expose targeted reload through admin APIs and runtime console

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `console/packages/sdkwork-api-types/src/index.ts`
- Modify: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Modify: `console/packages/sdkwork-api-runtime/src/index.tsx`

- [ ] **Step 1: Add targeted reload request and response DTOs**

Support optional request body identifiers and return applied scope metadata.

- [ ] **Step 2: Resolve instance scope through persisted installation data**

Connector instances stay instance-scoped; native-dynamic or builtin instances resolve to extension scope.

- [ ] **Step 3: Update the runtime console**

Keep global reload and add per-runtime targeted actions plus last-reload scope details.

- [ ] **Step 4: Run focused admin and console verification**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `pnpm --dir console -r typecheck`

Expected: PASS.

## Chunk 4: Documentation And Final Verification

### Task 4: Document targeted reload control and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-targeted-extension-runtime-reload-design.md`
- Create: `docs/plans/2026-03-15-targeted-extension-runtime-reload-implementation.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`

- [ ] **Step 1: Document scoped reload control as implemented**

Clarify the new request contract, scope semantics, and native-dynamic reuse rule.

- [ ] **Step 2: Remove targeted reload control from previous remaining-gap lists**

Keep only the remaining multi-node, request-draining, and restartless static-config gaps.

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
