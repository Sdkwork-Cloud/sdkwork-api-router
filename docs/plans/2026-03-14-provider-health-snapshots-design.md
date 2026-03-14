# Provider Health Snapshots Design

**Date:** 2026-03-14

**Status:** Approved by the existing gateway baseline and the user's standing instruction to continue autonomously

## Goal

Close the next strongest routing and runtime-governance gap by persisting provider health snapshots, exposing them through the control plane, and running lightweight background supervision that periodically records the current runtime health state.

## Why This Batch

The repository already supports:

- live connector and native dynamic runtime status inspection
- health-aware routing based on in-memory runtime status
- weighted and deterministic routing strategies
- admin APIs for viewing current runtime statuses

What is still missing relative to the original design is durable health state:

- no `routing_provider_health` persistence exists yet
- routing cannot fall back to recent health evidence when no live runtime status is available
- operators cannot inspect health history or trends
- services do not supervise runtime health over time

This is now the best next step because it deepens the existing routing and extension architecture without forcing a premature SLO or quota engine.

## Scope

This batch will implement:

1. a persisted `ProviderHealthSnapshot` domain model
2. SQLite and PostgreSQL storage for routing provider health history
3. app-layer capture of health snapshots from current runtime statuses
4. background snapshot supervision in standalone services
5. admin API support for listing recent health snapshots
6. routing fallback to recent persisted snapshots when live runtime status is absent
7. console runtime page improvements to show runtime health history

This batch will not implement:

- active HTTP probing for non-runtime official upstream providers
- hot reload of runtime packages
- quota-aware or SLO-aware routing policies
- regional affinity strategies
- retention cleanup or compaction jobs

## Options Considered

### Option A: Keep health entirely in-memory

Pros:

- no schema changes
- smallest implementation batch

Cons:

- loses health history across restarts
- cannot explain past runtime degradations
- leaves the original `routing_provider_health` design promise unimplemented

### Option B: Persist snapshots and add lightweight background supervision

Pros:

- reuses the current runtime status pipeline
- creates durable health evidence for routing and operators
- keeps implementation bounded and compatible with both standalone services and embedded mode
- prepares inputs for later SLO or quota-aware policies

Cons:

- introduces new storage surfaces and service startup tasks
- needs careful matching between provider records, extension instances, and runtime status records

### Option C: Build a full health telemetry subsystem with retention, alerts, and probes

Pros:

- closest to a production observability platform
- could support official upstream endpoint probing and alerting

Cons:

- too large for the current gap
- would slow down delivery of the most important missing persistence layer
- would require much more configuration and operational behavior than the repository currently needs

## Recommendation

Use **Option B**.

The current runtime status APIs are already normalized. Persisting them and supervising them periodically is the right architectural midpoint between the current in-memory status model and a future full telemetry subsystem.

## Domain Model

Add a `ProviderHealthSnapshot` aggregate to the routing domain.

Each snapshot should carry:

- `provider_id`
- `extension_id`
- `runtime`
- `instance_id`
- `healthy`
- `running`
- optional `message`
- `observed_at_ms`

This model is intentionally provider-centric rather than process-centric because routing decisions select providers, not raw runtime records.

## Snapshot Capture Rules

Snapshot capture should derive provider health using the same real execution identity the gateway already uses:

1. prefer exact provider-to-instance match where `ExtensionInstance.instance_id == ProxyProvider.id`
2. fall back to package-level runtime status when the runtime record has no instance ID, such as current `native_dynamic` package-level status
3. persist one snapshot per matched provider per capture pass
4. do not synthesize snapshots for providers with no resolvable runtime signal

This keeps persistence aligned with the real dispatch semantics already established in the gateway and routing code.

## Supervision Model

The supervision task should be intentionally lightweight:

- it runs in the standalone service process
- it polls the existing `list_extension_runtime_statuses()` app function
- it writes snapshots on a fixed interval

Configuration should be explicit and simple:

- `SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS`

Rules:

- `0` disables background supervision
- a positive value enables periodic capture
- one immediate capture should happen on startup before the loop begins

This gives operators durable health evidence without creating a heavy scheduler subsystem.

## Routing Fallback Model

Live runtime status remains the strongest signal.

Routing should prefer:

1. live runtime status
2. latest persisted provider health snapshot
3. unknown health

Persisted snapshots are therefore a fallback, not a replacement for live status. This keeps current deterministic and weighted routing behavior honest while improving resilience when runtime hosts temporarily expose no in-memory records.

## Admin API Surface

Add a new native control-plane endpoint:

- `GET /admin/routing/health-snapshots`

It should return recent snapshots ordered from newest to oldest.

This batch does not need a create or delete endpoint because snapshots are machine-produced.

## Console Impact

The current runtime page is mostly static. It should become a real observability surface by showing:

- current deployment/runtime posture
- recent persisted health snapshots
- whether the latest snapshots are healthy or degraded
- which provider and runtime each snapshot came from

This keeps the console aligned with the original management vision where runtime and routing are observable governance surfaces, not hidden backend internals.

## Testing Strategy

The batch should be proven through:

1. domain tests for `ProviderHealthSnapshot`
2. store tests for snapshot persistence round-trip in SQLite and PostgreSQL
3. app-extension tests for snapshot capture from runtime statuses
4. admin API tests for listing recent snapshots
5. routing tests proving fallback to persisted snapshots when no live runtime status is available
6. config tests for the new supervision interval environment variable

## Follow-On Work

After this batch, the remaining strongest governance gaps should be:

1. active endpoint probing for official upstream providers
2. retention and roll-up for long-running health history
3. quota-aware admission
4. SLO-aware routing and decision logging
5. regional policy dimensions
