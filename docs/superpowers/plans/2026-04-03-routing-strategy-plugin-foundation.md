# Routing Strategy Plugin Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** formalize the routing strategy seam as a registry-backed plugin contract without changing current routing behavior, React package structure, or admin and portal product composition.

**Architecture:** add a dedicated routing policy contract crate that owns strategy-plugin metadata, execution contracts, built-in strategy implementations, and a registry. Keep `sdkwork-api-app-routing` responsible for candidate assessment and orchestration, but delegate final candidate selection to the registry so later billing and admission plugins can follow the same pattern.

**Tech Stack:** Rust workspace crates, `sdkwork-api-domain-routing`, `sdkwork-api-app-routing`, Cargo workspace tests

---

### Task 1: Write the routing strategy plugin plan into code-facing workspace structure

**Files:**
- Create: `crates/sdkwork-api-policy-routing/Cargo.toml`
- Create: `crates/sdkwork-api-policy-routing/src/lib.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the new crate to the workspace**
- [ ] **Step 2: Create the package manifest wired to workspace dependencies**
- [ ] **Step 3: Add an empty library entrypoint so tests can compile against the new seam**

### Task 2: Write failing contract tests for the new strategy registry

**Files:**
- Create: `crates/sdkwork-api-policy-routing/tests/strategy_registry.rs`

- [ ] **Step 1: Write a failing test asserting the builtin registry resolves every current `RoutingStrategy`**
- [ ] **Step 2: Write a failing test asserting weighted-random selection remains deterministic when a seed is provided**
- [ ] **Step 3: Write a failing test asserting geo-affinity reports fallback metadata when no region is supplied**
- [ ] **Step 4: Run `cargo test -p sdkwork-api-policy-routing strategy_registry -- --nocapture` and verify the tests fail for missing implementation, not unrelated compile errors**

### Task 3: Implement the routing policy contract crate

**Files:**
- Modify: `crates/sdkwork-api-policy-routing/src/lib.rs`

- [ ] **Step 1: Define plugin metadata and execution input/output contracts**
- [ ] **Step 2: Define `RoutingStrategyPlugin` and `RoutingStrategyPluginRegistry`**
- [ ] **Step 3: Port the existing built-in routing strategy logic into concrete plugin implementations**
- [ ] **Step 4: Expose a `builtin_routing_strategy_registry()` helper**
- [ ] **Step 5: Run `cargo test -p sdkwork-api-policy-routing strategy_registry -- --nocapture` and verify the new crate is green**

### Task 4: Add an app-routing integration test first

**Files:**
- Modify: `crates/sdkwork-api-app-routing/Cargo.toml`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Add the new crate as an app-routing dependency**
- [ ] **Step 2: Write a failing app-routing test asserting the builtin plugin registry remains aligned with the persisted `RoutingStrategy` values**
- [ ] **Step 3: Run the focused app-routing test and verify it fails before integration**

### Task 5: Integrate app-routing orchestration with the registry

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`

- [ ] **Step 1: Replace the local hard-coded strategy dispatch with registry-backed execution**
- [ ] **Step 2: Keep `CandidateSelection` output shape and routing evidence stable so admin and portal contracts do not change**
- [ ] **Step 3: Reuse the new plugin result for selection reason, fallback reason, SLO flags, and selection seed handling**
- [ ] **Step 4: Run `cargo test -p sdkwork-api-app-routing simulate_route -- --nocapture` and verify the focused routing suite stays green**

### Task 6: Verify no evidence-chain regressions

**Files:**
- Verify only: `crates/sdkwork-api-interface-portal/tests/portal_routing.rs`
- Verify only: `apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`

- [ ] **Step 1: Run `cargo test -p sdkwork-api-interface-portal portal_routing -- --nocapture`**
- [ ] **Step 2: Run `node apps/sdkwork-router-admin/tests/admin-architecture.test.mjs`**
- [ ] **Step 3: Run `pnpm.cmd --dir apps/sdkwork-router-admin typecheck`**
- [ ] **Step 4: Document remaining follow-on work for billing and admission plugins**
