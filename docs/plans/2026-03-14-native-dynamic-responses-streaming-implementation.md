# Native Dynamic Responses Streaming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add native dynamic SSE execution for `/v1/responses` so trusted in-process provider extensions can match the existing upstream stream relay path.

**Architecture:** Reuse the current host-owned provider stream abstraction and callback-based native ABI, extend the native fixture to emit streamed `responses.create` frames, and prove the behavior through runtime and HTTP end-to-end tests.

**Tech Stack:** Rust, Axum, tokio, serde_json, libloading

---

### Task 1: Add failing tests for native dynamic `/v1/responses` streaming

**Files:**
- Modify: `crates/sdkwork-api-extension-host/tests/native_dynamic_runtime.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- a loaded native dynamic provider can execute `ProviderRequest::ResponsesStream`
- the stateful HTTP `/v1/responses` route relays SSE from a signed native dynamic extension

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime executes_native_dynamic_responses_stream_request -- --exact`
- `cargo test -p sdkwork-api-interface-http --test responses_route stateful_responses_route_relays_stream_to_native_dynamic_provider -- --exact`

Expected: FAIL because the native fixture only streams chat completions today.

### Task 2: Extend the native fixture and manifest capability set

**Files:**
- Modify: `crates/sdkwork-api-ext-provider-native-mock/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`

**Step 1: Add response stream fixture output**

Make `responses.create` with `expects_stream = true` emit deterministic SSE frames.

**Step 2: Publish response stream capability metadata**

Add `responses.create` and `responses.stream` to the fixture manifest helpers.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime executes_native_dynamic_responses_stream_request -- --exact`

Expected: PASS

### Task 3: Wire HTTP end-to-end coverage and docs

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- Modify: `README.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Add native dynamic `/v1/responses` route coverage**

Prove that a signed native dynamic provider can be discovered, loaded, and streamed through `/v1/responses`.

**Step 2: Update docs**

Reflect that native dynamic runtime now supports:

- JSON provider operations
- chat SSE
- responses SSE

while generic binary streams and richer lifecycle hooks remain future work.

**Step 3: Run focused tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test responses_route stateful_responses_route_relays_stream_to_native_dynamic_provider -- --exact`

Expected: PASS

### Task 4: Run verification and commit

**Files:**
- Modify all files above

**Step 1: Run verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `$env:CARGO_BUILD_JOBS='1'; cargo test -p sdkwork-api-extension-host --test native_dynamic_runtime -q`
- `$env:CARGO_BUILD_JOBS='1'; cargo test -p sdkwork-api-interface-http --test responses_route -q`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --no-deps -p sdkwork-api-extension-host -p sdkwork-api-interface-http -p sdkwork-api-ext-provider-native-mock --all-targets -- -D warnings`

Expected: PASS

**Step 2: Commit**

```bash
git add README.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md docs/plans/2026-03-14-native-dynamic-responses-streaming-design.md docs/plans/2026-03-14-native-dynamic-responses-streaming-implementation.md crates/sdkwork-api-ext-provider-native-mock crates/sdkwork-api-extension-host crates/sdkwork-api-interface-http
git commit -m "feat: add native dynamic responses streaming"
git push
```
