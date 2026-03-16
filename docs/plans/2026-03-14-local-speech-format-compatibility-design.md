# Local Speech Format Compatibility Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Context

The gateway already exposes `/v1/audio/speech` in both stateless and stateful modes, and relay-backed execution paths are covered by tests.

However, the local fallback path still has a concrete compatibility defect:

- it advertises multiple response formats through HTTP content-type handling
- but the fallback audio generator only synthesizes bytes for `wav` and `pcm`
- requests for formats like `mp3`, `opus`, `aac`, and `flac` therefore return empty bodies

This is not just an implementation detail. It breaks the promise that local fallback remains a useful compatible mode when upstream relay is unavailable.

## Problem

The current local speech fallback has two issues:

1. supported-looking formats can silently produce empty audio payloads
2. unsupported formats are not validated explicitly, which keeps error behavior vague and leaves a handler-side `expect(...)` in place

That combination makes the local fallback look broader than it really is.

## Options Considered

### Option A: Force all local fallback speech to `wav`

Pros:

- smallest implementation
- avoids empty payloads

Cons:

- response format would not match the requested format
- weak compatibility for clients that depend on content-type and file extension

### Option B: Support all currently advertised fallback formats and reject unknown ones

Pros:

- preserves the current public surface
- fixes empty-payload behavior for common formats
- lets the local path return a clear client error for unknown formats
- removes one panic-prone `expect(...)` from the handler edge

Cons:

- requires a small amount of format normalization and error handling

### Option C: Restrict local fallback to `wav` and `pcm` only

Pros:

- contract becomes explicit
- implementation stays simple

Cons:

- narrows current compatibility
- creates divergence between relay and local modes

## Recommendation

Use **Option B**.

The HTTP layer already advertises a broader speech format surface. The best move is to make the fallback consistent with that advertised surface and fail clearly for truly unsupported values.

## Scope

This batch should:

- make local `/v1/audio/speech` return non-empty fallback bytes for:
  - `wav`
  - `mp3`
  - `opus`
  - `aac`
  - `flac`
  - `pcm`
- normalize and validate the requested format before building the local response
- return an OpenAI-style `400` error envelope when the requested local speech format is unsupported
- remove the local speech `expect(...)` from the HTTP edge

This batch should not:

- change upstream relay behavior
- introduce real text-to-speech synthesis
- redesign audio contract types outside the fallback path

## Architecture

The implementation should stay split cleanly across layers:

- `sdkwork-api-app-gateway`
  normalizes the requested local speech format and synthesizes placeholder bytes for each supported format
- `sdkwork-api-interface-http`
  maps local speech generation errors to a structured OpenAI-style `400` response

Relay logic stays unchanged. Only the local fallback path is being corrected.

## Testing Strategy

Use TDD with route-level verification:

1. add a failing test proving local `mp3` speech currently returns an empty body
2. add a failing test proving unsupported local speech formats should return a structured `400`
3. implement minimal normalization and error handling
4. run focused route tests
5. run package and workspace verification

## Success Criteria

This batch is complete when:

- local `mp3` speech returns `200`, `audio/mpeg`, and non-empty bytes
- unsupported local speech formats return `400` with OpenAI-style JSON error payloads
- relay-backed speech tests still pass
- workspace verification stays green
