# Stable Runtime Coordination Store Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without interactive sign-off

## Goal

Keep multi-node rollout coordination coherent when standalone services hot-swap `database_url` at runtime, so rollout heartbeats, participant workers, and admin rollout APIs do not split across different databases mid-process.

## Why This Batch

The repository already supports:

- restartless store replacement for standalone services
- shared-store coordinated extension-runtime rollout
- shared-store coordinated standalone-config rollout

What remains incorrect is the interaction between those capabilities.

Today the request path can move to a replacement live store after `database_url` changes, while rollout heartbeats or rollout APIs may continue to assume the previous coordination substrate. That creates a control-plane split:

1. active nodes heartbeat into one database
2. rollout creation reads another database
3. pending participants can remain permanently invisible to workers

This is a correctness issue in the current system, not a speculative future enhancement.

## Scope

This batch will implement:

1. a dedicated runtime coordination-store handle for multi-node rollout state
2. startup-time pinning of that coordination store for the current process lifetime
3. admin rollout endpoints that read and write the pinned coordination store instead of the live request-serving store
4. extension-runtime rollout workers that heartbeat and process work against the pinned coordination store
5. explicit service wiring so the startup coordination store choice is visible in binaries and tests
6. docs updates that explain the coordination-store lifetime clearly

This batch will not implement:

- live migration of rollout coordination rows from one database to another
- dual-write or replay of rollout state across multiple databases
- full distributed database cutover for the admin control plane

## Problem Boundary

`database_url` hot reload is designed to replace request-serving store dependencies safely for new requests. Multi-node rollout coordination is different:

- it depends on a shared substrate observed by multiple processes
- it needs durable heartbeats plus participant state continuity
- it cannot safely jump databases mid-rollout without a migration protocol

The correct bounded behavior is therefore:

1. keep request-serving store hot-swappable
2. keep rollout coordination on one stable store per process lifetime
3. require restart if operators want the coordination substrate itself to move

## Options Considered

### Option A: Keep using the current live store for coordination

Pros:

- no new state handle
- simple to describe

Cons:

- creates rollout split-brain immediately after `database_url` hot swap
- can strand pending participants in the previous database
- makes coordination behavior timing-dependent and hard to reason about

### Option B: Pin a dedicated coordination store at startup

Pros:

- keeps rollout heartbeats, creation, listing, and workers on one substrate
- preserves existing request-serving store hot-swap behavior
- small, testable change that matches the current architecture boundary

Cons:

- rollout control plane does not migrate automatically with `database_url`
- operators need a restart to move distributed coordination onto a new database

### Option C: Build a live coordination-store migration layer

Pros:

- would preserve hot-swap semantics for both request-serving and coordination data

Cons:

- requires migration, bridging, or dual-store reads and writes
- much larger than the current bug
- easy to get wrong in multi-node deployments

## Recommendation

Use **Option B**.

This fixes the real correctness problem with the smallest coherent change. The system should be explicit that request-serving store replacement and distributed coordination-store migration are different concerns.

## Coordination Semantics

Each standalone process should derive one `coordination_store` from the startup admin store and keep it for the lifetime of that process.

That pinned handle is used for:

- service-runtime node heartbeats
- extension-runtime rollout creation, listing, and detail reads
- standalone-config rollout creation, listing, and detail reads
- extension-runtime rollout workers
- standalone-config rollout workers

The hot-swappable live store continues to serve:

- admin request handlers for catalog, routing, tenants, usage, billing, credentials, and similar data APIs
- request-serving gateway and portal state
- local runtime reload of database-backed dependencies

## Failure Model

This batch makes the current limitation explicit:

1. if `database_url` changes, rollout coordination remains on the startup coordination store
2. new rollout APIs still work because they read the same pinned substrate as heartbeats and workers
3. if operators need rollout coordination to move to the new database, they must restart the process after cutover

That is preferable to silently splitting the control plane.

## Testing Strategy

This batch should be proven with:

1. admin API tests showing extension-runtime rollout creation still succeeds after the live store handle swaps to a replacement database
2. admin API tests showing standalone-config rollout creation still succeeds after the live store handle swaps
3. runtime tests showing extension-runtime rollout workers continue consuming work from the startup coordination store after the live store handle swaps
4. full workspace verification so the coordination fix does not regress other runtime-control behavior
