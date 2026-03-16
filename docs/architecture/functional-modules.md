# Functional Modules

This page maps user-facing capabilities to the workspace modules that implement them.

## Runtime Surfaces

| Module | What it does | Primary paths |
|---|---|---|
| gateway | exposes the OpenAI-compatible `/v1/*` surface | `services/gateway-service`, `crates/sdkwork-api-interface-http` |
| admin control plane | manages tenants, projects, credentials, routing, billing, and runtime operations | `services/admin-api-service`, `crates/sdkwork-api-interface-admin` |
| public portal | supports registration, login, workspace inspection, and self-service API key issuance | `services/portal-api-service`, `crates/sdkwork-api-interface-portal` |
| console | browser and desktop UI shell | `console/` |
| docs | operator and contributor documentation | `docs/` |

## Core Backend Modules

| Capability | Responsibility | Primary crates |
|---|---|---|
| gateway orchestration | maps API routes to provider execution and local fallback | `sdkwork-api-app-gateway` |
| routing | policy simulation, decision logging, health-aware dispatch | `sdkwork-api-app-routing`, `sdkwork-api-domain-routing` |
| billing and usage | usage records, billing summaries, quota enforcement | `sdkwork-api-app-usage`, `sdkwork-api-app-billing`, `sdkwork-api-domain-usage`, `sdkwork-api-domain-billing` |
| identity | admin JWTs, portal JWTs, gateway API keys | `sdkwork-api-app-identity`, `sdkwork-api-domain-identity` |
| tenant and catalog | tenants, projects, models, channels, providers | `sdkwork-api-app-tenant`, `sdkwork-api-app-catalog`, `sdkwork-api-domain-tenant`, `sdkwork-api-domain-catalog` |
| credentials | encrypted provider credential storage and secret resolution | `sdkwork-api-app-credential`, `sdkwork-api-secret-*`, `sdkwork-api-domain-credential` |
| runtime and extension loading | extension discovery, ABI boundaries, runtime supervision, embedded host | `sdkwork-api-app-runtime`, `sdkwork-api-extension-host`, `sdkwork-api-runtime-host`, `sdkwork-api-extension-*` |

## Provider and Protocol Modules

| Module | Responsibility |
|---|---|
| `sdkwork-api-provider-openai` | OpenAI-compatible upstream relay |
| `sdkwork-api-provider-openrouter` | OpenRouter-compatible relay |
| `sdkwork-api-provider-ollama` | local Ollama-compatible relay |
| `sdkwork-api-provider-core` | shared provider request and stream abstractions |
| `sdkwork-api-contract-openai` | OpenAI-compatible request and response contract shapes |
| `sdkwork-api-contract-gateway` | SDKWork-specific contract types |

## Storage Modules

| Module | Responsibility |
|---|---|
| `sdkwork-api-storage-core` | shared admin-store contracts |
| `sdkwork-api-storage-sqlite` | SQLite-backed admin store and migrations |
| `sdkwork-api-storage-postgres` | PostgreSQL-backed admin store |
| `sdkwork-api-storage-libsql` | libSQL backend work |
| `sdkwork-api-storage-mysql` | MySQL backend work |

## Developer and Operator Tooling

| Module | Responsibility |
|---|---|
| `scripts/dev/start-workspace.*` | one-command local startup |
| `scripts/dev/start-stack.mjs` | backend-only startup |
| `scripts/dev/start-console.*` | UI-only startup |
| `docs/` | docs site and historical plans |
| `console/src-tauri/` | desktop packaging and host integration |

## How Modules Fit Together

When working on a change, use this rule of thumb:

- HTTP behavior starts in an interface crate
- workflow and orchestration live in an app crate
- policy logic lives in a domain crate
- persistence concerns live in a storage crate
- upstream execution concerns live in provider or extension runtime crates

This keeps the workspace scalable and avoids burying product behavior inside transport or persistence code.

## Related Docs

- system-level design:
  - [Software Architecture](/architecture/software-architecture)
- workspace structure:
  - [Repository Layout](/reference/repository-layout)
