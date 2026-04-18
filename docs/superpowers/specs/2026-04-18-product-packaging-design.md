# Product Packaging Design

## Goal

Define a clean product-line contract for SDKWork API Router so the repository exposes exactly two official distributable products:

- `sdkwork-api-router-product-server`
- `sdkwork-router-portal-desktop`

The design intentionally treats the application as greenfield packaging work. Compatibility with the previous release taxonomy is not a goal.

## Product Line

### 1. Server Product

The server product is the only official server-side runtime artifact. It is built around `router-product-service` and exposes a single public web/API port. The bundle must include:

- service binaries
- portal static assets
- admin static assets
- bootstrap data
- deploy assets
- install/runtime metadata

The server bundle is the canonical input for:

- native server installs
- Docker image builds
- Docker Compose deployments
- Helm deployments

### 2. Desktop Product

The desktop product is the only official desktop SKU. It is based on `apps/sdkwork-router-portal` and opens the portal console by default.

Public release assets must not include:

- standalone `sdkwork-router-admin` desktop installers
- standalone `web` release bundles intended for GitHub release consumers

The portal desktop bundle may still embed admin static assets because the desktop runtime hosts both portal and admin surfaces behind one local product runtime.

## Release Policy

GitHub Releases must publish only official user-facing product assets:

- server product archives
- desktop product installers/bundles
- one release-level machine-readable asset index, `release-catalog.json`

Governance evidence, telemetry exports, sync audits, window snapshots, and similar artifacts remain workflow artifacts and attestations. They are not user-facing release downloads.

## Build and Packaging Policy

### Official Naming

The public asset naming scheme is:

- `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz`
- `sdkwork-api-router-product-server-<platform>-<arch>.tar.gz.sha256.txt`
- `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json`
- platform-native portal desktop installers produced from `sdkwork-router-portal`
- `release-catalog.json`

The server-side release contract is an archive set, not just a single tarball. The external manifest describes the archive identity and the embedded bundle payload contract. `release-catalog.json` aggregates those official per-asset manifests into one machine-readable release index and should expose enough metadata for automation to reason about the release without opening every manifest.

The catalog contract should include:

- a top-level `generatedAt` timestamp
- per-variant `variantKind`
- per-variant `primaryFileSizeBytes`
- per-variant explicit `checksumAlgorithm`

### Internal Build Outputs

Internal build outputs may still contain intermediate files such as:

- compiled Rust binaries
- built browser assets
- Tauri bundle directories

But only the official server and desktop products are considered release artifacts.

## Runtime Topology

### Server

- runtime entrypoint: `router-product-service`
- mode: `server`
- public bind default: `0.0.0.0:3001`
- database default: PostgreSQL
- configuration precedence:
  - built-in defaults
  - environment fallback
  - config file
  - CLI

### Desktop

- UI shell: `sdkwork-router-portal`
- hosted surfaces: portal plus admin
- unified public port contract: `3001`
- bind scope modes:
  - local mode: `127.0.0.1:3001`
  - shared access mode: `0.0.0.0:3001`

The desktop runtime contract must converge on a single user-visible endpoint so switching between local and shared access does not require clients to discover a new port each launch.

## Install Layout Standard

The native server install standard follows nginx/redis-style operational packaging:

- immutable versioned program payload
- mutable config outside the program payload
- mutable data outside the program payload
- mutable logs outside the program payload
- mutable run state outside the program payload
- service-manager startup assets

System installs should move toward a layout shaped like:

- `releases/<version>/`
- `current/`
- `config/`
- `data/`
- `log/`
- `run/`

This spec allows incremental implementation, but the release/build/install interfaces must stop treating raw repository build output as the public product contract.

## Desktop Packaging Direction

The desktop package standard is:

- portal desktop is the only official desktop installer
- admin desktop is development-only, not release-facing
- desktop bundling may include embedded site assets for admin and portal
- desktop settings must be able to control whether the local runtime is local-only or shared

## Documentation Standard

Documentation must describe the product line in terms of the two official SKUs instead of mixing:

- raw service builds
- web static bundles
- admin desktop builds
- portal desktop builds

User-facing docs should answer:

- how to build server
- how to build desktop
- how to publish a release
- how to deploy server online
- how to initialize PostgreSQL-backed production config
- how to install and run the desktop product

## Implementation Phases

### Phase 1

- converge release workflow to official SKUs
- stop publishing admin desktop and web public assets
- package only server bundle and portal desktop bundle

### Phase 2

- converge install/build helper plans to official server packaging language
- stop describing intermediate artifacts as public products

### Phase 3

- refine desktop runtime packaging and settings-centered runtime mode control

### Phase 4

- complete docs and verification coverage around release/install behavior
