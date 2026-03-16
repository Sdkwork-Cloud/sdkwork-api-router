# Extension Runtime Reload Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously and choose the best implementation path without pausing for sign-off

## Goal

Close the strongest remaining extension-runtime control-plane gap by adding an explicit reload operation that can:

1. shut down currently managed connector and native-dynamic runtimes
2. invalidate the gateway's cached discovered extension host
3. rebuild discovery state from the current extension search paths and trust policy
4. return fresh runtime status evidence to operators through the admin API and console

## Why This Batch

The extension architecture now already supports:

- manifest-based discovery from configured search paths
- connector runtime supervision and health probing
- native-dynamic runtime loading with lifecycle hooks and status reporting
- authenticated admin visibility for discovered packages and runtime statuses
- real gateway execution through builtin, connector, and native-dynamic providers

What remains missing across the design corpus is explicit runtime reload orchestration. The runtime-mode docs still call out hot reload as future work, and the native-dynamic lifecycle design explicitly leaves unload or reload control as follow-on work.

That means operators can inspect runtime state, but they still cannot apply package changes without restarting the process.

## Scope

This batch will implement:

1. a process-local extension runtime reload operation
2. gateway cache invalidation for the configured discovered extension host
3. orderly shutdown of managed connector and native-dynamic runtimes before rebuild
4. an authenticated admin endpoint for triggering reload
5. a typed reload response with fresh runtime status evidence
6. console runtime-page controls and visibility for reload results

This batch will not implement:

- filesystem watchers or fully automatic hot reload
- per-extension or per-instance targeted reload
- transactional multi-process rollout across multiple gateway nodes
- OS-specific guarantees for replacing a loaded native library while requests are still in flight

## Options Considered

### Option A: Automatic filesystem watching

Pros:

- closest to the phrase "hot reload"
- no operator action required after package updates

Cons:

- significantly larger scope
- hard to make safe across connector processes, native-dynamic libraries, and trust validation
- introduces platform-specific file-watching and library replacement edge cases immediately

### Option B: Admin-triggered explicit reload orchestration

Pros:

- solves the real operational gap now
- keeps shutdown and rebuild ordered and explainable
- works with the current runtime registries and gateway cache
- easy to audit and test end to end

Cons:

- operator must trigger the reload explicitly
- not fully automatic

### Option C: Only clear the gateway cache and let requests rebuild lazily

Pros:

- smallest code change
- minimal surface area

Cons:

- leaves old managed runtimes alive
- does not invoke native-dynamic shutdown hooks
- gives operators no positive evidence that reload actually happened

## Recommendation

Use **Option B**.

It closes the designed control-plane gap without pretending to solve the harder class of true live file-watcher hot reload. It also preserves the lifecycle guarantees already introduced by connector supervision and native-dynamic shutdown hooks.

## Reload Semantics

The reload operation should be global and process-local.

Sequence:

1. shut down all managed connector runtimes
2. shut down all registered native-dynamic runtimes
3. clear the cached configured extension host used by gateway assembly
4. rebuild discovery state from the current environment-driven discovery policy
5. return the fresh runtime status snapshot plus discovery counts

The rebuild step should eagerly rediscover trusted packages. Native-dynamic packages should load immediately because they register provider factories by loading the library during host construction. Connector packages should remain lazily started on demand, consistent with current runtime behavior.

## Reliability And Safety Rules

1. Reload is best-effort and process-local only.
2. Shutdown failures should abort the reload and return an error instead of pretending success.
3. Cache invalidation must happen even when the discovery-policy cache key is unchanged.
4. Old cached host instances must be dropped before rebuilding so native-dynamic libraries can be released when no handles remain.
5. The operation is intended for quiescent admin-triggered maintenance windows, not for forcefully rotating runtimes mid-request.

## Admin API Contract

Add:

- `POST /admin/extensions/runtime-reloads`

Response should include:

- `discovered_package_count`
- `loadable_package_count`
- `active_runtime_count`
- `reloaded_at_ms`
- `runtime_statuses`

This keeps the result directly useful to both automation and the runtime console without introducing a second fetch round-trip requirement.

## Console Contract

The runtime page should evolve from passive persisted-health visibility into an operational runtime view.

It should show:

- currently active managed runtime statuses
- the latest reload summary
- a button that triggers reload and refreshes the runtime status list

Persisted provider health snapshots should stay visible because they remain the routing-facing evidence trail, but the operator now also gets a direct control surface for runtime refresh.

## Testing Strategy

This batch should be proven with:

1. gateway tests proving repeated reload with the same policy still invokes native-dynamic shutdown plus re-init
2. admin route tests proving `POST /admin/extensions/runtime-reloads` is authenticated, succeeds, and returns fresh runtime status evidence
3. console typecheck coverage for the new runtime DTOs and reload action

The native mock plugin's lifecycle log is sufficient to prove ordered shutdown and re-init without introducing OS-specific file replacement behavior.

## Follow-On Work

After this batch, the later multi-node coordinated rollout follow-on has now landed through shared-store runtime-rollout records and node workers.

The remaining extension-runtime gaps are now mostly broader distributed-policy concerns:

1. coordinated rollout for non-extension runtime configuration changes
