# Multi-Node Coordinated Runtime Rollout Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without interactive sign-off

This design has since been complemented by a follow-on shared-store capability for coordinated standalone config rollout across gateway, admin, and portal nodes.

## Goal

Close the strongest remaining runtime-control gap by letting one admin node coordinate explicit extension-runtime rollout across all active standalone gateway and admin nodes that share the same control-plane database.

## Why This Batch

The repository already supports:

- process-local extension runtime reload
- targeted reload scope selection
- automatic extension hot reload
- request draining and timeout rollback for native-dynamic runtimes
- restartless single-process store, JWT, listener, and secret-manager reconfiguration

What still remains missing is coordination across multiple processes.

Today `POST /admin/extensions/runtime-reloads` only affects the current admin process. In a real deployment with multiple gateway and admin nodes behind load balancers, that means operators still need to fan out the same runtime control action node by node. The design corpus now treats that as the strongest remaining operational gap.

## Scope

This batch will implement:

1. active standalone service instance heartbeats stored in the shared admin database
2. cluster-scoped extension-runtime rollout records and per-node participant status
3. a background rollout worker in gateway and admin services that claims and applies pending rollout work for its own node
4. admin API endpoints that create, list, and inspect cluster rollout operations
5. cluster-coordinated rollout for explicit extension-runtime reload scopes:
   - all runtimes
   - one extension
   - one connector instance
6. status aggregation that reports pending, applying, succeeded, failed, or timed out rollout state

This batch will not implement:

- cross-node config-file synchronization
- distributed rollout for database, JWT, listener, or secret-manager config changes
- leader election or exactly-once distributed transactions
- automatic rollback to a previous extension package version
- external queue or consensus dependencies beyond the shared admin store

## Problem Boundaries

The existing runtime reload contract is already process-local and safe. The missing piece is an operator-visible control plane that can:

1. discover which nodes are currently active
2. snapshot the intended rollout participants
3. let each node apply the same scoped reload locally
4. surface per-node progress and failures back through the admin API

The best implementation is therefore a shared-store coordination layer, not a second transport control plane.

## Options Considered

### Option A: Keep manual fan-out outside the product

Pros:

- zero new coordination code
- avoids distributed-state modeling

Cons:

- leaves the documented strongest remaining gap open
- makes rollout observability external and inconsistent
- gives operators no first-class per-node status model

### Option B: Shared-database rollout ledger with node heartbeats

Pros:

- fits the current architecture because all standalone services already depend on the admin store
- needs no new infrastructure beyond SQLite or PostgreSQL
- works for both gateway and admin nodes
- gives the admin API durable rollout progress and per-node status

Cons:

- not a strict consensus system
- requires schema, API, and worker additions

### Option C: Push-based control plane over direct HTTP calls to every node

Pros:

- immediate delivery semantics when every node is reachable
- no background polling worker needed

Cons:

- requires per-node discovery plus routable control-plane addresses
- adds network topology assumptions the current product does not model
- harder to verify locally and across SQLite-backed deployments

## Recommendation

Use **Option B**.

The current product already has one shared coordination substrate: the admin store. A shared-store rollout ledger is the smallest correct extension of the existing architecture, and it can be exercised in tests without inventing new deployment assumptions.

## Coordination Model

### Service Nodes

Gateway and admin processes should heartbeat an active node record into the shared store.

Each node record carries:

- `node_id`
- `service_kind`
- `started_at_ms`
- `last_seen_at_ms`

`node_id` should be:

- `SDKWORK_SERVICE_INSTANCE_ID` if provided in the process environment
- otherwise an auto-generated identifier derived from service kind, process id, and startup time

Node liveness is heartbeat-based. A node is considered active for rollout targeting only if its latest heartbeat is within the freshness window.

### Rollout Operations

Each cluster rollout stores:

- `rollout_id`
- requested reload scope
- requested and resolved runtime identifiers
- `created_by`
- `created_at_ms`
- `deadline_at_ms`

At rollout creation time, the admin API snapshots the currently active target nodes and creates one participant record per node.

### Participant Records

Each participant record stores:

- `rollout_id`
- `node_id`
- `service_kind`
- `status`
- `message`
- `updated_at_ms`

Allowed statuses:

- `pending`
- `applying`
- `succeeded`
- `failed`

Timed out state is aggregated, not directly written by workers.

## Rollout Semantics

### Creation

When an operator creates a cluster rollout:

1. validate the requested scope exactly like the existing process-local reload endpoint
2. load currently active gateway and admin nodes from the shared store
3. reject the request if no active rollout-capable nodes exist
4. create one rollout operation row
5. create participant rows for the targeted nodes
6. return the created rollout plus participant snapshot immediately

### Node Processing

Each active gateway or admin node runs a polling worker:

1. heartbeat its node record
2. find pending participant rows for its own `node_id`, oldest first
3. mark one participant as `applying`
4. execute the existing local scoped runtime reload
5. mark that participant `succeeded` or `failed` with a message

Workers do not need cross-node locks because each participant row is owned by exactly one node.

### Aggregated Rollout Status

Rollout status is computed from participant state plus the rollout deadline:

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

This derived-status model avoids fragile multi-row transactional updates.

## API Shape

Keep the existing local endpoint:

- `POST /admin/extensions/runtime-reloads`

Add cluster endpoints:

- `POST /admin/extensions/runtime-rollouts`
- `GET /admin/extensions/runtime-rollouts`
- `GET /admin/extensions/runtime-rollouts/:rollout_id`

`POST /admin/extensions/runtime-rollouts` accepts the same scope selectors as the local reload endpoint plus an optional timeout:

- `extension_id`
- `instance_id`
- `timeout_secs`

The admin API should resolve instance requests the same way the existing local reload endpoint does:

- connector instance => instance-scoped rollout
- builtin or native-dynamic instance => extension-scoped rollout

## Runtime Worker Placement

The rollout worker belongs in `sdkwork-api-app-runtime`.

Reasons:

- it is service-local background orchestration
- it already owns standalone runtime supervision
- it already has access to service kind and live store handles

The worker should operate against the live store handle so store replacement keeps cluster coordination on the active database.

## Failure Model

This batch needs pragmatic, explicit failure behavior:

1. If one node fails the local reload, the rollout becomes failed.
2. Other nodes may still continue processing their own participant rows.
3. If a node disappears and stops heartbeating, its participant remains pending or applying.
4. Once the rollout deadline passes, aggregate status becomes timed out.
5. Operators can inspect participant-level messages to decide what to do next.

This is not atomic distributed rollout, but it is operationally honest and far better than blind manual fan-out.

## Testing Strategy

This batch should be proven with:

1. storage tests for node heartbeat persistence and rollout participant updates
2. admin API tests proving rollout creation snapshots active nodes and returns aggregated status
3. runtime worker tests proving two independent nodes sharing one store both pick up and complete the same rollout
4. timeout tests proving stale pending participants aggregate to `timed_out`
5. regression coverage proving the old process-local reload endpoint still works unchanged

## Follow-On Work

After this batch, the remaining runtime-control gap should be narrower and more policy-oriented:

1. optional rollout cancellation or retry controls
2. more advanced rollout strategies such as staged waves or quorum policies
