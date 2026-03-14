# Quota-Aware Admission Design

**Date:** 2026-03-14

**Status:** Approved by the original gateway roadmap and the user's standing instruction to continue autonomously

## Goal

Close the next strongest governance gap by turning quota from a placeholder bookkeeping concept into a real project-level admission control capability for the core stateful gateway APIs.

## Why This Batch

The repository already supports:

- tenant and project resolution through gateway API keys
- persisted usage records
- persisted billing ledger entries
- runtime-aware routing and provider health governance
- admin control-plane APIs for usage and billing inspection

What is still missing relative to the original design is actual quota enforcement:

- `check_quota()` is still a stub that always returns `true`
- there is no `QuotaPolicy` aggregate in the billing domain
- there is no persistent `billing_quota_policies` table
- the gateway does not reject requests when a project has exhausted its budget

This is now the highest-leverage gap because it closes the control loop between tenancy, billing, and request execution without needing a full SLO or regional policy engine first.

## Scope

This batch will implement:

1. a `QuotaPolicy` aggregate in the billing domain
2. SQLite and PostgreSQL persistence for `billing_quota_policies`
3. app-layer quota evaluation using existing ledger usage as the source of consumed billable units
4. admin APIs for creating and listing quota policies
5. synchronous quota admission for the core stateful create routes:
   - `/v1/chat/completions`
   - `/v1/completions`
   - `/v1/responses`
   - `/v1/embeddings`
6. OpenAI-compatible `429` error responses when quota is exceeded
7. console visibility for configured quota policies

This batch will not implement:

- rolling time windows
- provider-specific quota buckets
- rate limiting
- SLO-aware routing
- geo affinity
- runtime package hot reload

## Options Considered

### Option A: Leave quota as bookkeeping only

Pros:

- no schema changes
- no gateway behavior changes

Cons:

- leaves the original design promise unimplemented
- does not protect projects from overruns
- keeps billing and routing governance incomplete

### Option B: Project-level hard quota using existing ledger totals

Pros:

- reuses current project identity, usage, and ledger flows
- gives the gateway a real synchronous admission decision today
- keeps the model small and explainable
- establishes an extension point for later rolling windows and provider budgets

Cons:

- consumed units are lifetime totals rather than windowed budgets
- quota is project-scoped, not provider-scoped

### Option C: Full multi-dimensional quota engine first

Pros:

- closer to a large production billing platform
- could support monthly budgets, per-model limits, and burst rate control

Cons:

- far too large for the current gap
- would delay delivery of the missing admission-control primitive
- requires more policy semantics than the repository currently models

## Recommendation

Use **Option B**.

The platform already has enough primitives to support hard quota admission with a bounded design. A project-level quota policy backed by ledger totals closes a real system gap now and creates a clean base for later windows or richer dimensions.

## Domain Model

Add a `QuotaPolicy` aggregate to the billing domain.

Each policy should carry:

- `policy_id`
- `project_id`
- `max_units`
- `enabled`

This batch keeps policies intentionally project-scoped. If multiple enabled policies exist for one project, the gateway should enforce the strictest effective limit by choosing the smallest `max_units`.

## Quota Evaluation Model

Quota evaluation should use existing persisted ledger entries as the canonical consumed-unit source.

Rules:

1. sum `LedgerEntry.units` for the target `project_id`
2. if no enabled quota policy exists, allow the request
3. if `used_units + requested_units > max_units`, reject the request
4. when multiple enabled policies exist, use the smallest limit

This keeps the first version deterministic and easy to verify.

## HTTP Behavior

The gateway should reject over-quota requests before upstream dispatch for the core stateful create APIs.

Response shape:

- HTTP status `429 Too Many Requests`
- OpenAI-compatible error envelope
- error type `insufficient_quota`
- error code `quota_exceeded`

This keeps the public API honest and consistent with OpenAI-style error handling.

## Admin API Surface

Add native control-plane endpoints:

- `GET /admin/billing/quota-policies`
- `POST /admin/billing/quota-policies`

This batch only needs create and list. Update and delete can follow later if policy editing becomes a real operator need.

## Console Impact

The console usage page should start showing project quota policies so operators can correlate:

- configured hard budget
- current ledger-backed consumption
- whether quota admission is active

This keeps billing governance visible instead of burying it in backend-only state.

## Testing Strategy

The batch should be proven through:

1. domain tests for `QuotaPolicy`
2. storage tests for quota policy persistence in SQLite and PostgreSQL
3. app-billing tests for quota evaluation against persisted ledger totals
4. admin API tests for create and list quota policies
5. HTTP route tests proving quota rejection with `429` on over-budget requests
6. console typecheck coverage for the new quota types and SDK calls

## Follow-On Work

After this batch, the strongest remaining governance gaps should be:

1. SLO-aware routing and decision logging
2. regional affinity strategy dimensions
3. active endpoint probing for official upstream providers
4. rolling-window quota policy dimensions
5. runtime hot reload or unload orchestration
