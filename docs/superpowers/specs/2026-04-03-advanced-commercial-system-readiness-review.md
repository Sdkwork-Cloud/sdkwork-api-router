# Advanced Commercial System Readiness Review

**Status:** re-audited against the local source tree on 2026-04-03

**Goal:** determine whether the current `sdkwork-api-router` repository already meets the standard of an advanced commercial API router platform, identify the remaining gaps precisely, and define the best target-state solution plus landing priorities.

## Executive Verdict

`sdkwork-api-router` is already a strong clean-slate foundation, but it is **not yet** a finished elite commercial system.

The architecture direction is already better than most open-source routers:

- plugin-first backend boundaries are real
- storage, cache, runtime, policy, and product seams are explicit
- admin and portal are already package-split React products with module metadata
- multimodal API surface breadth is already broad

But the system is still behind the strongest commercial products in the areas that matter most for revenue correctness and day-2 operations:

- request-path settlement correctness
- durable async multimodal job orchestration
- canonical commercial control-plane coverage
- platform-wide plugin governance
- production-grade multi-database parity
- finops reproducibility, reconciliation, and export
- launch-readiness verification for SLOs, resilience, and operational drift

The correct strategy is still:

- keep the current architecture
- do not rewrite toward a `new-api`-style monolith
- finish the missing commercial kernel and governance layers in the right order

## Comparison Baseline

### What Is Already Better Than `new-api`

The current repository remains ahead in architecture quality and long-term evolvability:

- domain, app, storage, interface, runtime, and extension boundaries are explicit instead of collapsed into one fast-moving code path
- storage and cache already have registry-style seams
- routing, quota, and billing already use plugin-style registries
- runtime rollout, reload, and supervision are first-class backend concerns
- admin and portal already expose package-first route and module manifests

This is the correct long-term architecture for a serious commercial platform.

### What `new-api` Still Does Better

The local `new-api` comparison baseline still shows stronger day-to-day product density for media operations:

- provider-specific task adapters are denser
- media capability coverage is presented as first-class product behavior instead of only protocol breadth
- long-running media workflows are closer to an operator-facing product surface

The useful lesson is not to copy its monolithic transport shape. The useful lessons are:

- keep expanding provider density for image, audio, video, and music
- model long-running media work as a canonical job system
- provide control-plane depth for daily media operations

### What The OpenRouter-Inspired Standard Still Demands

The repository already follows the right general OpenRouter direction, but the final commercial bar still requires:

- first-class provider and model capability metadata
- fully observable routing policy and fallback evidence
- explicit cost, latency, privacy, and cache visibility
- deterministic settlement and pricing evidence per request
- a stronger commercial control plane than a routing-only dashboard

The current repository has already made progress on routing profiles, compiled routing snapshots, provider health, and billing event summaries. What is still missing is the fully correct settlement kernel underneath those views.

## Verified Current State

The following facts are already real in the source tree today.

### 1. Plugin-first backend direction is real

Evidence:

- `docs/superpowers/specs/2026-04-03-plugin-first-architecture-design.md`
- `crates/sdkwork-api-storage-core/src/lib.rs`
- registry-driven storage, cache, quota-policy, and billing-policy seams are already present in the codebase and prior audits

Assessment:

- the backend architecture is commercially credible
- the remaining work is completion, not re-architecture

### 2. Multimodal API surface breadth is already strong

Evidence:

- `crates/sdkwork-api-interface-http/src/lib.rs`
- `crates/sdkwork-api-interface-http/tests/images_route.rs`
- `crates/sdkwork-api-interface-http/tests/videos_route.rs`
- `crates/sdkwork-api-interface-http/tests/music_route.rs`
- `crates/sdkwork-api-interface-http/tests/audio_speech_route.rs`
- `crates/sdkwork-api-interface-http/tests/transcriptions_route.rs`
- `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- `crates/sdkwork-api-interface-http/tests/realtime_sessions_route.rs`
- `crates/sdkwork-api-interface-http/tests/webhooks_route.rs`
- `crates/sdkwork-api-interface-http/tests/vector_stores_route.rs`

Assessment:

- the router already exposes a broad OpenAI-compatible surface across text, images, audio, video, music, uploads, vector stores, realtime, and webhooks
- breadth is no longer the main blocker
- durability and commercial operating model are the blockers

### 3. Canonical account-kernel seams and schema are partially landed

Evidence:

- `crates/sdkwork-api-storage-core/src/lib.rs`
- `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`
- `crates/sdkwork-api-app-billing/src/lib.rs`

What is already present:

- canonical account, lot, hold, ledger, request-fact, pricing-plan, pricing-rate, and settlement record types
- SQLite CRUD for the canonical account kernel
- PostgreSQL schema creation for the canonical account kernel
- balance projection and hold planning in `sdkwork-api-app-billing`
- canonical payable-account resolution via `resolve_payable_account_for_gateway_subject(...)`

Assessment:

- the data model is now credible
- the mutation path is still incomplete

### 4. Admin and portal package modularity is real

Evidence:

- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routeManifest.ts`
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/index.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/application/router/routeManifest.ts`
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/index.ts`

