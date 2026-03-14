# Observability Metrics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a real shared HTTP metrics registry and expose Prometheus-compatible `/metrics` endpoints on both admin and gateway routers.

**Architecture:** Build the metrics registry and middleware in `sdkwork-api-observability`, then compose it into the interface crates so both stateless and stateful routers share the same instrumentation pattern. Keep the first version bounded to request count and duration summaries keyed by service, method, matched route, and status.

**Tech Stack:** Rust, Axum, Tower, serde-free Prometheus text formatting

---

### Task 1: Define observability registry behavior with failing tests

**Files:**
- Modify: `crates/sdkwork-api-observability/src/lib.rs`
- Modify: `crates/sdkwork-api-observability/tests/telemetry_smoke.rs`

**Step 1: Write the failing test**

Extend the observability crate test to require:

- a registry bound to a service name
- request recording
- Prometheus text output containing `sdkwork_service_info`
- request counters and duration summaries

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-observability --test telemetry_smoke -q`

Expected: FAIL because the registry and rendering functions do not exist yet.

**Step 3: Write minimal implementation**

Add:

- `HttpMetricsRegistry`
- request recording API
- Prometheus text rendering
- shared Axum middleware function

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-observability --test telemetry_smoke -q`

Expected: PASS

### Task 2: Expose gateway metrics through `/metrics`

**Files:**
- Modify: `crates/sdkwork-api-interface-http/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/health_route.rs`

**Step 1: Write the failing route test**

Add a gateway route test that:

- hits `/health`
- hits `/metrics`
- asserts the metrics output includes gateway request counters for `/health`

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test health_route -q`

Expected: FAIL because `/metrics` is not exposed yet.

**Step 3: Write minimal implementation**

Wire the shared registry and middleware into both gateway router constructors and add the `/metrics` route.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test health_route -q`

Expected: PASS

### Task 3: Expose admin metrics through `/metrics`

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`

**Step 1: Write the failing route test**

Add an admin route test that:

- performs login
- performs an authenticated request
- reads `/metrics`
- asserts the metrics output includes request counters for login or projects

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: FAIL because `/metrics` is not exposed yet.

**Step 3: Write minimal implementation**

Wire the shared registry and middleware into both admin router constructors and add the `/metrics` route.

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`

Expected: PASS

### Task 4: Update docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Document `/metrics`**

Add the new metrics capability to the runtime and compatibility docs.

**Step 2: Run verification**

Run:

- `cargo test -p sdkwork-api-observability --test telemetry_smoke -q`
- `cargo test -p sdkwork-api-interface-http --test health_route -q`
- `cargo test -p sdkwork-api-interface-admin --test admin_auth_guard -q`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`

Expected: all commands exit `0`

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-observability-metrics-design.md docs/plans/2026-03-14-observability-metrics-implementation.md crates/sdkwork-api-observability crates/sdkwork-api-interface-http crates/sdkwork-api-interface-admin README.md README.zh-CN.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md
git commit -m "feat: add shared metrics endpoints"
```
