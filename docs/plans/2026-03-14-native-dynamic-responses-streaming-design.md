# Native Dynamic Responses Streaming Design

**Date:** 2026-03-14

**Status:** Approved by the existing extension and gateway baseline for direct implementation

## Goal

Close the next native dynamic parity gap by allowing trusted `native_dynamic` provider extensions to satisfy `/v1/responses` streaming requests with real SSE output.

## Why This Batch

The gateway already supports:

- built-in OpenAI-compatible upstream relay for `/v1/responses` JSON and SSE
- native dynamic JSON execution for provider operations
- native dynamic SSE relay for `/v1/chat/completions`

What is still missing is native dynamic parity for `/v1/responses` when `stream = true`.

That leaves the extension runtime one step behind the current OpenAI-compatible HTTP provider path for one of the primary inference APIs in the original project scope.

## Scope

This batch will implement:

1. native fixture support for streamed `responses.create`
2. runtime tests proving `ProviderRequest::ResponsesStream` works through `native_dynamic`
3. HTTP end-to-end tests proving `/v1/responses` can relay SSE from a signed native dynamic extension
4. manifest and documentation updates reflecting the new stream capability

This batch will not implement:

- generic binary stream support for audio, files, or videos
- native lifecycle hooks such as `init`, `health`, or `shutdown`
- weighted routing, health scoring, or geo-aware failover

## Options Considered

### Option A: Reuse `responses.create` only and rely on `expects_stream = true`

Pros:

- keeps the ABI mapping compact
- matches the existing extension invocation contract

Cons:

- manifest capability metadata remains less explicit about stream support unless we also publish a dedicated stream capability

### Option B: Add a dedicated manifest capability for `responses.stream`

Pros:

- explicit extension metadata for stream-capable response generation
- aligns with the existing `chat.completions.stream` capability naming

Cons:

- adds one more manifest capability string to keep in sync

## Recommendation

Use **Option B** while still executing the operation through `responses.create` plus `expects_stream = true`.

That keeps runtime invocation backward-compatible and makes capability discovery explicit for future routing, validation, and admin observability.

## Target Behavior

When a native dynamic provider receives `ProviderRequest::ResponsesStream`:

- extension-host maps it to `responses.create`
- `expects_stream = true` is set in the invocation envelope
- the plugin emits `text/event-stream`
- the gateway relays SSE bytes to `/v1/responses` without buffering
- the response body contains deterministic event frames followed by `[DONE]`

Non-streaming `/v1/responses` behavior remains unchanged.

## Testing Strategy

The batch will be proven through:

1. extension-host runtime tests for `ProviderRequest::ResponsesStream`
2. HTTP route tests for stateful `/v1/responses` relay through a signed native dynamic extension
3. fixture capability updates so manifest metadata reflects real runtime support

## Follow-On Work

After this batch, the most important remaining native dynamic gaps are:

1. generic binary stream parity for audio/file/video content
2. lifecycle hooks and richer runtime health contracts
3. more advanced routing strategies over multiple provider instances
