# Builtin Upstream Health Probing Design

**Date:** 2026-03-15

**Status:** Approved for direct implementation under the standing autonomous delivery instruction

## Goal

Close the next observability and routing gap by actively probing official builtin upstream providers when no live runtime status exists, so provider health snapshots and routing fallback work for first-party HTTP upstreams as well as managed runtimes.

## Why This Batch

The repository already supports:

- persisted `ProviderHealthSnapshot` history
- routing fallback to the latest persisted health evidence
- runtime-backed health capture for connector and native dynamic runtimes
- explicit runtime reload for managed extension hosts

What remained missing relative to the design baseline was builtin upstream coverage:

- builtin OpenAI, OpenRouter, and Ollama providers do not register runtime status records
- snapshot capture currently skips providers with no runtime signal
- routing therefore cannot benefit from recent health evidence for builtin HTTP upstreams unless snapshots are inserted manually

This is now the best next batch because it extends the existing health snapshot architecture without forcing a larger telemetry subsystem.

## Scope

This batch will implement:

1. active HTTP probing for official builtin upstream providers during provider health snapshot capture
2. support for instance or installation `health_path` overrides when operators want a non-default probe target
3. explicit runtime-family labeling of probed snapshots as `builtin`
4. negative guardrails so connector and native dynamic providers are not misclassified as probe-only builtin upstreams

This batch will not implement:

- background latency histograms or rolling SLO budgets
- credential-aware authenticated probes
- quota-aware admission or rate-limit-specific routing behavior
- generalized probing for arbitrary third-party or custom extension runtimes

## Options Considered

### Option A: Keep builtin providers on persisted or manual evidence only

Pros:

- no new HTTP traffic
- smallest code change

Cons:

- builtin providers still have no automatic health evidence
- routing fallback remains incomplete for the default provider families

### Option B: Probe builtin upstreams inline during the existing snapshot capture pass

Pros:

- reuses the current snapshot supervision loop and persistence model
- keeps runtime-backed providers and builtin providers on one common health evidence path
- bounded implementation with no new scheduler or schema changes

Cons:

- introduces outbound probe requests during snapshot capture
- requires conservative interpretation of unauthenticated HTTP responses

### Option C: Build a separate active telemetry worker with richer probe semantics

Pros:

- best long-term foundation for latency, quota, and SLO-aware routing
- could add per-provider concurrency or retention policies later

Cons:

- too large for the current gap
- duplicates infrastructure the current supervision loop already provides

## Recommendation

Use **Option B**.

The snapshot supervision loop already has the correct lifecycle and persistence contract. Extending it with bounded builtin upstream probes closes the functional gap with minimal architecture churn.

## Probe Eligibility Model

Only providers that clearly map to the official builtin HTTP families should be actively probed.

Rules:

1. if a live runtime status exists, keep using runtime-backed capture and do not probe
2. if a provider resolves to a connector or native dynamic installation, do not probe
3. if a provider uses the official builtin extension IDs or a builtin installation with a supported compatibility family, probe it

This avoids incorrectly treating managed runtimes as simple HTTP upstreams.

## Probe Target Rules

Probe targets should stay aligned with the current execution semantics.

Rules:

1. prefer `ExtensionInstance.base_url` when present because runtime dispatch already honors instance overrides
2. otherwise use `ProxyProvider.base_url`
3. prefer an explicit `health_path` from instance config
4. otherwise fall back to installation-level `health_path`
5. otherwise use the builtin default path `/v1/models` for the current OpenAI-compatible builtin families

The current builtin OpenAI, OpenRouter, and Ollama adapters all execute against the same OpenAI-compatible path family, so `/v1/models` is the correct default probe path for this batch.

## Health Interpretation Rules

Builtin upstream probes are intentionally conservative and unauthenticated.

Interpretation:

- `2xx` means `running = true` and `healthy = true`
- `401`, `403`, or `429` means the upstream is reachable and should still be treated as healthy for transport availability
- other HTTP responses mean `running = true` and `healthy = false`
- connection or URL failures mean `running = false` and `healthy = false`

This keeps routing health focused on endpoint availability while leaving credential, quota, and policy-specific admission to later batches.

## Snapshot Model

Builtin probe snapshots should reuse the existing `ProviderHealthSnapshot` aggregate.

Fields:

- `runtime = "builtin"`
- `instance_id` when a matching extension instance exists
- `message` containing the probe path and the result summary

This keeps builtin probe evidence compatible with the existing admin API and routing fallback logic.

## Testing Strategy

This batch should be proven through app-extension observability tests that cover:

1. a builtin provider with no runtime status producing a healthy snapshot from the default `/v1/models` probe
2. a builtin provider with explicit `health_path` producing an unhealthy snapshot when the probe endpoint returns failure
3. a connector-managed provider with no runtime status not being probed at all

## Follow-On Work

After this batch, the remaining strongest health-routing gaps should be:

1. authenticated or provider-specific probe strategies where transport-only reachability is insufficient
2. latency-aware builtin probing and histogram persistence
3. quota-aware and SLO-aware admission that can distinguish healthy-but-throttled providers from true transport degradation
