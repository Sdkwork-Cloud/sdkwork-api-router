# Restartless Store And JWT Reconfiguration Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let standalone services hot-swap database store dependencies and admin or portal JWT signing secrets from config-file changes without restarting the process.

**Architecture:** Add a small reloadable-value handle in `sdkwork-api-storage-core`, then teach gateway, admin, and portal router states to snapshot live dependencies per request. Extend standalone runtime supervision so config reload can build a replacement `AdminStore`, swap the live store handle on success, rotate admin or portal JWT handles, and bring `portal-api-service` under the same config supervision model.

**Tech Stack:** Rust, Axum, Tokio, sqlx-backed store crates, existing standalone config loader and runtime supervision

---

## Chunk 1: Failing Live-Handle And Supervision Tests

### Task 1: Add focused RED coverage for live store and JWT replacement

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/models_route.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/admin_auth_guard.rs`
- Modify: `crates/sdkwork-api-interface-portal/tests/portal_auth.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add a failing gateway live-store test**

Write a test that builds a gateway router from a live store handle, swaps the underlying store, and expects a new request to observe the replacement store contents.

- [ ] **Step 2: Add a failing admin live-JWT test**

Write a test that rotates the live admin JWT secret, expects an old token to fail, and expects a newly minted token to succeed.

- [ ] **Step 3: Add a failing portal live-JWT test**

Write the portal equivalent of the admin JWT rotation test.

- [ ] **Step 4: Add a failing config-supervision test**

Write a runtime-supervision test that:

1. starts with one database file and one JWT secret
2. starts standalone supervision with live handles
3. rewrites config to a second database file and a second JWT secret
4. expects the live handles to update without restarting the process

- [ ] **Step 5: Run the focused RED tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-http --test models_route gateway_router_uses_replaced_live_store_for_new_requests -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-admin --test admin_auth_guard admin_routes_apply_rotated_live_jwt_secret_to_new_requests -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-interface-portal --test portal_auth portal_routes_apply_rotated_live_jwt_secret_to_new_requests -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_reloads_store_and_jwt_after_config_file_change -q`

Expected: FAIL because router states are static snapshots and runtime supervision cannot yet replace stores or JWT secrets.

## Chunk 2: Reloadable Handles And Router State Snapshots

### Task 2: Implement request-scoped live dependency snapshots

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-portal/src/lib.rs`

- [ ] **Step 1: Add a generic reloadable value handle**

Expose a small cloneable handle that can snapshot and replace a current value.

- [ ] **Step 2: Teach gateway state to snapshot the current store per request**

Add live-store-aware constructors and keep existing fixed constructors working.

- [ ] **Step 3: Teach admin state to snapshot the current store and JWT secret per request**

Keep the secret manager fixed for this batch.

- [ ] **Step 4: Teach portal state to snapshot the current store and JWT secret per request**

Add the same live-handle support used by admin.

- [ ] **Step 5: Re-run the interface-focused tests**

Run the first three commands from Chunk 1.

Expected: PASS.

## Chunk 3: Config-Driven Store And JWT Replacement

### Task 3: Extend standalone supervision and service wiring

**Files:**
- Modify: `crates/sdkwork-api-app-runtime/Cargo.toml`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`

- [ ] **Step 1: Add store-construction helpers in app runtime**

Build `Arc<dyn AdminStore>` instances from `StandaloneConfig` for sqlite and postgres.

- [ ] **Step 2: Extend standalone supervision to manage live store and JWT handles**

Handle gateway, admin, and portal service modes, replace the store on successful `database_url` changes, rotate admin or portal JWT handles on secret changes, and keep listener changes restart-bound.

- [ ] **Step 3: Bring portal under config-loader supervision**

Switch portal startup from one-shot `StandaloneConfig::from_env()` to `StandaloneConfigLoader::from_env()` plus the shared supervision task.

- [ ] **Step 4: Re-run the app-runtime RED test to reach GREEN**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_reloads_store_and_jwt_after_config_file_change -q`

Expected: PASS.

## Chunk 4: Docs And Full Verification

### Task 4: Update runtime docs and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-restartless-store-and-jwt-reconfiguration-design.md`
- Create: `docs/plans/2026-03-15-restartless-store-and-jwt-reconfiguration-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-targeted-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-request-draining-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-drain-timeout-rollback-design.md`

- [ ] **Step 1: Update the documented reload contract**

Move `database_url`, `admin_jwt_signing_secret`, and `portal_jwt_signing_secret` into the live-reloadable set for supervised standalone services. Keep listener and secret-manager settings restart-bound.

- [ ] **Step 2: Document the discovered secret-manager safety constraint**

Call out that runtime secret-manager reconfiguration still needs credential metadata plus migration support.

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
