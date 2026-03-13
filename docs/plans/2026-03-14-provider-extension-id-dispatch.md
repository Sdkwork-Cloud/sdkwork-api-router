# Provider Extension ID Dispatch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make provider execution dispatch extension-ID-driven so provider records bind to a concrete extension package instead of treating `adapter_kind` as the primary runtime identity.

**Architecture:** Keep the current OpenAI-compatible runtime behavior intact, but promote `extension_id` into the provider domain model and persistence schema. The extension host will resolve provider execution by extension ID first, while `adapter_kind` remains as a compatibility and protocol hint for older records and future translated runtimes.

**Tech Stack:** Rust, Axum, serde, sqlx, SQLite, PostgreSQL

---

### Task 1: Add failing tests for provider `extension_id`

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/tests/provider_channel_binding.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/tests/catalog_bindings.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`

**Step 1: Add a domain test**

Assert that `ProxyProvider::new(...)` derives a default `extension_id`, and that an explicit `extension_id` can override it.

**Step 2: Add a storage round-trip test**

Persist a provider with `extension_id = "sdkwork.provider.openrouter"` and verify list results return the same value.

**Step 3: Add an admin API test**

Create a provider through `/admin/providers` with an explicit `extension_id` and verify the list response returns it.

**Step 4: Add a gateway dispatch test**

Create a provider record whose `adapter_kind` is intentionally non-resolvable but whose `extension_id` points at a built-in provider extension. Verify gateway dispatch still resolves through the extension host.

**Step 5: Run the failing tests**

Run:

- `cargo test -p sdkwork-api-domain-catalog --test provider_channel_binding -q`
- `cargo test -p sdkwork-api-storage-sqlite --test catalog_bindings -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`

Expected: FAIL because `extension_id` is not yet modeled end-to-end.

### Task 2: Implement `extension_id` in the provider domain and storage

**Files:**
- Modify: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`

**Step 1: Add provider extension identity**

Add `extension_id` to `ProxyProvider` with:

- sensible default derivation from current built-in adapter kinds
- explicit override support for admin-created providers

**Step 2: Persist `extension_id` additively**

Add `extension_id` columns in SQLite and PostgreSQL migrations with compatibility defaults for existing rows.

**Step 3: Verify focused tests**

Run:

- `cargo test -p sdkwork-api-domain-catalog --test provider_channel_binding -q`
- `cargo test -p sdkwork-api-storage-sqlite --test catalog_bindings -q`
- `cargo test -p sdkwork-api-storage-postgres -q`

Expected: PASS

### Task 3: Route gateway provider execution through `extension_id`

**Files:**
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

**Step 1: Add extension-ID-first resolution**

The extension host should resolve provider execution by:

1. `extension_id`
2. legacy adapter-kind alias fallback

**Step 2: Update admin provider create/list**

Accept optional `extension_id` input and return the persisted value.

**Step 3: Update gateway dispatch helpers**

Whenever a provider record is loaded, the gateway should prefer `provider.extension_id` when resolving the runtime extension factory.

**Step 4: Verify**

Run:

- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes -q`
- `cargo test -p sdkwork-api-interface-http -q`

Expected: PASS

### Task 4: Final verification and commit

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Refresh docs**

Document that provider runtime identity is now `extension_id`-driven and `adapter_kind` is compatibility metadata.

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q`

Expected: PASS

**Step 3: Commit**

```bash
git add README.md docs/architecture/runtime-modes.md docs/plans/2026-03-14-provider-extension-id-dispatch.md crates/sdkwork-api-domain-catalog crates/sdkwork-api-app-catalog crates/sdkwork-api-extension-host crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-admin crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres
git commit -m "feat: route providers by extension identity"
```
