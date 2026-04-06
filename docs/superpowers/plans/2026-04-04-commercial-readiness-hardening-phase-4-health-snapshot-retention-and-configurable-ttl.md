# Commercial Readiness Hardening Phase 4 Health Snapshot Retention And Configurable TTL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** eliminate provider health snapshot write amplification and make routing freshness TTL configurable for production operations.

**Architecture:** keep the existing persisted health snapshot surface, but change persistence semantics from append-only to keyed replacement per provider/runtime/instance so hot providers do not grow unbounded history. Routing will stop relying on a hard-coded TTL and instead read an environment-backed freshness window, preserving the current default while enabling production tuning without code changes.

**Tech Stack:** Rust, sqlx, SQLite, Postgres, cargo test

---

### Task 1: Make provider health snapshot persistence idempotent per provider/runtime/instance

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`

- [ ] **Step 1: Add failing SQLite store test showing repeated writes for the same provider/runtime/instance keep only the latest record**
- [ ] **Step 2: Add failing Postgres integration test with the same expectation when `SDKWORK_TEST_POSTGRES_URL` is provided**
- [ ] **Step 3: Run targeted storage tests and confirm the new assertions fail first**
- [ ] **Step 4: Implement keyed replacement semantics in both stores before inserting the latest snapshot**
- [ ] **Step 5: Re-run the targeted storage tests**

### Task 2: Make routing provider health freshness TTL configurable

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Add failing routing tests showing a custom env TTL can preserve a 5-minute-old snapshot and a tiny TTL can expire a recent snapshot**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture` and confirm the new assertions fail first**
- [ ] **Step 3: Replace the hard-coded freshness constant with an env-backed TTL loader that defaults to the current 60 seconds**
- [ ] **Step 4: Re-run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**

### Task 3: Run targeted regressions across storage, routing, and execution paths

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-storage-sqlite --lib -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 4: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test anthropic_messages_route --test gemini_generate_content_route --test runtime_execution -- --nocapture`**
