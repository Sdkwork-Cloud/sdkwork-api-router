# API Key Group, Routing Profiles, and Billing 2.0 Design

**Status:** approved for autonomous phase-by-phase execution

**Goal:** evolve `sdkwork-api-router` from a capable API router into a stronger product-grade gateway by introducing API key groups as a first-class policy boundary, establishing a route toward OpenRouter-style provider selection and generation observability, and preparing the billing domain for richer multimodal pricing.

This design now sits under the broader plugin-first architecture program described in [2026-04-03-plugin-first-architecture-design.md](/D:/javasource/spring-ai-plus/spring-ai-plus-business/apps/sdkwork-api-router/docs/superpowers/specs/2026-04-03-plugin-first-architecture-design.md). API key groups are no longer treated as a standalone feature; they are the first durable policy attachment point for later routing, billing, quota, and product-module plugins.

## Context

The current router already has a strong architectural base:

- domain-separated identity, catalog, credential, routing, usage, billing, and runtime crates
- canonical `ai_*` storage tables
- stateful and stateless gateway execution
- broad OpenAI-compatible surface including chat, responses, embeddings, files, images, audio, realtime, and videos
- extension runtime support with hot reload and rollout coordination

The current weakness is not the platform core. The weakness is product density and the lack of a few first-class product concepts:

- API keys do not have a first-class grouping boundary
- routing policy is stronger than many peers, but is not yet packaged into reusable customer-facing selection profiles
- billing records are too coarse for multimodal and provider-cost-aware accounting
- operator and tenant experience does not yet expose enough evidence for route choice, provider fallback, cache benefit, or per-capability cost posture

## External Lessons Incorporated

### Lessons from `new-api`

- treat grouping as a real product concept, not a loose tag
- make channel and model operations easy to run day to day
- optimize for product composition and operator workflow density
- precompute a runtime-friendly routing index when canonical policy data is too expensive to evaluate repeatedly

### Lessons from OpenRouter

- expose model and provider capability metadata as a first-class contract
- make provider selection configurable through structured request or policy objects
- treat cost, latency, fallback, privacy, and cache effects as routing inputs and observable outputs
- support multimodal discovery and generation evidence rather than only endpoint parity

## Decision

Do not collapse the current architecture into a monolith.

Keep the current `domain -> app -> storage -> interface -> runtime` split and add the missing product concepts directly into that architecture.

## Phase Structure

This design deliberately separates a large platform evolution into smaller deliverable phases.

### Phase 1: API Key Group Foundation

Introduce API key groups as a first-class identity and policy object.

This phase adds:

- `ApiKeyGroupRecord` in the identity domain
- durable storage for API key groups
- API key membership in a group
- admin and portal CRUD flows for groups
- API key create and update flows that can bind a key to a group

This phase does not yet add:

- full provider selection profiles
- request-level provider override objects
- billing 2.0 event ledger
- async media job system

### Phase 2: Routing Profile Layer

Introduce reusable group-bound routing policy bundles.

This phase adds:

- provider selection profile records
- model selection profile records
- group-level default routing profile binding
- compiled routing snapshots derived from canonical routing records
- admin observability for selected candidate, rejected candidate, fallback reason, and route evidence

### Phase 3: Billing 2.0

Introduce fine-grained billable usage events and separate upstream-cost from customer-charge accounting.

This phase adds:

- generation or execution event records
- multimodal billable dimensions such as image count, audio seconds, video seconds, and cached token deltas
- separate upstream provider cost and customer charge
- group-level and capability-level summaries
- BYOK and platform-credit accounting modes

### Phase 4: Async Media and Product Density

Introduce a unified job model for image, video, and music-style long-running tasks, plus a denser operator workflow.

This phase adds:

- async generation job records
- asset lifecycle records
- webhook and polling visibility
- operator command center for provider balance, model sync, bulk test, and key-pool operations

## API Key Group Domain Model

API key groups are the main new abstraction.

They represent a stable policy boundary shared by many API keys.

### Why a first-class group object is required

A simple `group_name` field on the key record would not be enough because:

- keys rotate more frequently than policies
- multiple keys need to share one policy bundle
- billing and observability need aggregation by group
- future routing, quota, and commerce policies need a stable attach point

### Proposed record

`ApiKeyGroupRecord`

- `group_id`
- `tenant_id`
- `project_id`
- `name`
- `slug`
- `description`
- `environment`
- `color`
- `default_capability_scope`
- `active`
- `created_at_ms`
- `updated_at_ms`

### Key membership change

`GatewayApiKeyRecord` should gain:

- `api_key_group_id: Option<String>`

