# P0 Gateway Execution Context Metrics And Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** remove the gateway test-target compile blocker and expose execution-context failure reasons as first-class metrics so failover, timeout, overload, and deadline behavior are commercially observable.

**Architecture:** keep the current gateway execution-context model and shared Prometheus registry, but add a narrow metric series keyed by capability, provider, and local failure reason. Fix the existing `extension_dispatch` test compile regression first so larger gateway verification can run again, then record execution-context failure reasons at the same gateway decision points that already emit upstream outcome and retry metrics.

**Tech Stack:** Rust, Tokio, Prometheus text exposition, cargo test

---

### Task 1: Remove the gateway test compile blocker

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

- [ ] **Step 1: Reproduce the compile failure**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test extension_dispatch native_dynamic_fixture_library_path -- --nocapture`
Expected: FAIL with `use of undeclared type 'Sha256'`

- [ ] **Step 2: Apply the smallest scope fix**

Move the `sha2::{Digest, Sha256}` import to a scope shared by both `sign_native_dynamic_package` and `sha256_hex_path`, without changing test behavior.

- [ ] **Step 3: Re-run the same test target**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test extension_dispatch native_dynamic_fixture_library_path -- --nocapture`
Expected: PASS compilation and execute the targeted test

### Task 2: Add execution-context failure reason metrics with TDD

**Files:**
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/execution_context.rs`

- [ ] **Step 1: Add failing assertions for execution-context failure reason metrics**

Extend gateway metrics coverage to require a Prometheus counter that distinguishes local control-plane failures such as `request_timeout`, `deadline_exceeded`, and `provider_overloaded`.

- [ ] **Step 2: Run the targeted test first and confirm failure**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context relay_chat_completion_fails_over_when_primary_provider_is_locally_overloaded -- --nocapture`
Expected: FAIL because the new metric series is not implemented yet

- [ ] **Step 3: Implement the metric series**

Add a shared observability counter and record it from the gateway failure path only when the error is a gateway execution-context error, keeping provider-health persistence behavior unchanged.

- [ ] **Step 4: Re-run the targeted execution-context test**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context relay_chat_completion_fails_over_when_primary_provider_is_locally_overloaded -- --nocapture`
Expected: PASS

### Task 3: Run focused regressions

**Files:**
- Verify only

- [ ] **Step 1: Re-run the full execution-context target**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test execution_context`
Expected: PASS

- [ ] **Step 2: Re-run the extension dispatch target**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --test extension_dispatch`
Expected: PASS

- [ ] **Step 3: Re-run the gateway library metrics target**

Run: `cargo test -j 1 -p sdkwork-api-app-gateway --lib gateway_upstream_outcomes_are_recorded_to_shared_gateway_metrics -- --nocapture`
Expected: PASS
