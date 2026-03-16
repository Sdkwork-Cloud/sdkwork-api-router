# Native-Dynamic Request Draining Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add single-process native-dynamic request draining so runtime shutdown waits for in-flight plugin calls before unload or reload proceeds.

**Architecture:** Extend `sdkwork-api-extension-host` native-dynamic runtime state with drain-aware invocation accounting. Reuse the existing shutdown helpers so explicit reload, targeted reload, and automatic hot reload all inherit the safer behavior without adding a second control path. Use deterministic fixture-controlled delays in the native mock plugin to prove JSON and stream drains in tests.

**Tech Stack:** Rust, Tokio, std synchronization primitives, existing native-dynamic ABI fixture

---

## Chunk 1: Failing Drain Tests

### Task 1: Add failing extension-host tests for in-flight drain behavior

**Files:**
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`

- [ ] **Step 1: Add a failing JSON drain test**

Write a test that starts a delayed native-dynamic JSON invocation, triggers shutdown while the call is still running, and expects the invocation to finish before the plugin shutdown hook runs.

- [ ] **Step 2: Run the focused JSON drain test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime shutting_down_native_dynamic_runtimes_waits_for_in_flight_json_invocation -q`
Expected: FAIL because the fixture does not yet hold the call open and shutdown does not yet drain in-flight invocations.

- [ ] **Step 3: Add a failing stream drain test**

Write a test that starts a delayed native-dynamic stream invocation, triggers shutdown after the stream thread starts, and expects the stream thread to finish before plugin shutdown runs.

- [ ] **Step 4: Run the focused stream drain test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime shutting_down_native_dynamic_runtimes_waits_for_in_flight_stream_invocation -q`
Expected: FAIL because stream execution currently spawns a thread that is not tracked by shutdown.

## Chunk 2: Drain-Aware Runtime State

### Task 2: Implement native-dynamic request draining

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`

- [ ] **Step 1: Add fixture delay and invocation-log controls**

Support environment-driven test hooks in the native mock provider for delayed JSON and stream execution plus invocation log events.

- [ ] **Step 2: Add drain-aware runtime accounting in the extension host**

Implement:

- active invocation counting
- draining gate that rejects new invocations once shutdown starts
- shutdown wait for active JSON, stream, and health-check calls
- stream-thread accounting so the lease ends only after plugin execution finishes

- [ ] **Step 3: Run focused extension-host verification**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`
Expected: PASS.

## Chunk 3: Docs And Final Verification

### Task 3: Document request draining as implemented and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-native-dynamic-request-draining-design.md`
- Create: `docs/plans/2026-03-15-native-dynamic-request-draining-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-15-targeted-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`

- [ ] **Step 1: Update runtime docs**

Move request draining from remaining gap to implemented capability and clarify that reload now waits for in-flight native-dynamic work before unload.

- [ ] **Step 2: Keep only the remaining unresolved gaps**

Leave:

- multi-node coordinated rollout
- migration-safe secret-manager reconfiguration

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