The group is optional during migration so older keys remain valid.

## Policy Attachment Model

The group object is the future attach point for higher-level policy.

The system should evolve toward these bindings:

- group -> routing profile
- group -> billing plan
- group -> quota and rate-limit policy bundle
- group -> capability policy
- group -> privacy or residency policy

Phase 1 only establishes the attach point. It does not implement every policy type.

## Storage Design

Add a new canonical table:

- `ai_app_api_key_groups`

Fields:

- `group_id TEXT PRIMARY KEY`
- `tenant_id TEXT NOT NULL`
- `project_id TEXT NOT NULL`
- `name TEXT NOT NULL`
- `slug TEXT NOT NULL`
- `description TEXT`
- `environment TEXT NOT NULL`
- `color TEXT`
- `default_capability_scope TEXT`
- `active INTEGER NOT NULL DEFAULT 1`
- `created_at_ms INTEGER NOT NULL`
- `updated_at_ms INTEGER NOT NULL`

Unique constraints:

- `(tenant_id, project_id, environment, slug)`

Add one nullable column to `ai_app_api_keys`:

- `api_key_group_id TEXT`

No hard foreign key is required immediately because existing SQLite migrations avoid strict relational coupling in several areas and backward compatibility is more important than relational strictness for this phase.

## Admin and Portal API Design

### Admin

Add:

- `GET /admin/api-key-groups`
- `POST /admin/api-key-groups`
- `PATCH /admin/api-key-groups/{group_id}`
- `POST /admin/api-key-groups/{group_id}/status`
- `DELETE /admin/api-key-groups/{group_id}`

Extend:

- `POST /admin/api-keys`
- `PATCH /admin/api-keys/{hashed_key}`

with optional `api_key_group_id`.

### Portal

Add:

- `GET /portal/api-key-groups`
- `POST /portal/api-key-groups`
- `PATCH /portal/api-key-groups/{group_id}`
- `POST /portal/api-key-groups/{group_id}/status`
- `DELETE /portal/api-key-groups/{group_id}`

Extend:

- `POST /portal/api-keys`

with optional `api_key_group_id`.

Portal routes remain scoped to the authenticated user's workspace and may only see groups inside the caller-owned `tenant_id + project_id`.

## Validation Rules

- group name must be non-empty after trim
- slug must be normalized to lowercase ASCII with dash separators
- environment must remain non-empty and continue to align with current API key semantics
- a key may only bind to a group in the same tenant, project, and environment
- inactive groups may not receive newly created keys
- deleting a group should not delete keys; it should clear future usability and either reject deletion while bound keys exist or detach keys explicitly through application logic

Phase 1 chooses the safer behavior:

- reject deletion while bound keys exist

## Application-Layer Behavior

The application layer, not the interface layer, owns group validation.

New app-identity responsibilities:

- create group
- list groups globally
- list groups by tenant and project
- update group metadata
- set group active state
- delete group with safe checks
- validate key-to-group attachment during key create and key update

## Routing and Billing Follow-Through

API key groups matter because they become the stable control point for the next two big evolutions.

### Routing follow-through

Groups will later own reusable routing profiles inspired by OpenRouter provider-selection semantics:

- preferred providers
- deny list
- fallback allowed or disabled
- max price ceiling
- capability-specific routing rules
- privacy or residency constraints
- latency or SLA posture

### Billing follow-through

Groups will later own billing summaries and policy bundles:

- chargeback by group
- group-specific rate cards
- platform credit vs BYOK mode
- capability-specific quota summaries
- route-level cost visibility

The current implementation has now crossed the first real billing-policy threshold:

- API key groups can optionally persist `default_accounting_mode`
- accepted values are canonicalized to:
  - `platform_credit`
  - `byok`
  - `passthrough`
- gateway billing-event inference now resolves accounting mode from the bound API key group when present and falls back to `platform_credit` otherwise

## Testing Strategy

Phase 1 must add tests at three levels:

### Domain and app tests

- create group success
- duplicate slug rejection within same tenant or project environment scope
- key create fails when group belongs to another workspace
- key create fails when group environment mismatches
- key create succeeds when group is valid
- deleting a group with attached keys fails

### Storage tests

- SQLite migration creates `ai_app_api_key_groups`
- records round-trip correctly
- existing key rows still round-trip when `api_key_group_id` is null

### HTTP tests

- admin CRUD for groups
- portal CRUD for groups in caller workspace
- admin key create with `api_key_group_id`
- portal key create with `api_key_group_id`

## Non-Goals for Phase 1

- request-body provider routing overrides
- compiled route snapshots
- generation stats endpoints
- multimodal async job model
- music generation contracts
- front-end management pages

