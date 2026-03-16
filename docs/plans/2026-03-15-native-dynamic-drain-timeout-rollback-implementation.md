# Native-Dynamic Drain Timeout And Rollback Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a configurable native-dynamic drain timeout with safe rollback so stuck plugin calls do not wedge reload forever and do not trigger unsafe unload.

**Architecture:** Extend standalone runtime config with a process-wide drain-timeout setting exported through `SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS`. Refactor native-dynamic shutdown orchestration so runtime drain waits can time out before plugin shutdown begins, clear draining mode on failure, and remove registry entries only after successful shutdown. Prove the behavior with config, extension-host, and gateway reload tests.

**Tech Stack:** Rust, Tokio, std synchronization primitives, existing native-dynamic mock fixture, current standalone config system

---

## Chunk 1: Failing Timeout And Rollback Tests

### Task 1: Add focused failing tests for timeout rollback

**Files:**
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

- [ ] **Step 1: Add a config-loading test for the new drain timeout**

Cover env parsing and file-backed reload behavior for `native_dynamic_shutdown_drain_timeout_ms`.

- [ ] **Step 2: Add a failing extension-host timeout rollback test**

Write a test that starts a delayed native-dynamic JSON invocation, sets a short drain timeout, triggers shutdown, expects a timeout error, proves no plugin shutdown hook ran, and proves a second invocation can start after rollback.

- [ ] **Step 3: Add a failing gateway reload test**

Write a test that triggers configured host reload while a delayed native-dynamic invocation is still in flight under a short drain timeout, and expects reload to fail safely.

- [ ] **Step 4: Run the focused RED tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-config --test config_loading parses_native_dynamic_shutdown_drain_timeout_from_pairs_and_reload_inputs -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime shutting_down_native_dynamic_runtimes_rolls_back_after_drain_timeout -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch configured_extension_host_reload_fails_safely_when_native_dynamic_drain_times_out -q`

Expected: FAIL because the timeout field does not yet exist and native-dynamic shutdown still waits unbounded.

## Chunk 2: Configurable Timeout And Safe Rollback

### Task 2: Implement timeout-aware native-dynamic shutdown

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`

- [ ] **Step 1: Add the new standalone config field**

Support:

- `native_dynamic_shutdown_drain_timeout_ms`
- `SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS`

Include it in defaults, file overlays, env parsing, resolved env export, and runtime-dynamic config export.

- [ ] **Step 2: Add timeout-aware drain waiting in the extension host**

Implement:

- process-wide timeout lookup
- clear timeout error reporting
- drain wait with timeout
- draining rollback when timeout expires before plugin shutdown starts

- [ ] **Step 3: Change registry shutdown helpers to remove runtimes only after successful shutdown**

Ensure timed-out runtimes remain registered and callable.

- [ ] **Step 4: Re-run the focused tests to reach GREEN**

Run the same focused commands from Chunk 1.

Expected: PASS.

## Chunk 3: Docs And Full Verification

### Task 3: Update docs and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-native-dynamic-drain-timeout-rollback-design.md`
- Create: `docs/plans/2026-03-15-native-dynamic-drain-timeout-rollback-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-request-draining-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-request-draining-implementation.md`
- Modify: `docs/plans/2026-03-15-targeted-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`

- [ ] **Step 1: Update runtime docs**

Document that native-dynamic shutdown now supports an optional drain timeout with rollback-before-unload behavior.

- [ ] **Step 2: Remove timeout rollback from the remaining-gap lists**

Leave:

- multi-node coordinated runtime rollout
- migration-safe secret-manager reconfiguration

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