What is already true:

- admin and portal are split into React packages
- both products already consume dedicated backend API packages
- route manifests already carry `moduleId`, `pluginId`, `pluginKind`, permissions, navigation, and loading metadata

Assessment:

- the frontend product composition model is correct
- the missing work is commercial surface completeness, not packaging direction

## Commercial Readiness Scorecard

This scorecard is intentionally strict. It measures commercial readiness, not architectural promise.

- architecture and extensibility: `8.5/10`
- multimodal API breadth: `8/10`
- request-path settlement correctness: `3/10`
- admin commercial control plane: `5/10`
- portal commercial self-service plane: `5/10`
- async multimodal operations: `2/10`
- plugin governance and module operations: `4/10`
- multi-database production parity: `4/10`
- finops reproducibility and export: `4/10`
- performance, resilience, and launch gates: `4/10`

Overall commercial readiness: **foundation-grade, not launch-complete**

## Gaps That Still Block An Elite Commercial System

### 1. Request-path settlement is still not transaction-safe

Evidence:

- `crates/sdkwork-api-app-billing/src/lib.rs` currently exposes `plan_account_hold(...)` and `resolve_payable_account_for_gateway_subject(...)`, but the live mutation path still centers on `persist_ledger_entry(...)` and `check_quota(...)`
- `crates/sdkwork-api-interface-http/src/lib.rs` still calls `enforce_project_quota(...)` and persists coarse ledger usage after the request instead of executing canonical hold and settlement flow
- `crates/sdkwork-api-storage-core/src/lib.rs` still lacks a public account-command transaction seam

Consequence:

- partial failures can still leave commercial state inconsistent
- the most important commercial invariant is not yet enforced

### 2. Admin and portal are still operating on compatibility-era billing views

Evidence:

- `crates/sdkwork-api-interface-admin/src/lib.rs` currently exposes billing summary, ledger, billing events, quota policies, routing, and extension operations, but not canonical account balance, benefit-lot, hold, pricing-plan, pricing-rate, or settlement control-plane APIs
- `crates/sdkwork-api-interface-portal/src/lib.rs` exposes workspace, usage, billing summaries, commerce, routing, and API key groups, but not canonical tenant-facing account, settlement, or job surfaces
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/routes.ts` has no admin routes for pricing plans, commercial accounts, settlements, plugin inventory, or finops operations
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-core/src/routes.ts` has no routes for media jobs, settlement explorer, or canonical pricing posture

Consequence:

- the products look structured, but they still do not expose the commercial kernel that a real operator or customer needs

### 3. Durable async multimodal jobs are still absent

Evidence:

- the repository has broad route coverage, but current code search still shows no shipped canonical `ai_async_job`, `ai_async_job_attempt`, `ai_generated_asset`, or callback-delivery kernel
- the gap is explicitly called out in:
  - `docs/superpowers/specs/2026-04-03-router-implementation-audit-and-upgrade-plan.md`
  - `docs/superpowers/specs/2026-04-03-commercial-system-gap-assessment-and-target-solution.md`

Consequence:

- long-running image, video, audio, and music workloads still lack durable retries, callback reconciliation, asset lifecycle tracking, and operator job visibility

### 4. Plugin governance is incomplete beyond extension runtime operations

Evidence:

- `crates/sdkwork-api-interface-admin/src/lib.rs` already exposes extension installation, instance, runtime-status, and rollout endpoints
- but the current codebase does not yet expose a unified backend plugin inventory, compatibility snapshot center, module-level feature flags, or backend route ownership by product module

Consequence:

- the architecture is plugin-first in direction
- the platform is not yet plugin-governed end to end

### 5. Multi-database support is not commercially complete

Evidence:

