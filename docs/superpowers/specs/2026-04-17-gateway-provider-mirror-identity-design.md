# Gateway Provider Mirror Identity Design

**Date:** 2026-04-17

## Goal

Preserve provider-specific mirror identity across the catalog, admin and portal provider views, and stateless upstream configuration without breaking the existing execution-oriented `protocol_kind` contract. This slice creates the foundation required for future provider-specific public mirror routes such as `music.suno`, `images.kling`, or `video.google-veo` to be added without reworking provider metadata again.

## Current State

- provider metadata already distinguishes three execution-level protocol families cleanly:
  - `openai`
  - `anthropic`
  - `gemini`
- all other providers collapse into `protocol_kind = custom`
- admin and portal views already expose `protocol_kind` plus `integration.mode`
- stateless upstream configuration already derives and normalizes `protocol_kind`
- that execution contract is correct for routing and runtime behavior, but it loses the provider-specific mirror identity needed for future heterogeneous public protocol families

## Problem Statement

The gateway now has a stable public mirror taxonomy for:

- `code.openai`
- `code.claude`
- `code.gemini`
- `images.openai`
- `audio.openai`
- `video.openai`
- `music.openai`

The next major requirement is provider-specific media mirror expansion. Today that work is blocked by metadata shape:

- `protocol_kind` intentionally expresses execution semantics, not public mirror identity
- once a provider falls into `custom`, the current surface no longer preserves whether it is `ollama`, `suno`, `kling`, `midjourney`, or another heterogeneous protocol
- future public mirror routes would therefore need to rediscover provider identity from unrelated fields or invent new ad hoc rules at route time
- that would recreate the exact taxonomy drift this project is trying to eliminate

## Options Considered

### Option A: Reuse `protocol_kind` for both execution semantics and public mirror identity

Pros:

- smallest field count
- no additional output shape

Cons:

- mixes two different responsibilities again
- would force `protocol_kind` to grow beyond `openai|anthropic|gemini|custom`
- risks breaking existing routing, readiness, and passthrough assumptions

### Option B: Keep `protocol_kind` stable and add an additive derived mirror identity

Pros:

- preserves existing execution semantics
- gives future public mirror routes a stable provider-facing identity key
- can be derived from existing provider metadata for stateful surfaces
- can be added to stateless upstream configuration without breaking current constructors

Cons:

- requires additive API and test updates across catalog, admin, portal, and stateless config
- needs precise derivation rules so standard passthrough families do not regress into provider-brand labels

### Option C: Skip the metadata layer and solve provider identity later inside each future route family

Pros:

- no immediate cross-cutting changes

Cons:

- future public mirror routes would each reinvent provider identity rules
- guarantees drift between admin, portal, stateless setup, and gateway routing
- raises the cost of every later provider-specific mirror slice

## Recommendation

Adopt Option B.

`protocol_kind` should remain the execution-semantic field. Add a separate additive `mirror_protocol_identity` that represents the public mirror family key future routes and docs should use.

## Design

### Contract Split

Keep these responsibilities separate:

- `protocol_kind`: execution semantics and passthrough/runtime behavior
- `mirror_protocol_identity`: public mirror family identity

Examples:

- official OpenAI provider:
  - `protocol_kind = openai`
  - `mirror_protocol_identity = openai`
- official Anthropic provider:
  - `protocol_kind = anthropic`
  - `mirror_protocol_identity = anthropic`
- official Gemini provider:
  - `protocol_kind = gemini`
  - `mirror_protocol_identity = gemini`
- OpenRouter default-plugin provider:
  - `protocol_kind = openai`
  - `mirror_protocol_identity = openai`
- Ollama default-plugin provider:
  - `protocol_kind = custom`
  - `mirror_protocol_identity = ollama`
- future Suno custom provider:
  - `protocol_kind = custom`
  - `mirror_protocol_identity = suno`

### Derivation Rules

For persisted providers:

1. normalize `protocol_kind`
2. if it is `openai`, `anthropic`, or `gemini`, use that exact value as `mirror_protocol_identity`
3. otherwise, if the provider is on a builtin default-plugin path that intentionally preserves its own heterogeneous protocol family, use that family key
   - current example: `ollama`
4. otherwise, if the provider has a custom extension identity like `sdkwork.provider.suno.official`, derive the provider segment from that extension identity
5. otherwise, fall back to a sanitized `adapter_kind`

For stateless upstream configuration:

1. existing constructors continue to derive the identity automatically from `runtime_key + protocol_kind`
2. add an explicit constructor that lets library callers override `mirror_protocol_identity` when the runtime key is too generic
3. existing callers remain source-compatible

### Output Surfaces

The new field should be additive and visible where provider integration metadata is already exposed:

- `sdkwork-api-app-catalog::ProviderIntegrationView`
- `/admin/providers`
- `/admin/tenants/{tenant_id}/providers/readiness`
- portal routing provider options
- `sdkwork-api-interface-http::StatelessGatewayUpstream`

This slice does not change public gateway data-plane routes or `/openapi.json`.

### Non-Goals

- no new public provider-specific mirror routes yet
- no change to runtime `route_readiness` semantics
- no change to how `protocol_kind` is stored or interpreted
- no provider-specific media OpenAPI tags in the public gateway document yet

## Testing And Acceptance

Acceptance requires tests in four layers:

### App Catalog

- default plugin families derive the expected `mirror_protocol_identity`
- standard passthrough providers keep `mirror_protocol_identity == protocol_kind`
- custom providers derive provider identity from extension or adapter identity without mutating `protocol_kind`

### Admin API

- provider create/list responses expose `integration.mirror_protocol_identity`
- OpenRouter remains `openai`
- Ollama becomes `ollama`
- explicit custom providers can surface a non-generic identity through their runtime identity metadata

### Portal API

- provider option summaries expose `integration.mirror_protocol_identity`
- existing OpenAI/OpenRouter semantics remain unchanged

### Stateless HTTP

- legacy constructors still work
- derived identities stay correct for standard passthrough and builtin default-plugin cases
- new explicit constructor preserves a caller-supplied `mirror_protocol_identity`

## Risks And Mitigations

### Risk: `mirror_protocol_identity` drifts into a second execution field

Mitigation:

- keep routing and readiness logic on `protocol_kind`
- keep `mirror_protocol_identity` strictly additive and descriptive in this slice

### Risk: standard passthrough providers regress into provider-brand labels

Mitigation:

- hard-pin `openai`, `anthropic`, and `gemini` identities when those are the normalized protocol kinds
- add regression tests for OpenAI, Anthropic, Gemini, OpenRouter, and Ollama

### Risk: future custom plugins still lose provider identity

Mitigation:

- derive identity from custom extension IDs when available
- add an explicit stateless override constructor for cases where runtime keys are too generic

## Final Principle

Before the gateway can publish provider-specific mirror routes cleanly, it must preserve provider-specific mirror identity cleanly.

That means:

- execution semantics stay in `protocol_kind`
- public mirror identity lives in `mirror_protocol_identity`
- standard passthrough families stay standard
- heterogeneous provider families keep their own identity instead of collapsing into `custom`
