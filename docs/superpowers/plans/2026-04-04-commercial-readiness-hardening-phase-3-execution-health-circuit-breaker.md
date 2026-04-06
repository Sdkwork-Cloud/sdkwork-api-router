# Commercial Readiness Hardening Phase 3 Execution Health Circuit Breaker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** turn chat failover into a traffic-stable execution circuit breaker by persisting short-lived upstream execution health and teaching routing to prefer healthy backups on subsequent requests.

**Architecture:** reuse the existing provider health snapshot surface instead of creating a new in-memory breaker. Gateway execution helpers will write transient healthy/unhealthy snapshots on upstream attempt outcomes, and routing will only honor fresh execution snapshots within a bounded TTL so bad primaries cool down briefly instead of being suppressed forever.

**Tech Stack:** Rust, Axum, sqlx, SQLite, cargo test

---

### Task 1: Make routing treat only fresh persisted health snapshots as authoritative

**Files:**
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`

- [ ] **Step 1: Add failing routing tests for fresh unhealthy snapshots and stale snapshot expiry**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture` and confirm the new assertions fail first**
- [ ] **Step 3: Implement a bounded freshness window for persisted provider health snapshots**
- [ ] **Step 4: Re-run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**

### Task 2: Persist execution-driven health snapshots from gateway upstream outcomes

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_route.rs`

- [ ] **Step 1: Add a failing end-to-end chat regression test showing that a second request bypasses a provider that just failed**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route stateful_chat_route_skips_recently_failed_primary_provider_on_following_request -- --nocapture` and confirm it fails first**
- [ ] **Step 3: Persist transient healthy/unhealthy provider health snapshots from gateway execution helpers on success and failure**
- [ ] **Step 4: Re-run the targeted failover regression test**

### Task 3: Run targeted regressions across routing and gateway compatibility surfaces

**Files:**
- Verify only

- [ ] **Step 1: Run `cargo test --offline -p sdkwork-api-app-gateway --lib -- --nocapture`**
- [ ] **Step 2: Run `cargo test --offline -p sdkwork-api-interface-http --test chat_route --test anthropic_messages_route --test gemini_generate_content_route --test runtime_execution -- --nocapture`**
- [ ] **Step 3: Run `cargo test --offline -p sdkwork-api-app-routing --test simulate_route -- --nocapture`**
