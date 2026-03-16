# Targeted Extension Runtime Reload Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the next strongest extension-runtime control gap by adding targeted runtime reload control so operators can reload:

1. all managed extension runtimes
2. one extension package runtime family
3. one connector instance runtime

without always forcing unrelated managed runtimes to churn.

## Why This Batch

The repository already supports:

- explicit process-local runtime reload
- polling-based automatic extension hot reload
- dynamic runtime config reload for extension policy and supervision intervals
- authenticated runtime visibility in the admin API and runtime console

What still remains missing across the design corpus is scope control. The control plane can currently only reload everything, even when an operator only changed one extension package or one connector instance. The latest extension-runtime docs explicitly leave per-instance or per-extension targeted reload as the strongest remaining control gap.

## Scope

This batch will implement:

1. a targeted reload request contract for `/admin/extensions/runtime-reloads`
2. extension-scoped managed runtime shutdown for connector and native-dynamic runtimes
3. connector-instance-scoped runtime shutdown for managed connector processes
4. native-dynamic runtime reuse during host cache rebuild so unrelated runtimes do not reinitialize
5. runtime console controls for global reload plus per-runtime targeted reload
6. response metadata explaining what scope was requested and what scope was actually applied

This batch will not implement:

- multi-node coordinated reload
- per-instance native-dynamic unload semantics
- partial manifest discovery that only scans one package root

## Options Considered

### Option A: Keep one global reload endpoint and only improve messaging

Pros:

- smallest change
- no new scope resolution rules

Cons:

- does not solve the real operational gap
- still churns unrelated runtimes for single-package updates

### Option B: Targeted runtime shutdown plus full host cache rebuild with native-dynamic reuse

Pros:

- solves the operator problem now
- preserves the current single host-cache rebuild seam
- avoids unrelated native-dynamic lifecycle churn
- keeps connector targeting precise where the runtime model is already instance-scoped

Cons:

- requires careful runtime registry reuse behavior
- targeted reload remains process-local

### Option C: True partial host rebuild per package

Pros:

- potentially even narrower churn

Cons:

- much larger architectural change
- current configured host cache is whole-host, not package-partitioned
- unnecessary for closing the current gap

## Recommendation

Use **Option B**.

The existing architecture already has one safe rebuild seam. The right move is to make shutdown selective and rebuild reuse-aware, not to invent a second partial-host architecture.

## Scope Resolution Model

The admin endpoint should accept an optional JSON body:

- no body or empty body:
  - reload everything
- `extension_id` only:
  - reload that extension runtime family
- `instance_id` only:
  - resolve the persisted instance and installation
  - if the installation runtime is `connector`, reload only that managed connector instance
  - if the installation runtime is `native_dynamic` or `builtin`, apply extension-scoped reload using the instance's `extension_id`

`extension_id` and `instance_id` together should be rejected as invalid input for this batch.

## Reload Semantics

### Global Scope

Keep today's behavior:

1. shut down all managed connector runtimes
2. shut down all native-dynamic runtimes
3. clear the configured host cache
4. rebuild discovery state

### Extension Scope

1. shut down connector runtimes whose `extension_id` matches the target extension
2. shut down native-dynamic runtimes whose loaded manifest ID matches the target extension
3. clear the configured host cache
4. rebuild discovery state across the full configured policy
5. reuse already-loaded unrelated native-dynamic runtimes instead of reinitializing them

### Connector Instance Scope

1. shut down the managed connector process for the target `instance_id`
2. clear the configured host cache
3. rebuild discovery state across the full configured policy
4. reuse unrelated native-dynamic runtimes

Because configured host discovery remains whole-host, the rebuild step is still global. The selectivity guarantee in this batch applies to runtime shutdown and native-dynamic lifecycle churn, not to filesystem scanning breadth.

## Native-Dynamic Reuse Rule

The critical architectural rule is:

1. if a native-dynamic runtime for the same resolved entrypoint is already loaded and was not part of the targeted shutdown set, host rebuild should reuse it
2. if the runtime was part of the targeted shutdown set, rebuild should load it fresh and re-run lifecycle init

This closes the main correctness gap that would otherwise cause unrelated native-dynamic runtimes to reinitialize on every targeted reload.

## Admin API Contract

`POST /admin/extensions/runtime-reloads`

Request body:

- optional
- supports:
  - `extension_id`
  - `instance_id`

Response should include:

- `scope`
- `requested_extension_id`
- `requested_instance_id`
- `resolved_extension_id`
- `discovered_package_count`
- `loadable_package_count`
- `active_runtime_count`
- `reloaded_at_ms`
- `runtime_statuses`

`scope` is the actually applied scope:

- `all`
- `extension`
- `instance`

This allows the control plane to explain when an instance request degraded to extension scope for process-wide runtimes.

## Console Contract

The runtime page should keep the current global reload button and add targeted controls per active runtime row:

- connector rows:
  - reload this instance
- native-dynamic rows:
  - reload this extension

The last reload summary should show:

- applied scope
- requested target identifiers
- resolved extension identifier when relevant

## Testing Strategy

This batch should be proven with:

1. gateway tests proving targeted reload of an unrelated scope does not reinitialize an already loaded native-dynamic runtime
2. gateway tests proving targeted extension reload does invoke native-dynamic shutdown plus init
3. admin route tests proving targeted reload request bodies are validated, scope is resolved correctly, and responses return the right scope metadata
4. console typecheck coverage for the new reload request and response fields

## Follow-On Work

After this batch, the multi-node follow-on has since been implemented separately through shared-store coordinated `POST /admin/extensions/runtime-rollouts`.

The remaining extension-runtime control gaps are now narrower:

1. broader distributed rollout for non-extension runtime configuration changes
