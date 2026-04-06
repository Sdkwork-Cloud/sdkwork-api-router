# P0 Default Recovery Probe Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ensure stale-unhealthy primary providers can automatically re-enter service through a conservative default recovery-probe cohort, even when operators have not explicitly configured recovery probing.

**Architecture:** Keep the existing routing recovery-probe mechanism intact, but change its default configuration from disabled to a small non-zero percentage so deterministic-priority routing can periodically probe stale-unhealthy primaries. Verify this via routing simulation tests that cover both explicit config and default behavior.

**Tech Stack:** Rust, Tokio, existing routing simulation and provider-health model

---

### Task 1: Add a Failing Default-Behavior Test

**Files:**
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Test: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Write the failing test**

Add a routing simulation test showing that:
- the primary provider has a stale unhealthy snapshot
- the backup provider is currently healthy
- the recovery probe percentage env is left empty so the default path is exercised
- a deterministic seed inside the default cohort causes the primary to be selected for recovery probing

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -j 1 -p sdkwork-api-app-routing --test simulate_route route_simulation_selects_stale_primary_for_recovery_probe_by_default`
Expected: FAIL because the default recovery probe percentage is currently 0

### Task 2: Enable a Conservative Default Recovery Probe Cohort

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Test: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Add a non-zero default recovery probe percentage**

Introduce a small default value, keeping explicit env configuration authoritative.

- [ ] **Step 2: Re-run the new targeted test**

Run: `cargo test -j 1 -p sdkwork-api-app-routing --test simulate_route route_simulation_selects_stale_primary_for_recovery_probe_by_default`
Expected: PASS

### Task 3: Regress Existing Recovery Probe Behavior

**Files:**
- Test: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Re-run existing explicit recovery probe tests**

Run: `cargo test -j 1 -p sdkwork-api-app-routing --test simulate_route route_simulation_selects_stale_primary_for_recovery_probe_when_probe_cohort_enabled`
Expected: PASS

- [ ] **Step 2: Re-run the gating test**

Run: `cargo test -j 1 -p sdkwork-api-app-routing --test simulate_route route_simulation_keeps_backup_when_request_is_outside_recovery_probe_cohort`
Expected: PASS
