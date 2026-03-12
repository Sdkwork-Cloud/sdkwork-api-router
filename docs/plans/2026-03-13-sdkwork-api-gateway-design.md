# SDKWork API Gateway Design

**Date:** 2026-03-13

**Status:** Approved for planning

## Goal

Build a Rust-based API gateway server that is fully compatible with the OpenAI API style, supports local and server deployment modes, can also run embedded inside a Tauri application, and provides a complete multi-tenant control plane for channels, proxy providers, routing, credentials, usage, quota, billing, and audit.

The first delivery stage must:

- define the full OpenAI-compatible contract surface
- implement `/v1/chat/completions`
- implement `/v1/responses`
- implement `/v1/models`
- implement `/v1/embeddings`
- support SSE streaming
- support both `Gateway API Key` and `JWT`
- support embedded and standalone runtime modes
- follow controller/service/repository layering through DDD-oriented Rust boundaries

## Product Positioning

The system is not a single-provider proxy. It is a multi-tenant gateway platform that separates:

- `Channel`: model ecosystem or vendor capability family such as OpenAI, Google, Anthropic, Aliyun, DeepSeek
- `ProxyProvider`: a concrete upstream path inside or across channels such as official endpoints, OpenRouter, Ollama, or enterprise proxy infrastructure

One capability such as `/v1/chat/completions` may be delivered by many channels, and each channel may expose multiple proxy providers. Routing is therefore based on capability, policy, health, cost, latency, and quota, not on a hardcoded vendor decision.

## Architecture Summary

The recommended architecture is:

- logical full microservices
- physically composable runtime

This means service boundaries are real in code and contracts, but runtime deployment supports two modes:

1. `Server Mode`
   Separate processes for gateway, routing, identity, catalog, credential, usage, billing, and admin services.
2. `Embedded Mode`
   The same Rust crates are assembled into an in-process runtime host and embedded into a Tauri desktop application.

This design keeps long-term scalability without sacrificing local desktop embeddability.

## Technology Selection

### Backend

- `Rust`
- `Axum` for HTTP services and OpenAI-compatible API exposure
- `Tokio` for async runtime
- `tower` and `tower-http` for middleware, auth, limits, tracing, and HTTP policies
- `reqwest` for upstream provider calls
- `serde` and `serde_json` for protocol contracts
- `schemars` for schema generation and contract metadata
- `thiserror` and `anyhow` for error modeling
- `sqlx` as the storage execution layer
- `sea-query` as the SQL builder and dialect unifier
- `tracing`, `tracing-subscriber`, `OpenTelemetry`, and Prometheus for observability

### Frontend and Desktop

- `pnpm` workspace
- `Turbo`
- `React`
- `TypeScript`
- `Vite`
- `Tauri 2`
- `TanStack Router`
- `TanStack Query`
- `zustand` for lightweight local UI state where needed

## Backend Service Boundaries

### Data Plane

- `gateway-service`
  Exposes `/v1/*`, handles auth, protocol normalization, error mapping, and stream relay.
- `routing-service`
  Computes routing decisions based on policy, capability, health, quota, latency, and cost.
- `streaming relay`
  Treated as a core gateway concern with explicit SSE handling.

### Control Plane

- `identity-service`
  Users, roles, permissions, JWT sessions, Gateway API keys.
- `tenant-project-service`
  Tenants, projects, membership, environment boundaries.
- `catalog-service`
  Channels, proxy providers, models, capabilities, aliases.
- `credential-service`
  Upstream credential storage, rotation, and secure retrieval.
- `usage-service`
  Request facts, token usage, errors, latency, provider usage.
- `billing-service`
  Quotas, limits, ledgers, settlement, pricing policies.
- `admin-api-service`
  Control plane API used by React Web and Tauri hosts.

## Domain Model

The system is organized around these core domains:

- `Identity`
- `Tenant`
- `Catalog`
- `Credential`
- `Routing`
- `Gateway`
- `Usage`
- `Billing`

### Key Aggregate Roots

- `Tenant`
- `Project`
- `GatewayApiKey`
- `Channel`
- `ProxyProvider`
- `ModelCatalogEntry`
- `UpstreamCredential`
- `RoutingPolicy`
- `ProviderHealthSnapshot`
- `UsageStatement`
- `QuotaPolicy`
- `LedgerAccount`

### Critical Domain Decisions

