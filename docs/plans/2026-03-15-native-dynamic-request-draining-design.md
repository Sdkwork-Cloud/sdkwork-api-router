# Native-Dynamic Request Draining Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the strongest remaining single-process native-dynamic unload-safety gap by ensuring runtime shutdown waits for in-flight plugin calls to finish before unload or reload proceeds.

## Why This Batch

The repository already supports:

- trusted native-dynamic provider execution for JSON and stream-capable operations
- lifecycle hooks including `init`, `health_check`, and `shutdown`
- explicit runtime reload
- automatic extension hot reload
- targeted extension runtime reload

What still remains missing across the latest runtime-control design corpus is in-flight request draining. Today a native-dynamic runtime can be removed from the registry and shut down while a JSON or stream invocation is still executing inside the plugin library. That creates an avoidable unload-safety risk in the exact control paths that now support explicit and automatic reload.

## Scope

This batch will implement:

1. native-dynamic invocation leases that count active plugin calls
2. shutdown behavior that blocks new invocations, waits for in-flight calls to finish, and only then invokes plugin shutdown
3. coverage for both JSON and stream invocation paths
4. test-fixture controls that let runtime tests hold a plugin call open long enough to prove drain ordering
5. docs updates that move request draining from remaining gap to implemented capability

This batch will not implement:

- multi-node coordinated rollout
- forceful cancellation of in-flight native-dynamic work
- configurable drain timeouts or rollback policies
- per-instance native-dynamic unload targeting beyond today's extension-scoped runtime model
- new admin or console contract fields for draining state

## Options Considered

### Option A: Reference-count active plugin calls and wait until zero before shutdown

Pros:

- directly solves the unload-safety problem in the current architecture
- works for both explicit reload and automatic hot reload because both already flow through the same shutdown helpers
- does not require any ABI change for external plugins

Cons:

- shutdown can wait indefinitely if a plugin call never returns
- does not give operators new draining observability fields yet

### Option B: Add drain timeout and fail reload if the timeout expires

Pros:

- bounds operator wait time
- surfaces hung plugin behavior explicitly

Cons:

- requires more complex state rollback because the runtime must stay callable if reload fails
- introduces new config and error-policy questions that the current runtime-control docs have not yet settled

### Option C: Force-cancel in-flight plugin calls

Pros:

- shortest reload latency

Cons:

- unsafe for the current in-process FFI model
- no existing cooperative cancellation ABI for plugins
- would trade a correctness gap for a memory-safety risk

## Recommendation

Use **Option A**.

The current architecture has one correct move available: quiesce the runtime, reject new calls, and wait for active plugin work to finish before invoking `shutdown` and unloading the library. Timeouts and forceful cancellation can be layered later, but unload safety has to come first.

## Drain Semantics

Native-dynamic runtime state should track:

1. whether the runtime is currently draining for shutdown
2. how many plugin calls are in flight

The runtime should acquire an invocation lease before:

- `execute_json`
- `execute_stream_json`
- `health_check`

The lease should:

1. fail immediately if the runtime is not running
2. fail immediately if shutdown has entered draining mode
3. increment the active-call counter on acquisition
4. decrement the counter when the plugin call fully finishes

For stream invocations, the lease must stay alive until the spawned plugin-execution thread finishes, not merely until the host returns an initial `ProviderStreamOutput`. That is the point where plugin code has actually stopped running.

## Shutdown Semantics

When native-dynamic shutdown begins:

1. mark the runtime as draining
2. block new invocation lease acquisition
3. wait until the active-call counter reaches zero
4. invoke the plugin `shutdown` hook if exported
5. mark the runtime stopped and fully shut down

This should apply automatically to:

- full native-dynamic shutdown
- targeted extension shutdown
- reload-driven shutdown paths
- hot-reload-driven shutdown paths

because they all already route through the same extension-host shutdown helpers.

## Error Handling

New invocations that arrive after draining begins should fail with a clear runtime error instead of racing shutdown.

This batch should keep shutdown wait unbounded rather than adding partial timeout behavior. If a plugin wedges forever, the safe outcome in the current architecture is a blocked reload, not an unsafe unload.

## Testing Strategy

This batch should be proven with:

1. an extension-host test proving shutdown waits for an in-flight JSON invocation to finish before the plugin shutdown hook runs
2. an extension-host test proving shutdown waits for an in-flight stream invocation thread to finish before unload
3. lifecycle or invocation log evidence showing ordered completion before shutdown

The native mock plugin fixture should expose test-only delay controls through environment variables so the runtime can be held in-flight long enough to prove the drain behavior deterministically.

## Follow-On Work

After this batch and the later secret-manager reconfiguration follow-on, the strongest remaining runtime-control gap is:

1. multi-node coordinated runtime rollout
