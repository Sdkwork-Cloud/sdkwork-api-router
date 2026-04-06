# Commercial Readiness Hardening Phase 2 Flow Control And Observability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close the most visible commercial traffic-governance gaps by making gateway rate limiting observable to callers and by preparing the execution path for later request-level failover.

**Architecture:** move gateway rate-limit enforcement into middleware so policy evaluation can attach standard response headers on both success and rejection paths, instead of burying enforcement inside request extractors. Keep the existing storage-backed rate-limit engine intact and layer observability on top of it before expanding into request-level upstream failover.

**Tech Stack:** Rust, Axum, sqlx, SQLite, cargo test

---

### Task 1: Move gateway rate limiting into middleware with response headers

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/rate_limit_enforcement.rs`

- [ ] **Step 1: Add failing tests for success and rejection responses to expose rate-limit headers**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-http --test rate_limit_enforcement -- --nocapture` and confirm the new assertions fail first**
- [ ] **Step 3: Implement middleware-backed gateway rate-limit enforcement that records the evaluation in request extensions**
- [ ] **Step 4: Append `retry-after` and `x-ratelimit-*` headers on allowed and rejected gateway responses**
- [ ] **Step 5: Re-run `cargo test --offline -p sdkwork-api-interface-http --test rate_limit_enforcement -- --nocapture`**

### Task 2: Capture upstream execution outcomes for observability and later failover

**Files:**
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/tests/telemetry_smoke.rs`

- [ ] **Step 1: Add failing metrics tests for upstream attempt, success, and failure counters**
- [ ] **Step 2: Run the targeted observability tests and confirm they fail before implementation**
- [ ] **Step 3: Introduce low-cardinality upstream outcome metrics keyed by service, capability, provider, and outcome**
- [ ] **Step 4: Emit the metrics from gateway provider execution helpers without changing routing behavior yet**
- [ ] **Step 5: Re-run the targeted observability tests**

### Task 3: Add chat-completion request failover on upstream execution failure

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/runtime_execution.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [ ] **Step 1: Add failing regression tests for retrying the next ranked provider when the selected provider execution fails**
- [ ] **Step 2: Run the targeted gateway/runtime tests and confirm they fail for the expected reason**
- [ ] **Step 3: Implement bounded request-level failover for chat-completion JSON and stream paths using ranked routing candidates**
- [ ] **Step 4: Ensure usage/billing attribution uses the actual executed provider, not only the planned routing decision**
- [ ] **Step 5: Re-run the targeted runtime and gateway tests**