- `Channel` and `ProxyProvider` are separate concepts.
- `ModelCatalogEntry` is not just a string model name. It stores capability, lifecycle, modality, context length, routing metadata, and billing metadata.
- `Capability` is a first-class domain concept.
- `GatewayApiKey` and `UpstreamCredential` are fully separate security objects.
- `Usage` and `Ledger` are different domains and must not collapse into a single request record.

## Contract-First API Design

The protocol layer is split into three contract levels:

1. `External OpenAI Contract`
   Public OpenAI-compatible request and response models.
2. `Canonical Gateway Contract`
   Internal normalized request, response, usage, and streaming models.
3. `Provider Adapter Contract`
   Upstream-specific request and response mapping boundary.

### External Contract Scope

The project should define the structure for the broader OpenAI-compatible API families, even if only a subset is implemented in the first stage:

- `responses`
- `chat_completions`
- `models`
- `embeddings`
- `files`
- `uploads`
- `audio`
- `images`
- `moderations`
- `realtime`
- `assistants`
- `vector_stores`
- `batches`
- `webhooks`
- `evals`

### Stage 1 Execution Scope

Only these APIs must be implemented in the first delivery stage:

- `/v1/chat/completions`
- `/v1/responses`
- `/v1/models`
- `/v1/embeddings`
- `text/event-stream`

### Stream Handling

Streaming is modeled explicitly through:

- canonical stream event types
- OpenAI-compatible SSE frames
- terminal events
- stream error events

Streaming must not be treated as plain chunk forwarding.

## Storage Architecture

The storage layer follows a storage driver model:

- `sdkwork-api-storage-core`
- `sdkwork-api-storage-sqlite`
- `sdkwork-api-storage-postgres`
- `sdkwork-api-storage-mysql`
- `sdkwork-api-storage-libsql`

### Storage Principles

- domain and application layers depend only on repository traits
- each service owns its data boundary
- embedded mode may share one physical database, but logical service ownership is preserved
- migration execution is dialect-aware
- outbox and inbox patterns are first-class from the start

### Main Data Areas

- `identity_*`
- `tenant_*`
- `catalog_*`
- `credential_*`
- `routing_*`
- `usage_*`
- `billing_*`
- `audit_*`
- `outbox_*`

### Representative Tables

- `identity_users`
- `identity_gateway_api_keys`
- `tenant_tenants`
- `tenant_projects`
- `catalog_channels`
- `catalog_proxy_providers`
- `catalog_models`
- `catalog_model_aliases`
- `credential_records`
- `routing_policies`
- `routing_provider_health`
- `routing_decision_logs`
- `usage_requests`
- `usage_token_records`
- `billing_quota_policies`
- `billing_ledger_entries`
- `billing_settlements`
- `audit_events`
- `outbox_events`

## Request Flow

For `/v1/chat/completions` and `/v1/responses`, the synchronous request path is:

1. gateway receives request
2. JWT or Gateway API key is validated
3. tenant and project context are resolved
4. billing and quota are checked synchronously
5. routing service selects a target provider
6. catalog and credential data are resolved
7. provider adapter issues the upstream request
8. upstream response is mapped back to OpenAI-compatible output
9. usage facts are recorded
10. billing ledger and settlement facts are emitted

`/v1/models` is served from the platform catalog and filtered by policy and visibility.

`/v1/embeddings` follows the same core chain but with embedding-specific usage and dimension metadata.

## Security Model

Security is split into four independent chains:

1. `User Authentication`
   JWT-based control plane sessions.
2. `Gateway Access Authentication`
   Gateway API keys for `/v1/*`.
3. `Upstream Credential Management`
   Encrypted upstream API secrets.
4. `Runtime Trust Boundary`
   Server mode versus embedded desktop trust assumptions.

### Security Rules

- management console uses `JWT`
- data plane primarily uses `Gateway API Key`
- upstream credentials are stored as decryptable encrypted envelopes
- Tauri embedded mode prefers OS keyring when available
- embedded runtime binds to loopback by default
- security decisions follow `RBAC + Scope + Resource Binding`

## Monorepo Structure

The repository should be a polyglot monorepo:

```text
sdkwork-api-server/
├── Cargo.toml
├── services/
├── crates/
├── console/
├── deploy/
├── scripts/
├── docs/
└── examples/
```

### Rust Workspace Layout

