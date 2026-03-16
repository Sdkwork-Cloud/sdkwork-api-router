# Native-Dynamic Drain Timeout And Rollback Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the next strongest native-dynamic runtime-control gap by adding a configurable drain timeout and safe rollback policy so reload or shutdown does not block forever when a plugin invocation wedges.

## Why This Batch

The repository now already supports:

- native-dynamic runtime execution for JSON and stream-capable operations
- lifecycle hooks and runtime observability
- explicit runtime reload
- automatic extension hot reload
- targeted runtime reload
- request draining that rejects new invocations and waits for in-flight work before unload

What still remains missing across the latest runtime-control design corpus is a bounded failure policy. Today the safe behavior is to wait forever for a stuck plugin call. That preserves unload safety, but it can also wedge explicit reload, automatic hot reload, and config-driven runtime correction indefinitely.

## Scope

This batch will implement:

1. a configurable native-dynamic drain timeout exposed through standalone config and process env
2. timeout-aware drain waiting in the extension host
3. rollback semantics that clear draining mode and keep the runtime callable if the timeout expires before plugin shutdown begins
4. shutdown orchestration that only removes runtimes from the registry after a successful shutdown
5. tests proving timeout rollback keeps the runtime live and reload control paths fail safely
6. docs updates that move hung-drain timeout and rollback policy from remaining gap to implemented capability

This batch will not implement:

- forceful cancellation of in-flight native-dynamic plugin code
- multi-node coordinated rollout
- restartless listener rebinding in the same batch
  - this gap was closed later by `2026-03-15-restartless-listener-rebinding-design.md`
- migration-safe secret-manager reconfiguration
  - this gap was closed later by `2026-03-15-migration-safe-secret-manager-reconfiguration-design.md`
- new admin or console fields for drain timeout status
- transactional rollback after a plugin `shutdown` hook itself has already started and failed

## Options Considered

### Option A: Keep unbounded draining

Pros:

- smallest change
- preserves current safety behavior

Cons:

- reload can block forever on one bad plugin call
- leaves the documented remaining gap unresolved

### Option B: Add a configurable drain timeout with pre-shutdown rollback

Pros:

- preserves unload safety because timeout returns before plugin shutdown runs
- keeps the runtime callable after failure by clearing draining mode
- integrates naturally with explicit reload, automatic hot reload, and config reload because they already route through the same shutdown helpers

Cons:

- requires more lifecycle state handling than the current unbounded wait
- reload can still fail and require operator attention

### Option C: Force-unload or cancel stuck plugin calls after timeout

Pros:

- shortest reload latency

Cons:

- unsafe in the current in-process FFI model
- no cooperative cancellation ABI exists for plugins
- trades an operational problem for a memory-safety risk

## Recommendation

Use **Option B**.

The runtime must stay safe first. The only acceptable timeout policy in the current architecture is one that aborts shutdown before plugin `shutdown` runs, clears draining mode, and leaves the runtime in service.

## Configuration Contract

Add:

- config file field: `native_dynamic_shutdown_drain_timeout_ms`
- environment variable: `SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS`

Default:

- `0`

Semantics:

- `0` means wait indefinitely, preserving today's behavior
- any positive value is the maximum time a shutdown or reload path waits for in-flight native-dynamic work to drain before rollback

This field is runtime-dynamic. It should be re-exported by the existing runtime config reload flow without requiring listener or store rebuild.

## Drain Timeout Semantics

When native-dynamic shutdown begins:

1. mark the runtime draining
2. reject new invocation lease acquisition
3. wait for the active invocation count to reach zero
4. if the wait completes before timeout:
   - invoke plugin `shutdown`
   - mark the runtime stopped
   - remove it from the registry
5. if the timeout expires first:
   - clear draining mode
   - keep the runtime running
   - return a timeout error
   - do not invoke plugin `shutdown`
   - do not remove the runtime from the registry

The timeout must be measured only across the drain wait, not across discovery rebuild.

## Registry Rollback Rule

The runtime registry must only lose a native-dynamic runtime after a successful shutdown.

That means shutdown orchestration should:

1. collect the targeted runtimes first
2. attempt drain and shutdown on the live runtime objects
3. remove only the successfully shut-down entrypoints from the registry

If a timeout occurs before plugin shutdown begins, the timed-out runtime must remain registered and callable.

## Control-Path Behavior

This timeout and rollback policy should apply automatically to:

- `shutdown_all_native_dynamic_runtimes()`
- `shutdown_native_dynamic_runtimes_for_extension(...)`
- explicit reload
- targeted reload
- automatic extension hot reload
- config-driven extension host reload

Reload should fail loudly when timeout occurs. It should not clear the configured host cache or pretend success.

## Testing Strategy

This batch should be proven with:

1. config tests proving the new timeout field parses from env and file-backed reload inputs
2. extension-host tests proving a timed-out shutdown:
   - returns an error
   - does not invoke plugin shutdown
   - keeps the runtime listed as running
   - clears draining so new invocations can start again
3. gateway reload tests proving a timed-out drain causes reload to fail safely instead of reporting success

The native mock plugin's existing delay and invocation-log controls are sufficient for deterministic timeout coverage.

## Follow-On Work

After this batch and its later listener plus secret-manager follow-ons, the strongest remaining runtime-control gap is:

1. multi-node coordinated runtime rollout
