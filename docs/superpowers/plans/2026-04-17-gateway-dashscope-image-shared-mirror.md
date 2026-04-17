# Gateway DashScope Image Shared Mirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish `images.aliyun` on the official shared DashScope image paths and make shared async task routing resolve the exact owning provider.

**Architecture:** Replace the Kling-only image mirror handler with a shared DashScope image transport handler. Create requests choose a provider from an allowed mirror-identity set using the request model, and task polling resolves the exact provider from persisted billing events by `reference_id`.

**Tech Stack:** Rust, axum, utoipa, reqwest, existing gateway routing and billing stores.

---

### Task 1: Write the failing contract tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/images_route.rs`

- [ ] **Step 1: Add failing OpenAPI assertions**

Assert:
- `images.aliyun` tag exists
- shared DashScope image paths exist
- shared DashScope operations are tagged with both `images.kling` and `images.aliyun`
- wrapper paths for Kling and Aliyun remain absent

- [ ] **Step 2: Add failing route tests**

Cover:
- stateless shared DashScope image paths relay for `aliyun`
- stateful create routes to Aliyun based on model/provider selection
- stateful task polling resolves the owner provider from billing events

- [ ] **Step 3: Run focused tests and verify red**

Run:
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test images_route -- --nocapture`

Expected:
- failures because shared DashScope routing and `images.aliyun` publication are not implemented yet

### Task 2: Add gateway routing helpers

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/gateway_routing.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`

- [ ] **Step 1: Add multi-identity planned execution helper**

Support selecting a planned execution context from an allowed set such as `["kling", "aliyun"]`.

- [ ] **Step 2: Add provider-id-specific planned execution helper**

Support reconstructing a planned execution context from an exact `provider_id` so async task polling can route to the recorded owner provider.

- [ ] **Step 3: Run focused gateway checks**

Run:
- `cargo check -p sdkwork-api-app-gateway`

Expected:
- helper APIs compile and are available to the HTTP layer

### Task 3: Refactor the shared DashScope image handlers

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/inference_handlers/image_dashscope.rs`
- Create: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/image_dashscope.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/mod.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateful_route_groups/inference_and_storage.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_stateless_route_groups/inference_and_storage.rs`
- Delete: `crates/sdkwork-api-interface-http/src/inference_handlers/image_kling.rs`
- Delete: `crates/sdkwork-api-interface-http/src/inference_stateless_handlers/image_kling.rs`

- [ ] **Step 1: Implement shared stateless DashScope relay**

Allow only `kling` or `aliyun` stateless upstream identities and forward the official request unchanged.

- [ ] **Step 2: Implement shared stateful create routing**

Parse the request body, extract `model`, select a provider from allowed mirror identities, relay the official request, and persist lightweight ownership evidence keyed by `task_id`.

- [ ] **Step 3: Implement shared stateful task polling**

Resolve the exact owning provider from billing events by `task_id`, reconstruct the planned execution context, and relay to that provider.

- [ ] **Step 4: Run image tests to green**

Run:
- `cargo test -p sdkwork-api-interface-http --test images_route -- --nocapture`

Expected:
- shared DashScope image routing passes for Kling and Aliyun

### Task 4: Publish the OpenAPI and docs

**Files:**
- Create: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_images_dashscope.rs`
- Modify: `crates/sdkwork-api-interface-http/src/gateway_openapi.rs`
- Delete: `crates/sdkwork-api-interface-http/src/gateway_openapi_paths_images_kling.rs`
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/api-reference/gateway-api.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Publish shared DashScope operations**

Expose the two official operations with shared operation ids and dual tags:
- `images.kling`
- `images.aliyun`

- [ ] **Step 2: Update docs**

Describe the official shared DashScope image transport as active for both providers and remove `images.aliyun` from reserved-only wording.

- [ ] **Step 3: Run final verification**

Run:
- `cargo fmt -p sdkwork-api-interface-http -p sdkwork-api-app-gateway`
- `cargo test -p sdkwork-api-interface-http --test openapi_route openapi_routes_expose_gateway_api_inventory -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test images_route -- --nocapture`
- `cargo check -p sdkwork-api-interface-http -p sdkwork-api-app-gateway`

Expected:
- formatting clean
- OpenAPI/image tests pass
- both crates compile
