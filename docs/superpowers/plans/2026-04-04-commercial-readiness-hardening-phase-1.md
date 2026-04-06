# Commercial Readiness Hardening Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** restore release viability and close the most dangerous default security gaps in the deployed gateway, admin, and portal services.

**Architecture:** keep the existing modular router architecture intact, fix the Rust dependency drift at the identity boundary first, then harden service startup and HTTP exposure paths with configuration-backed safeguards instead of scattering ad hoc checks throughout product code. Preserve current test helpers where possible, but make production services fail closed by default.

**Tech Stack:** Rust, Axum, sqlx, SQLite, cargo test

---

### Task 1: Rebaseline the identity crate after dependency drift

**Files:**
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/jwt_and_api_key.rs`

- [ ] **Step 1: Add or extend a failing regression test for gateway API key hashing output shape if needed**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-app-identity jwt_and_api_key -- --nocapture` and capture the current compile failure**
- [ ] **Step 3: Replace the incompatible RNG import with the password-hash compatible RNG source**
- [ ] **Step 4: Replace digest formatting with an explicit hex encoding path that works with the current `sha2` output type**
- [ ] **Step 5: Re-run `cargo test -p sdkwork-api-app-identity jwt_and_api_key -- --nocapture`**

### Task 2: Add production startup guards for insecure defaults

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`
- Create: `crates/sdkwork-api-config/tests/security_posture.rs`

- [ ] **Step 1: Write failing config validation tests covering default JWT secrets, default credential master key, and explicit insecure override**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-config security_posture -- --nocapture` and confirm the new tests fail for the right reason**
- [ ] **Step 3: Implement a configuration-level security validation API that fails closed for production service startup unless explicitly overridden**
- [ ] **Step 4: Call that validation from the three service entrypoints before listener startup**
- [ ] **Step 5: Re-run `cargo test -p sdkwork-api-config security_posture -- --nocapture`**

### Task 3: Protect metrics and tighten browser CORS defaults

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/health_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/cors_preflight.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_auth.rs`

- [ ] **Step 1: Write failing tests for metrics bearer protection and configured browser origin allowlists**
- [ ] **Step 2: Run the targeted router tests and verify the new expectations fail before implementation**
- [ ] **Step 3: Implement env/config-backed metrics protection shared by gateway, admin, and portal routers**
- [ ] **Step 4: Replace `allow_origin(Any)` with explicit configured origins and safe local defaults for browser-facing routes**
- [ ] **Step 5: Re-run the targeted HTTP, admin, and portal router tests**

### Task 4: Strengthen password validation without breaking authenticated flows

**Files:**
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/admin_identity.rs`
- Modify: `crates/sdkwork-api-app-identity/tests/portal_identity.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_auth.rs`

- [ ] **Step 1: Add failing tests for weak-password rejection and strong-password acceptance**
- [ ] **Step 2: Run the targeted identity and auth tests and confirm the new weak-password tests fail first**
- [ ] **Step 3: Implement stronger password rules with clear validation errors**
- [ ] **Step 4: Update affected auth-flow tests to use strong passwords where they represent legitimate users**
- [ ] **Step 5: Re-run the targeted identity, admin, and portal auth tests**

### Task 5: Verify the phase-1 hardening slice and decide the next iteration

**Files:**
- No code changes required unless verification exposes new regressions

- [ ] **Step 1: Run `cargo test -p sdkwork-api-app-identity -- --nocapture`**
- [ ] **Step 2: Run `cargo test -p sdkwork-api-config -- --nocapture`**
- [ ] **Step 3: Run `cargo test -p sdkwork-api-interface-http health_route cors_preflight -- --nocapture`**
- [ ] **Step 4: Run `cargo test -p sdkwork-api-interface-admin admin_auth_guard -- --nocapture` and `cargo test -p sdkwork-api-interface-portal portal_auth -- --nocapture`**
- [ ] **Step 5: Use the fresh verification evidence to select phase 2 priority between runtime failover, distributed flow control, and observability uplift**
