# Gateway Public OpenAPI Schema Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the public gateway OpenAPI document from route-inventory HTML/JSON generation to `utoipa` schema-driven OpenAPI 3.1 generation for developer-facing endpoints.

**Architecture:** Keep the existing gateway router and handlers intact, but replace the old route-inventory OpenAPI builder with a dedicated `utoipa` document layer. Public developer endpoints will be described through a focused `openapi_paths` module backed by real request and response DTOs from `sdkwork-api-contract-openai`, so `/openapi.json` and `/docs` always reflect current Rust types instead of inferred path strings.

**Tech Stack:** Rust, axum, utoipa, utoipa-axum, utoipa-swagger-ui, serde, existing `sdkwork-api-contract-openai` DTOs

---

### Task 1: Lock The Gateway Regression Surface

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/openapi_route.rs`

- [ ] **Step 1: Write the failing test**

Add assertions for:
- `components.schemas` exists
- `CreateChatCompletionRequest`
- `ChatCompletionResponse`
- `CreateResponseRequest`
- `ResponseObject`
- `CreateEmbeddingRequest`
- `CreateEmbeddingResponse`
- `ListModelsResponse`
- request/response `$ref` values for `/v1/chat/completions`, `/v1/responses`, `/v1/embeddings`, `/v1/models`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`
Expected: FAIL because the current gateway OpenAPI output has no schema components or typed `$ref` mappings.

### Task 2: Make Public OpenAI Contract DTOs Schema-Aware

**Files:**
- Modify: `crates/sdkwork-api-contract-openai/Cargo.toml`
- Modify: `crates/sdkwork-api-contract-openai/src/models.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/chat_completions.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/responses.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/embeddings.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/errors.rs`

- [ ] **Step 1: Add `utoipa` dependency**

Use the workspace version and keep the contract crate lightweight.

- [ ] **Step 2: Add `ToSchema` derives to public request and response DTOs**

Cover the types used by the gateway public OpenAPI surface:
- `ModelObject`
- `ListModelsResponse`
- `CreateChatCompletionRequest`
- `ChatMessageInput`
- `ChatCompletionMessage`
- `ChatCompletionChoice`
- `ChatCompletionResponse`
- `CreateResponseRequest`
- `CountResponseInputTokensRequest`
- `CompactResponseRequest`
- `ResponseObject`
- `ResponseOutputItem`
- `ResponseInputTokensObject`
- `CreateEmbeddingRequest`
- `EmbeddingObject`
- `CreateEmbeddingResponse`
- `OpenAiErrorEnvelope`
- `OpenAiErrorResponse`

- [ ] **Step 3: Run a focused compile check**

Run: `cargo test -p sdkwork-api-interface-http openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`
Expected: still FAIL at the OpenAPI assertions, but compile cleanly with the new schema derives available.

### Task 3: Replace Gateway Route Inventory OpenAPI Generation

**Files:**
- Modify: `crates/sdkwork-api-interface-http/Cargo.toml`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

- [ ] **Step 1: Add `utoipa`, `utoipa-axum`, and `utoipa-swagger-ui` dependencies**

Use workspace versions already adopted by the portal migration.

- [ ] **Step 2: Remove the old manual OpenAPI builder path**

Delete or stop using:
- `sdkwork_api_openapi` imports
- `GATEWAY_OPENAPI_SPEC`
- route inventory extraction
- `gateway_openapi_document()`
- `gateway_docs_html()`

- [ ] **Step 3: Introduce a dedicated `openapi_paths` module**

Describe these developer-facing endpoints with `#[utoipa::path]` stubs:
- `GET /health`
- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/responses`
- `POST /v1/embeddings`
- `POST /v1/messages`
- `POST /v1/messages/count_tokens`
- `POST /v1beta/models/{tail}`

Use real DTOs where possible and `serde_json::Value` only for compatibility wrappers that currently do not have first-class typed contracts.

- [ ] **Step 4: Build the gateway OpenAPI document through `OpenApiRouter`**

Expose:
- `/openapi.json` as live generated OpenAPI JSON
- `/docs` as a stable no-redirect HTML entry
- `/docs/ui/` as the embedded Swagger UI shell when needed

- [ ] **Step 5: Preserve security semantics**

Public docs must keep bearer auth on gateway execution routes while leaving `/health`, `/openapi.json`, and `/docs` unauthenticated.

- [ ] **Step 6: Run the focused regression test**

Run: `cargo test -p sdkwork-api-interface-http openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`
Expected: PASS with schema components and `$ref` assertions.

### Task 4: Reconcile The Public Portal API Reference Surface

**Files:**
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-reference/src/index.tsx`
- Modify: `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts` (only if new copy is introduced)

- [ ] **Step 1: Ensure gateway entries match the real generated routes**

Keep the page aligned with the now-live:
- `/openapi.json`
- `/docs`

- [ ] **Step 2: Keep portal entries aligned with live generated routes**

Keep the page aligned with:
- `/api/portal/openapi.json`
- `/api/portal/docs`

- [ ] **Step 3: Re-run the public API Reference tests**

Run:
- `node apps/sdkwork-router-portal/tests/portal-api-reference-center.test.mjs`
- `node apps/sdkwork-router-portal/tests/portal-public-site-i18n-detail.test.mjs`

Expected: PASS.

### Task 5: Final Verification

**Files:**
- No additional files required unless verification reveals drift

- [ ] **Step 1: Run backend verification**

Run:
- `cargo test -p sdkwork-api-interface-http openapi_routes_expose_gateway_api_inventory -- --exact --nocapture`
- `cargo test -p sdkwork-api-interface-portal openapi_routes_expose_portal_api_inventory_with_schema_components -- --exact --nocapture`

- [ ] **Step 2: Run frontend verification**

Run:
- `pnpm -C apps/sdkwork-router-portal typecheck`
- `pnpm -C apps/sdkwork-router-portal build`
- `node apps/sdkwork-router-portal/tests/portal-api-reference-center.test.mjs`
- `node apps/sdkwork-router-portal/tests/portal-public-site-cta-polish.test.mjs`
- `node apps/sdkwork-router-portal/tests/portal-public-site-i18n-detail.test.mjs`

- [ ] **Step 3: Record any remaining scope**

If admin is still on route-inventory OpenAPI after this gateway migration, document that it remains an internal follow-up instead of implying full convergence.
