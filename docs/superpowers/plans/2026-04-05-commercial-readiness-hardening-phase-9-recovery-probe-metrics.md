# Commercial Readiness Hardening Phase 9 Recovery Probe Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** expose provider health recovery probe outcomes as first-class metrics so operators can see whether stale-primary revalidation is succeeding, being lease-blocked, or being conservatively suppressed by lock backend failures.

**Architecture:** add structured recovery probe outcome metadata onto routing decisions, but keep persistence schemas unchanged. Gateway will record Prometheus counters from fresh routing decisions only, keyed by provider and recovery probe outcome, while routing tests pin the selected and lease-blocked states.

**Tech Stack:** Rust, serde, Prometheus text exposition, cargo test

---

### Task 1: Add failing tests for structured recovery probe outcomes

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Extend recovery probe routing tests to require structured probe outcome metadata on selected and lease-blocked decisions**
- [ ] **Step 2: Extend gateway metrics tests to require a Prometheus counter for recovery probe outcomes**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route route_simulation_selects_stale_primary_for_recovery_probe_when_probe_lease_is_available -- --nocapture` and confirm it fails first**

### Task 2: Implement recovery probe outcome propagation and metrics

**Files:**
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Add a structured recovery probe outcome model onto routing decisions**
- [ ] **Step 2: Populate selected / lease_contended / lease_error outcomes during routing**
- [ ] **Step 3: Record `sdkwork_provider_health_recovery_probes_total` from fresh gateway routing decisions**

### Task 3: Run targeted regressions

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test runtime_execution -- --nocapture`**
