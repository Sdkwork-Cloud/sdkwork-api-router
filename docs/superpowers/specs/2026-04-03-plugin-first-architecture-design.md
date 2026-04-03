# Plugin-First Architecture Design

**Status:** approved for autonomous architecture-first execution

**Goal:** turn `sdkwork-api-router` into a plugin-first platform with explicit extension standards, pluggable component seams, storage and cache driver abstractions, and a stable evolution path for provider, product, and infrastructure modules.

## Why This Is the New Primary Direction

The repository already has important plugin-first ingredients:

- extension runtimes for `builtin`, `connector`, and `native_dynamic`
- a shared storage contract through `AdminStore`
- dialect-aware config that already recognizes `sqlite`, `postgres`, `mysql`, and `libsql`
- hot reload, rollout supervision, and embedded or standalone runtime composition

What is missing is not another runtime feature. What is missing is a unified standard that says:

- what counts as a plugin
- which seams are required to be pluggable
- how plugins declare capabilities
- how infrastructure drivers are loaded and selected
- how lifecycle, health, reload, permissions, and compatibility are enforced

Without that standard, the codebase remains extensible in several places but not yet systematically pluggable.

## Target State

SDKWork should evolve into a plugin-first platform with four extension classes:

### 1. Core component plugins

These are compile-time or in-process components selected through stable Rust traits and driver registries.

Examples:

- storage drivers
- cache drivers
- secret backends
- telemetry sinks
- queue or scheduler backends
- billing strategy engines

### 2. Runtime service plugins

These are runtime-loaded components that execute provider and media behavior.

Examples:

- provider adapters
- connector processes
- native dynamic libraries
- future WASM sandbox runtimes if adopted later

### 3. Policy plugins

These are policy engines that do not own transport, but do own decision logic.

Examples:

- routing strategies
- quota strategies
- billing strategies
- fraud or abuse detection
- residency or compliance checks

### 4. Product plugins

These are feature packs that compose backend APIs, workflow rules, assets, admin views, and portal views.

Examples:

- API key management modules
- image or video job centers
- commerce modules
- provider ops center modules

## Architecture Principles

### Principle 1: Every pluggable seam gets a contract crate

A component is not considered pluggable unless it has:

- a dedicated contract trait or ABI
- capability metadata
- configuration schema
- health or readiness semantics
- lifecycle hooks where applicable
- compatibility versioning

### Principle 2: Canonical core stays small

The platform core should own:

- identity
- tenancy
- project scope
- credentials
- capability registry
- route execution orchestration
- durable audit and usage records

Everything else should be attachable.

### Principle 3: Plugins must not bypass governance

A plugin may extend behavior, but it may not bypass:

- auth boundaries
- tenant or project isolation
- secret management
- routing evidence
- billing event emission
- observability hooks

### Principle 4: Runtime plugins and infrastructure drivers are different

Do not force every extensibility seam into one loading model.

The best model differs by component type:

- storage drivers should use compile-time trait implementations and factory selection
- cache drivers should use compile-time trait implementations and factory selection
- provider runtimes may use builtin, connector, native dynamic, or later WASM execution
- product modules may use backend registration plus frontend route manifests

## The Required Plugin Standard

Every plugin-capable component class should satisfy the same standard vocabulary.

### Identity

- `plugin_id`
- `plugin_kind`
- `version`
- `compatibility_range`

### Capability declaration

- `declared_capabilities`
- `required_permissions`
- `exposed_operations`
- `supported_modalities`

### Configuration

- JSON-schema-like config definition
- default values
- environment overrides
- validation hook
- redactable secret fields

### Lifecycle

- `discover`
- `validate`
- `start`
- `health_check`
- `reload`
- `drain`
- `shutdown`

### Observability

- structured logs
- metrics labels
- trace annotations
- event hooks for admin evidence

### Safety

- trust policy
- signature or provenance requirement where needed
- resource constraints
- failure isolation mode
- rollback strategy

## Component Seams That Must Become Formally Pluggable

### Storage

Current state:

- `AdminStore` already abstracts durable control-plane persistence
- SQLite and PostgreSQL already exist
- config already understands `mysql` and `libsql`, but they are not implemented here

Required end state:

- split the monolithic `AdminStore` into capability facets while keeping a facade for compatibility
- add a `StorageDriverFactory` standard
- support storage driver registration by dialect
- make migrations and compatibility checks part of the driver contract

Proposed storage facets:

- `IdentityStore`
- `TenantStore`
- `CatalogStore`
- `CredentialStore`
- `RoutingStore`
- `UsageStore`
- `BillingStore`
- `ExtensionStore`
- `CommerceStore`

The facade may still exist as `AdminStore`, but internally it should compose these facets rather than force every backend to implement one giant trait without structure.

### Cache

Current state:

- there are local caches and runtime reuse paths, but no single cache contract

Current implementation progress:

- `sdkwork-api-cache-core` now defines:
  - `CacheStore`
  - `DistributedLockStore`
  - `CacheDriverFactory`
  - `CacheDriverRegistry`
