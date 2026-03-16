# Runtime Config Hot Reload Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without pausing for sign-off

## Goal

Close the next strongest extension-runtime control gap by letting standalone services re-read dynamic runtime config file values without a process restart.

## Why This Batch

The repository already supports:

- local JSON and YAML config loading with stable default paths
- explicit process-local extension runtime reload
- polling-based automatic extension tree hot reload
- background provider health snapshot supervision

What still remained missing was config-driven runtime correction after the process had started. Operators could change extension paths, trust policy, or supervision intervals on disk, but those changes still required a manual process restart to take effect.

## Scope

This batch implements:

1. a preserved config loader that keeps the original process-start environment override set
2. polling-based config file watch state for the resolved config file or default config candidates
3. standalone runtime supervision that re-applies dynamic config changes for `gateway-service` and `admin-api-service`
4. automatic extension host reload when extension discovery or trust policy changes
5. dynamic restart of background health snapshot and extension hot reload supervisors when their intervals change
6. admin extension package listing that reads the current runtime discovery policy instead of caching startup state forever
7. follow-on standalone service supervision that now hot-swaps database stores plus admin or portal JWT secrets for all three standalone services
8. follow-on standalone listener supervision that now rebinds `gateway_bind`, `admin_bind`, or `portal_bind` without restarting the process

This batch does not implement:

- observation of environment variable changes made outside the running process after startup

## Options Considered

### Option A: Full process re-bootstrap inside the service

Pros:

- could eventually cover bind addresses, stores, and secret managers

Cons:

- too large for the current gap
- would require replacing listeners, stores, and router state safely
- introduces lifecycle complexity far beyond extension-runtime control

### Option B: Scoped dynamic runtime config reload

Pros:

- solves the highest-value operational gap now
- reuses the existing extension reload and snapshot supervision seams
- keeps static config explicitly restart-bound instead of pretending to hot reload everything

Cons:

- only a subset of config fields become live-reloadable
- requires careful handling of process-exported resolved env values

### Option C: Re-read config files only for extension discovery helpers

Pros:

- smallest implementation

Cons:

- leaves snapshot and hot-reload intervals stale
- does not produce one coherent runtime config contract

## Recommendation

Use **Option B**.

The best move is to add a narrow dynamic runtime layer over the existing config system, while keeping non-reloadable settings explicit and honest.

## Dynamic Reload Contract

Reloadable fields:

- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `native_dynamic_shutdown_drain_timeout_ms`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `extension_hot_reload_interval_secs`
- `runtime_snapshot_interval_secs`
- `database_url`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `secret_backend`
- `credential_master_key`
- `credential_legacy_master_keys`
- `secret_local_file`
- `secret_keyring_service`

Restart-required fields:
- none in the current single-process standalone design corpus beyond changes injected outside the preserved process-start env override set

When restart-required fields change on disk, the process should log that the change was detected but ignored until restart.

## Config Source Semantics

The standalone services already export their resolved config back into `SDKWORK_*` process variables for older env-based consumers.

That means a naive `from_env()` reload would read the previously exported resolved values and mask new file changes.

The runtime config loader must therefore:

1. capture the original process-start environment override set before resolved values are exported
2. keep using that original environment snapshot for all future reloads
3. re-read the config file layer against that preserved snapshot

This preserves the documented merge order instead of accidentally making the first resolved config sticky forever.

## Watch Model

Use the same cross-platform polling strategy already chosen for extension tree hot reload.

The config supervisor should:

1. capture the initial config watch state before spawning the background task
2. poll once per second
3. watch either:
   - the explicit `SDKWORK_CONFIG_FILE` target, or
   - the default `config.yaml`, `config.yml`, and `config.json` candidates
4. advance its baseline only after a reload pass succeeds or after a purely static-only file change is acknowledged

## Runtime Supervision Semantics

For `gateway-service` and `admin-api-service`:

1. start the current provider health snapshot supervisor from the initial config
2. start the current extension hot reload supervisor from the initial config
3. watch the config file set
4. on dynamic config change:
   - reload extension runtime state when extension policy changed
   - re-export only the dynamic runtime env values
   - restart the background supervisors when their intervals changed
   - restart extension hot reload supervision when extension policy changed so its watch baseline matches the new policy

For all standalone services:

1. rebuild and replace the live `AdminStore` when `database_url` changes and the new store initializes successfully
2. rotate the active admin JWT secret when `admin_jwt_signing_secret` changes
3. rotate the active portal JWT secret when `portal_jwt_signing_secret` changes
4. rebind the current service listener when its bind address changes by pre-binding the replacement socket, activating the new server generation, and gracefully draining the previous listener
5. keep request handling consistent by snapshotting the current store and JWT values at request entry instead of re-reading them mid-request

Gateway and admin secret-manager settings are now live-reloadable too, provided operators keep any still-needed historical decrypt keys in `credential_legacy_master_keys`. Persisted credential metadata now carries the backend locator and master-key identity needed to keep historical secrets readable after backend-path, backend-service, or master-key changes.

## Testing Strategy

This batch should be proven with:

1. config tests proving reload uses the original external env override set even after resolved env export
2. config tests proving config watch state changes when a config file appears
3. admin route tests proving `/admin/extensions/packages` reflects the current runtime discovery policy instead of startup state
4. gateway, admin, and portal tests proving live store or JWT handles are applied to new requests without restarting the router
5. runtime supervision tests proving a config file change can shut down and re-enable a native-dynamic runtime without restarting the process
6. runtime supervision tests proving a config file change can replace the active store and JWT handles without restarting the process
7. runtime supervision tests proving a config file change can move a live listener to a new bind and retry on bind failure without dropping the old listener first

## Follow-On Work

After this batch, explicit multi-node extension-runtime rollout and coordinated standalone-config rollout have since been implemented as separate shared-store control-plane capabilities.

The remaining runtime-config and extension-control gaps are now:

1. cross-node synchronization or versioning of local config content before rollout begins
2. richer rollout policy such as staged waves, canaries, retries, or cancellation for distributed config changes
