# Restartless Store And JWT Reconfiguration Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the highest-value safe subset of the remaining standalone reconfiguration gap by allowing database store dependencies and admin or portal JWT signing secrets to change on disk without a process restart.

## Why This Batch

The repository already supports:

- process-local extension runtime reload
- automatic extension hot reload
- runtime config file polling for gateway and admin services
- native-dynamic request draining with bounded timeout and rollback

What still remained missing in the local process-control path was service dependency reconfiguration. Today a standalone process must restart to pick up a new database URL or a new admin or portal JWT signing secret, even though those changes can be modeled as runtime dependency replacement rather than listener replacement.

At the time this batch was selected, the broader remaining-gap bucket grouped together listener rebinding, database replacement, and secret-manager replacement. Those items were not equally safe. Database and JWT secret replacement could be implemented cleanly with request-scoped dependency snapshots. Secret-manager replacement required a later follow-on that persisted credential metadata and added legacy-key-aware resolution.

## Scope

This batch will implement:

1. request-scoped router state snapshots for gateway, admin, and portal services
2. a shared reloadable store handle that lets future requests pick up a newly built `AdminStore`
3. runtime replacement of the active store when `database_url` changes and the new store initializes successfully
4. runtime replacement of the active admin JWT signing secret when `admin_jwt_signing_secret` changes
5. runtime replacement of the active portal JWT signing secret when `portal_jwt_signing_secret` changes
6. config-file supervision for `portal-api-service` so it participates in the same local runtime correction model
7. tests proving both direct live-handle swaps and config-driven reconfiguration behave correctly without restarting the process

This batch will not implement:

- restartless listener rebinding for `gateway_bind`, `admin_bind`, or `portal_bind`
  - this gap was closed later by `2026-03-15-restartless-listener-rebinding-design.md`
- runtime secret-manager backend replacement
- runtime credential master-key rotation
  - this gap was later closed by `2026-03-15-migration-safe-secret-manager-reconfiguration-design.md`
- migration or replication of data between old and new databases
- multi-node coordinated rollout

## Discovered Safety Constraint

Secret-manager hot replacement was intentionally out of scope for this batch.

The current credential model stores:

- credential identity
- selected secret backend kind

It does not store:

- historical local encrypted file paths
- historical keyring service names
- key lineage or multiple active master keys

That meant changing `secret_local_file`, `secret_keyring_service`, or `credential_master_key` at runtime could make previously stored secrets unreadable immediately. The later fix was to persist explicit credential metadata and add legacy-key-aware resolution rather than performing a blind live swap.

## Options Considered

### Option A: Keep database and JWT settings restart-bound

Pros:

- smallest change
- preserves the current simple startup model

Cons:

- keeps a major operational gap open
- prevents config-driven correction of service dependencies after startup

### Option B: Hot-swap dependencies by reading the current handle on every operation

Pros:

- simpler than request snapshots
- avoids rebuilding routers

Cons:

- a single request can span old and new stores if a swap happens mid-flight
- creates inconsistent behavior during multi-step request handling

### Option C: Use stable live handles plus per-request snapshots

Pros:

- gives new requests the latest dependencies
- preserves per-request consistency after extraction
- fits naturally into Axum state extraction without rebuilding handlers

Cons:

- requires custom router-state cloning behavior
- adds a small runtime indirection layer

### Option D: Full in-process service re-bootstrap

Pros:

- could eventually cover listeners and secret managers too

Cons:

- much larger than the current safe gap
- unnecessary when only store and JWT state need to change

## Recommendation

Use **Option C**.

The right move is to introduce reloadable handles for runtime-owned dependencies, but snapshot those handles when a request enters the router. That keeps the live process correct without allowing a single request to mix old and new backing resources.

## Request Consistency Model

Each router state should hold:

- a live store handle
- optional live JWT secret handle
- request-snapshot fields for the current store and JWT secret

When Axum clones router state for request handling, the custom `Clone` implementation should:

1. read the current live handle values
2. place those values into the request-snapshot fields
3. keep references to the same live handles for future request clones

This produces:

- existing in-flight requests keep their original store and JWT view
- future requests observe the latest successfully applied config

## Store Reload Semantics

When `database_url` changes:

1. load the next config
2. attempt to initialize and migrate a new store for the target dialect
3. if store initialization fails:
   - keep the existing store live
   - log the failure
   - do not advance the config watch baseline
4. if store initialization succeeds:
   - replace the live store handle
   - restart any background supervision tasks that should now write to the new store
   - log the applied change

No attempt should be made to migrate data from the old database to the new one. A new `database_url` is treated as a new source of truth selected by the operator.

## JWT Reload Semantics

When `admin_jwt_signing_secret` or `portal_jwt_signing_secret` changes:

- new logins must mint tokens with the new secret
- authenticated routes must validate tokens against the new secret
- tokens minted with the previous secret may stop working immediately after the swap

That immediate cutoff is acceptable and desirable for explicit secret rotation.

## Service Coverage

`gateway-service`:

- runtime config reload remains active
- store replacement becomes live
- gateway bind remained restart-bound in this batch and was closed later by the dedicated listener-rebinding batch

`admin-api-service`:

- runtime config reload remains active
- store replacement becomes live
- admin JWT secret rotation becomes live
- admin bind remained restart-bound in this batch and was closed later by the dedicated listener-rebinding batch

`portal-api-service`:

- adopts config-loader and watch supervision
- store replacement becomes live
- portal JWT secret rotation becomes live
- portal bind remained restart-bound in this batch and was closed later by the dedicated listener-rebinding batch

## Implementation Shape

Use these boundaries:

- `sdkwork-api-storage-core`: generic reloadable value handle
- `sdkwork-api-interface-http`: gateway state snapshots over a live store handle
- `sdkwork-api-interface-admin`: admin state snapshots over live store and JWT handles
- `sdkwork-api-interface-portal`: portal state snapshots over live store and JWT handles
- `sdkwork-api-app-runtime`: config supervision that rebuilds stores and updates live handles
- `services/*`: startup wiring that builds live handles once and passes them to both the router and the supervisor

## Testing Strategy

This batch should be proven with:

1. gateway router tests proving a live store swap changes what a new request reads
2. admin router tests proving a live JWT secret swap invalidates old tokens and accepts newly minted ones
3. portal router tests proving the same JWT rotation behavior for portal auth
4. runtime supervision tests proving a config file change updates the live store and JWT handles without restarting the process
5. existing runtime supervision tests continuing to pass so extension-control behavior does not regress

## Follow-On Work

After this batch and its later listener plus secret-manager follow-ons, the strongest remaining standalone reconfiguration gap is:

1. multi-node coordinated rollout
