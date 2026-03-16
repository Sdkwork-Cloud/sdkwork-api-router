# Migration-Safe Secret-Manager Reconfiguration Design

**Date:** 2026-03-15

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive sign-off

## Goal

Close the remaining single-process secret-management gap by allowing supervised standalone services to change:

- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

without restarting the process and without making previously stored credentials unreadable.

## Why This Batch

The repository already supports:

- process-local extension runtime reload
- automatic extension hot reload
- restartless store replacement
- restartless JWT rotation
- restartless listener rebinding

What still remains restart-bound is secret-manager configuration. The current system stores only the selected secret backend kind per credential record. That is enough to survive a default backend switch in some cases, but it is not enough to survive:

- local encrypted file path changes
- keyring service changes
- master-key rotation

The missing data is credential-era metadata. Without it, a live process cannot know which file path, which keyring service, or which master key lineage is required to read a historical secret after configuration changes.

## Scope

This batch will implement:

1. per-credential persisted secret metadata for backend-specific locator data and master-key identity
2. support for a current credential master key plus legacy decrypt-only master keys
3. secret resolution based on credential-record metadata instead of the process-wide current defaults
4. live secret-manager handles for gateway and admin routers, with request-scoped snapshots like the existing store and JWT models
5. runtime supervision support that hot-swaps the active secret manager when secret-manager config changes
6. validation that blocks an unsafe secret-manager reload if the next manager cannot still resolve existing credentials
7. tests proving old credentials remain readable after backend-path, keyring-service, or master-key changes

This batch will not implement:

- eager online rewriting of all stored credentials to the newest backend or key
- deletion of superseded secrets from legacy local files or legacy keyring services
- explicit admin APIs for historical credential cleanup
- multi-node coordinated rollout

## Core Design Constraint

Online full-credential rewriting is not the right first move.

If the system rewrites credential metadata to new backend locations or a new master key while in-flight requests still hold an old request-scoped secret-manager snapshot, those requests can start failing mid-flight. That is the same consistency class we already avoided for stores and JWTs.

The correct first batch is therefore:

1. make the new manager capable of reading old records safely
2. hot-swap to that new manager for new requests
3. keep old request snapshots valid because existing credential metadata is not rewritten during the swap

That closes the live-reconfiguration gap without introducing per-request credential inconsistency.

## Options Considered

### Option A: Keep secret-manager settings restart-bound

Pros:

- no new credential metadata
- no runtime complexity

Cons:

- leaves the last major single-process reconfiguration gap open
- prevents safe master-key rotation or backend relocation without restart

### Option B: Persist credential metadata plus legacy master keys, then live-swap managers

Pros:

- closes the real correctness gap
- keeps old credentials readable after config changes
- fits the existing reloadable handle plus request snapshot pattern
- avoids rewriting records during in-flight request windows

Cons:

- adds schema and domain model changes
- requires validation before applying a new manager

### Option C: Live-swap the manager and immediately rewrite all credentials to the new backend or key

Pros:

- converges all credential storage to the newest config quickly
- would eventually let operators retire old secret locations and keys

Cons:

- creates request-consistency risks for in-flight requests holding the old manager snapshot
- introduces partial-migration lifecycle complexity
- is larger than the current gap

## Recommendation

Use **Option B**.

The system needs metadata-complete credential records and multi-key read support first. That is the minimum architecture required to make secret-manager reconfiguration safe at all.

## Credential Metadata Model

Each credential record should carry:

- `secret_backend`
- `secret_local_file`
- `secret_keyring_service`
- `secret_master_key_id`

Rules:

- `database_encrypted` records may leave `secret_local_file` and `secret_keyring_service` empty
- `local_encrypted_file` records persist the exact file path used when the secret was written
- `os_keyring` records persist the exact keyring service name used when the secret was written
- encrypted records persist the derived key identifier for the master key used to write them

This metadata is what makes future secret resolution independent of the process's current defaults.

## Master-Key Identity Model

The system should derive a stable, non-secret key identifier from the raw master key and persist only that identifier.

The runtime config should then support:

- one current `credential_master_key`
- zero or more `credential_legacy_master_keys`

Semantics:

- new writes always use the current master key
- decrypts use the record's `secret_master_key_id` when present
- if a legacy record has no persisted key id, the manager may fall back to trying current and legacy keys in order for backward compatibility

## Secret Resolution Semantics

When resolving one credential:

1. read the credential record
2. select the backend from the record metadata
3. select the backend locator from the record metadata
4. load the encrypted envelope from the right storage location
5. pick the matching master key using `secret_master_key_id`, or use backward-compatible fallback when the record predates the new field
6. decrypt and return the plaintext secret

This makes the current process default relevant only for new writes, not for historical reads.

## Runtime Reload Semantics

When secret-manager config changes:

1. build the candidate next manager from the next config
2. validate that existing credentials in the active store are still resolvable through that candidate manager
3. if validation fails:
   - keep the current live manager
   - log the failure
   - do not advance the config-watch baseline
4. if validation succeeds:
   - replace the live secret-manager handle
   - let new requests pick up the new manager snapshot
   - keep old requests using their old manager snapshot until they finish

This is the same request-consistency rule already used for stores, JWTs, and listeners.

## Service Coverage

`gateway-service`:

- live secret-manager replacement becomes active
- old credentials remain readable after config changes when their metadata and legacy keys are available

`admin-api-service`:

- live secret-manager replacement becomes active
- new credential creation uses the new manager defaults after reload

`portal-api-service`:

- unaffected because it does not use upstream credential secret resolution

## Config Contract

Add one new config field:

- `credential_legacy_master_keys`

Recommended parsing contract:

- config file: list of strings
- environment variable: semicolon-delimited string

This field is live-reloadable together with the other secret-manager settings.

## Testing Strategy

This batch should be proven with:

1. config tests proving `credential_legacy_master_keys` parse from env and config files
2. app-credential tests proving credentials written under old file, keyring service, or master key remain resolvable through a new manager
3. gateway or admin state tests proving a replaced live secret-manager handle affects only new requests
4. runtime supervision tests proving config-file secret-manager changes hot-swap the live manager and preserve historical credential readability

## Follow-On Work

After this batch, the strongest remaining reconfiguration and runtime-control gap should be:

1. multi-node coordinated rollout

Optional future improvement:

- explicit credential convergence tooling that rewrites old credentials onto the newest backend and current master key once operators want to retire legacy key material or secret storage locations
