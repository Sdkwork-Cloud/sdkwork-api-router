# Native Dynamic Extension Runtime Design

**Date:** 2026-03-14

**Status:** Approved by existing extension architecture baseline for direct implementation

## Goal

Close the last major runtime gap in the extension architecture by turning `native_dynamic` from a discovery-only manifest concept into a real, loadable, policy-gated runtime for provider extensions.

## Why This Batch

The current gateway already has:

- built-in provider extensions
- connector extension discovery, supervision, and dispatch
- extension trust policy with signature verification

What is still missing is the in-process plugin boundary promised by the approved architecture. Until `native_dynamic` can actually load and execute, the extension system is not yet a complete three-runtime model.

## Scope

This batch will implement:

1. a new `sdkwork-api-extension-abi` crate
2. a stable C ABI for provider plugin manifest and JSON execution
3. host-side native dynamic library loading through `libloading`
4. runtime registration for discovered `native_dynamic` provider plugins
5. real gateway dispatch through native dynamic provider plugins for JSON operations

This batch will not implement:

- hot reload
- unload safety guarantees beyond process lifetime
- stream ABI support
- cross-language SDKs for plugin authors

## Options Considered

### Option A: Metadata-only ABI

Define an ABI crate and exported manifest symbols, but do not execute provider operations.

Pros:

- minimal change
- low loader risk

Cons:

- keeps `native_dynamic` operationally incomplete
- does not satisfy the original runtime promise

### Option B: Minimal JSON execution ABI

Define a narrow ABI where a plugin exports:

- ABI version
- manifest JSON
- provider execute function
- string free function

The host serializes gateway provider requests into a stable JSON invocation envelope and receives a JSON result envelope back.

Pros:

- real runtime execution
- stable enough across crate boundaries
- no trait object ABI leakage
- small enough to test and reason about

Cons:

- stream support remains out of scope
- execution payload must stay JSON-friendly

### Option C: Full in-process runtime with stream and lifecycle ABI

Add load/init/shutdown/stream functions in one batch.

Pros:

- broadest capability surface

Cons:

- much more operational complexity
- harder to keep safe on Windows
- larger blast radius for the current codebase

## Recommendation

Use **Option B**.

It turns `native_dynamic` into a real runtime with bounded risk and a clean upgrade path toward richer lifecycle and stream support later.

## ABI Shape

The ABI will be explicit and JSON-based.

### Exported symbols

- `sdkwork_extension_abi_version`
- `sdkwork_extension_manifest_json`
- `sdkwork_extension_provider_execute_json`
- `sdkwork_extension_free_string`

### Invocation envelope

The host will send:

- operation name
- path parameters
- request body as JSON
- `api_key`
- `base_url`
- whether the request expects stream output

### Result envelope

The plugin will return one of:

- `json`
- `unsupported`
- `error`

This keeps the ABI narrow, stable, and independent from Rust trait object layout.

## Runtime Validation

For a discovered `native_dynamic` package to become executable, all of the following must hold:

1. discovery policy enables `native_dynamic`
2. trust policy allows the package to load
3. the entrypoint library loads successfully
4. the library exports the required ABI symbols
5. the library manifest matches the package manifest on runtime-critical fields

If any of these fail, the package remains visible through the admin API but is not registered into the executable provider host.

## Gateway Execution Model

Native dynamic provider plugins will participate in the same `extension_id` based runtime dispatch path as builtin and connector providers.

The gateway will:

1. discover the package
2. verify trust policy
3. register a provider factory backed by the native dynamic loader
4. build runtime load plans from installation and instance state
5. execute JSON-capable provider operations through the loaded plugin

If a native dynamic plugin is not loadable, the gateway falls back conservatively instead of failing the entire runtime assembly.

## Testing Strategy

The implementation will be proven with a real fixture plugin compiled as a `cdylib`.

Tests will cover:

- ABI crate envelope contract
- host loading and manifest verification
- host provider resolution for native dynamic plugins
- gateway relay through a native dynamic plugin
- policy gating when native dynamic runtime is disabled or the plugin is not loadable

## Follow-On Work

After this batch, the next logical steps are:

1. stream ABI for native dynamic plugins
2. explicit lifecycle hooks such as `init` and `shutdown`
3. weighted and health-scored routing over multiple extension instances
4. further reduction of `emulated` data-plane paths
