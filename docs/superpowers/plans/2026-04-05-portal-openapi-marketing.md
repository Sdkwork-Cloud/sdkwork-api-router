# Portal OpenAPI Marketing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose working `/portal/openapi.json` and `/portal/docs` endpoints and include the new portal marketing coupon routes in the published contract.

**Architecture:** Keep the implementation narrow and compatible with current `HEAD` by adding a lightweight portal docs router inside `sdkwork-api-interface-portal`. Publish a stable OpenAPI 3.1 document that reflects the current portal router surface, then verify the existing tests plus the new marketing assertions against the live router.

**Tech Stack:** Rust, Axum, existing portal router, portal integration tests, PowerShell-safe file editing for the large portal library.

---

### Task 1: Extend the failing OpenAPI regression test

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/tests/openapi_route.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/openapi_route.rs`

- [ ] **Step 1: Add marketing coupon route assertions**

Add assertions for:
- `POST /portal/marketing/coupon-validations`
- `POST /portal/marketing/coupon-reservations`
- `POST /portal/marketing/coupon-redemptions/confirm`
- `POST /portal/marketing/coupon-redemptions/rollback`
- `GET /portal/marketing/my-coupons`
- `GET /portal/marketing/reward-history`
- `GET /portal/marketing/redemptions`
- `GET /portal/marketing/codes`

- [ ] **Step 2: Add schema component assertions**

Add assertions for the request and response schemas needed by the new marketing routes:
- `PortalCouponValidationRequest`
- `PortalCouponValidationResponse`
- `PortalCouponReservationRequest`
- `PortalCouponReservationResponse`
- `PortalCouponRedemptionConfirmRequest`
- `PortalCouponRedemptionConfirmResponse`
- `PortalCouponRedemptionRollbackRequest`
- `PortalCouponRedemptionRollbackResponse`
- `PortalMarketingCodesResponse`
- `PortalMarketingRedemptionsResponse`

- [ ] **Step 3: Run the focused test to verify RED**

Run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\admin\.codex\memories\cargo-target-sdkwork'; cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture
```

Expected: fail because `/portal/openapi.json` and `/portal/docs` are missing or do not publish the required portal marketing paths and schema components.

### Task 2: Implement portal docs routes and OpenAPI payload

**Files:**
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: Add a small docs router**

Mount:
- `GET /portal/openapi.json`
- `GET /portal/docs`

into both `portal_router()` and `portal_router_with_state(...)`.

- [ ] **Step 2: Publish the OpenAPI 3.1 payload**

Return a stable OpenAPI document with:
- portal health, auth, and workspace routes already expected by the existing test
- the new portal marketing coupon routes
- `bearerAuth` security scheme
- schema components referenced by the tested routes

- [ ] **Step 3: Publish the docs landing page**

Return HTML that identifies the portal API and links to `/portal/openapi.json`.

### Task 3: Verify GREEN on the focused scope

**Files:**
- Test: `crates/sdkwork-api-interface-portal/tests/openapi_route.rs`
- Test: `crates/sdkwork-api-interface-portal/tests/marketing_coupon_routes.rs`

- [ ] **Step 1: Re-run the focused OpenAPI test**

Run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\admin\.codex\memories\cargo-target-sdkwork'; cargo test -p sdkwork-api-interface-portal --test openapi_route -- --nocapture
```

Expected: PASS.

- [ ] **Step 2: Re-run the portal marketing route test**

Run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\admin\.codex\memories\cargo-target-sdkwork'; cargo test -p sdkwork-api-interface-portal --test marketing_coupon_routes -- --nocapture
```

Expected: PASS to prove the docs work did not regress the marketing flow.

### Task 4: Run broader portal verification

**Files:**
- Test: `crates/sdkwork-api-interface-portal`

- [ ] **Step 1: Run the portal crate test slice**

Run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\admin\.codex\memories\cargo-target-sdkwork'; cargo test -p sdkwork-api-interface-portal -- --nocapture
```

Expected: PASS, or isolate any unrelated pre-existing failure with exact evidence.
