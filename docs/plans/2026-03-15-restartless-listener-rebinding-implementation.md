# Restartless Listener Rebinding Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let standalone gateway, admin, and portal services rebind their configured TCP listener address from config-file changes without restarting the process.

**Architecture:** Add a shared Axum listener host in `sdkwork-api-app-runtime` that can pre-bind a replacement socket, activate a new server generation, and gracefully drain the old generation. Extend standalone runtime supervision to treat the current service's bind field as live-reloadable, prepare rebinds before applying other safe replacements, and wire all three standalone services to serve through the shared host.

**Tech Stack:** Rust, Axum, Tokio, existing standalone config loader and runtime supervision

---

## Chunk 1: RED Listener Rebinding Tests

### Task 1: Add focused failing coverage for listener host and config-driven rebinding

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/Cargo.toml`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add a failing direct listener-host test**

Write a test that starts a simple health router on one bind, rebinds to a second bind through a planned listener-host API, and expects the new address to respond while the old address stops accepting new connections.

- [ ] **Step 2: Add a failing config-supervision rebind test**

Write a test that starts standalone supervision for the gateway service, rewrites `gateway_bind` in the config file, and expects the listener to move to the new address without restarting the process.

- [ ] **Step 3: Add a failing bind-fallback test**

Write a test that changes `gateway_bind` to an address already occupied by another listener, proves the old listener keeps serving, then releases the target address and expects supervision to retry the same pending rebind successfully.

- [ ] **Step 4: Run the focused RED test target**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_listener_host_rebinds_requests_to_new_bind -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_rebinds_listener_after_config_file_change -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_retries_listener_rebind_after_bind_failure -q`

Expected: FAIL because there is no shared listener-host abstraction and runtime supervision still treats bind changes as restart-only.

## Chunk 2: Listener Host Implementation

### Task 2: Implement a shared restartless listener host

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/Cargo.toml`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`

- [ ] **Step 1: Add the listener-host types**

Introduce a host plus reload-handle pair that can own the active Axum server generation and expose current-bind inspection.

- [ ] **Step 2: Add pre-bound rebind support**

Support preparing a replacement `TcpListener` before activation so config supervision can fail fast without dropping the old listener.

- [ ] **Step 3: Add graceful cutover and shutdown handling**

Start the new server before signaling the old server's graceful shutdown, and keep enough exit reporting to surface unexpected active-server termination.

- [ ] **Step 4: Re-run the direct listener-host test**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_listener_host_rebinds_requests_to_new_bind -q`

Expected: PASS.

## Chunk 3: Config-Supervised Bind Reload

### Task 3: Extend standalone runtime supervision to manage listener binds

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`

- [ ] **Step 1: Treat the current service bind field as live-reloadable**

Add service-relevant bind detection and stop reporting unrelated service binds as local restart requirements.

- [ ] **Step 2: Prepare listener rebinds during reload evaluation**

When the current service bind changes, pre-bind the replacement listener before applying safe live replacements.

- [ ] **Step 3: Activate the prepared listener after successful reload preflight**

Complete the rebind only after other preflight work succeeds, and keep retrying on later polls when binding fails.

- [ ] **Step 4: Re-run the two config-supervision tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_rebinds_listener_after_config_file_change -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_retries_listener_rebind_after_bind_failure -q`

Expected: PASS.

## Chunk 4: Service Wiring

### Task 4: Move standalone services onto the shared listener host

**Files:**
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`

- [ ] **Step 1: Build the router once and start the listener host**

Replace one-shot `TcpListener::bind + axum::serve` startup with the shared listener host for each binary.

- [ ] **Step 2: Pass listener reload handles into runtime supervision**

Extend the existing supervision wiring so bind changes become live for all three services.

- [ ] **Step 3: Wait on the shared host instead of a single serve future**

Keep the process lifetime anchored to the active listener host so unexpected server exits still surface as errors.

- [ ] **Step 4: Run the app-runtime test target again**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision -q`

Expected: PASS.

## Chunk 5: Docs And Full Verification

### Task 5: Update runtime docs and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-restartless-listener-rebinding-design.md`
- Create: `docs/plans/2026-03-15-restartless-listener-rebinding-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-restartless-store-and-jwt-reconfiguration-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-request-draining-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-drain-timeout-rollback-design.md`

- [ ] **Step 1: Mark listener rebinding implemented**

Move `gateway_bind`, `admin_bind`, and `portal_bind` into the live-reloadable set for supervised standalone services.

- [ ] **Step 2: Keep the remaining-gap list accurate**

Reduce the standalone reconfiguration list to secret-manager reconfiguration and multi-node rollout.

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
