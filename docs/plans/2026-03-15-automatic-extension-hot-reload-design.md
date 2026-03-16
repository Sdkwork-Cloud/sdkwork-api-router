# Automatic Extension Hot Reload Design

**Date:** 2026-03-15

**Status:** Approved for direct implementation under the standing autonomous delivery instruction

## Goal

Close the next strongest extension-runtime gap by adding automatic process-local hot reload supervision that watches configured extension search paths and invokes the existing runtime reload orchestration whenever those paths change.

## Why This Batch

The repository already supports:

- manifest-based extension discovery from configured search paths
- connector and native-dynamic runtime shutdown plus rebuild through explicit reload orchestration
- process-local runtime status inspection and health snapshot supervision
- local config runtime with stable extension-path resolution

What still remained missing across the design corpus was true automatic refresh:

- explicit reload exists, but it must be triggered manually
- runtime-mode docs still describe watcher-driven hot reload as future work
- package changes on disk are therefore not applied until an operator intervenes

This is now the highest-value next step because the dangerous part of reload semantics is already implemented. What remains is change detection and safe process-local triggering.

## Scope

This batch will implement:

1. a polling-based extension tree watcher for configured search paths
2. automatic process-local invocation of the existing reload orchestrator when the watched tree changes
3. standalone config support for `extension_hot_reload_interval_secs`
4. startup wiring in standalone gateway and admin services

This batch will not implement:

- OS-specific native filesystem watcher backends
- hot reload for runtime config file changes or arbitrary environment mutations
- per-extension or per-instance selective reload
- coordinated multi-node rollout

## Options Considered

### Option A: Keep explicit reload only

Pros:

- smallest code surface
- no background tasks

Cons:

- disk changes still require manual operator action
- leaves the most visible remaining runtime gap unresolved

### Option B: Add a polling-based watcher over extension search paths

Pros:

- cross-platform with no new native watcher dependency
- easy to reason about with current process-local reload semantics
- reuses the existing shutdown and rebuild path directly

Cons:

- change detection is interval-based rather than instant
- watches extension trees only, not all config inputs

### Option C: Add full native filesystem watcher integration

Pros:

- closest to classic hot reload
- lower latency than polling

Cons:

- larger dependency and platform-behavior surface
- more fragile around library replacement, transient events, and cross-platform semantics

## Recommendation

Use **Option B**.

It is the best architectural midpoint: small enough to land safely now, but complete enough to remove the “manual-only reload” limitation from the runtime story.

## Supervision Model

Each standalone process should supervise its own extension-runtime state.

Rules:

1. build a baseline signature from the current extension discovery policy and extension tree fingerprint
2. poll on a fixed interval
3. when either the discovery-policy cache key or the watched tree fingerprint changes, call the existing `reload_configured_extension_host()`
4. update the baseline only after a successful reload

This preserves the current process-local semantics and avoids pretending that an admin API call in one process can mutate another process directly.

## Fingerprint Rules

The watched fingerprint should be intentionally simple and deterministic.

Include:

- configured extension search path entries
- file and directory paths
- file sizes
- last-modified timestamps
- explicit markers for missing watched roots

Sort the collected entries before comparison so the result is stable across directory enumeration order.

## Safety Rules

1. `extension_hot_reload_interval_secs = 0` disables automatic hot reload
2. reload remains best-effort and process-local
3. failed reloads must not advance the baseline, so the process keeps retrying after the next observed interval
4. background supervision should log failures rather than panic

## Configuration Contract

Add:

- config file field: `extension_hot_reload_interval_secs`
- environment variable: `SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS`

Default:

- `0`

This matches the existing `runtime_snapshot_interval_secs` style and keeps enablement explicit.

## Testing Strategy

This batch should be proven with:

1. config tests proving the new hot-reload interval parses correctly
2. gateway tests proving a watched extension-tree change triggers shutdown plus rebuild for a native-dynamic runtime
3. workspace verification to ensure the new background supervisor wiring does not regress service builds

## Follow-On Work

After this batch, the coordinated multi-node rollout follow-on has since been implemented for explicit extension-runtime control through the shared admin store.

The remaining runtime-control gaps are now narrower:

1. coordinated rollout for broader non-extension runtime changes
