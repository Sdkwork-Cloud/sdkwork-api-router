# Commercial Readiness Hardening Phase 6 Recovery Probe Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** allow controlled automatic failback probing so a stale-unhealthy primary can be retried by a small request cohort instead of staying permanently stuck behind a healthy backup.

**Architecture:** keep the short-lived health snapshot circuit breaker as the default guardrail, then add an env-controlled routing override for deterministic-priority routes. When the top-ranked provider only has a stale unhealthy snapshot and a healthy backup is currently winning, a seeded request cohort can intentionally select the primary as a half-open recovery probe and refresh health state based on the execution outcome.

**Tech Stack:** Rust, Axum, SQLite, cargo test

---

### Task 1: Add failing routing regressions for recovery probe selection

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Add a failing test showing a stale-unhealthy primary is selected for a recovery probe when the probe cohort is enabled**
- [ ] **Step 2: Add a failing test showing a request outside the configured probe cohort stays on the healthy backup**
- [ ] **Step 3: Run the targeted routing tests and confirm they fail first**

### Task 2: Implement seeded recovery probe override in routing

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`

- [ ] **Step 1: Add env-backed recovery probe percentage parsing with a safe disabled default**
- [ ] **Step 2: Detect a stale-unhealthy top-ranked provider that is currently losing to a healthy backup**
- [ ] **Step 3: Override candidate selection only for requests that fall into the configured probe cohort**
- [ ] **Step 4: Re-run the targeted routing recovery probe tests**

### Task 3: Run targeted routing and gateway regressions

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test runtime_execution -- --nocapture`**
