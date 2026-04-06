# Commercial Pricing Architecture Design

**Status:** design aligned to the local `sdkwork-api-router` source tree on 2026-04-03

**Goal:** define a standardized, extensible, commercially correct pricing architecture for `sdkwork-api-router` so operators can manage model and API pricing across token, request, image, audio, video, and music billing modes from the admin product while keeping settlement-time pricing explicit and auditable.

## Executive Decision

`sdkwork-api-router` should keep **two distinct pricing planes**.

1. `catalog model price`
   This remains the market-reference and upstream-cost read model used to describe provider and model offerings.

2. `canonical commercial pricing`
   This becomes the authoritative settlement-time pricing layer used by billing, commercial governance, admin operations, and tenant-facing display.

The correct architecture is **not** to overload `ai_model_price` with more columns and special cases.

The correct architecture is:

- preserve `ai_model_price` as the provider and market reference layer
- upgrade `ai_pricing_plan` and `ai_pricing_rate` into the canonical pricing source of truth
- expose canonical pricing through dedicated admin management surfaces and portal display surfaces
- normalize price semantics around reusable charge units and pricing methods instead of provider-specific column sprawl

## Why The Current State Is Not Enough

The current canonical pricing model already exists, but it is too thin for real commercial use.

Today `PricingRateRecord` only carries:

- `metric_code`
- `model_code`
- `provider_code`
- `quantity_step`
- `unit_price`

That is insufficient for:

- multimodal pricing definitions
- explicit token input versus token output pricing
- flat request billing versus per-unit billing
- image, audio, video, and music pricing
- operator-facing validation and display
- future tiering, included usage, and effective-time governance

The current implementation can list pricing rows, but it does not yet define a professional pricing grammar.

## Pricing Plane Separation

### Catalog Model Price

This plane answers:

- what models and providers exist
- what upstream list price or market price is associated with them
- how operators compare provider offerings in the catalog

This layer may contain market-style units such as:

- `per_1m_tokens`
- `per_request`
- `per_image`
- `per_second_audio`
- `per_minute_video`
- `per_track`

That is correct for a catalog workbench, but it is not enough for settlement-time billing.

### Canonical Commercial Pricing

This plane answers:

- which commercial plan is active
- which rate applies to a capability, metric, model, or provider scope
- how quantity is rounded and billed
- which currency or credit unit is charged
- how admin and portal surfaces explain the resulting charge

This layer must be explicit, stable, auditable, and extensible.

## Canonical Pricing Model

### Pricing Plan

`PricingPlanRecord` remains the plan header and version container.

It should continue to own:

- `pricing_plan_id`
- `tenant_id`
- `organization_id`
- `plan_code`
- `plan_version`
- `display_name`
- `currency_code`
- `credit_unit_code`
- `status`
- `created_at_ms`
- `updated_at_ms`

For admin governance and display, the plan header should also carry:

- `description`
- `billing_scope`
- `is_default`
- `effective_from_ms`
- `effective_to_ms`

The first implementation slice can safely defer some of those fields if needed, but the structure should be designed with them in mind.

### Pricing Rate

`PricingRateRecord` must become a general-purpose charge rule instead of a bare numeric row.

The canonical shape should include the existing scoping keys plus richer charging semantics:

- `pricing_rate_id`
- `tenant_id`
- `organization_id`
- `pricing_plan_id`
- `metric_code`
- `model_code`
- `provider_code`
- `capability_code`
- `charge_unit`
- `pricing_method`
- `quantity_step`
- `unit_price`
- `display_price_unit`
- `minimum_billable_quantity`
- `minimum_charge`
- `rounding_increment`
- `rounding_mode`
- `included_quantity`
- `priority`
- `notes`
- `status`
- `created_at_ms`
- `updated_at_ms`

The first implementation slice should land at least:

- `capability_code`
- `charge_unit`
- `pricing_method`
- `display_price_unit`
- `minimum_billable_quantity`
- `minimum_charge`
- `rounding_increment`
- `rounding_mode`
- `included_quantity`
- `priority`
- `notes`
- `status`
- `updated_at_ms`

This is enough to cover professional operator configuration while leaving room for tier tables later.

## Standard Charge Unit Taxonomy

Canonical charge units should be standardized so admin forms, portal display, and settlement logic all speak the same language.

Recommended first-class units:

- `input_token`
- `output_token`
- `cache_read_token`
- `cache_write_token`
- `request`
- `image`
- `audio_second`
- `audio_minute`
- `video_second`
- `video_minute`
- `music_track`
- `character`
- `storage_mb_day`
- `tool_call`

These are not vendor-specific names. They are platform billing primitives.

Each rate still keeps `metric_code`, which lets the request-metering layer map normalized usage facts onto billing rules.

## Pricing Method Taxonomy

Pricing methods should be explicit instead of being inferred from missing fields.

Recommended first slice:

- `per_unit`
- `flat`
- `step`
- `included_then_per_unit`

Planned future expansion:

- `tiered`
- `volume`
- `package`

This lets the admin product show the billing semantics directly instead of forcing operators to decode numeric fields manually.

## Rounding And Minimums

Commercial billing needs deterministic quantity normalization.

Canonical fields should support:

- `quantity_step`
  The base charge quantity, such as `1000` tokens or `1` request.

- `minimum_billable_quantity`
  The minimum quantity eligible for billing after usage normalization.

