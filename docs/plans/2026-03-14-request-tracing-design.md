# Request Tracing Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Add a real request-level tracing baseline for the gateway and admin services, including stable request IDs, shared HTTP tracing middleware, and runtime log initialization that works in both standalone binaries and embedded composition.

## Current Problem

The repository now exposes `/metrics`, but request-level observability is still incomplete:

- there is no shared request ID generation or propagation
- responses do not return a correlation header that operators can use
- HTTP requests are not wrapped in structured tracing spans
- standalone binaries do not initialize a tracing subscriber

This leaves the system below the originally approved observability architecture, which explicitly called for tracing in addition to Prometheus-style metrics.

## Options Considered

### Option A: Initialize `tracing_subscriber` only

Pros:

- very small diff
- gives runtime logs immediately

Cons:

- no request correlation boundary
- interface crates still own no reusable tracing behavior
- hard to connect logs to operator-facing incidents

### Option B: Add a shared request-tracing layer with request IDs and thin runtime initialization

Pros:

- closes the most important request-correlation gap
- keeps the observability boundary centralized in one crate
- composes cleanly into Axum routers and standalone services
- preserves room for future OpenTelemetry exporters

Cons:

- does not yet provide distributed tracing or external exporters

### Option C: Jump directly to full OpenTelemetry propagation and exporters

Pros:

- closest to long-term architecture
- future-proofs trace shipping

Cons:

- much larger operational and dependency surface
- would slow down delivery of the first reliable tracing baseline
- adds deployment concerns that the current repository does not need yet

## Recommendation

Use **Option B**.

The highest-value gap is not exporter plumbing. It is the lack of a stable request-correlation contract. Shared request IDs plus HTTP tracing spans create immediate operational value, align with the current Axum architecture, and keep the system ready for a later OpenTelemetry phase.

## Proposed Capability

Add a shared request tracing capability in `sdkwork-api-observability` that provides:

- request ID extraction from `x-request-id` when supplied by the caller
- generated request IDs when the caller does not provide one
- response propagation through the `x-request-id` header
- structured tracing spans around every HTTP request with:
  - `service`
  - `request_id`
  - `method`
  - `route`
  - `status`
  - `duration_ms`
- idempotent tracing subscriber initialization for standalone services

## Architecture

The implementation should remain layered:

- `sdkwork-api-observability`
  owns request ID generation, response propagation, tracing middleware, and subscriber initialization
- `sdkwork-api-interface-http`
  composes the shared tracing middleware into the gateway router
- `sdkwork-api-interface-admin`
  composes the shared tracing middleware into the admin router
- `gateway-service` and `admin-api-service`
  initialize tracing once during startup and then stay thin

This keeps request tracing reusable across standalone, embedded, and test router construction paths.

## Request ID Contract

The first version should use one stable header:

- `x-request-id`

Behavior:

1. if the inbound request already includes `x-request-id`, preserve it
2. otherwise, generate a compact unique ID locally
3. always return the resolved ID on the response
4. include the same value in the tracing span

The ID does not need to be globally cryptographic in this batch. It must be unique enough for operator correlation and deterministic in tests.

## Logging Shape

The tracing middleware should emit one info-level completion event per request after the response is known.

The event should include:

- service name
- request ID
- method
- matched route pattern
- status code
- duration in milliseconds

This batch should avoid chatty per-chunk streaming logs. Streaming endpoints should still produce one request completion event.

## Error Handling

- tracing must never block or fail request handling
- if matched route metadata is unavailable, use `unmatched`
- invalid non-UTF8 request ID headers should be ignored and replaced with a generated ID
- subscriber initialization should be safe to call more than once

## Testing Strategy

Add tests for:

- generated request IDs being added to responses
- supplied request IDs being preserved on responses
- admin and gateway routers both exposing the same header behavior
- tracing subscriber initialization being safe to call repeatedly

## Scope

This batch will:

- add shared request ID and tracing middleware
- propagate `x-request-id` through admin and gateway responses
- initialize tracing in both standalone binaries
- document request correlation in the README and runtime docs

This batch will not:

- add OpenTelemetry exporters
- add W3C traceparent propagation
- add sampling, collector config, or external log shipping
