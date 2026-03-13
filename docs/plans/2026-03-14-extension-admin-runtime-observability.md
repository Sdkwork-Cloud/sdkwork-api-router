# Extension Admin Runtime Observability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a real admin control-plane surface for discovered extension packages and active connector runtime status so the plugin system is observable and operable instead of only configurable.

**Architecture:** Keep the current extension architecture additive. Reuse `sdkwork-api-extension-host` as the source of truth for filesystem package discovery and connector process supervision, expose thin application-layer DTOs in `sdkwork-api-app-extension`, then wire authenticated Axum admin endpoints that read the same extension discovery policy contract the runtime already uses.

**Tech Stack:** Rust, Axum, serde, sqlx-backed admin tests, extension host discovery and connector supervision

---

### Task 1: Add failing tests for extension package discovery and runtime status APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Create: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`
- Modify: `crates/sdkwork-api-app-extension/Cargo.toml`

**Step 1: Write the failing tests**

Add tests that prove:

- the app layer can list discovered extension packages from an `ExtensionDiscoveryPolicy`
- the app layer can list active connector runtime statuses from the host registry
- the admin API exposes `GET /admin/extensions/packages`
- the admin API exposes `GET /admin/extensions/runtime-statuses`

Use a temporary `sdkwork-extension.toml` package for discovery and a Windows PowerShell connector process for active runtime status.

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`

Expected: FAIL because the app layer does not expose discovery or runtime-status DTOs and the admin router has no such endpoints.

### Task 2: Expose discovery and connector runtime status from the application and host layers

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/Cargo.toml`
- Modify: `crates/sdkwork-api-app-extension/src/lib.rs`
- Modify: `crates/sdkwork-api-app-extension/Cargo.toml`

**Step 1: Add host runtime status listing**

Expose a public host function that returns all currently running supervised connector runtime statuses while pruning exited processes.

**Step 2: Add application DTOs**

Introduce serializable DTOs for:

- discovered extension packages
- connector runtime statuses

Provide app-layer functions:

- `list_discovered_extension_packages(policy)`
- `list_connector_runtime_statuses()`

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 3: Wire authenticated admin endpoints and runtime policy state

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Extend admin state**

Add extension discovery policy to `AdminApiState`, defaulting existing constructors to the current environment-driven extension search path and runtime toggles.

**Step 2: Add endpoints**

Expose authenticated handlers for:

- `GET /admin/extensions/packages`
- `GET /admin/extensions/runtime-statuses`

Return app-layer DTOs directly as JSON.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-admin -q`

Expected: PASS

### Task 4: Update docs and verify the workspace

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Document the new control-plane capability**

Document that the admin API now exposes extension discovery and active connector runtime status for operational visibility.

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `cargo clippy --no-deps -p sdkwork-api-app-extension -p sdkwork-api-interface-admin -p sdkwork-api-extension-host --all-targets -- -D warnings`

Expected: PASS

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-extension-admin-runtime-observability.md README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md crates/sdkwork-api-extension-host crates/sdkwork-api-app-extension crates/sdkwork-api-interface-admin
git commit -m "feat: add extension runtime observability APIs"
git push
```