- SQLite is the strongest real commercial backend today
- PostgreSQL currently proves canonical table creation in `crates/sdkwork-api-storage-postgres/tests/integration_postgres.rs`, but not the full transaction-backed commercial mutation kernel
- `crates/sdkwork-api-storage-mysql/src/lib.rs` only returns `dialect_name() == "mysql"`
- `crates/sdkwork-api-storage-libsql/src/lib.rs` only returns `dialect_name() == "libsql"`

Consequence:

- the platform still has one true commercial backend and one partial backend
- this is not enough for an elite commercial platform

### 6. Finops reproducibility is still incomplete

Evidence:

- canonical pricing-plan and pricing-rate tables exist
- billing events already separate upstream cost and customer charge
- API key groups already carry default accounting mode
- but the live request path does not yet resolve a canonical pricing plan during settlement or attach an immutable settlement-time pricing snapshot

Consequence:

- finance and BI cannot yet reproduce every historical charge from canonical settlement evidence alone

### 7. Performance, resilience, and launch gates are not yet frozen

Evidence:

- the current closure plan prioritizes correctness, which is correct
- but the repository still does not define a formal commercial launch gate for:
  - p95 latency and error budgets
  - duplicate callback suppression
  - retry and replay behavior
  - settlement drift detection
  - operator-facing backlog and degraded-runtime posture

Consequence:

- even after feature completion, the platform would still need an explicit launch-readiness gate before it should be called elite or production-complete

## The Best Target-State Solution

The best target state is a six-layer commercial platform.

### Layer 1: Canonical identity and payable subject

- canonical API key identity
- explicit auth subject resolution
- active primary payable account resolution

### Layer 2: Transaction-safe commercial settlement kernel

- hold planning
- hold creation, release, and capture
- ledger and allocation evidence
- request facts and metrics
- immutable settlement records with deterministic pricing evidence

### Layer 3: Durable multimodal execution kernel

- synchronous fast-path where appropriate
- async jobs for long-running media work
- attempts, callbacks, assets, and finalization

### Layer 4: Commercial control plane

Admin should gain dedicated modules or workbenches for:

- commercial accounts
- pricing plans and pricing rates
- hold and settlement explorer
- reconciliation and finance export
- async media jobs
- plugin inventory and compatibility
- module rollout and feature flags

Portal should gain dedicated modules or workbenches for:

- canonical account balance and lot posture
- settlement history and evidence
- recharge and pricing posture
- media job lifecycle and generated assets
- customer-facing routing and spend evidence

### Layer 5: Platform governance and portability

- unified plugin inventory across runtime, policy, storage, cache, secrets, and product modules
- compatibility evidence and config revision tracking
- PostgreSQL parity first, then MySQL and LibSQL
- explicit unsupported behavior until parity is real

### Layer 6: Launch-readiness assurance

- SLO dashboards and operator alerts
- resilience and replay test matrix
- settlement drift detection
- finops reproducibility checks
- final commercial-v1 release gates

## Best Execution Order

The best landing order remains:

1. transaction-safe account mutation kernel
2. gateway cutover from quota-only admission to canonical settlement admission
3. admin and portal cutover to canonical account and settlement APIs
4. durable async multimodal job kernel
5. plugin inventory and backend product-module governance
6. PostgreSQL parity, then MySQL and LibSQL
7. reconciliation, finance export, and enterprise governance polish
8. performance, resilience, and commercial launch gate freeze

This order is still optimal because it fixes correctness before density, and density before portability or launch polish.

## Release Gates For A True Commercial V1

The platform should not call itself a finished elite commercial system until all of the following are true:

- every billable gateway request resolves canonical subject, payable account, hold, and settlement through a transaction-safe command path
- long-running media work uses a durable job kernel with retries, callbacks, and asset records
- admin exposes pricing, account, settlement, plugin, rollout, and finops workbenches
- portal exposes canonical account, settlement, recharge, and media-job evidence
- SQLite and PostgreSQL are both production-valid for the commercial kernel
- MySQL and LibSQL either work for the same kernel or fail honestly as unsupported
- finance can reproduce historical charges from immutable settlement evidence
- performance and resilience gates are documented, tested, and visible in the control plane

## Final Assessment

The current repository is already architecturally strong and commercially promising.

It is **not yet** at the level of the best commercial API router systems.

The missing work is now sharply bounded:

- finish the transaction-safe settlement kernel
- cut the control plane onto that kernel
- add the durable async media job layer
- complete plugin governance and database parity
- freeze launch-readiness gates for performance, resilience, and finops correctness

That is the shortest path from the current strong foundation to an elite commercial API router platform.
