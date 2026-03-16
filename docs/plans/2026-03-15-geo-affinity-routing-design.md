# Geo-Affinity Routing Design

**Date:** 2026-03-15

**Status:** Approved by the existing routing architecture baseline and the user's standing instruction to continue autonomously without pausing for approval

## Goal

Close the strongest remaining routing-policy gap by adding a first-class `geo_affinity` strategy that prefers region-matching providers when the request carries an explicit routing region, while degrading safely to the current health-aware ranking when no regional match exists.

## Why This Batch

The repository already supports:

- persisted routing policies
- deterministic, weighted, and SLO-aware routing strategies
- persisted routing decision logs
- extension instance config carrying region-like deployment metadata
- authenticated stateful gateway execution
- admin route simulation and routing console visibility

What is still missing relative to the documented architecture is a real regional selection dimension. The docs still call out geo affinity as future work, which leaves a clear gap between the current routing substrate and the production-oriented routing story.

## Scope

This batch will implement:

1. a new `geo_affinity` routing strategy
2. request-scoped routing-region propagation for real gateway traffic
3. admin simulation support for an optional `requested_region`
4. candidate assessment metadata for resolved instance region and region-match status
5. persisted decision-log capture of the requested region
6. console visibility for region-aware simulation and decision-log evidence

This batch will not implement:

- IP geolocation or country inference
- multi-region latency telemetry
- provider-specific regional pricing policies
- connector or native dynamic hot reload
- unload safety for in-process plugins

## Options Considered

### Option A: Policy-level fixed preferred region

Pros:

- smallest schema change
- easy to explain

Cons:

- cannot vary by request
- multiple policies for the same capability and model still need an external discriminator
- does not actually solve regional request affinity

### Option B: Request-scoped geo affinity with gateway header propagation

Pros:

- gives the runtime a real request-level regional signal
- keeps the policy model small
- reuses existing extension instance config that already stores region metadata
- allows admin simulation to reproduce gateway behavior

Cons:

- touches routing, gateway, HTTP middleware, admin DTOs, and console types
- requires careful caching so one region does not reuse another region's routing decision

### Option C: Full geolocation subsystem first

Pros:

- closest to a long-term edge-routing vision
- could infer region without client hints

Cons:

- much larger scope
- introduces correctness and privacy questions unrelated to the core routing gap
- slows down delivery of the actual missing routing strategy

## Recommendation

Use **Option B**.

The codebase already has the right substrate for explainable route selection and persisted audit logs. The narrow, high-leverage move is to let requests carry an explicit region hint and let routing prefer matching provider instances without weakening the current health-aware fallback behavior.

## Request Region Model

Real gateway traffic should treat the request region as optional metadata supplied by the caller through `x-sdkwork-region`.

Rules:

1. the header is optional
2. values are normalized by trimming whitespace and lowercasing
3. an empty normalized value behaves as absent
4. when absent, `geo_affinity` degrades to the current top-ranked candidate and records that no region hint was available

Admin simulation should accept the same metadata through JSON as `requested_region` so operators can reproduce and explain the same selection path.

## Candidate Region Model

Provider region should come from mounted extension instance config using the same precedence model already used for routing hints:

1. `config.routing.region`
2. `config.region`

Candidate assessments should expose:

- `region`
- `region_match`

This keeps routing explanations and persisted logs inspectable without inventing a second region-specific DTO family.

## Selection Semantics

`geo_affinity` should operate in three stages:

1. build the current assessed candidate list and preserve the existing stable ranking
2. determine the healthy-eligible pool:
   - only available candidates participate
   - if at least one healthy available candidate exists, unhealthy candidates are excluded from geo-affinity selection
3. prefer exact region matches inside that pool:
   - if one or more candidates match the requested region, choose the top-ranked matching candidate
   - otherwise degrade to the top-ranked candidate from the healthy-eligible pool

If there are no available candidates at all, the strategy should still fall back to the existing top-ranked candidate path so behavior remains conservative and explainable.

This keeps reliability ahead of locality while still making locality a real routing dimension.

## Decision And Logging Model

Routing decisions and persisted decision logs should carry `requested_region` as optional metadata.

Candidate reasons should include region evidence such as:

- `resolved region = us-east`
- `eligible for geo-affinity selection because region matches requested region us-east`
- `excluded from geo-affinity selection because region eu-west does not match requested region us-east`
- `excluded from geo-affinity selection because candidate region is unknown`

This preserves the governance substrate introduced by decision logging and makes future regional debugging practical.

## Gateway Integration

The gateway already routes all stateful requests through a small routing-selection seam. The cleanest integration is:

1. inject a request-scoped routing region at the HTTP layer
2. let gateway routing helpers read that request-scoped value
3. include the effective region in routing cache keys so decisions do not bleed across regions

This avoids widening dozens of public function signatures just to thread one optional header.

## Testing Strategy

The batch should be proven through:

1. routing tests for:
   - selecting a matching region over a non-matching higher-ranked candidate
   - degrading to the top-ranked healthy candidate when no match exists
   - preferring healthy non-matching candidates over unhealthy matching candidates
2. admin API tests for:
   - accepting `requested_region`
   - returning requested-region decision metadata
   - persisting requested-region decision logs
3. gateway tests for:
   - propagating `x-sdkwork-region`
   - selecting the matching provider instance
   - storing requested-region evidence in decision logs
4. console typecheck coverage for the new routing fields

## Follow-On Work

After this batch, the strongest remaining gaps should be:

1. automatic geography derivation instead of explicit request hints
2. region-aware latency evidence and adaptive routing
3. extension hot reload and unload orchestration
