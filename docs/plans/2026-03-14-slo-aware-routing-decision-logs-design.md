# SLO-Aware Routing And Decision Logs Design

**Date:** 2026-03-14

**Status:** Approved by the original gateway roadmap and the user's standing instruction to continue autonomously without pausing for approval checkpoints

## Goal

Close the next strongest governance gap by turning routing from an explainable candidate ranking system into an SLO-aware selection system with persisted decision logs for both gateway execution and admin simulation.

## Why This Batch

The repository already supports:

- persisted routing policies
- runtime-aware candidate assessment
- weighted and deterministic routing strategies
- persisted provider health snapshots
- project-scoped quota-aware admission
- admin routing simulation and console visualization of candidate assessment

What is still missing relative to the original design is the governance loop around routing quality itself:

- routing cannot express first-class SLO intent
- there is no persisted audit trail for why a provider was selected
- operators cannot inspect recent real routing decisions outside a one-off simulation response
- future policy dimensions such as geo affinity still have no durable decision log substrate

This is now the highest-leverage batch because quota closes budget admission, while SLO-aware routing closes execution-quality governance.

## Scope

This batch will implement:

1. a new `slo_aware` routing strategy
2. SLO fields on routing policies for:
   - `max_cost`
   - `max_latency_ms`
   - `require_healthy`
3. richer candidate assessment fields for SLO eligibility and violations
4. persisted routing decision logs in SQLite and PostgreSQL
5. log emission for:
   - admin routing simulations
   - real gateway routing decisions
6. admin APIs for listing recent routing decision logs
7. console visibility for recent routing decisions

This batch will not implement:

- hard routing rejection when no candidate meets the SLO
- geo or regional affinity
- provider-specific monthly budget policies
- adaptive latency sampling from live upstream timings
- connector or native dynamic hot reload

## Options Considered

### Option A: Decision logs only

Pros:

- smallest schema change
- gives operators a history view quickly

Cons:

- routing still has no SLO intent
- logs would mostly capture the same deterministic or weighted behavior already visible in simulation

### Option B: SLO-aware routing only

Pros:

- materially improves selection behavior
- minimal UI work

Cons:

- weak operator trust because degraded or surprising decisions remain opaque after the fact
- does not build the audit substrate needed for future geo and policy debugging

### Option C: SLO-aware routing plus persisted decision logs

Pros:

- closes both policy and observability gaps together
- reuses the current assessment pipeline cleanly
- creates a durable governance substrate for later geo and provider-budget dimensions
- keeps first version bounded by using deterministic best-effort fallback instead of hard failure

Cons:

- touches more layers in one batch
- requires careful routing-store and gateway integration to avoid excessive churn

## Recommendation

Use **Option C**.

The current codebase already has the hard part: a normalized routing assessment pipeline. The right next step is to let policies express a small SLO profile and then persist the resulting decision for later inspection.

## Strategy Model

Add a new routing strategy:

- `slo_aware`

This strategy should remain deterministic in the first version.

It should reuse the current ranked assessment list, but apply an SLO eligibility pass before selecting the final provider.

## Routing Policy Model

Extend `RoutingPolicy` with optional SLO fields:

- `max_cost`
- `max_latency_ms`
- `require_healthy`

Rules:

1. these fields are meaningful for `slo_aware`
2. they can be stored on any policy, but non-`slo_aware` strategies ignore them in selection
3. `require_healthy` defaults to `false` to preserve backward compatibility

This keeps the model small and avoids inventing a second policy aggregate prematurely.

## Candidate Assessment Model

Extend `RoutingCandidateAssessment` with:

- `slo_eligible`
- `slo_violations`

Violation examples:

- `health requirement not satisfied`
- `cost 0.40 exceeds max_cost 0.25`
- `latency 450ms exceeds max_latency_ms 200`
- `latency evidence missing for required SLO threshold`

This keeps the decision explainable both in simulation and in persisted logs.

## Selection Semantics

For `slo_aware`:

1. compute the current ranked candidate assessment list first
2. evaluate each candidate against the policy's SLO fields
3. if one or more candidates are SLO-eligible, choose the top-ranked eligible candidate
4. if no candidates are SLO-eligible, fall back to the top-ranked candidate overall
5. record in the decision reason whether the route was SLO-compliant or degraded

This first version is intentionally best-effort rather than rejecting traffic. Quota already governs hard admission. SLO-aware routing should govern quality-biased selection.

## Decision Log Model

Add a persisted `RoutingDecisionLog` aggregate with:

- `decision_id`
- `decision_source`
- `tenant_id`
- `project_id`
- `capability`
- `route_key`
- `selected_provider_id`
- `matched_policy_id`
- `strategy`
- `selection_seed`
- `selection_reason`
- `slo_applied`
- `slo_degraded`
- `created_at_ms`
- `assessments`

`decision_source` should distinguish:

- `gateway`
- `admin_simulation`

`tenant_id` and `project_id` should be optional because admin simulations are not always tied to a project context.

Store `assessments` as JSON in the first version. That keeps the schema small while preserving the full decision evidence.

## Gateway Integration

The real gateway should emit a decision log whenever it selects a provider for an upstream-capable route.

To avoid scattering logging logic across handlers, selection should move through one application-layer entry point that:

1. computes the routing decision
2. persists the decision log
3. returns the selected provider context

This keeps the gateway and routing layers aligned and avoids handler-level duplication.

## Admin API Surface

Add:

- `GET /admin/routing/decision-logs`

Enhance:

- `POST /admin/routing/simulations`

Simulation responses should still return the decision inline as they do today, but the simulation should also persist a decision log with `decision_source = admin_simulation`.

## Console Impact

The routing console should start showing:

- recent decision logs
- selected provider
- source
- policy
- strategy
- SLO degraded vs compliant state

This keeps routing observable as a governance system rather than a black-box picker.

## Testing Strategy

The batch should be proven through:

1. routing domain tests for:
   - `slo_aware` strategy
   - assessment SLO eligibility metadata
   - decision log aggregate construction
2. SQLite and PostgreSQL persistence tests for decision logs and extended routing policies
3. app-routing tests for:
   - preferring SLO-compliant candidates
   - degrading gracefully when no candidate satisfies the SLO
   - persisting decision logs
4. admin API tests for:
   - listing decision logs
   - simulation log persistence
5. gateway tests proving real dispatch writes routing decision logs
6. console typecheck coverage for new log payloads

## Follow-On Work

After this batch, the strongest remaining governance gaps should be:

1. regional affinity or geo dimensions
2. rolling-window quota and provider-budget dimensions
3. official upstream active probing with live latency evidence
4. hot reload and runtime unload orchestration for extensions
