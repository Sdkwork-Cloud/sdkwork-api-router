# Request Tracing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add shared request ID propagation and structured HTTP request tracing to the admin and gateway services.

**Architecture:** Extend `sdkwork-api-observability` with a reusable tracing layer that resolves `x-request-id`, emits completion logs, and exposes safe subscriber initialization. Compose the middleware into both Axum interface crates and initialize tracing in the standalone service binaries.

**Tech Stack:** Rust, Axum, Tokio, tracing, tracing-subscriber

---

### Task 1: Define request tracing behavior with failing observability tests

**Files:**
- Modify: `crates/sdkwork-api-observability/Cargo.toml`
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/tests/telemetry_smoke.rs`

**Step 1: Write the failing tests**

Extend the observability tests to require:

- request ID generation when none is provided
- request ID preservation when a header is supplied
- safe repeated tracing subscriber initialization

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-observability --test telemetry_smoke -q`

Expected: FAIL because the request tracing helpers do not exist yet.

**Step 3: Write minimal implementation**

Add:

- request ID resolver and generator
- response header propagation
- shared Axum tracing middleware
- safe tracing subscriber initialization

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-observability --test telemetry_smoke -q`

Expected: PASS

### Task 2: Expose request IDs on gateway responses

**Files:**
- Modify: `crates/sdkwork-api-interface-http/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/health_route.rs`

**Step 1: Write the failing route tests**

Extend the gateway health route test to require:

- generated `x-request-id` on `/health`
- preserved `x-request-id` when the caller sends one

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test health_route -q`

Expected: FAIL because the gateway router does not yet propagate request IDs.

**Step 3: Write minimal implementation**

Wire the shared tracing middleware into the gateway router constructors.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test health_route -q`

Expected: PASS

### Task 3: Expose request IDs on admin responses

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`

**Step 1: Write the failing route tests**

Extend the admin auth guard test to require:

- generated `x-request-id` on login
- preserved `x-request-id` on an authenticated route

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: FAIL because the admin router does not yet propagate request IDs.

**Step 3: Write minimal implementation**

Wire the shared tracing middleware into the admin router constructors.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: PASS

### Task 4: Initialize tracing in standalone binaries and update docs

**Files:**
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Add startup initialization**

Initialize the shared tracing subscriber early in both binaries before binding the listener.

**Step 2: Document request tracing**

Add `x-request-id` correlation and structured HTTP tracing to operator-facing docs.

**Step 3: Run verification**

Run:

- `cargo test -p sdkwork-api-observability --test telemetry_smoke -q`
- `cargo test -p sdkwork-api-interface-http --test health_route -q`
- `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`

Expected: all commands exit `0`

### Task 5: Commit and push

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Commit**

```bash
git add docs/plans/2026-03-14-request-tracing-design.md docs/plans/2026-03-14-request-tracing-implementation.md crates/sdkwork-api-observability crates/sdkwork-api-interface-http crates/sdkwork-api-interface-admin services/gateway-service/src/main.rs services/admin-api-service/src/main.rs README.md README.zh-CN.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md Cargo.toml Cargo.lock
git commit -m "feat: add request tracing and correlation ids"
```

**Step 2: Push**

```bash
git push origin feature/bootstrap-workspace-skeleton
```
