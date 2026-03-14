# Observability Metrics Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for checkpoints

## Goal

Turn the current placeholder observability crate into a real shared HTTP metrics layer that exposes scrapeable service metrics for both the gateway and admin APIs.

## Current Problem

The repository includes `sdkwork-api-observability`, but it is still a stub. The standalone services expose `/health`, yet they do not expose:

- scrapeable metrics
- shared HTTP telemetry middleware
- service-level request counters or latency summaries

This leaves a large gap versus the approved architecture, which explicitly called for tracing and Prometheus-style observability.

## Options Considered

### Option A: Add tracing initialization only

Pros:

- small diff
- low risk

Cons:

- no operator-visible metrics surface
- does not create a reusable observability boundary
- still leaves Prometheus integration effectively unimplemented

### Option B: Add shared HTTP metrics registry and `/metrics` endpoints

Pros:

- creates immediate operational value
- fits the current Axum router architecture cleanly
- keeps observability centralized in one crate
- provides a stable base for future tracing and exporter work

Cons:

- first version focuses on HTTP request metrics only

### Option C: Add full OpenTelemetry + Prometheus exporter stack now

Pros:

- closest to the end-state architecture
- future-proof telemetry model

Cons:

- higher dependency and integration cost
- harder to validate safely in the current batch
- likely to slow down delivery of the first real metrics capability

## Recommendation

Use **Option B**.

The repository needs a real metrics surface before it needs a full telemetry platform. A shared HTTP metrics layer plus `/metrics` endpoints closes the most visible observability gap while keeping scope disciplined.

## Proposed Capability

Add a shared `HttpMetricsRegistry` in `sdkwork-api-observability` that records, per service:

- total request count
- cumulative request duration in milliseconds
- request count by method
- request count by matched route pattern
- request count by final HTTP status

Expose the data in Prometheus text exposition format through:

- `GET /metrics` on the gateway router
- `GET /metrics` on the admin router

## Architecture

The implementation should keep observability reusable and layered:

- `sdkwork-api-observability`
  owns the metrics registry, formatting, and Axum middleware function
- `sdkwork-api-interface-http`
  creates a gateway registry and attaches the shared middleware
- `sdkwork-api-interface-admin`
  creates an admin registry and attaches the shared middleware

This keeps services thin and avoids duplicating instrumentation logic.

## Metric Shape

The first version should expose stable Prometheus-compatible metrics such as:

- `sdkwork_service_info`
- `sdkwork_http_requests_total`
- `sdkwork_http_request_duration_ms_sum`
- `sdkwork_http_request_duration_ms_count`

Labels should include:

- `service`
- `method`
- `route`
- `status`

The route label should prefer Axum's matched route pattern, not raw request URIs, so the cardinality stays bounded.

## Security and Exposure

`/metrics` should remain unauthenticated, similar to `/health`, because metrics scrapers usually do not perform JWT login flows. This is acceptable in the current architecture because standalone services already bind to local or operator-controlled addresses by configuration.

## Error Handling

- metrics collection must never break request handling
- if matched route metadata is unavailable, the route label should fall back to `"unmatched"`
- metrics rendering should always succeed, even for empty registries

## Testing Strategy

Add tests for:

- registry rendering in the observability crate
- gateway `/metrics` exposure after one or more requests
- admin `/metrics` exposure after login and authenticated routes

## Scope

This batch will:

- implement a real shared observability registry
- expose `/metrics` in admin and gateway routers
- instrument requests through shared middleware
- document the new metrics endpoints

This batch will not:

- add full OpenTelemetry tracing pipelines
- add external Prometheus dependencies in the runtime environment
- add histogram buckets, alert rules, or distributed trace correlation
