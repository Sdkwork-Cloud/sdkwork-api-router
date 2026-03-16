# Runtime Config Hot Reload Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reload dynamic standalone runtime config file values for extension discovery and background supervision without restarting `gateway-service` or `admin-api-service`.

**Architecture:** Preserve the original process-start env override set through a config loader, project `StandaloneConfig` into a reloadable dynamic runtime subset, and run one shared standalone runtime supervision loop that manages extension reload plus background supervisor restarts.

**Tech Stack:** Rust, tokio, existing config crate, existing extension reload orchestration, existing admin store abstractions

---

## Chunk 1: Config Loader And Watch State

### Task 1: Add reloadable config primitives

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`

- [ ] **Step 1: Add failing config tests**
- [ ] **Step 2: Add `StandaloneConfigLoader`, config watch state, and a dynamic runtime config projection**
- [ ] **Step 3: Verify config tests pass**

## Chunk 2: Shared Runtime Supervision

### Task 2: Add standalone runtime config supervision

**Files:**
- Create: `crates/sdkwork-api-app-runtime/Cargo.toml`
- Create: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Create: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Add a failing runtime supervision test**
- [ ] **Step 2: Expose a policy-driven extension host reload helper**
- [ ] **Step 3: Implement shared runtime config supervision with startup race avoidance**
- [ ] **Step 4: Verify the new runtime supervision test passes**

## Chunk 3: Service And Admin Integration

### Task 3: Connect services and current admin discovery views

**Files:**
- Modify: `services/gateway-service/Cargo.toml`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/Cargo.toml`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

- [ ] **Step 1: Switch standalone services to the shared runtime supervision**
- [ ] **Step 2: Make admin extension package listing read current runtime discovery policy**
- [ ] **Step 3: Verify the focused admin route test passes**

## Chunk 4: Documentation And Verification

### Task 4: Document the runtime config reload contract and re-run verification

**Files:**
- Create: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`
- Create: `docs/plans/2026-03-15-runtime-config-hot-reload-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/zh/operations/configuration.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`

- [ ] **Step 1: Document which config fields hot reload and which still require restart**
- [ ] **Step 2: Update prior follow-on lists so runtime config reload is no longer listed as missing**
- [ ] **Step 3: Run targeted verification**
- [ ] **Step 4: Run full verification**
