# Migration-Safe Secret-Manager Reconfiguration Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let gateway and admin services hot-swap secret-manager configuration without restart while preserving the ability to resolve credentials written under previous backend locations or master keys.

**Architecture:** Persist backend locator and master-key identity metadata in credential records, extend `CredentialSecretManager` to read historical records through per-record metadata plus legacy decrypt keys, and move gateway/admin secret managers onto reloadable request-scoped handles. Runtime supervision validates a candidate manager against existing credentials before swapping it live.

**Tech Stack:** Rust, Axum, Tokio, sqlx-backed admin stores, existing secret-core/local/keyring crates, standalone config loader and runtime supervision

---

## Chunk 1: RED Metadata And Reload Tests

### Task 1: Add failing coverage for metadata-complete secret-manager reload

**Files:**
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `crates/sdkwork-api-app-credential/tests/save_credential.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add a failing config parsing test**

Write a test proving `credential_legacy_master_keys` parse from env pairs and file-backed reload input.

- [ ] **Step 2: Add a failing app-credential historical-read test**

Write a test that:

1. saves a credential under one local secret file or one master key
2. builds a new manager with different defaults plus the legacy key
3. expects the new manager to still resolve the original secret

- [ ] **Step 3: Add a failing runtime-supervision reload test**

Write a test that starts standalone supervision with a live secret-manager handle, rewrites secret-manager config, and expects the new live manager to resolve a credential written before the change.

- [ ] **Step 4: Run the focused RED tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-config --test config_loading parses_credential_legacy_master_keys_from_pairs_and_reload_inputs -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-credential --test save_credential resolves_historical_credentials_after_secret_manager_reconfiguration -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_reloads_secret_manager_after_config_file_change -q`

Expected: FAIL because credential records do not yet persist locator or key metadata, config has no legacy-key field, and runtime supervision cannot swap secret-manager handles.

## Chunk 2: Credential Metadata And Multi-Key Resolution

### Task 2: Persist complete credential metadata and resolve through it

**Files:**
- Modify: `crates/sdkwork-api-domain-credential/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Modify: `crates/sdkwork-api-secret-core/src/lib.rs`
- Modify: `crates/sdkwork-api-app-credential/src/lib.rs`
- Modify: `crates/sdkwork-api-app-credential/tests/save_credential.rs`

- [ ] **Step 1: Extend the credential domain model**

Add optional backend locator and master-key identity fields while keeping older constructors usable.

- [ ] **Step 2: Extend storage schema and query mappings**

Add the new credential metadata columns, write them on insert or update, and return them from credential lookup methods.

- [ ] **Step 3: Add master-key identity helpers in secret-core**

Expose a stable derived key identifier for a raw master key.

- [ ] **Step 4: Teach `CredentialSecretManager` to read historical records**

Use record metadata to choose file path, keyring service, and decrypt key; preserve backward-compatible fallback for records created before the new fields exist.

- [ ] **Step 5: Re-run the app-credential RED test**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-credential --test save_credential resolves_historical_credentials_after_secret_manager_reconfiguration -q`

Expected: PASS.

## Chunk 3: Config And Live Secret-Manager Handles

### Task 3: Make secret-manager settings live-reloadable

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `crates/sdkwork-api-app-runtime/tests/standalone_runtime_supervision.rs`

- [ ] **Step 1: Add config support for `credential_legacy_master_keys`**

Parse from env and config files, include it in resolved env export, and mark it as part of secret-manager live reload.

- [ ] **Step 2: Move gateway and admin secret managers onto reloadable request snapshots**

Mirror the store and JWT pattern so a request sees one consistent manager instance.

- [ ] **Step 3: Extend runtime supervision with secret-manager reload handles**

Build the candidate next manager, validate it against existing credentials in the active store, and hot-swap the live handle on success.

- [ ] **Step 4: Wire gateway and admin services to pass live secret-manager handles**

Keep portal unchanged.

- [ ] **Step 5: Re-run the focused config and runtime tests**

Run:

- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-config --test config_loading parses_credential_legacy_master_keys_from_pairs_and_reload_inputs -q`
- `source "$HOME/.cargo/env" && cargo test -p sdkwork-api-app-runtime --test standalone_runtime_supervision standalone_runtime_supervision_reloads_secret_manager_after_config_file_change -q`

Expected: PASS.

## Chunk 4: Docs And Full Verification

### Task 4: Update docs and verify the workspace

**Files:**
- Create: `docs/plans/2026-03-15-migration-safe-secret-manager-reconfiguration-design.md`
- Create: `docs/plans/2026-03-15-migration-safe-secret-manager-reconfiguration-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/plans/2026-03-15-runtime-config-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-restartless-store-and-jwt-reconfiguration-design.md`
- Modify: `docs/plans/2026-03-15-restartless-listener-rebinding-design.md`
- Modify: `docs/plans/2026-03-15-targeted-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-extension-runtime-reload-design.md`
- Modify: `docs/plans/2026-03-15-automatic-extension-hot-reload-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-request-draining-design.md`
- Modify: `docs/plans/2026-03-15-native-dynamic-drain-timeout-rollback-design.md`

- [ ] **Step 1: Mark secret-manager reconfiguration implemented**

Move secret-manager settings into the live-reloadable set for supervised gateway and admin services, with the legacy-key requirement called out explicitly.

- [ ] **Step 2: Reduce remaining-gap lists to multi-node rollout**

Keep any mention of explicit credential cleanup tooling as an optional future improvement rather than a correctness gap.

- [ ] **Step 3: Run full verification**

Run:

- `source "$HOME/.cargo/env" && cargo fmt --all --check`
- `source "$HOME/.cargo/env" && cargo clippy --workspace --all-targets -- -D warnings`
- `source "$HOME/.cargo/env" && cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS.
