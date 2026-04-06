# Commercial Readiness Hardening Phase 5 Provider Health Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** expose live provider health state through `/metrics` so execution failover and short-lived circuit breaker decisions are directly observable in Prometheus.

**Architecture:** extend the shared `HttpMetricsRegistry` with provider health gauges keyed by service, provider, and runtime. Gateway execution health persistence will update those gauges on every success and failure, and the existing chat failover integration tests will verify the rendered metrics surface end to end.

**Tech Stack:** Rust, Prometheus text exposition, Axum, cargo test

---

### Task 1: Add failing metric assertions for provider health state

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`

- [ ] **Step 1: Extend the existing chat failover metrics assertion to require provider health gauges for the failed primary and recovered backup**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route stateful_chat_route_fails_over_to_backup_provider_and_records_actual_provider -- --nocapture` and confirm it fails first**

### Task 2: Add provider health gauges to the shared metrics registry

**Files:**
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Extend the shared registry state and Prometheus renderer with provider health gauge series**
- [ ] **Step 2: Update gateway execution health snapshot persistence to publish provider health into the metrics registry on every execution outcome**
- [ ] **Step 3: Run the targeted failover test again**

### Task 3: Run targeted regressions across observability and gateway compatibility surfaces

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test health_route -- --nocapture`**
