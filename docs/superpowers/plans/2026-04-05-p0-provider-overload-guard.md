# P0 Provider Overload Guard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add gateway-side provider in-flight protection so saturated providers fail fast and execution failover can route traffic to backups instead of amplifying overload.

**Architecture:** Add a lightweight in-memory per-provider in-flight guard in `sdkwork-api-app-gateway` with configurable limits and a dedicated gateway execution-context error kind for overload. Apply the guard inside the lowest provider execution helpers so JSON and stream paths share the same behavior, and classify overload as retryable/failover-capable without poisoning persisted provider-health snapshots.

**Tech Stack:** Rust, Tokio, Axum, existing gateway/provider adapter architecture

---

### Task 1: Capture Overload Behavior With Failing Tests

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/tests/execution_context.rs`
- Test: `crates/sdkwork-api-app-gateway/tests/execution_context.rs`

- [ ] **Step 1: Write the failing test**

Add a test that:
- configures provider in-flight limit to `1`
- keeps the primary provider request open long enough to occupy the only slot
- starts a second request while the primary is busy
- expects the second request to fail over to backup immediately
- verifies the primary only received one attempt while backup handled the overflow request

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context relay_chat_completion_fails_over_when_primary_provider_is_locally_overloaded`
Expected: FAIL because the gateway currently has no provider in-flight guard

### Task 2: Implement Provider In-Flight Guard

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Test: `crates/sdkwork-api-app-gateway/tests/execution_context.rs`

- [ ] **Step 1: Add a provider overload execution-context error kind**

Extend the current gateway execution-context error model with a `ProviderOverloaded` case and classify it as retryable with a stable retry reason.

- [ ] **Step 2: Add an in-memory per-provider in-flight guard**

Implement:
- configurable max in-flight limit
- non-blocking slot acquisition
- RAII release on completion
- shared state keyed by `provider_id`

- [ ] **Step 3: Apply the guard inside shared provider execution helpers**

Wrap the current adapter execution path so the guard applies before upstream dispatch for both JSON and stream requests.

- [ ] **Step 4: Prevent local overload from poisoning provider health**

Ensure overload and expired-deadline local control-plane errors do not persist an unhealthy provider-health snapshot.

- [ ] **Step 5: Re-run the targeted test**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context relay_chat_completion_fails_over_when_primary_provider_is_locally_overloaded`
Expected: PASS

### Task 3: Regress Existing Timeout / Failover Behavior

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/tests/execution_context.rs`
- Test: `crates/sdkwork-api-app-gateway/tests/execution_context.rs`

- [ ] **Step 1: Re-run the existing timeout failover test**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context relay_chat_completion_times_out_primary_and_fails_over_to_backup`
Expected: PASS

- [ ] **Step 2: Re-run the whole execution-context test target**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context`
Expected: PASS with all execution-context tests green