- `sdkwork-api-cache-memory` is now the first concrete driver
- standalone runtime cache initialization now resolves linked cache backends through a registry-driven build path
- gateway route-decision reuse now runs through the standardized cache backend instead of a custom process-local map
- gateway model list and retrieve reads now have a first `CapabilityCatalogCache` helper on top of `CacheStore`
- admin catalog mutations and gateway-side model deletion now invalidate capability catalog entries through shared cache tags
- capability catalog cache stays opt-in by runtime mode: the combined product runtime enables it, while standalone gateway and admin services enable it only when the configured backend supports shared cross-process coherence and keep it off by default with `memory`
- `redis` is now linked into the standalone runtime as the first shared cache backend target

Required end state:

- add `sdkwork-api-cache-core`
- define `CacheStore` for key/value, TTL, atomic get-or-populate, invalidation, and tagged purge
- define `DistributedLockStore` separately from generic cache
- define `RouteDecisionCache`, `CapabilityCatalogCache`, and `SessionStateCache` on top of those primitives

Driver targets:

- in-memory
- Redis
- optional no-op cache for test or embedded modes

### Secret backends

Current state:

- secret backend strategy already exists and is more mature than many codebases

Required end state:

- keep the current abstraction
- reframe it as one standardized plugin seam under the same lifecycle and config rules

### Provider runtimes

Current state:

- `builtin`, `connector`, and `native_dynamic` are already present

Required end state:

- standardize provider runtime manifests around capability and operation declarations
- make provider operation compatibility testable through a golden contract suite
- support richer multimodal and async operation declarations without special-casing each route family

### Policy engines

Current state:

- routing logic is rich, but policy engines are embedded in app logic

Required end state:

- formalize routing strategy plugins
- formalize billing strategy plugins
- formalize admission policy plugins
- make policy bundles attachable to tenant, project, API key group, and request context

### Product modules

Current state:

- admin and portal are products, but not yet organized as backend-frontend modules with a stable module manifest

Required end state:

- define a product-module manifest with backend routes, admin surfaces, portal surfaces, migrations, and permissions
- allow operator-only modules and tenant-visible modules
- support feature flags and staged rollout per module

## The Best Architectural Path

Do not replace the current architecture.

Refactor the current architecture into a plugin-first standard using these steps:

### Step 1: Standardize seams

Define contract crates and manifests before broad feature work.

### Step 2: Introduce infrastructure driver registries

Storage, cache, secret, and telemetry drivers should be selected through registries, not hidden conditionals.

### Step 3: Normalize runtime plugin manifests

Provider runtime packages should declare operations and modalities in a way the router can inspect generically.

### Step 4: Make policy attach points explicit

API key groups, project profiles, and capability policies become stable policy anchors.

### Step 5: Turn feature surfaces into modules

Large operator and portal features should ship as modules rather than as permanent hard-coded product assumptions.

## Data and Control Plane Consequences

A plugin-first platform needs more metadata, not less.

New core registries should eventually exist for:

- plugin packages
- plugin installations
- plugin instances
- plugin capabilities
- plugin compatibility snapshots
- plugin health state
- plugin configuration revisions
- plugin permissions

The current extension and rollout tables are a strong starting point, but they should become generic enough to describe more than provider runtimes over time.

## Database and Cache Strategy

### Database

The database layer should follow this standard:

- canonical domain models remain storage-agnostic
- each dialect backend implements facet traits
- a storage driver factory resolves the concrete driver from config
- migration ownership belongs to the driver
- compatibility checks run before the backend becomes live

Initial driver targets:

- SQLite
- PostgreSQL
- LibSQL
- MySQL

The repo already anticipates MySQL and LibSQL at the config level. The architecture work should make those official driver targets rather than dormant enum values.

### Cache

The cache layer should follow this standard:

- cache access goes through `CacheStore`
- lock coordination goes through `DistributedLockStore`
- queue-like deferred work does not overload the cache contract
- local in-memory fallback remains available for embedded or single-node mode

Initial cache driver targets:

- memory
- Redis

## Multimodal and Async Consequences

The plugin-first architecture must be compatible with a broad media future:

- text
- image
- audio
- video
- music
- async generation jobs

This means capability metadata must be modality-aware and operation-aware.

The platform should not treat image, video, and music as isolated product hacks. They should be routable and billable capabilities backed by plugin-declared operations.

## Recommended Immediate Execution Order

The best next path is:

### Phase A: Architecture foundation

- write the plugin-first standard
- add architecture docs
- define contract crates and seam inventory

### Phase B: Infrastructure pluggability

- split storage facets from `AdminStore`
- add storage driver factory
- add cache-core and memory driver
- add config support for cache backend selection

### Phase C: Runtime and policy standardization

- formalize plugin manifest shape
- formalize policy engine seams
- attach policy to project and API key group

### Phase D: Product module uplift

- make major admin and portal capabilities module-driven
- surface plugin inventory and capability registry in the control plane

## Why This Path Is Better Than a Monolith

It preserves the strongest parts of the current system:

- durable domain boundaries
- explicit storage contracts
- extension runtime supervision
- hot reload and rollout control

while adding what is still missing:

- explicit seam ownership
- reusable plugin standards
- infrastructure driver modularity
- a clear path to more databases and real cache backends
- a future-proof base for multimodal and product-density growth

## Success Criteria

This architecture program succeeds when:

- every major component seam has a named contract and loading model
- storage and cache are driver-based, not conditionally wired
- provider and policy plugins share a consistent capability vocabulary
- plugin lifecycle and trust rules are explicit
- the control plane can inspect installed plugins, health, compatibility, and config revision
- adding a new database, cache, provider runtime, or product module no longer requires ad hoc cross-cutting rewrites
