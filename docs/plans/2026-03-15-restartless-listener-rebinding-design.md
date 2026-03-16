# Restartless Listener Rebinding Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the highest-value remaining standalone reconfiguration gap by allowing `gateway_bind`, `admin_bind`, and `portal_bind` to change on disk without requiring a process restart.

## Why This Batch

The repository already supports:

- process-local extension runtime reload
- automatic extension hot reload
- native-dynamic request draining with rollback support
- restartless database-store replacement
- restartless admin and portal JWT secret rotation

That leaves listener rebinding as the strongest remaining single-process runtime-correction gap. Today each standalone service binds one `TcpListener` at startup and keeps that socket for the lifetime of the process. Any port or host change still requires a full restart even though the request router, live store handles, and live JWT handles can now survive independently of the listener itself.

At the time this batch was selected, secret-manager replacement was still blocked on credential metadata and migration support. Listener rebinding was therefore the next safe batch because it could be implemented with host-owned server orchestration and did not require changing credential semantics.

## Scope

This batch will implement:

1. a shared standalone listener host that owns the active Axum server task for one service
2. restartless listener rebinding for `gateway-service`, `admin-api-service`, and `portal-api-service`
3. config-supervision integration so a changed bind address triggers a live rebind instead of only logging a restart requirement
4. graceful cutover semantics where the new listener starts before the old listener begins shutdown
5. failure handling where bind errors keep the previous listener active and keep retrying the pending config change
6. tests proving successful rebind, config-driven rebind, and failed-rebind fallback behavior

This batch will not implement:

- secret-manager backend hot replacement
- credential master-key rotation
  - both gaps were later closed by `2026-03-15-migration-safe-secret-manager-reconfiguration-design.md`
- TLS certificate reload
- unix-domain-socket serving
- multi-node coordinated rollout

## Historical Safety Constraint

At the time of this batch, secret-manager settings remained out of scope:

- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

That boundary was later removed by the migration-safe secret-manager follow-on once persisted credential metadata and legacy-key-aware resolution were in place.

## Options Considered

### Option A: Keep listeners restart-bound

Pros:

- zero implementation risk
- preserves the current simple startup path

Cons:

- leaves the strongest remaining standalone runtime gap open
- prevents config-driven correction of service endpoints

### Option B: Shut down the old listener before binding the new one

Pros:

- small implementation
- avoids running two listeners briefly

Cons:

- introduces an avoidable outage window
- makes rebind failure user-visible because service availability drops before replacement succeeds

### Option C: Pre-bind the new listener, activate it, then gracefully drain the old server

Pros:

- avoids downtime during successful cutover
- keeps the old listener serving if the new bind fails
- fits the current Axum-per-service deployment model
- composes cleanly with live store and JWT handles already present in router state

Cons:

- requires a small listener-host abstraction
- adds server-task lifecycle management to runtime supervision

### Option D: Full in-process service re-bootstrap

Pros:

- could eventually unify listener, store, JWT, and secret-manager replacement

Cons:

- much larger than the current safe gap
- would duplicate work already solved with reloadable request-scoped handles

## Recommendation

Use **Option C**.

The right move is to add a service-local listener host that can:

1. pre-bind a replacement socket
2. start a new Axum server task on that socket
3. atomically mark the new server as active
4. signal graceful shutdown to the old server

That keeps the service continuously available on successful cutover and leaves the existing listener untouched when the new bind cannot be acquired.

## Cutover Model

For a relevant bind change:

1. the config supervisor detects the changed bind field for the current service
2. the listener host attempts to bind the new socket while the old server remains active
3. if binding fails:
   - log the failure
   - keep the old listener active
   - do not advance the config-watch baseline
   - retry on the next supervision poll
4. if binding succeeds:
   - start the replacement server task on the new socket
   - swap the active listener generation
   - signal graceful shutdown to the old server
   - let in-flight requests on the old listener complete
5. new incoming connections are accepted only on the new bind after cutover completes

## Router And State Model

The listener host should reuse the existing router instance shape rather than rebuilding service state during rebinding.

That is safe because:

- gateway, admin, and portal routers are already pure Axum routers
- request entry already snapshots live store and JWT dependencies per request
- secret-manager instances remained fixed for the lifetime of the process during this batch and were later made reloadable by the migration-safe secret-manager follow-on

This means listener rebinding is only a transport concern. Store and JWT live handles continue working exactly as they do now.

## Service-Relevant Config Semantics

Each standalone process should only react to fields relevant to that service:

- `gateway-service`: `gateway_bind`, `database_url`, extension-runtime dynamic config, and, at the time of this batch, secret-manager restart-only fields
- `admin-api-service`: `admin_bind`, `database_url`, `admin_jwt_signing_secret`, extension-runtime dynamic config, and, at the time of this batch, secret-manager restart-only fields
- `portal-api-service`: `portal_bind`, `database_url`, `portal_jwt_signing_secret`

Changes for other services should be ignored by the current process instead of being reported as local restart requirements.

## Failure Semantics

If multiple relevant changes are present in one config update:

- the supervisor may pre-build new stores and pre-bind new listeners first
- a listener bind failure must prevent the new bind from activating
- already-safe live replacements such as store and JWT swaps should still only be applied after the bind preflight succeeds

The implementation does not need distributed transaction semantics across all runtime subsystems, but it should avoid needless partial application when the target socket cannot even be acquired.

## Implementation Shape

Use these boundaries:

- `sdkwork-api-app-runtime`: listener-host abstraction, relevant-field filtering, and config-supervision orchestration
- `services/gateway-service`: create the listener host from `gateway_router_with_state(...)`
- `services/admin-api-service`: same pattern for the admin router
- `services/portal-api-service`: same pattern for the portal router
- `docs/*`: move listener rebinding from the remaining-gap list to implemented capability

## Testing Strategy

This batch should be proven with:

1. a direct listener-host test proving requests move from the old bind to the new bind
2. a config-supervision test proving a standalone service rebinds after a config file change
3. a failed-rebind test proving the old listener stays active and the supervisor retries until the new socket becomes available
4. full workspace verification so runtime, services, and console still build together

## Follow-On Work

After this batch and its later secret-manager follow-on, the strongest remaining standalone reconfiguration gap is:

1. multi-node coordinated rollout
