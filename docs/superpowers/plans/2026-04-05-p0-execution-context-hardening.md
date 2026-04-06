# P0 Execution Context Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real execution context foundation for gateway upstream calls so timeout, deadline, idempotency, trace propagation, and OpenRouter routing/privacy preferences become first-class behavior.

**Architecture:** Extend `ProviderRequestOptions` into a small execution-context carrier that remains backward-compatible with existing header passthrough. Apply request-level timeout/deadline enforcement in gateway execution helpers, and translate OpenRouter-specific execution hints into upstream headers from the OpenAI-compatible adapter path.

**Tech Stack:** Rust, Tokio, Axum, Reqwest, existing gateway/provider adapter architecture

---

### Task 1: Expand Provider Request Options

**Files:**
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`
- Test: `crates/sdkwork-api-provider-openrouter/tests/http_execution.rs`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run the targeted provider tests to verify the new execution-context expectations fail**
- [ ] **Step 3: Add timeout/deadline/idempotency/trace/OpenRouter preference fields and builder helpers to `ProviderRequestOptions`**
- [ ] **Step 4: Re-run provider tests and keep them green**

### Task 2: Map OpenRouter Preferences Into Upstream Calls

**Files:**
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Test: `crates/sdkwork-api-provider-openrouter/tests/http_execution.rs`

- [ ] **Step 1: Write failing tests for OpenRouter-specific execution-context headers**
- [ ] **Step 2: Run the single test target and verify the failure is caused by missing propagation**
- [ ] **Step 3: Implement header translation in the OpenAI-compatible adapter path used by OpenRouter**
- [ ] **Step 4: Re-run the provider-openrouter test target and keep it green**

### Task 3: Enforce Timeout and Deadline In Gateway Execution

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Test: `crates/sdkwork-api-app-gateway/tests/routing_policy_dispatch.rs`

- [ ] **Step 1: Write failing tests for request timeout and expired deadline behavior**
- [ ] **Step 2: Run the targeted gateway test target to verify correct failure mode**
- [ ] **Step 3: Implement timeout/deadline enforcement in gateway execution helpers without breaking existing retry/failover logic**
- [ ] **Step 4: Re-run the targeted gateway tests and confirm they pass**

### Task 4: Verify End-To-End Slice

**Files:**
- Modify: `crates/sdkwork-api-provider-openrouter/tests/http_execution.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/routing_policy_dispatch.rs`

- [ ] **Step 1: Re-run all touched provider and gateway test targets together**
- [ ] **Step 2: Inspect failures and apply minimal corrective refactors**
- [ ] **Step 3: Re-run verification commands and record the exact passing evidence**
