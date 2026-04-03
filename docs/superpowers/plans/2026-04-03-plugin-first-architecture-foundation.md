# Plugin-First Architecture Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** establish a formal plugin-first architecture foundation for SDKWork by standardizing pluggable seams, introducing infrastructure driver contracts, and preparing the runtime for generalized component plugins.

**Architecture:** start with the lowest-risk, highest-leverage seams that already exist in the codebase: storage, cache, secret handling, provider runtimes, and policy attachment points. Preserve the current runtime and domain split, but make extension standards explicit through new core contracts and registries instead of one-off conditionals.

**Tech Stack:** Rust, Axum, existing `sdkwork-api-*` crates, sqlx, serde, hot-reload runtime, cargo tests

---

### Task 1: Document the seam inventory in the product architecture

**Files:**
- Modify: `docs/architecture/software-architecture.md`
- Create: `docs/architecture/plugin-first-architecture.md`
- Modify: `docs/index.md`

- [ ] **Step 1: Write the failing doc-oriented verification**

Prepare a grep check that expects the new plugin-first architecture page and references from the main architecture doc.

- [ ] **Step 2: Run verification to confirm the gap exists**

Run: `rg -n "plugin-first-architecture|StorageDriverFactory|CacheStore" docs/architecture docs/index.md`
Expected: no authoritative plugin-first architecture page yet.

- [ ] **Step 3: Write the minimal documentation**

Add:

- a plugin-first architecture page
- a seam inventory in the software architecture page
- docs index references

- [ ] **Step 4: Re-run verification**

Run: `rg -n "plugin-first-architecture|StorageDriverFactory|CacheStore" docs/architecture docs/index.md`
Expected: PASS with all new references present.

- [ ] **Step 5: Commit**

```bash
git add docs/architecture/software-architecture.md docs/architecture/plugin-first-architecture.md docs/index.md
git commit -m "docs: define plugin-first architecture standard"
```

### Task 2: Add cache-core contracts

**Files:**
- Create: `crates/sdkwork-api-cache-core/Cargo.toml`
- Create: `crates/sdkwork-api-cache-core/src/lib.rs`
- Create: `crates/sdkwork-api-cache-core/tests/cache_contract.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write the failing test**

Define tests for a minimal cache contract:

- set with TTL
- get
- delete
- tagged invalidation

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-cache-core -- --nocapture`
Expected: FAIL because the crate and trait do not exist.

- [ ] **Step 3: Write minimal implementation**

Create the contract crate with:

- `CacheStore`
- `CacheEntry`
- `CacheTag`
- `DistributedLockStore`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-cache-core -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-cache-core
git commit -m "feat: add cache core contracts"
```

### Task 3: Add memory cache driver

**Files:**
- Create: `crates/sdkwork-api-cache-memory/Cargo.toml`
- Create: `crates/sdkwork-api-cache-memory/src/lib.rs`
- Create: `crates/sdkwork-api-cache-memory/tests/memory_cache.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write the failing test**

Add tests proving the memory cache satisfies the cache-core contract, including TTL expiry and tagged invalidation.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-cache-memory -- --nocapture`
Expected: FAIL because the driver crate does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement an in-memory cache driver suitable for embedded, test, and single-node mode.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-cache-memory -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-cache-memory
git commit -m "feat: add memory cache driver"
```

### Task 4: Introduce storage driver factory contracts

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Create: `crates/sdkwork-api-storage-core/tests/driver_registry.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests that expect:

- storage dialect selection through a registry or factory
- unsupported configured dialects to fail with a clear error

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-storage-core driver_registry -- --nocapture`
Expected: FAIL because storage selection is still hard-coded.

- [ ] **Step 3: Write minimal implementation**

Add:

- `StorageDriverFactory`
- a driver registry shape
- runtime selection through the registry

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-storage-core driver_registry -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-storage-core/src/lib.rs crates/sdkwork-api-storage-core/tests/driver_registry.rs crates/sdkwork-api-app-runtime/src/lib.rs
git commit -m "feat: add storage driver factory"
```

### Task 5: Split `AdminStore` into capability facets without breaking callers

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Test: backend trait tests under both storage crates

- [ ] **Step 1: Write the failing test**

Add tests that assert a backend can satisfy smaller facet contracts and still expose the compatibility facade.

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p sdkwork-api-storage-sqlite admin_store_trait -- --nocapture
cargo test -p sdkwork-api-storage-postgres admin_store_trait -- --nocapture
```

Expected: FAIL once the tests reference the new facet traits before implementation exists.

- [ ] **Step 3: Write minimal implementation**

Add capability facets such as:

- `IdentityStore`
- `TenantStore`
- `CatalogStore`
- `CredentialStore`
- `RoutingStore`
- `UsageStore`
- `BillingStore`
- `ExtensionStore`

Keep `AdminStore` as a compatibility supertrait during migration.

- [ ] **Step 4: Run test to verify it passes**

Run the same targeted trait suites.

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite crates/sdkwork-api-storage-postgres
git commit -m "refactor: split admin store into pluggable facets"
```

### Task 6: Introduce cache backend configuration

**Files:**
- Modify: `crates/sdkwork-api-config/src/lib.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `docs/operations/configuration.md`

- [ ] **Step 1: Write the failing test**

Add config tests for:

- `cache_backend: memory`
- `cache_backend: redis`
- optional `cache_url`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-config config_loading -- --nocapture`
Expected: FAIL because cache backend config does not exist.

- [ ] **Step 3: Write minimal implementation**

Add cache backend config parsing plus docs.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-config config_loading -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-config/src/lib.rs crates/sdkwork-api-config/tests/config_loading.rs docs/operations/configuration.md
git commit -m "feat: add cache backend config"
```

### Task 7: Formalize plugin capability manifests for runtime packages

**Files:**
- Modify: `crates/sdkwork-api-extension-core/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/tests/*`
- Modify: `docs/architecture/runtime-modes.md`

- [ ] **Step 1: Write the failing test**

Add tests that require manifests to expose:

- operation declarations
- modality support
- compatibility version
- config schema version

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-extension-host -- --nocapture`
Expected: FAIL because manifests are not yet normalized to that contract.

- [ ] **Step 3: Write minimal implementation**

Extend runtime manifests and validation without breaking existing built-in packages.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-extension-host -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-extension-core crates/sdkwork-api-extension-host docs/architecture/runtime-modes.md
git commit -m "feat: formalize plugin capability manifests"
```

### Task 8: Record next-phase hooks for API key groups, policy plugins, and billing 2.0

**Files:**
- Modify: `docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md`
- Modify: `docs/superpowers/specs/2026-04-03-plugin-first-architecture-design.md`

- [ ] **Step 1: Update the API key group spec**

Position API key groups as policy attachment points in the plugin-first architecture rather than a standalone feature.

- [ ] **Step 2: Update the master architecture spec**

List:

- API key groups
- routing profiles
- billing strategy plugins

as the next execution tier after infrastructure pluggability.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-04-03-api-key-group-routing-billing-design.md docs/superpowers/specs/2026-04-03-plugin-first-architecture-design.md
git commit -m "docs: align phase-two policy work with plugin-first architecture"
```
