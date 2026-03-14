# Native Dynamic Extension Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn `native_dynamic` extensions into a real loadable provider runtime with a stable ABI, host-side library loading, and gateway dispatch.

**Architecture:** Add a dedicated ABI crate with a narrow C ABI over JSON envelopes, then teach `sdkwork-api-extension-host` and `sdkwork-api-app-gateway` to load trusted native dynamic provider libraries and execute JSON provider operations through them. Keep stream support and hot reload out of scope so the runtime boundary stays small and verifiable.

**Tech Stack:** Rust, `libloading`, Axum, serde, `cdylib` fixture plugin, workspace tests

---

### Task 1: Add failing tests for native dynamic ABI and loader execution

**Files:**
- Create: `crates/sdkwork-api-extension-abi/tests/abi_contract.rs`
- Create: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- Create: `crates/sdkwork-api-ext-provider-native-mock/Cargo.toml`
- Create: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- the ABI crate serializes invocation and result envelopes
- the host can load a trusted native dynamic plugin manifest from a compiled library
- the host can resolve a provider adapter for a native dynamic plugin
- the gateway can execute a non-stream chat completion through a native dynamic plugin

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-abi --test abi_contract -q`
- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`
- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch native_dynamic_extension_can_relay_through_loaded_library -- --exact`

Expected: FAIL because no ABI crate, no loader, and no native dynamic provider execution path exist.

### Task 2: Implement the ABI crate and fixture plugin

**Files:**
- Create: `crates/sdkwork-api-extension-abi/Cargo.toml`
- Create: `crates/sdkwork-api-extension-abi/src/lib.rs`
- Create: `crates/sdkwork-api-extension-abi/tests/abi_contract.rs`
- Create: `crates/sdkwork-api-ext-provider-native-mock/Cargo.toml`
- Create: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`
- Modify: `Cargo.toml`

**Step 1: Add ABI types and exported symbol constants**

Define:

- ABI version constant
- exported symbol names
- invocation envelope
- result envelope
- CString helpers

**Step 2: Add a real fixture plugin**

Create a `cdylib` fixture that exports:

- ABI version
- manifest JSON
- execute function
- free-string function

Make it return a deterministic JSON chat completion for test coverage.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-abi --test abi_contract -q`

Expected: PASS

### Task 3: Implement native dynamic loading in the extension host

**Files:**
- Modify: `crates/sdkwork-api-extension-host/Cargo.toml`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`
- Create: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`

**Step 1: Add host loader and adapter**

Implement:

- library loading through `libloading`
- ABI version validation
- manifest loading from library exports
- manifest match verification against package manifest
- provider adapter backed by the plugin execute function

**Step 2: Register discovered native dynamic providers**

When a discovered provider manifest uses `native_dynamic`, register a provider factory that loads the declared library and executes through the ABI.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 4: Wire gateway dispatch and runtime policy

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/tests/extension_dispatch.rs`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `README.md`

**Step 1: Use native dynamic factories in runtime dispatch**

Ensure discovered native dynamic providers become executable through the same `extension_id` dispatch path used by builtin and connector providers.

**Step 2: Keep policy conservative**

If loading fails, do not crash runtime assembly; skip provider registration so the gateway falls back cleanly.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-app-gateway --test extension_dispatch native_dynamic_extension_can_relay_through_loaded_library -- --exact`
- `cargo test -p sdkwork-api-app-gateway -q`

Expected: PASS

### Task 5: Run verification and commit

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`
- Add or modify all implementation files above

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-extension-abi -p sdkwork-api-extension-host -p sdkwork-api-app-gateway --all-targets -- -D warnings`
- `$env:CARGO_BUILD_JOBS='1'; cargo test --workspace -q -j 1`

Expected: PASS

**Step 2: Commit**

```bash
git add Cargo.toml Cargo.lock README.md docs/architecture/runtime-modes.md docs/api/compatibility-matrix.md docs/plans/2026-03-14-native-dynamic-extension-runtime-design.md docs/plans/2026-03-14-native-dynamic-extension-runtime-implementation.md crates/sdkwork-api-extension-abi crates/sdkwork-api-ext-provider-native-mock crates/sdkwork-api-extension-host crates/sdkwork-api-app-gateway
git commit -m "feat: add native dynamic extension runtime"
git push
```
