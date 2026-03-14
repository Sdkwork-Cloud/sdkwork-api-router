# OpenAI Parity Gap Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the highest-value remaining OpenAI API parity gaps by adding audio voices and voice consents, evals query and run routes, fine-tuning events and checkpoints, and video character and extend routes across the full gateway stack.

**Architecture:** Extend the existing typed provider contract instead of introducing a generic passthrough. Add tests first, then implement the minimal contract, provider, extension-host, gateway, and HTTP routing changes so stateful and stateless execution paths stay aligned.

**Tech Stack:** Rust, Axum, Reqwest, serde, SQLx, existing SDKWork provider and extension crates

---

### Task 1: Add failing route tests for the missing API families

**Files:**
- Create: `crates/sdkwork-api-interface-http/tests/audio_voices_route.rs`
- Create: `crates/sdkwork-api-interface-http/tests/audio_voice_consents_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/evals_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/fine_tuning_route.rs`
- Modify: `crates/sdkwork-api-interface-http/tests/videos_route.rs`

**Step 1: Write the failing tests**

Add tests that assert the gateway exposes:

- `GET /v1/audio/voices`
- `POST /v1/audio/voice_consents`
- eval list, retrieve, update, delete, run list, run create, run retrieve
- fine-tuning events list and checkpoints list
- video characters list, retrieve, update, and extend

Cover:

- local compatible fallback behavior on the plain router
- stateless upstream relay behavior
- stateful relay behavior through configured provider data

**Step 2: Run tests to verify they fail**

Run:

- `cargo test -p sdkwork-api-interface-http --test audio_voices_route -q`
- `cargo test -p sdkwork-api-interface-http --test audio_voice_consents_route -q`
- `cargo test -p sdkwork-api-interface-http --test evals_route -q`
- `cargo test -p sdkwork-api-interface-http --test fine_tuning_route -q`
- `cargo test -p sdkwork-api-interface-http --test videos_route -q`

Expected: FAIL because the routes and provider request variants do not exist yet.

### Task 2: Add missing contract and provider request types

**Files:**
- Modify: `crates/sdkwork-api-contract-openai/src/audio.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/evals.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/fine_tuning.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/videos.rs`
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`

**Step 1: Add minimal additive contract structs**

Add only the request and response shapes needed by the new handlers and local fallbacks.

**Step 2: Extend `ProviderRequest`**

Add variants for all new routes so builtin providers and dynamic extensions share the same execution vocabulary.

**Step 3: Run compile-targeted tests**

Run:

- `cargo test -p sdkwork-api-provider-core -q`

Expected: PASS or compile forward to the next failing layer.

### Task 3: Implement provider and extension runtime support

**Files:**
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-extension-host/src/lib.rs`

**Step 1: Implement upstream OpenAI request mapping**

Map the new provider variants to the official OpenAI-compatible upstream routes.

**Step 2: Implement extension operation mapping**

Map the same variants to stable extension invocation names:

- `audio.voices.list`
- `audio.voice_consents.create`
- `evals.*`
- `fine_tuning.jobs.events.list`
- `fine_tuning.jobs.checkpoints.list`
- `videos.characters.*`
- `videos.extend`

**Step 3: Run targeted tests**

Run:

- `cargo test -p sdkwork-api-provider-openai -q`
- `cargo test -p sdkwork-api-extension-host -q`

Expected: PASS

### Task 4: Implement gateway relay helpers and HTTP routes

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Add stateful relay helpers and local fallback functions**

Follow the existing naming and selection pattern so the new routes participate in routing, credentials, and extension resolution like the existing ones.

**Step 2: Wire stateless and stateful Axum routes**

Add the new paths to both routers and keep handler semantics consistent with the current relay-first behavior.

**Step 3: Run focused route tests**

Run:

- `cargo test -p sdkwork-api-interface-http --test audio_voices_route -q`
- `cargo test -p sdkwork-api-interface-http --test audio_voice_consents_route -q`
- `cargo test -p sdkwork-api-interface-http --test evals_route -q`
- `cargo test -p sdkwork-api-interface-http --test fine_tuning_route -q`
- `cargo test -p sdkwork-api-interface-http --test videos_route -q`

Expected: PASS

### Task 5: Update compatibility docs and verify the workspace

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/api/compatibility-matrix.md`
- Modify: `docs/architecture/runtime-modes.md`

**Step 1: Update documentation**

Document the new parity coverage and plugin operation standard additions.

**Step 2: Run full verification**

Run:

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`
- `pnpm --dir console build`
- `$env:CARGO_BUILD_JOBS='1'; cargo clippy --workspace --all-targets -- -D warnings`

Expected: all commands exit `0`

**Step 3: Commit**

```bash
git add docs/plans/2026-03-14-openai-parity-gap-closure-design.md docs/plans/2026-03-14-openai-parity-gap-closure-implementation.md crates/sdkwork-api-contract-openai/src/audio.rs crates/sdkwork-api-contract-openai/src/evals.rs crates/sdkwork-api-contract-openai/src/fine_tuning.rs crates/sdkwork-api-contract-openai/src/videos.rs crates/sdkwork-api-provider-core/src/lib.rs crates/sdkwork-api-provider-openai/src/lib.rs crates/sdkwork-api-extension-host/src/lib.rs crates/sdkwork-api-app-gateway/src/lib.rs crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-interface-http/tests README.md README.zh-CN.md docs/api/compatibility-matrix.md docs/architecture/runtime-modes.md
git commit -m "feat: close openai parity gaps"
```
