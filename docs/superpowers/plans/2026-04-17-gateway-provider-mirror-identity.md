# Gateway Provider Mirror Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a derived `mirror_protocol_identity` across provider integration surfaces so future provider-specific mirror protocols can expand without breaking current execution semantics.

**Architecture:** Keep `protocol_kind` unchanged as the runtime execution field and add a separate additive identity field derived from existing provider metadata. Implement the derivation in shared provider metadata helpers, then expose it through app-catalog integration views, admin and portal responses, and stateless upstream configuration.

**Tech Stack:** Rust, axum, serde, utoipa, cargo test

---

### Task 1: Write failing identity-regression tests

**Files:**
- Modify: `crates/sdkwork-api-app-catalog/tests/create_provider.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/stateless_upstream_protocol.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/stateless_runtime.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes/providers_models_coupons.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_routing.rs`

- [ ] **Step 1: Add failing app-catalog assertions**

Require `mirror_protocol_identity` in `ProviderIntegrationView` and verify:

- OpenAI => `openai`
- OpenRouter => `openai`
- Ollama => `ollama`
- a custom plugin with a provider-specific extension ID derives the provider-specific identity

- [ ] **Step 2: Add failing stateless upstream assertions**

Require:

- existing constructors expose derived mirror identities
- a new explicit constructor preserves an override identity

- [ ] **Step 3: Add failing admin and portal assertions**

Require `integration.mirror_protocol_identity` in:

- `/admin/providers`
- `/admin/tenants/{tenant_id}/providers/readiness`
- portal routing provider options

- [ ] **Step 4: Run the focused tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-app-catalog --test create_provider -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`

Expected: FAIL because the new identity field does not exist yet.

### Task 2: Implement shared mirror identity derivation

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`

- [ ] **Step 1: Add shared derivation helpers in domain catalog**

Implement shared helper logic that derives a public mirror identity from:

- normalized `protocol_kind`
- builtin default-plugin families
- provider extension IDs
- sanitized adapter identity fallback

- [ ] **Step 2: Expose the field through `ProviderIntegrationView`**

Add `mirror_protocol_identity: String` and update `provider_integration_view(...)`.

- [ ] **Step 3: Re-run the app-catalog test**

Run: `cargo test -p sdkwork-api-app-catalog --test create_provider -- --nocapture`

Expected: PASS

### Task 3: Implement stateless upstream mirror identity support

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/stateless_gateway/config.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/stateless_upstream_protocol.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/stateless_runtime.rs`

- [ ] **Step 1: Add derived mirror identity to stateless upstream**

Keep existing constructors source-compatible and add:

- a getter for `mirror_protocol_identity`
- an explicit constructor for override cases

- [ ] **Step 2: Re-run the focused stateless tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_runtime -- --nocapture`

Expected: PASS

### Task 4: Expose the identity in admin and portal responses

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/types/catalog.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/catalog/provider.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes/providers_models_coupons.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/routing.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_routing.rs`

- [ ] **Step 1: Thread the new integration field through admin responses**

No new response wrapper is needed if `ProviderIntegrationView` remains shared and serialized additively.

- [ ] **Step 2: Verify admin provider regressions**

Run:

- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_provider_accepts_default_plugin_family_for_openrouter -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes create_provider_accepts_default_plugin_family_for_ollama -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_tenant_provider_readiness_exposes_focused_tenant_overlay_inventory -- --nocapture`

Expected: PASS

- [ ] **Step 3: Verify portal routing regression**

Run: `cargo test -p sdkwork-api-interface-portal --test portal_routing portal_routing_preferences_preview_and_logs_are_project_scoped -- --nocapture`

Expected: PASS

### Task 5: Run final verification and document follow-up boundaries

**Files:**
- Verify only

- [ ] **Step 1: Run the aggregated package checks**

Run:

- `cargo check -p sdkwork-api-app-catalog`
- `cargo check -p sdkwork-api-interface-http`
- `cargo check -p sdkwork-api-interface-admin`
- `cargo check -p sdkwork-api-interface-portal`

- [ ] **Step 2: Run the final targeted regression bundle**

Run:

- `cargo test -p sdkwork-api-app-catalog --test create_provider -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_upstream_protocol -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test stateless_runtime -- --nocapture`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -- --nocapture`
- `cargo test -p sdkwork-api-interface-portal --test portal_routing -- --nocapture`

- [ ] **Step 3: Record the next deferred slice explicitly**

After this slice, the remaining work is still:

- formalize one concrete provider-specific public mirror protocol
- publish its official path/auth/body contract
- add gateway runtime routes only after the metadata layer is stable
