# Commercial Readiness Hardening Phase 7 Health Persistence Failure Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** expose provider health snapshot persistence failures as first-class metrics so production alerting can detect when routing health state is no longer being durably refreshed.

**Architecture:** extend the shared observability registry with a counter keyed by provider and runtime for health persistence failures. Gateway execution health persistence will increment that counter whenever snapshot storage fails, while preserving the current best-effort request semantics.

**Tech Stack:** Rust, Prometheus text exposition, cargo test

---

### Task 1: Add failing metric assertions for provider health persistence failures

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Extend the existing gateway metrics unit test to require a provider health persistence failure counter series**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-gateway --lib gateway_upstream_outcomes_are_recorded_to_shared_gateway_metrics -- --nocapture` and confirm it fails first**

### Task 2: Add persistence failure counters to observability and gateway

**Files:**
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Extend the shared registry state and Prometheus renderer with `sdkwork_provider_health_persist_failures_total`**
- [ ] **Step 2: Update gateway health snapshot persistence to increment the failure counter when store writes fail**
- [ ] **Step 3: Re-run the targeted gateway metrics unit test**

### Task 3: Run targeted regressions for metrics surfaces

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-http --test health_route --test chat_route -- --nocapture`**
