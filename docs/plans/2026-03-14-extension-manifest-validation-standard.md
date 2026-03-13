# Extension Manifest Validation Standard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the extension manifest standard with explicit permissions and health contracts, then expose discovery-time validation results through the admin control plane.

**Architecture:** Extend `sdkwork-api-extension-core` with additive manifest metadata only, keeping existing runtime behavior stable. Put validation logic in `sdkwork-api-extension-host` so discovery and runtime-host policy remain the single source of truth. Surface the validation summary through `sdkwork-api-app-extension` DTOs and existing authenticated admin package discovery APIs.

**Tech Stack:** Rust, serde, Axum, extension host discovery, workspace tests

---

### Task 1: Add failing tests for manifest security and health metadata

**Files:**
- Modify: `crates/sdkwork-api-extension-core/tests/extension_standard.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/discovery.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- `ExtensionManifest` can carry explicit permissions and a health contract
- discovered packages produce a validation summary
- the admin package listing returns validation details for discovered packages

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-core --test extension_standard -q`
- `cargo test -p sdkwork-api-extension-host --test discovery -q`
- `cargo test -p sdkwork-api-interface-admin --test sqlite_admin_routes list_discovered_extension_packages_from_admin_api -- --exact`

Expected: FAIL because the manifest lacks explicit permission and health fields and package records do not expose validation summaries.

### Task 2: Implement manifest standard metadata and host validation

**Files:**
- Modify: `crates/sdkwork-api-extension-core/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Extend manifest metadata**

Add:

- `ExtensionPermission`
- `ExtensionHealthContract`
- builder helpers for permissions and health contract

**Step 2: Add validation report types**

Add host-side validation report structures and a manifest validation function that checks at least:

- explicit permissions declared
- at least one channel binding
- at least one capability
- connector runtime has an entrypoint
- connector runtime health contract presence

Use `error` and `warning` severities so the standard can tighten without breaking runtime compatibility.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-core --test extension_standard -q`
- `cargo test -p sdkwork-api-extension-host --test discovery -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 3: Surface validation through app and admin APIs

**Files:**
- Modify: `crates/sdkwork-api-app-extension/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/tests/sqlite_admin_routes.rs`

**Step 1: Extend discovered package DTOs**

Include:

- `distribution_name`
- `crate_name`
- `validation`

**Step 2: Keep endpoint shape additive**

Do not add a new endpoint in this batch; enrich `GET /admin/extensions/packages` so existing observability gets stronger without route sprawl.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-extension -q`
- `cargo test -p sdkwork-api-interface-admin -q`

Expected: PASS

### Task 4: Update docs and verify the workspace

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Document the standard**

Document:

- explicit extension permissions
- manifest health contract
- discovery-time validation surfaced in admin APIs

**Step 2: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo clippy --no-deps -p sdkwork-api-extension-core -p sdkwork-api-extension-host -p sdkwork-api-app-extension -p sdkwork-api-interface-admin --all-targets -- -D warnings`
- `cargo test --workspace -q -j 1`

Expected: PASS

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-extension-manifest-validation-standard.md README.md docs/architecture/runtime-modes.md crates/sdkwork-api-extension-core crates/sdkwork-api-extension-host crates/sdkwork-api-app-extension crates/sdkwork-api-interface-admin
git commit -m "feat: validate extension manifest standards"
git push
```
