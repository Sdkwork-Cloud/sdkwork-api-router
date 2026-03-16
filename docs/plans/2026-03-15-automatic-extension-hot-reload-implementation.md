# Automatic Extension Hot Reload Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add automatic extension hot reload supervision so standalone processes detect extension tree changes and invoke the existing runtime reload orchestration without operator intervention.

**Architecture:** Extend `sdkwork-api-app-gateway` with a polling-based extension-tree fingerprint supervisor keyed by the current discovery policy, add standalone config support for `extension_hot_reload_interval_secs`, and start the supervisor from standalone service binaries. Reuse `reload_configured_extension_host()` as the only reload path so shutdown and rebuild semantics stay centralized.

**Tech Stack:** Rust, Tokio, std filesystem metadata, existing extension runtime reload orchestration

---

### Task 1: Add failing tests for config and watcher-driven reload

**Files:**
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

- [ ] **Step 1: Add a failing config parse test**

Assert that `StandaloneConfig` exposes `extension_hot_reload_interval_secs` and parses `SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS`.

- [ ] **Step 2: Run the focused config test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-config parses_extension_discovery_settings_from_pairs -q`
Expected: FAIL because the config field does not exist yet.

- [ ] **Step 3: Add a failing gateway hot reload supervision test**

Add a serial native-dynamic test that:

1. builds the initial configured host
2. starts a new automatic hot reload supervisor
3. rewrites the signed manifest with equivalent content to change its timestamp
4. expects the lifecycle log to become `init`, `shutdown`, `init`

- [ ] **Step 4: Run the focused gateway test to confirm RED**

Run: `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch configured_extension_host_hot_reload_supervision_reloads_after_extension_tree_change -q`
Expected: FAIL because no automatic hot reload supervisor exists yet.

### Task 2: Implement the watcher-driven supervisor and config field

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `services/gateway-service/Cargo.toml`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/Cargo.toml`
- Modify: `services/admin-api-service/src/main.rs`

- [ ] **Step 1: Add config support**

Model `extension_hot_reload_interval_secs` in `StandaloneConfig`, file overlay parsing, environment override parsing, and resolved environment exports.

- [ ] **Step 2: Add a gateway-owned hot reload supervisor**

Implement a polling loop that:

1. captures a baseline watch state
2. recomputes the extension tree fingerprint on each interval
3. invokes `reload_configured_extension_host()` when the watch state changes
4. updates the baseline only after successful reload

- [ ] **Step 3: Start the supervisor in standalone services**

Wire the new supervisor into gateway and admin service startup using the new config field.

- [ ] **Step 4: Run focused tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-config parses_extension_discovery_settings_from_pairs -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch configured_extension_host_hot_reload_supervision_reloads_after_extension_tree_change -q`

Expected: PASS

### Task 3: Update documentation

**Files:**
- Create: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Create: `docs/plans/2026-03-15-automatic-extension-hot-reload-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/zh/operations/configuration.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`

- [ ] **Step 1: Record the design and implementation plan**

Describe the polling-based watch model, scope boundaries, and safety rules.

- [ ] **Step 2: Refresh runtime and configuration docs**

Document that watcher-driven automatic hot reload is now active for standalone services and that `SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS` enables it.

### Task 4: Verify the workspace

**Files:**
- Modify: repository worktree from previous tasks

- [ ] **Step 1: Run focused package verification**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-config -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`

Expected: PASS

- [ ] **Step 2: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS
