# OpenAI Parity Gap Closure Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Close the highest-value OpenAI API parity gaps that remain after the stateless relay expansion by adding the missing route families that already fit the current SDKWork gateway, provider, and extension architecture.

## Current Problem

The repository now covers most of the OpenAI-compatible data plane, but an audit against the current official API reference still shows several route groups that are either absent or only partially implemented:

- `/v1/audio/voices`
- `/v1/audio/voice_consents`
- `/v1/evals` query and nested run flows
- `/v1/fine_tuning/jobs/{job_id}/events`
- `/v1/fine_tuning/jobs/{job_id}/checkpoints`
- `/v1/videos/{video_id}/characters`
- `/v1/videos/{video_id}/characters/{character_id}`
- `/v1/videos/{video_id}/extend`

This gap matters because the architecture already claims broad OpenAI compatibility, and these remaining omissions now stand out more than deeper architectural concerns.

## Options Considered

### Option A: Keep the gaps documented and defer them

Pros:

- zero implementation risk
- no new types or route handlers

Cons:

- leaves the compatibility matrix knowingly incomplete
- weakens the claim of broad API-reference coverage
- keeps plugin runtimes from handling these operations through the same extension contract

### Option B: Add the missing routes end-to-end using the existing request abstraction

Pros:

- reuses `ProviderRequest`, the existing provider adapters, and extension runtime invocation naming
- keeps stateful and stateless routing behavior aligned
- extends the plugin contract instead of bypassing it
- fits the current DDD layering without adding a second execution path

Cons:

- touches several crates in one batch
- requires additive contract modeling for newly surfaced request and response bodies

### Option C: Introduce a generic catch-all passthrough proxy for unknown OpenAI routes

Pros:

- could reduce route-specific work in the short term
- might handle future API drift faster

Cons:

- weakens contract clarity and typing
- bypasses the current provider capability model
- makes extension ABI naming and compatibility guarantees ambiguous

## Recommendation

Use **Option B**.

The system already has the correct architecture for this work. The remaining gaps are now best solved by extending the existing typed provider contract, not by weakening the architecture with a generic passthrough layer.

## Scope

This batch should add first-class support for:

- `GET /v1/audio/voices`
- `POST /v1/audio/voice_consents`
- `GET /v1/evals`
- `GET /v1/evals/{eval_id}`
- `POST /v1/evals/{eval_id}`
- `DELETE /v1/evals/{eval_id}`
- `GET /v1/evals/{eval_id}/runs`
- `POST /v1/evals/{eval_id}/runs`
- `GET /v1/evals/{eval_id}/runs/{run_id}`
- `GET /v1/fine_tuning/jobs/{job_id}/events`
- `GET /v1/fine_tuning/jobs/{job_id}/checkpoints`
- `GET /v1/videos/{video_id}/characters`
- `GET /v1/videos/{video_id}/characters/{character_id}`
- `POST /v1/videos/{video_id}/characters/{character_id}`
- `POST /v1/videos/{video_id}/extend`

This batch should also extend:

- local compatible fallback behavior for stateless mode
- stateful relay support through the admin-backed provider catalog
- extension-host runtime invocation mapping so connector and native-dynamic packages can participate
- compatibility and README documentation

## Architecture

The batch should preserve the current layering:

- `sdkwork-api-contract-openai`
  defines additive request and response structures
- `sdkwork-api-provider-core`
  gains additive `ProviderRequest` variants only
- `sdkwork-api-provider-openai`
  maps those variants to concrete upstream HTTP calls
- `sdkwork-api-extension-host`
  maps the same variants to stable plugin operation names
- `sdkwork-api-app-gateway`
  exposes stateful relay helpers plus local fallback behavior
- `sdkwork-api-interface-http`
  wires Axum routes in both stateless and stateful routers

No new service, runtime layer, or transport abstraction is needed.

## Extension Standard Impact

The plugin architecture should remain explicit and typed.

New operations should extend the existing naming convention instead of inventing a second protocol:

- `audio.voices.list`
- `audio.voice_consents.create`
- `evals.list`
- `evals.retrieve`
- `evals.update`
- `evals.delete`
- `evals.runs.list`
- `evals.runs.create`
- `evals.runs.retrieve`
- `fine_tuning.jobs.events.list`
- `fine_tuning.jobs.checkpoints.list`
- `videos.characters.list`
- `videos.characters.retrieve`
- `videos.characters.update`
- `videos.extend`

This keeps channel and provider extensions dynamically loadable while preserving a stable execution vocabulary across builtin, connector, and native-dynamic runtimes.

## Local Fallback Semantics

The current runtime behavior should stay conservative:

1. stateless mode tries upstream first when configured
2. unresolved stateless upstream still falls back locally
3. resolved upstream failures still return `502 Bad Gateway`
4. stateful mode still prefers configured provider relay when routing resolves
5. missing stateful provider configuration still falls back to local compatible responses where the current router already does so

The new routes should match that behavior exactly.

## Testing Strategy

Use additive route tests first, then minimal implementation:

- contract coverage through compile-time usage in handlers and provider requests
- stateless route tests for upstream relay success
- stateful route tests for provider relay success
- local route tests for compatible fallback responses

The tests should verify:

- correct HTTP methods and paths
- response shape compatibility
- authorization propagation to upstream providers
- binary versus JSON behavior where applicable

## Out of Scope

This batch does not:

- add OpenAI organization admin APIs
- add containers or code-interpreter container files
- implement extension hot reload
- redesign routing or quota semantics
- weaken typed routing into a generic opaque passthrough
