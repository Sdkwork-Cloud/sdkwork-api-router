# Plugin-First Architecture

This page defines the target plugin-first architecture standard for SDKWork API Router.

## Purpose

SDKWork already supports extension runtimes and storage abstraction, but the platform needs a consistent standard for all pluggable seams.

The target model is:

- component seams are explicit
- plugins declare capabilities and permissions
- infrastructure backends load through driver registries
- runtime modules expose lifecycle and health contracts
- the control plane can inspect installed plugins and backend drivers

## Plugin Classes

SDKWork should support four plugin classes:

- core component plugins
  - storage
  - cache
  - secret backends
  - telemetry sinks
- runtime service plugins
  - builtin providers
  - connector processes
  - native-dynamic libraries
- policy plugins
  - routing strategies
  - quota strategies
  - billing strategies
- product plugins
  - admin modules
  - portal modules
  - media workflow modules

Product plugins must preserve the current React workspace topology:

- `admin` and `portal` stay package-based applications
- plugin/module boundaries are layered onto existing React packages, route manifests, and workbench registries
- pluginization does not imply splitting `admin` or `portal` into new top-level apps
- shared package contracts should carry capability metadata, navigation descriptors, lazy-load hooks, and permission declarations

## Standard Contract

Every plugin-capable seam should expose:

- identity
  - plugin id
  - plugin kind
  - version
  - compatibility range
- capability declaration
  - operations
  - modalities
  - permissions
- configuration
  - schema
  - defaults
  - validation
  - secret fields
- lifecycle
  - discover
  - validate
  - start
  - health check
  - reload
  - drain
  - shutdown
- observability
  - logs
  - metrics
  - traces
  - admin evidence

## Pluggable Infrastructure

### Storage

Storage now exposes a compatibility-first facet model: `AdminStore` remains the stable facade, while focused capability traits project narrower seams for `IdentityStore`, `TenantStore`, `CatalogStore`, `CredentialStore`, `RoutingStore`, `UsageStore`, `BillingStore`, and `ExtensionStore`.

Driver selection now routes through a storage driver factory registry rather than hard-coded runtime matching.

Documented or anticipated dialects:

- SQLite
- PostgreSQL
- MySQL
- LibSQL

### Cache

Cache should become a first-class seam with:

- `CacheStore`
- `DistributedLockStore`
- backend selection through config and a cache driver registry

Initial targets:

- memory
- Redis

Current implementation status:

- `sdkwork-api-cache-core` now defines the cache primitives plus the cache driver registry contract
- the standalone runtime now resolves linked `memory` and `redis` cache drivers through that registry
- the gateway route-decision handoff cache is now the first runtime consumer migrated onto the standard cache seam instead of a bespoke global map
- a first `CapabilityCatalogCache` helper now exists for gateway model list and retrieve reads
- catalog cache entries are tag-invalidated through admin catalog mutations and gateway-side model deletion
- catalog cache activation is opt-in today: the shared product runtime enables it explicitly, while standalone gateway and admin services enable it only when the configured cache backend supports shared cross-process coherence, which is currently `redis`

## Runtime Plugins

Provider runtimes remain the most advanced plugin seam today.

The current manifest contract now makes runtime packages explicit about:

- supported operations through `capabilities`
- supported modalities through `supported_modalities`
- runtime compatibility through `runtime_compat_version`
- configuration schema versions through `config_schema_version`

## Policy and Product Follow-On

API key groups, routing profiles, billing strategies, and product modules should be layered on top of the same plugin-first standards.

That keeps policy and product growth aligned with the same capability, lifecycle, and observability model used by infrastructure and provider runtimes.

For the product layer specifically, the implementation target is a package-first React plugin model:

- `apps/sdkwork-router-admin/packages/*` remains the host surface for admin modules
- `apps/sdkwork-router-portal/packages/*` remains the host surface for portal modules
- each package can expose manifest-style metadata for routes, permissions, navigation, preload strategy, and SDK dependencies
- the host shell composes those package manifests rather than owning hard-coded feature wiring