- `rounding_increment`
  The increment used when applying rounding.

- `rounding_mode`
  Standardized values such as `ceil`, `floor`, `half_up`, or `none`.

- `minimum_charge`
  A flat lower bound applied after quantity normalization.

This keeps token, image, and media pricing on a common foundation.

## Capability And Scope Matching

Pricing rates need stronger scope matching than `metric_code` alone.

The canonical rate matcher should support:

- `capability_code`
  Examples: `responses`, `chat`, `image_generation`, `video_generation`, `audio_transcription`, `speech_synthesis`, `music_generation`

- `provider_code`
- `model_code`

Matching precedence should stay simple in the first implementation slice:

1. capability + provider + model
2. capability + model
3. capability + provider
4. capability only
5. global metric fallback

This is sufficient for admin governance and later settlement resolution work.

## How This Relates To OpenRouter And Model Markets

OpenRouter-style model markets expose pricing in a way operators understand quickly:

- explicit modality-aware units
- separate input and output token pricing
- provider and model-specific surfaces
- human-readable display of unit economics

That should be borrowed at the product layer.

What should **not** be copied is a market-optimized schema directly into settlement-time billing.

The better design for `sdkwork-api-router` is:

- keep market style display and catalog comparisons in the catalog plane
- keep auditable commercial charging rules in the canonical pricing plane
- project canonical pricing into admin and portal views that read like a professional model marketplace

## Admin Product Responsibilities

The admin product needs a dedicated pricing management module rather than burying all pricing work inside catalog dialogs.

Recommended first-class admin module:

- `sdkwork-router-admin-pricing`

This module should manage:

- pricing plan list and activation posture
- pricing rate list and filters
- charge unit taxonomy visibility
- billing method visibility
- create and edit flows for canonical plans and rates
- validation for modality-specific rate shapes
- side-by-side view of market reference price versus canonical retail price

This module should **not** replace catalog management.

Catalog remains responsible for:

- channels
- providers
- models
- provider-model availability
- market reference pricing

Pricing management becomes responsible for:

- commercial plan definition
- settlement-facing billing rules
- operator governance of retail price posture

## Portal Responsibilities

Portal should not expose the full authoring surface, but it should display pricing clearly enough for a commercial tenant.

Recommended portal responsibilities:

- show active plan name and version
- show applicable pricing rates for the authenticated workspace scope
- present modality-aware charge units and display units
- explain whether billing is per token, per request, per image, per minute, or per track

This remains read-only unless commercial self-serve pricing becomes a future product requirement.

## API Design

Admin must expose canonical read and write APIs for pricing.

Required admin endpoints:

- `GET /admin/billing/pricing-plans`
- `POST /admin/billing/pricing-plans`
- `GET /admin/billing/pricing-rates`
- `POST /admin/billing/pricing-rates`

Recommended near-term extensions:

- `PUT /admin/billing/pricing-plans/{pricing_plan_id}`
- `PUT /admin/billing/pricing-rates/{pricing_rate_id}`
- `POST /admin/billing/pricing-plans/{pricing_plan_id}/activate`

Portal should continue to expose read-only surfaces:

- `GET /portal/billing/pricing-plans`
- `GET /portal/billing/pricing-rates`

## Storage Design

The storage layer should keep canonical pricing in `ai_pricing_plan` and `ai_pricing_rate`.

The immediate upgrade is to add missing commercial fields to `ai_pricing_rate` for both SQLite and PostgreSQL.

This preserves the existing plugin-first storage abstraction and avoids inventing a storage side channel.

The first implementation slice should not add separate tier tables yet.

YAGNI applies here:

- land robust per-unit and flat pricing first
- keep the record shape extensible for tiering later
- only add `ai_pricing_tier` when settlement logic and operator workflows actually need it

## Validation Rules

The admin module and admin API should apply standardized validation.

Examples:

- `per_unit` requires positive `quantity_step`
- `flat` should use `quantity_step = 1`
- token-based units should use `metric_code` values consistent with request meter metrics
- image and media units should require a matching capability code
- `rounding_mode = none` should not require `rounding_increment`
- `minimum_charge` and `minimum_billable_quantity` must be non-negative

## Implementation Order

The best implementation order is:

1. enrich canonical pricing domain and storage schema
2. expose admin write APIs
3. add admin TypeScript types and API client methods
4. add a dedicated admin pricing package and route
5. sync workspace snapshot and operator display
6. extend portal display after admin authoring is stable

This order keeps the backend source of truth ahead of the product surface.

## Non-Goals For This Slice

Do not attempt all future billing sophistication in one pass.

Explicit non-goals for the first slice:

- tier table implementation
- discount stacks
- contract pricing inheritance trees
- invoice engine generation
- tax engine calculation
- provider-specific pricing adapters inside the canonical record

Those are valid future layers, but not needed to land a correct commercial pricing foundation now.

## Final Recommendation

The best architecture for `sdkwork-api-router` pricing is:

- keep `catalog model price` as the model market and upstream reference layer
- upgrade canonical pricing plans and rates into the commercial source of truth
- standardize charge units and billing methods across text, image, audio, video, and music capabilities
- expose operator authoring through a dedicated admin pricing module
- keep portal pricing display read-only and derived from canonical pricing

That design is more extensible than `new-api`, more commercially correct than a pure market-price table, and better aligned with the existing plugin-first architecture of this codebase.
