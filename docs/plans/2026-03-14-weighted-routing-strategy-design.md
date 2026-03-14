# Weighted Routing Strategy Design

**Date:** 2026-03-14

**Status:** Approved by the existing gateway architecture baseline and the user's standing instruction to continue autonomously

## Goal

Promote routing strategy from an implicit hardcoded ranking rule into an explicit first-class policy capability, then use that strategy layer to add weighted random provider balancing without regressing the current health-aware deterministic path.

## Why This Batch

The repository already supports:

- capability and model-pattern policy matching
- policy priority and ordered provider preference
- runtime-aware candidate assessment
- operator-configured `weight`, `cost`, and `latency_ms` hints
- explainable routing simulation through the admin API

What is still missing relative to the original gateway design is the ability to express different routing behaviors for the same capability surface. Right now every policy implicitly means "deterministic best-ranked provider wins".

That creates three structural problems:

- the platform cannot do load distribution across equivalent providers
- `weight` exists only as a tie-break input, not as an actual traffic-shaping control
- future strategies such as quota-aware or SLO-aware selection have no clean extension point

## Scope

This batch will implement:

1. a first-class `RoutingStrategy` domain model
2. persisted routing policy strategy support in SQLite and PostgreSQL
3. seeded weighted-random provider selection for eligible candidates
4. admin API support for creating and simulating weighted policies
5. explainable decision metadata that includes the selection seed for reproducibility

This batch will not implement:

- persisted provider health snapshots
- scheduled connector or native runtime health supervision
- quota-aware admission
- SLO budget enforcement
- geo affinity or regional policy dimensions

## Options Considered

### Option A: Keep a single deterministic strategy and only reinterpret `weight`

Pros:

- smallest diff
- no storage changes

Cons:

- `weight` would still not mean actual probabilistic balancing
- no formal extension point for future routing strategies
- strategy intent would remain implicit and hard to explain

### Option B: Add explicit policy strategy plus seeded weighted selection

Pros:

- creates a durable extension seam for future routing behaviors
- keeps weighted routing reproducible in tests and admin simulations
- preserves deterministic ranking as the default strategy
- requires only a narrow schema evolution

Cons:

- introduces strategy plumbing across domain, storage, and admin layers
- requires careful eligibility rules so unhealthy or disabled candidates are not randomly favored

### Option C: Build a full pluggable routing-engine subsystem first

Pros:

- closest to a long-term micro-kernel routing engine vision
- could support third-party strategy plugins later

Cons:

- too large for the current gap
- would add framework complexity before basic strategy diversity exists
- would slow down delivery of the most obvious missing control-plane feature

## Recommendation

Use **Option B**.

The current codebase already has a clean policy boundary and assessment pipeline. The right next step is to formalize routing strategy in the policy model and route selection function, not to invent a larger plugin runtime prematurely.

## Strategy Model

Add a new `RoutingStrategy` enum in the routing domain with at least:

- `deterministic_priority`
- `weighted_random`

Policy defaults should remain deterministic so all existing stored data and callers continue to behave as before.

## Weighted Routing Semantics

Weighted random selection should work in two stages.

### Stage 1: Eligibility

Start from the same assessed candidate list already produced by health-aware routing and keep only candidates that are:

- available
- not runtime-unhealthy if at least one healthier candidate exists

This preserves the current architectural rule that unhealthy runtimes are demoted rather than erased, while still avoiding obviously bad weighted choices when healthy capacity exists.

### Stage 2: Selection

For each eligible candidate, resolve an effective weight:

- explicit `routing.weight` or root `weight` from instance config
- otherwise default weight `100`

Then:

1. sum all effective weights
2. derive a deterministic bucket from `selection_seed % total_weight`
3. walk candidates in stable ranked order until the bucket lands inside a candidate's weight range

Stable ranked order must still come from the current assessment sort so weighted selection remains explainable and predictable when weights match.

## Seed Model

Weighted routing must be testable and explainable.

Add an optional `selection_seed` input for admin simulation and a matching optional `selection_seed` output on `RoutingDecision`.

Rules:

- if the caller provides a seed, use it
- if the caller omits a seed, generate one at runtime
- always return the seed used in the decision

This gives the admin console a reproducible simulation path and allows gateway behavior to remain probabilistic in live traffic.

## Assessment and Explanation Model

Keep the current `RoutingCandidateAssessment` structure and extend routing explanations with weighted-selection context:

- strategy name
- selection seed
- total eligible weight
- candidate-level reasons such as:
  - `eligible for weighted selection`
  - `excluded from weighted selection because a healthier candidate exists`
  - `resolved weight = 250`

The system does not need a new complex score object yet. Reasons are sufficient for this batch.

## Storage Compatibility

Add a `strategy` column to `routing_policies` in both SQLite and PostgreSQL with default value `deterministic_priority`.

Compatibility rules:

- old rows load as deterministic without migration breakage
- old API clients may omit `strategy`
- the new field must round-trip through both SQLite and PostgreSQL stores

## Gateway and API Surface

No gateway handler contract needs to change for this batch.

The main surface changes are:

- `create_routing_policy` accepts optional strategy
- admin route simulation accepts optional `selection_seed`
- admin route simulation returns the effective seed

The gateway can continue calling the existing routing entrypoint, which will internally generate a seed only when weighted selection is requested.

## Testing Strategy

The batch should be test-driven around four behaviors:

1. policy strategy defaults to deterministic
2. storage round-trips the new strategy field
3. weighted selection chooses providers reproducibly for known seeds
4. unhealthy or disabled candidates do not win weighted selection when healthier eligible alternatives exist

## Follow-On Extensions

This design intentionally sets up the next routing phases:

- `quota_aware`
- `slo_aware`
- `geo_affinity`
- persisted health snapshot inputs

Those future strategies can share the same policy enum and selection dispatch point introduced here.
