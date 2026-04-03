# Clean-Slate Account Billing Service Design

**Goal:** move `sdkwork-api-app-billing` off the compatibility-era `project_id + quota + ledger` model and turn it into the orchestration layer for the canonical account kernel already defined in the billing domain and storage layer.

## Context

The storage layer now persists the canonical account kernel:

- `ai_account`
- `ai_account_benefit_lot`
- `ai_account_hold`
- `ai_account_hold_allocation`
- `ai_account_ledger_entry`
- `ai_account_ledger_allocation`
- `ai_request_meter_fact`
- `ai_request_meter_metric`
- `ai_request_settlement`
- `ai_pricing_plan`
- `ai_pricing_rate`

But `sdkwork-api-app-billing` still centers its public behavior around:

- project-scoped quota checks
- coarse `LedgerEntry`
- compatibility-era billing summaries

That gap blocks clean cutover in gateway, admin, and portal because the application layer still has no reusable account-kernel service that can:

- project real account balances
- pick lots for a new hold
- later persist hold and settlement mutations through one consistent flow

## Approaches

### 1. Extend the legacy quota service

Keep `sdkwork-api-app-billing` centered on `QuotaPolicy` and `LedgerEntry`, then slowly bolt account concepts onto it.

Pros:

- low short-term churn
- minimal interface changes

Cons:

- cements the wrong business boundary
- keeps gateway admission quota-first instead of hold-first
- makes multimodal pricing and settlement evidence harder to reason about

### 2. Adapter-first account-kernel service

Introduce clean-slate account orchestration APIs in `sdkwork-api-app-billing` on top of `AccountKernelStore`, while leaving compatibility summaries in place only as temporary consumers.

Pros:

- preserves a single application-layer billing boundary
- gives gateway, admin, and portal one reusable source of truth
- lets storage and domain investments pay off immediately

Cons:

- requires new service types and tests in `sdkwork-api-app-billing`
- temporary duality remains until gateway and UI cut over

### 3. Gateway-first direct orchestration

Skip app-billing evolution and wire hold allocation or settlement logic directly inside `sdkwork-api-interface-http`.

Pros:

- fastest path to one request lifecycle

Cons:

- puts core accounting rules in the wrong layer
- duplicates logic later for admin, portal, jobs, and reconciliation
- makes plugin-first architecture weaker

## Recommendation

Choose approach 2.

`sdkwork-api-app-billing` should become the canonical orchestration layer for account balance projection, lot selection, hold planning, and later hold or settlement persistence. The gateway should call that layer, not rebuild billing logic inline.

## Design

### Service boundary

Keep the compatibility helpers for now, but add clean-slate APIs that depend on `AccountKernelStore` instead of only `AdminStore`.

The first service slice should expose:

- `summarize_account_balance(store, account_id, now_ms)`
- `plan_account_hold(store, account_id, requested_quantity, now_ms)`

These are the minimum safe primitives needed before a mutation flow:

- balance projection proves the new kernel can power admin and portal views
- hold planning proves the spend-priority logic exists in one reusable place

### Read models

Add explicit application-layer result types rather than leaking raw storage records:

- `AccountBalanceSnapshot`
- `AccountLotBalanceSnapshot`
- `PlannedHoldAllocation`
- `AccountHoldPlan`

These should capture:

- available balance
- held balance
- consumed balance
- grant balance
- eligible lots in spend order
- requested quantity, covered quantity, and shortfall quantity

### Eligibility rules

An eligible lot for a new hold must satisfy all of the following:

- same `account_id`
- `status = active`
- not expired at `now_ms`
- positive free balance where `remaining_quantity - held_quantity > 0`

### Spend priority

Default hold planning should follow the clean-slate account design:

1. earliest expiry first
2. scoped lots before unscoped lots
3. non-cash benefits before cash credit
4. lower acquisition cost first
5. deterministic `lot_id` tie-break

This gives stable and auditable lot selection while preserving customer value.

### Balance projection rules

For the first slice, project balances from benefit lots directly:

- `grant_balance = sum(original_quantity)`
- `held_balance = sum(held_quantity)`
- `consumed_balance = sum(original_quantity - remaining_quantity)`
- `available_balance = sum(max(remaining_quantity - held_quantity, 0))`

This is sufficient for account views and hold planning because lot rows are already the canonical inventory state.

### Next mutation slice

After the projection or planning slice is stable, the next service slice should add mutation orchestration:

- create hold record
- create hold allocations
- decrement lot free balance into held balance
- emit hold-create ledger entry plus ledger allocations
- later capture or release the hold into settlement and immutable ledger evidence

That next step should stay in `sdkwork-api-app-billing`, not in the HTTP interface layer.

## Testing

The service must be driven by TDD:

- unit tests for spend-priority ordering and balance math
- SQLite-backed integration tests for real account, lot, and hold-planning flows
- follow-up gateway tests once request admission switches to hold planning

## Rollout

1. land account balance projection and hold planning in `sdkwork-api-app-billing`
2. replace quota-only gateway admission with planned hold evaluation
3. point admin and portal balance pages to the account snapshot service
4. add mutation orchestration for hold creation, release, and settlement

## Decision

Proceed with an adapter-first `sdkwork-api-app-billing` upgrade. This is the cleanest path from the current partial kernel to a real commercial settlement system without rebuilding business logic in multiple layers.
