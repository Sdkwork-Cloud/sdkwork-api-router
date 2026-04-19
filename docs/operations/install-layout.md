# Install Layout

This page defines the production install layout for the official server product, `sdkwork-api-router-product-server`.

## Packaging Model

Every server install is split into three layers:

- product root: the stable top-level install directory
- current control home: the stable runtime wrapper layer used by operators and service managers
- versioned release payload: the immutable program payload under `releases/<version>/`

Mutable state is never stored inside the versioned release payload.

The immutable `releases/<version>/` payload is always materialized from the official packaged server bundle under `artifacts/release/native/<platform>/<arch>/bundles/`.
The installer selects and resolves that canonical bundle directly from `artifacts/release/release-catalog.json`.
Any archive, checksum, or external manifest outside the published catalog entry is rejected before materializing `releases/<version>/`.
That packaged release payload preserves the bundled `bin/`, `sites/*/dist/`, `data/`, `deploy/`, `release-manifest.json`, and `README.txt` entries exactly as published.

## Portable Layout

Portable installs are intended for local validation, CI smoke tests, and explicit non-system installs.

Default portable product root:

- `artifacts/install/sdkwork-api-router/`

Portable layout:

- `current/`
  - `bin/`
  - `service/`
  - `release-manifest.json`
- `releases/<version>/`
  - `bin/`
  - `sites/admin/dist/`
  - `sites/portal/dist/`
  - `data/`
  - `deploy/`
  - `release-manifest.json`
  - `README.txt`
- `config/`
  - `router.yaml`
  - `router.env`
  - `router.env.example`
  - `conf.d/`
- `data/`
- `log/`
- `run/`

Notes:

- `current/` is the control layer. It contains wrapper scripts and service descriptors.
- `current/bin/` is the stable operator surface for `start.*`, `stop.*`, `validate-config.*`, `backup.*`, and `restore.*`.
- `releases/<version>/` is the active immutable payload copied from the packaged server bundle.
- `config/`, `data/`, `log/`, and `run/` stay writable and upgrade-safe.

## System Layout

System installs follow OS-standard mutable roots while keeping the program payload under a dedicated product root.

### Linux

- product root: `/opt/sdkwork-api-router/`
- current control home: `/opt/sdkwork-api-router/current/`
- versioned release payload: `/opt/sdkwork-api-router/releases/<version>/`
- config home: `/etc/sdkwork-api-router/`
- config file: `/etc/sdkwork-api-router/router.yaml`
- config fragments: `/etc/sdkwork-api-router/conf.d/`
- env file: `/etc/sdkwork-api-router/router.env`
- data home: `/var/lib/sdkwork-api-router/`
- log home: `/var/log/sdkwork-api-router/`
- run home: `/run/sdkwork-api-router/`

### macOS

- product root: `/usr/local/lib/sdkwork-api-router/`
- current control home: `/usr/local/lib/sdkwork-api-router/current/`
- versioned release payload: `/usr/local/lib/sdkwork-api-router/releases/<version>/`
- config home: `/Library/Application Support/sdkwork-api-router/`
- config file: `/Library/Application Support/sdkwork-api-router/router.yaml`
- config fragments: `/Library/Application Support/sdkwork-api-router/conf.d/`
- env file: `/Library/Application Support/sdkwork-api-router/router.env`
- data home: `/Library/Application Support/sdkwork-api-router/data/`
- log home: `/Library/Logs/sdkwork-api-router/`
- run home: `/Library/Application Support/sdkwork-api-router/run/`

### Windows

- product root: `C:\Program Files\sdkwork-api-router\`
- current control home: `C:\Program Files\sdkwork-api-router\current\`
- versioned release payload: `C:\Program Files\sdkwork-api-router\releases\<version>\`
- config home: `C:\ProgramData\sdkwork-api-router\`
- config file: `C:\ProgramData\sdkwork-api-router\router.yaml`
- config fragments: `C:\ProgramData\sdkwork-api-router\conf.d\`
- env file: `C:\ProgramData\sdkwork-api-router\router.env`
- data home: `C:\ProgramData\sdkwork-api-router\data\`
- log home: `C:\ProgramData\sdkwork-api-router\log\`
- run home: `C:\ProgramData\sdkwork-api-router\run\`

## Config Discovery

Primary config discovery order:

1. `router.yaml`
2. `router.yml`
3. `router.json`
4. `config.yaml`
5. `config.yml`
6. `config.json`

Supported overlays under `conf.d/*.{yaml,yml,json}` are loaded after the primary file in lexical order.

## Config Precedence

Effective precedence from lowest to highest:

- built-in defaults
- environment fallback
- config file
- CLI

Discovery exception:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`

These two variables are read first so the runtime can locate the config files. After discovery completes, file-defined business fields override environment fallback values.

## Release Manifest Contract

The generated `current/release-manifest.json` is the control-plane bridge between `current/` and `releases/<version>/`.

It records:

- manifest schema and generation metadata: `layoutVersion`, `installedAt`
- install topology and release selection: `installMode`, `productRoot`, `controlRoot`, `releasesRoot`, `releaseRoot`, `releaseVersion`
- resolved target descriptor: `target`
- installed service payload inventory: `installedBinaries`
- active release version
- active release root
- resolved router binary path
- resolved admin and portal site roots
- the installed bootstrap data and `deploy/` asset roots inside the active release payload: `bootstrapDataRoot`, `deploymentAssetRoot`
- the installed release payload `release-manifest.json` and `README.txt` paths: `releasePayloadManifest`, `releasePayloadReadmeFile`
- mutable config, data, log, and run roots plus the canonical config file path: `configRoot`, `configFile`, `mutableDataRoot`, `logRoot`, `runRoot`

Operators should treat `current/release-manifest.json` as generated state. Do not hand-edit it during normal operation.

## Database Defaults

- `portable`
  - defaults to SQLite under the portable `data/` root
- `system`
  - defaults to PostgreSQL

In `system` mode, PostgreSQL is the standard contract. SQLite remains a local-validation convenience, not the production default.
