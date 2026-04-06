# Commercial Readiness Hardening Phase 8 Recovery Probe Single-Probe Lease Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** upgrade provider health recovery probes from cohort-only routing to a single-probe lease so multi-instance gateways do not stampede back onto a stale-unhealthy primary.

**Architecture:** extend routing selection context with an optional distributed lock store and a configurable probe lease TTL. Gateway will inject the cache runtime distributed lock implementation, while routing will only probe a stale primary after the request matches the configured cohort and successfully acquires the provider-scoped recovery probe lease.

**Tech Stack:** Rust, cache runtime distributed locks, cargo test

---

### Task 1: Add failing tests for lease-gated recovery probes

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Modify: `crates/sdkwork-api-app-routing/Cargo.toml`

- [ ] **Step 1: Add routing integration tests that require a recovery probe lease to be acquired before stale-primary probing is allowed**
- [ ] **Step 2: Add a second test that pre-holds the same probe lease and requires routing to keep the healthy backup selected**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route route_simulation_selects_stale_primary_for_recovery_probe_when_probe_lease_is_available -- --nocapture` and confirm it fails first**

### Task 2: Wire lease-aware routing selection into production code

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/Cargo.toml`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `services/gateway-service/src/main.rs`

- [ ] **Step 1: Add cache distributed lock support and configurable recovery probe lease TTL to routing**
- [ ] **Step 2: Require successful lease acquisition before gateway-triggered recovery probes can select the stale primary**
- [ ] **Step 3: Inject the cache runtime distributed lock store from gateway startup into routing decisions**

### Task 3: Verify regressions stay green

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test runtime_execution -- --nocapture`**
