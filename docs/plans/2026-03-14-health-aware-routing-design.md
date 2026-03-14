# Health-Aware Routing Design

**Date:** 2026-03-14

**Status:** Approved by the existing gateway, extension runtime, and control-plane baseline for direct implementation

## Goal

Close the strongest remaining platform gap in the original gateway design by turning routing from a static policy ordering mechanism into a runtime-aware decision process that considers health and operator-configured cost or weight hints.

## Why This Batch

The current repository already supports:

- policy selection by capability, model pattern, and priority
- ordered provider preference with optional default provider fallback
- real runtime status visibility for:
  - connector runtimes
  - native dynamic runtimes
- extension instance config persistence

What is still missing is the architecture promised in the original design:

- routing decisions are not health-aware
- runtime status does not influence candidate ranking
- route simulation cannot explain why one provider was chosen over another
- the console cannot show score or health reasons behind routing outcomes

This is now the highest-leverage gap because the control plane, extension runtime, and runtime status signals already exist. The missing piece is joining them into the routing service.

## Scope

This batch will implement:

1. runtime-aware provider candidate assessment in `sdkwork-api-app-routing`
2. health-informed ranking for provider candidates
3. operator-configured routing hints read from extension instance config
4. richer `RoutingDecision` metadata with per-candidate explanation
5. admin API and console support for visualizing candidate assessment and selection reasons

This batch will not implement:

- active background health polling for official upstream HTTP providers
- persisted latency histograms or SLO budgets
- quota-aware admission or billing-based hard rejection
- weighted random load balancing across multiple live providers

## Options Considered

### Option A: Keep routing policy static and only expose runtime statuses separately

Pros:

- no routing behavior changes
- very small implementation batch

Cons:

- leaves the central architecture gap unresolved
- forces operators to manually correlate routing and runtime screens
- does not improve real gateway execution

### Option B: Add runtime-aware deterministic scoring using existing state

Pros:

- reuses current store, extension instance config, and runtime status data
- keeps behavior deterministic and easy to reason about
- improves both admin simulation and real gateway dispatch immediately
- avoids schema churn in the current batch

Cons:

- only as strong as the available runtime signals
- latency and quota remain declarative hints rather than observed facts

### Option C: Build a full routing telemetry subsystem first

Pros:

- closest to the long-term routing vision
- supports richer policy dimensions later

Cons:

- too large for the current batch
- requires new persistence, collectors, and probe jobs
- would delay obvious value already unlocked by the current runtime host

## Recommendation

Use **Option B**.

The platform already has enough structure to make routing health-aware today. A deterministic scoring layer closes the architectural gap without prematurely introducing a new telemetry subsystem.

## Routing Inputs

Candidate routing should use four input classes:

1. `Catalog availability`
   Derived from provider records and model catalog entries.
2. `Policy ordering`
   Derived from `RoutingPolicy`.
3. `Control-plane enablement`
   Derived from extension installation and instance state.
4. `Runtime health and operator hints`
   Derived from:
   - extension runtime statuses
   - extension instance config

## Candidate Metadata Standard

The routing service should support a normalized per-candidate assessment model.

Each candidate assessment should include:

- `provider_id`
- `available`
- `healthy`
- `policy_rank`
- `weight`
- optional `cost`
- optional `latency_ms`
- `reasons`

This model should remain generic enough to cover:

- built-in providers without extension instances
- connector-backed providers
- native dynamic providers

## Health Model

Health is intentionally conservative in this batch.

Rules:

1. if a provider resolves to one or more enabled extension instances and at least one matching runtime status exists, the provider is healthy when any matching runtime is healthy
2. if a provider resolves only to unhealthy matching runtime statuses, the provider is unhealthy
3. if no runtime signal exists for the provider, treat health as unknown but not failed
4. disabled installation or disabled instance state makes the candidate unavailable

This gives runtime-backed providers meaningful health-aware ordering while preserving compatibility for providers that do not currently expose runtime health.

## Operator Hint Model

To avoid schema churn, this batch will reuse extension instance config.

Supported routing hints:

- `weight`
- `cost`
- `latency_ms`

The router should read:

1. root-level keys for backward compatibility
2. `routing.*` nested keys for the standardized future shape

Example:

```json
{
  "region": "global",
  "weight": 100,
  "routing": {
    "cost": 0.35,
    "latency_ms": 120
  }
}
```

## Ranking Strategy

Ranking must stay deterministic.

Recommended comparison order:

1. `available = true` before `false`
2. `healthy = true` before unhealthy when health is known
3. lower `cost`
4. lower `latency_ms`
5. higher `weight`
6. lower policy rank index
7. lexicographic `provider_id`

This keeps:

- explicit policy ordering relevant
- health failures able to demote bad runtimes
- cost and latency hints meaningful
- selection reproducible

## Decision Output

`RoutingDecision` should remain backward compatible but gain richer explanation.

In addition to:

- `selected_provider_id`
- `candidate_ids`
- `matched_policy_id`

it should also expose:

- `strategy`
- `selection_reason`
- `assessments`

This lets the admin API and console present:

- why a provider won
- why others were standby or demoted
- whether runtime health or config hints affected the outcome

## Real Gateway Impact

The real gateway already uses `simulate_route_with_store` to choose providers before dispatch.

That means this batch improves both:

- admin simulation
- live request routing

without requiring a separate second integration layer.

## Console Impact

The routing console page should evolve from a minimal candidate list into a basic decision explainer.

It should show:

- selected provider
- decision strategy
- candidate count
- per-candidate availability and health
- cost or latency hints when present
- human-readable reasons

This keeps the console aligned with the approved control-plane architecture where routing is a governance surface, not just a hidden backend helper.

## Testing Strategy

The batch will be proven through:

1. domain tests for enriched routing decision metadata
2. application tests for:
   - unhealthy runtime demotion
   - lower-cost healthy provider preference
   - disabled instance exclusion
3. admin route tests for the richer simulation payload
4. console typecheck coverage for the expanded routing result model

## Follow-On Work

After this batch, the strongest remaining routing gaps should be:

1. persisted provider health snapshots and historical trend views
2. active health probing for non-runtime upstream providers
3. probabilistic weighted balancing and failover policies
4. quota and SLO-aware admission control
