# Music API Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first-class `music` API family to the gateway so the router covers music generation, lyrics generation, resource retrieval, content fetch, and billing-ready metering.

**Architecture:** Extend the existing OpenAI-compatible Rust surface the same way `videos` was added: define a contract module, add provider request variants, wire stateless and stateful gateway routes, reuse the existing extension runtime and provider adapters, and record `music_seconds` when create requests include duration metadata. Keep the API resource-oriented instead of copying `new-api`'s `/suno/*` transport shape directly.

**Tech Stack:** Rust, Axum, Serde, sqlx, existing gateway/provider abstractions, OpenAPI route inventory generation, cargo tests.

---

### Task 1: Define music contracts

**Files:**
- Create: `crates/sdkwork-api-contract-openai/src/music.rs`
- Modify: `crates/sdkwork-api-contract-openai/src/lib.rs`
- Test: `crates/sdkwork-api-contract-openai/tests/music_contract.rs`

- [ ] **Step 1: Write the failing test**

Add a contract test that serializes:
- `CreateMusicRequest`
- `CreateMusicLyricsRequest`
- `MusicObject`
- `MusicTracksResponse`
- `DeleteMusicResponse`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-contract-openai music_contract -- --nocapture`
Expected: FAIL because the `music` module and related types do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add a `music` module with resource-oriented request and response structs:
- create track
- create lyrics
- track object
- lyrics object
- list response
- delete response

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-contract-openai music_contract -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-contract-openai/src/lib.rs crates/sdkwork-api-contract-openai/src/music.rs crates/sdkwork-api-contract-openai/tests/music_contract.rs
git commit -m "feat: add music API contracts"
```

### Task 2: Add provider and gateway app support

**Files:**
- Modify: `crates/sdkwork-api-provider-core/src/lib.rs`
- Modify: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Test: `crates/sdkwork-api-interface-http/tests/music_route.rs`

- [ ] **Step 1: Write the failing route test**

Add route tests that expect:
- `POST /v1/music`
- `GET /v1/music`
- `GET /v1/music/{music_id}`
- `DELETE /v1/music/{music_id}`
- `GET /v1/music/{music_id}/content`
- `POST /v1/music/lyrics`

Also add stateful and stateless relay checks against an OpenAI-compatible upstream stub.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`
Expected: FAIL because the router and provider request variants do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add:
- provider request enum variants for music
- OpenAI provider adapter methods that hit `/v1/music*`
- local fallback music objects and bytes in app gateway
- relay helpers that route music capability through the store-backed provider selection path

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-provider-core/src/lib.rs crates/sdkwork-api-provider-openai/src/lib.rs crates/sdkwork-api-app-gateway/src/lib.rs crates/sdkwork-api-interface-http/tests/music_route.rs
git commit -m "feat: add music gateway relay support"
```

### Task 3: Wire HTTP routes and billing

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Test: `crates/sdkwork-api-interface-http/tests/music_route.rs`

- [ ] **Step 1: Extend the failing test**

Add assertions that stateful music create requests:
- record usage under capability `music`
- propagate `music_seconds` into billing events when duration metadata is present

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`
Expected: FAIL because the router does not yet meter music duration.

- [ ] **Step 3: Write minimal implementation**

Add stateless and stateful handlers plus route registrations, then meter:
- create track
- list
- retrieve
- delete
- content fetch
- lyrics create

For create requests, derive `music_seconds` from request duration first, then from upstream response if present.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-interface-http/tests/music_route.rs
git commit -m "feat: add music HTTP routes and billing metering"
```

### Task 4: Update docs and compatibility references

**Files:**
- Modify: `docs/api-reference/gateway-api.md`
- Modify: `docs/reference/api-compatibility.md`
- Modify: `docs/zh/reference/api-compatibility.md`

- [ ] **Step 1: Write the doc gap check**

Confirm docs currently list images, audio, and videos but not music as a first-class route family.

- [ ] **Step 2: Run targeted verification**

Run: `rg -n "/v1/music|music" docs/api-reference/gateway-api.md docs/reference/api-compatibility.md docs/zh/reference/api-compatibility.md`
Expected: music missing or incomplete.

- [ ] **Step 3: Write minimal implementation**

Document the new `music` family and its current route set.

- [ ] **Step 4: Re-run targeted verification**

Run: `rg -n "/v1/music|music" docs/api-reference/gateway-api.md docs/reference/api-compatibility.md docs/zh/reference/api-compatibility.md`
Expected: updated route references present.

- [ ] **Step 5: Commit**

```bash
git add docs/api-reference/gateway-api.md docs/reference/api-compatibility.md docs/zh/reference/api-compatibility.md
git commit -m "docs: document music gateway routes"
```

### Task 5: Final verification

**Files:**
- Verify the modified files from Tasks 1-4

- [ ] **Step 1: Run focused contract and route tests**

Run:
- `cargo test -p sdkwork-api-contract-openai music_contract -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test music_route -- --nocapture`

Expected: PASS

- [ ] **Step 2: Run broader gateway regression slices**

Run:
- `cargo test -p sdkwork-api-app-gateway -- --nocapture`
- `cargo test -p sdkwork-api-interface-http --test videos_route -- --nocapture`

Expected: PASS

- [ ] **Step 3: Review docs references**

Run:
- `rg -n "/v1/music|music_seconds" docs crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-app-gateway/src/lib.rs`

Expected: music routes and billing references present in code and docs.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/plans/2026-04-03-music-api-foundation.md
git commit -m "docs: add music API foundation plan"
```