```text
services/
├── gateway-service/
├── routing-service/
├── identity-service/
├── catalog-service/
├── credential-service/
├── usage-service/
├── billing-service/
└── admin-api-service/

crates/
├── sdkwork-api-kernel/
├── sdkwork-api-config/
├── sdkwork-api-observability/
├── sdkwork-api-contract-openai/
├── sdkwork-api-contract-gateway/
├── sdkwork-api-storage-core/
├── sdkwork-api-storage-sqlite/
├── sdkwork-api-storage-postgres/
├── sdkwork-api-storage-mysql/
├── sdkwork-api-storage-libsql/
├── sdkwork-api-secret-core/
├── sdkwork-api-secret-local/
├── sdkwork-api-secret-keyring/
├── sdkwork-api-provider-core/
├── sdkwork-api-provider-openai/
├── sdkwork-api-provider-openrouter/
├── sdkwork-api-provider-ollama/
├── sdkwork-api-domain-identity/
├── sdkwork-api-domain-tenant/
├── sdkwork-api-domain-catalog/
├── sdkwork-api-domain-credential/
├── sdkwork-api-domain-routing/
├── sdkwork-api-domain-usage/
├── sdkwork-api-domain-billing/
├── sdkwork-api-app-gateway/
├── sdkwork-api-app-routing/
├── sdkwork-api-app-identity/
├── sdkwork-api-app-catalog/
├── sdkwork-api-app-credential/
├── sdkwork-api-app-usage/
├── sdkwork-api-app-billing/
├── sdkwork-api-interface-http/
├── sdkwork-api-interface-admin/
├── sdkwork-api-runtime-host/
└── sdkwork-api-testkit/
```

### Frontend Workspace Layout

The `console/` directory follows the SDKWork standard:

```text
console/
├── package.json
├── pnpm-workspace.yaml
├── turbo.json
├── tsconfig.json
├── vite.config.ts
├── src/
├── packages/
│   ├── sdkwork-api-types/
│   ├── sdkwork-api-i18n/
│   ├── sdkwork-api-commons/
│   ├── sdkwork-api-core/
│   ├── sdkwork-api-auth/
│   ├── sdkwork-api-workspace/
│   ├── sdkwork-api-tenant/
│   ├── sdkwork-api-project/
│   ├── sdkwork-api-user/
│   ├── sdkwork-api-channel/
│   ├── sdkwork-api-provider/
│   ├── sdkwork-api-model/
│   ├── sdkwork-api-routing/
│   ├── sdkwork-api-credential/
│   ├── sdkwork-api-usage/
│   ├── sdkwork-api-billing/
│   ├── sdkwork-api-audit/
│   ├── sdkwork-api-runtime/
│   └── sdkwork-api-admin-sdk/
└── src-tauri/
```

## Control Plane Information Architecture

The management console is organized into four top-level workspaces:

- `Workspace`
- `Connectivity`
- `Governance`
- `Operations`

### Mandatory Stage 1 Pages

- dashboard overview
- tenant and project management
- gateway API key management
- channel and proxy provider registry
- credential vault
- model catalog
- routing policy editor
- route simulation
- usage dashboard
- request explorer
- runtime status

Web and Tauri must use the same admin APIs and share the same business packages.

## Recommended Delivery Strategy

The recommended delivery approach is `contract-first plus vertical slice`.

### Milestone 1

- create repository skeleton
- create Cargo workspace and frontend workspace
- establish runtime, config, and observability baseline

### Milestone 2

- implement protocol crates
- implement domain crates
- implement storage and secret abstractions

### Milestone 3

- implement first end-to-end slice
- JWT and Gateway API key
- tenant and project context
- channel, provider, credential, and model catalog
- routing
- `/v1/models`
- `/v1/chat/completions`
- `/v1/responses`
- `/v1/embeddings`
- streaming
- usage and billing baseline

### Milestone 4

- implement React console core pages
- implement Tauri embedded runtime integration
- polish SQLite local mode
- validate PostgreSQL and future storage-driver integrations

## Open Questions Deferred to Implementation

- event bus transport choice for distributed deployment
- exact settlement calculation formula and commercial packaging
- official support level for non-OpenAI protocol variants
- local first-run bootstrap UX in Tauri mode
- migration orchestration for mixed standalone and embedded deployments

## Current Repository Constraint

The target directory was empty at the time of design authoring and is not initialized as a git repository. Because of that:

- the design document can be written locally
- the design document cannot be committed yet

This should be resolved before implementation begins if commit-based checkpoints are required.
