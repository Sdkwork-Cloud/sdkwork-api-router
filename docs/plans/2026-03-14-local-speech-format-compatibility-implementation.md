# Local Speech Format Compatibility Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix local `/v1/audio/speech` fallback so supported response formats return real placeholder bytes and unsupported formats fail with structured client errors instead of ambiguous behavior.

**Architecture:** Add route-level regression tests in `sdkwork-api-interface-http`, then implement local speech format normalization in `sdkwork-api-app-gateway` and map local generation failures to OpenAI-style `400` responses in the HTTP layer.

**Tech Stack:** Rust, Axum, anyhow, serde_json, cargo test

---

### Task 1: Add failing route tests

**Files:**
- Modify: `crates/sdkwork-api-interface-http/tests/audio_speech_route.rs`

**Step 1: Write the failing tests**

Add tests for:

- local `mp3` speech returns non-empty bytes and `audio/mpeg`
- unsupported local speech format returns `400` with OpenAI-style error JSON

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test -p sdkwork-api-interface-http --test audio_speech_route audio_speech_route_returns_local_mp3_content_when_requested -q
cargo test -p sdkwork-api-interface-http --test audio_speech_route audio_speech_route_returns_bad_request_for_unsupported_local_format -q
```

Expected: failures because fallback bytes are empty for `mp3` and unsupported formats are not handled explicitly.

### Task 2: Implement minimal production fix

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Normalize supported speech formats**

Teach the local fallback generator to synthesize placeholder bytes for the supported format set.

**Step 2: Reject unsupported formats clearly**

Return an error for unsupported local speech formats.

**Step 3: Map that error to a structured `400`**

Add an OpenAI-style bad-request helper in the HTTP layer and use it for local speech generation failures.

**Step 4: Run focused tests**

Run:

```powershell
cargo test -p sdkwork-api-interface-http --test audio_speech_route audio_speech_route_returns_local_mp3_content_when_requested -q
cargo test -p sdkwork-api-interface-http --test audio_speech_route audio_speech_route_returns_bad_request_for_unsupported_local_format -q
```

Expected: PASS

### Task 3: Re-run broader verification

**Files:**
- Review: `sdkwork-api-interface-http`
- Review: workspace

**Step 1: Run package verification**

Run:

```powershell
cargo test -p sdkwork-api-interface-http --test audio_speech_route -q
cargo test -p sdkwork-api-interface-http -q
```

Expected: PASS

**Step 2: Run workspace verification**

Run:

```powershell
cargo test --workspace -q -j 1
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS

### Task 4: Commit the batch

**Files:**
- Include updated tests, implementation, and plan docs

**Step 1: Commit**

Run:

```powershell
git add crates/sdkwork-api-app-gateway/src/lib.rs crates/sdkwork-api-interface-http/src/lib.rs crates/sdkwork-api-interface-http/tests/audio_speech_route.rs docs/plans/2026-03-14-local-speech-format-compatibility-design.md docs/plans/2026-03-14-local-speech-format-compatibility-implementation.md
git commit -m "fix: harden local speech fallback formats"
```
