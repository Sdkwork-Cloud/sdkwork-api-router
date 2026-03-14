# Local Config Runtime Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive checkpoints

## Context

The repository currently has:

- one runtime config crate: `sdkwork-api-config`
- environment-variable loading for standalone services
- hardcoded local-friendly defaults inside `StandaloneConfig`

What is still missing is a proper local configuration system:

- no JSON or YAML config file support
- no stable default config directory
- no search order for local config files
- no normalized local default paths for secrets and extension discovery
- some production code still reads `SDKWORK_EXTENSION_*` directly from the process environment

## Goal

Add a local configuration system that supports JSON and YAML files, uses `~/.sdkwork/router/` as the default cross-platform config root, preserves environment variable overrides, and makes the final resolved configuration consistent across services.

## Approaches

### Option A: Environment variables only

Pros:

- simplest implementation
- no new file parsing logic

Cons:

- poor local UX
- hard to persist stable machine-local defaults
- weak discoverability for extension search paths, secrets file, and database location

### Option B: File config plus environment overrides in the config crate

Pros:

- centralizes configuration logic in one place
- supports stable local defaults
- keeps deployment flexibility because environment variables still win
- maps well to standalone binaries and future embedded runtimes

Cons:

- requires path normalization and file parsing
- requires care to avoid divergence with production code that still reads environment variables directly

### Option C: Add a separate external configuration service

Pros:

- highly extensible long term

Cons:

- disproportionate to the current repository
- adds deployment and operational complexity
- unnecessary before the local config story is solved

## Recommendation

Choose **Option B**.

The repository already has a dedicated config boundary. The right move is to make `sdkwork-api-config` the single source of truth for default values, local file loading, and environment overrides.

## Target Design

### Default Config Root

Default root:

- `~/.sdkwork/router/`

Cross-platform resolution:

- Unix-like systems:
  - `${HOME}/.sdkwork/router`
- Windows:
  - `${USERPROFILE}\\.sdkwork\\router`
  - fallback to `${HOMEDRIVE}${HOMEPATH}\\.sdkwork\\router` if needed

Override support:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

### File Discovery

When `SDKWORK_CONFIG_FILE` is not set, search in the config directory using this order:

1. `config.yaml`
2. `config.yml`
3. `config.json`

The first existing file wins.

### Merge Order

The resolved config order should be:

1. built-in local defaults
2. discovered local config file
3. `SDKWORK_*` environment variables

This preserves deployment compatibility while making local development predictable.

### Local Defaults

The runtime local defaults should be normalized around the config root:

- gateway bind:
  - `127.0.0.1:8080`
- admin bind:
  - `127.0.0.1:8081`
- portal bind:
  - `127.0.0.1:8082`
- default SQLite database:
  - `~/.sdkwork/router/sdkwork-api-server.db`
- extension search path:
  - `~/.sdkwork/router/extensions`
- local secret file:
  - `~/.sdkwork/router/secrets.json`

### File Schema

Use a flat top-level schema that mirrors `StandaloneConfig` field names:

- `gateway_bind`
- `admin_bind`
- `portal_bind`
- `database_url`
- `extension_paths`
- `enable_connector_extensions`
- `enable_native_dynamic_extensions`
- `extension_trusted_signers`
- `require_signed_connector_extensions`
- `require_signed_native_dynamic_extensions`
- `admin_jwt_signing_secret`
- `portal_jwt_signing_secret`
- `runtime_snapshot_interval_secs`
- `secret_backend`
- `credential_master_key`
- `secret_local_file`
- `secret_keyring_service`

### Relative Path Resolution

In file-based config:

- `secret_local_file` relative paths should resolve against the config file directory
- `extension_paths` relative paths should resolve against the config file directory
- relative SQLite file URLs should resolve against the config file directory and become absolute SQLite URLs

### Consistency With Existing Production Code

Some production code still reads `SDKWORK_EXTENSION_*` directly from the process environment.

To keep the runtime behavior consistent in this batch:

- services should load the merged config once through `StandaloneConfig::from_env()`
- services should then export the resolved config back into process environment variables before constructing runtime components that still read env directly

This avoids a split-brain configuration result without requiring a broad refactor in one batch.

## Testing Strategy

Use TDD-first coverage for:

- default config root resolution from a provided home directory
- discovery precedence across `config.yaml`, `config.yml`, and `config.json`
- JSON and YAML parsing
- file config merge before environment overrides
- relative path normalization for `secret_local_file`, `extension_paths`, and SQLite file URLs
- export of resolved config back into process env pairs

## Non-Goals

This batch should not:

- add remote config distribution
- add hot-reload for config files
- redesign every env-based helper in the workspace
- add TOML as a runtime config format for standalone service startup
