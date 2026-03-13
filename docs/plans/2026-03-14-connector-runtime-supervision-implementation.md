# Connector Runtime Supervision Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn connector-style extensions from discovery-only metadata into supervised external runtimes that can be started, health-checked, and used by the gateway during real provider relay.

**Architecture:** Keep the current protocol-mapped provider adapters so connector processes can remain OpenAI-compatible HTTP runtimes, but add a supervised process layer in `sdkwork-api-extension-host`. The host will derive a connector launch contract from `ExtensionLoadPlan`, resolve relative entrypoints against discovered package roots, start external processes on demand, poll a health endpoint, and reuse the running process for subsequent gateway calls. The gateway will then ensure a connector runtime is healthy before delegating to the existing protocol adapter for upstream relay.

**Tech Stack:** Rust, std::process, std::net, serde_json, Axum tests, existing extension host and gateway crates

---

### Task 1: Add failing tests for supervised connector runtime behavior

**Files:**
- Create: `crates/sdkwork-api-extension-host/tests/connector_runtime.rs`
- Create: `crates/sdkwork-api-app-gateway/tests/connector_runtime_dispatch.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- the extension host can start a connector process from a load plan entrypoint
- the host resolves a relative entrypoint against a discovered package root
- the host waits for a health endpoint before reporting the connector as ready
- the gateway can relay chat completions through a supervised discovered connector runtime instead of only through a pre-existing in-process HTTP server

Use a temporary PowerShell-based test connector process in Windows integration tests to keep the proof end-to-end.

**Step 2: Run focused tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-host --test connector_runtime -q`
- `cargo test -p sdkwork-api-app-gateway --test connector_runtime_dispatch -q`

Expected: FAIL because the host has no connector process manager, load plans do not preserve package roots, and the gateway never starts connector processes before resolving providers.

### Task 2: Extend extension host load plans with connector runtime metadata

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/Cargo.toml`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

**Step 1: Preserve package root metadata**

Update extension registration so discovered packages retain their filesystem root in the host. Then expose that root in `ExtensionLoadPlan` so relative connector entrypoints can be resolved safely and deterministically.

**Step 2: Add connector runtime settings parsing**

Standardize a minimal connector launch contract from merged installation or instance config:

- `command_args`
- `environment`
- `working_directory`
- `health_path`
- `startup_timeout_ms`
- `startup_poll_interval_ms`

Do not require all fields. Defaults should keep local development simple.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test connector_runtime -q`
- `cargo test -p sdkwork-api-extension-host --test load_planning -q`

Expected: still FAIL on process supervision behavior, but package-root-aware load plans should now compile and behave correctly.

### Task 3: Implement supervised connector process lifecycle

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Test: `crates/sdkwork-api-extension-host/tests/connector_runtime.rs`

**Step 1: Add runtime manager**

Implement a host-owned connector runtime registry that can:

- start a process once per `instance_id`
- detect exited processes
- kill and replace unhealthy or exited processes
- expose runtime status for the current process
- shut down one or all managed connector processes for test cleanup and future host lifecycle integration

**Step 2: Add readiness probing**

Implement a lightweight HTTP health probe against the resolved `base_url + health_path` and block readiness until:

- a `200 OK` response is received, or
- the startup timeout is exceeded

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test connector_runtime -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 4: Make gateway execution start connector runtimes on demand

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/Cargo.toml`
- Test: `crates/sdkwork-api-app-gateway/tests/connector_runtime_dispatch.rs`

**Step 1: Start connector before relay**

When a provider resolves to a connector `ExtensionLoadPlan`, ensure the managed runtime is healthy before building the `ProviderExecutionTarget`. Reuse the same resolved `base_url` for both health checking and relay.

**Step 2: Keep protocol mapping intact**

Continue using the existing `openai`, `openrouter`, and `ollama` protocol adapters after supervision. Do not invent a second execution protocol in this batch.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-gateway --test connector_runtime_dispatch -q`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`

Expected: PASS

### Task 5: Update docs and verify the workspace

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Update runtime truth**

Document that:

- connector runtime loading is now supervised and executable for discovered packages
- protocol-mapped connector relay is active through the managed runtime
- native dynamic plugins remain unsupported

**Step 2: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`

Optional targeted lint:

- `cargo clippy --no-deps -p sdkwork-api-extension-host -p sdkwork-api-app-gateway --all-targets -- -D warnings`

Expected: PASS

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-connector-runtime-supervision-implementation.md README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md crates/sdkwork-api-extension-host crates/sdkwork-api-app-gateway
git commit -m "feat: supervise connector extension runtimes"
git push
```