Backend correctness comes first. UI can layer on top of the stable API after the group model is in place.

## Success Criteria

Phase 1 is complete when:

- API key groups exist as durable canonical records
- keys can optionally bind to a group
- admin and portal can manage groups through real APIs
- invalid cross-workspace and mismatched-environment group usage is rejected
- tests cover domain, storage, and HTTP behavior
- the change preserves backward compatibility for existing ungrouped keys

## Implementation Notes

Phase 1 implementation currently follows the design with these concrete notes:

- application-layer group mutations now use structured input objects instead of long argument lists:
  - `ApiKeyGroupInput`
  - `PortalApiKeyGroupInput`
- admin and portal both expose first-class API key group CRUD and status routes backed by real storage and app-identity validation
- `GatewayApiKeyRecord` and `CreatedGatewayApiKey` now carry optional `api_key_group_id`
- canonical persistence now spans:
  - `ai_app_api_key_groups`
  - `ai_app_api_keys.api_key_group_id`

### Intentional compatibility deviation

The design proposed `PATCH /admin/api-keys/{hashed_key}` for API key metadata updates.

The current implementation keeps the existing `PUT /admin/api-keys/{hashed_key}` route for backward compatibility and extends that route with optional `api_key_group_id` support instead of introducing a second update verb in this phase.

### Validation outcomes now enforced

- API key group slug uniqueness is enforced within `tenant_id + project_id + environment`
- key creation rejects:
  - missing groups
  - cross-tenant groups
  - cross-project groups
  - cross-environment groups
  - inactive groups
- group deletion rejects groups that still have bound API keys

## Phase 2 Implementation Notes

Routing profile work now has two delivered slices:

- group-bound routing profiles are durable canonical records
- API key groups may bind an optional `default_routing_profile_id`
- request-time route selection now merges:
  - matched global routing policy
  - project routing preferences
  - group-bound routing profile
  - static fallback
- request-time region selection now prefers:
  - explicit requested region
  - group routing profile preferred region
  - project routing preferences preferred region
- compiled routing snapshots are now persisted as derived records so admin diagnostics can inspect the effective routing state without re-deriving it mentally from multiple source tables
- routing simulations now expose an explicit evidence contract for:
  - `compiled_routing_snapshot_id`
  - `selected_candidate`
  - `rejected_candidates`
  - `fallback_reason`

These changes intentionally keep routing semantics centralized in the existing routing pipeline instead of creating a second policy engine for snapshots or diagnostics.

## Phase 3 Implementation Notes

Billing 2.0 now has a delivered foundation slice:

- `BillingEventRecord` is the new canonical detailed accounting object
- canonical persistence now includes:
  - `ai_billing_events`
- billing events retain:
  - tenant and project scope
  - optional `api_key_group_id`
  - capability, route key, usage model, provider, and channel
  - multimodal billable dimensions:
    - request count
    - token usage
    - cache read and write token deltas
    - image count
    - audio seconds
    - video seconds
    - music seconds
  - separate `upstream_cost` and `customer_charge`
  - `accounting_mode`
  - routing evidence:
    - `applied_routing_profile_id`
    - `compiled_routing_snapshot_id`
    - `fallback_reason`
- admin now exposes:
  - `GET /admin/billing/events`
  - `GET /admin/billing/events/summary`
- portal now also exposes workspace-scoped event visibility:
  - `GET /portal/billing/events`
  - `GET /portal/billing/events/summary`
- gateway metering now dual-writes billing events alongside the existing:
  - usage record
  - coarse billing ledger entry

Billing-event capture has now started to move beyond text-only dimensions:

- image generation, edit, and variation requests now populate `image_count` from the actual response payload size
- `audio_seconds`, `video_seconds`, and `music_seconds` remain part of the canonical event contract but still require richer upstream or request metadata before they can be inferred accurately

This implementation intentionally keeps `LedgerEntry` and quota summaries alive for backward compatibility while establishing the durable event layer needed for later pricing engines, BYOK settlement logic, multimodal billing plugins, and richer tenant chargeback views.

## Next Phase Entry Criteria

Routing profile and Billing 2.0 work should start only after all of the following hold:

- admin and portal UI layers consume the new API key group APIs instead of using local-only grouping state
- route selection can read a stable group attachment from request context without another identity schema change
- billing ledger evolution has a dedicated event model for multimodal dimensions instead of overloading the current summary rows
- upstream provider cost attribution is designed separately from customer-facing charge policy
- request-time provider selection evidence has a durable schema contract for fallback reason, pricing posture, and capability matching
