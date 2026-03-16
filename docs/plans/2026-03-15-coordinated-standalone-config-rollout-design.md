# Coordinated Standalone Config Rollout Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the strongest remaining multi-process runtime-control gap by letting one admin node coordinate an explicit standalone-config reload rollout across active gateway, admin, and portal nodes that share the same admin-store database.

## Why This Batch

The repository already supports:

- restartless single-process replacement of store, JWT, listener, and secret-manager settings
- shared-store coordinated extension-runtime rollout across multiple gateway and admin nodes
- request-scoped live-handle snapshots so new requests observe the latest store or JWT values safely

What still remains missing is a first-class way to coordinate when multiple processes pick up already-staged local config file changes.

Today each standalone process polls its own config file and applies changes independently. That is safe, but it leaves operators without:

1. a durable multi-node control-plane record for runtime-config reload
2. a way to trigger all relevant nodes on demand instead of waiting for polling cadence
3. an aggregate progress view for store, JWT, listener, or secret-manager rollout across processes

## Scope

This batch will implement:

1. cluster-scoped standalone-config rollout records plus per-node participant status in the shared admin store
2. admin API endpoints for create, list, and inspect standalone-config rollouts
3. runtime-supervision participation for gateway, admin, and portal nodes
4. per-node heartbeats for all standalone services that participate in coordinated config rollout
5. on-demand execution of one local config reload pass against the node's current on-disk config
6. aggregate rollout status reporting as `pending`, `applying`, `succeeded`, `failed`, or `timed_out`

This batch will not implement:

- cross-node config file synchronization
- central config version storage in the admin database
- exactly-once distributed transactions
- staged waves, canaries, quorum policies, cancellation, or retry controls
- convergence verification that every node loaded byte-identical config content

## Problem Boundary

The current architecture deliberately keeps standalone config files local to each process. That means the correct product capability is not "push the same config bytes everywhere". The correct capability is:

1. discover which nodes are currently active
2. snapshot the intended rollout participants
3. ask each node to execute its existing local config reload path now
4. persist durable per-node success or failure evidence

This keeps the control plane honest about what the system actually knows.

## Options Considered

### Option A: Keep polling-only behavior

Pros:

- zero new coordination code
- preserves current local-file ownership model

Cons:

- operators still have no durable rollout record
- no on-demand coordination point across multiple processes
- leaves the strongest remaining runtime-control gap open

### Option B: Shared-store coordinated local reload pass

Pros:

- fits the existing standalone architecture and shared admin store
- reuses the already safe single-process reload implementation
- gives durable aggregate and per-node rollout visibility
- works for gateway, admin, and portal without inventing node addresses

Cons:

- does not guarantee local config contents are identical
- requires new schema, API, and runtime-supervision hooks

### Option C: Direct HTTP fan-out to every node

Pros:

- immediate command delivery when every node is routable

Cons:

- requires routable per-node control-plane addresses and discovery metadata
- introduces network topology assumptions the product does not model today
- harder to test and less compatible with SQLite-backed deployments

## Recommendation

Use **Option B**.

This closes the real operational gap without pretending the product has a centralized distributed config system when it does not.

## Coordination Model

### Node Identity And Liveness

Reuse the existing `service_runtime_nodes` heartbeat ledger.

Rollout-capable services are now:

- `gateway`
- `admin`
- `portal`

Each node uses:

- `SDKWORK_SERVICE_INSTANCE_ID` when provided
- otherwise a synthesized identifier derived from service kind, process id, and startup time

Nodes are considered active for config rollout targeting only when their heartbeat is within the freshness window.

### Rollout Records

Each standalone-config rollout stores:

- `rollout_id`
- optional requested `service_kind`
- `created_by`
- `created_at_ms`
- `deadline_at_ms`

At creation time the admin API snapshots currently active nodes matching the optional service-kind selector and creates one participant record per node.

### Participant Records

Each participant record stores:

- `rollout_id`
- `node_id`
- `service_kind`
- `status`
- optional `message`
- `updated_at_ms`

Allowed worker-written statuses:

- `pending`
- `applying`
- `succeeded`
- `failed`

Timed out remains a derived aggregate status.

## Rollout Semantics

### Creation

When an operator creates a standalone-config rollout:

1. optionally filter target nodes by one service kind
2. load currently active standalone nodes from the shared store
3. reject the request if no active matching nodes exist
4. insert one rollout row
5. insert one pending participant row per targeted node
6. return the rollout snapshot immediately

### Node Processing

Each standalone service already runs runtime supervision for local config reload. That supervisor should also:

1. heartbeat the node identity when rollout participation is enabled
2. poll pending standalone-config participant rows for its own node
3. mark one participant as `applying`
4. run one forced local config reload pass against its current config loader
5. mark the participant `succeeded` or `failed`

The forced reload pass reuses the existing store, JWT, listener, secret-manager, and extension-policy reload logic. It is not a second implementation path.

### Success Semantics

Participant success means:

- the node completed one local config reload pass successfully

That pass may either:

- apply one or more reloadable changes, or
- detect that there were no effective local changes to apply

Both outcomes are operationally valid and should be reported honestly in the participant message.

### Aggregate Status

Aggregate rollout status is derived from participant rows plus deadline:

- `succeeded`:
  - every participant succeeded
- `failed`:
  - at least one participant failed
- `timed_out`:
  - deadline passed and at least one participant is still pending or applying
- `applying`:
  - at least one participant is applying and none failed
- `pending`:
  - participants exist but none started yet

## API Shape

Add:

- `POST /admin/runtime-config/rollouts`
- `GET /admin/runtime-config/rollouts`
- `GET /admin/runtime-config/rollouts/{rollout_id}`

Create request:

- optional `service_kind`
- optional `timeout_secs`

Response should include:

- `rollout_id`
- `status`
- optional `requested_service_kind`
- `created_by`
- `created_at_ms`
- `deadline_at_ms`
- `participant_count`
- `participants`

## Runtime Placement

This capability belongs in `sdkwork-api-app-runtime`.

Reasons:

- it is service-local background orchestration
- it must reuse the existing local config reload machinery directly
- it already owns the standalone-service lifecycle and live handles

The implementation should extract the current one-off config reload pass from the polling loop into a reusable helper, then call that helper from both:

1. watch-state driven reload
2. shared-rollout driven reload

The runtime should keep a dedicated coordination-store handle for rollout heartbeats and participant transitions. By default that handle should pin the startup store snapshot so a hot-swapped `database_url` does not move an in-flight rollout ledger to a different database unexpectedly.

## Failure Model

This batch should be explicit and pragmatic:

1. a node can fail its reload because local config is invalid or a reloadable transition fails
2. other nodes continue independently
3. if a node disappears, its participant stays pending or applying until timeout
4. the control plane does not claim config convergence beyond what each node reports

## Testing Strategy

This batch should be proven with:

1. storage tests for standalone-config rollout records and participant updates
2. admin API tests proving rollout creation snapshots active nodes and service-kind filtering works
3. runtime-supervision tests proving two nodes can consume the same rollout and update live runtime state
4. timeout tests proving stale pending participants aggregate to `timed_out`
5. full workspace verification so the new coordination hooks do not regress existing runtime reload behavior

## Follow-On Work

After this batch, the remaining distributed runtime-control work should be more policy-oriented:

1. rollout cancellation or retry controls
2. staged waves or canary strategies
3. stronger convergence evidence for local config content or version identity
