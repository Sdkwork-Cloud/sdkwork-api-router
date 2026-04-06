# Enterprise Payment, Order, and Finance Closure Design

**Status:** aligned to the local `sdkwork-api-router` repository and refreshed against official Stripe, Adyen, Alipay, and WeChat Pay references on 2026-04-05

**Goal:** turn `sdkwork-api-router` into a production-grade commercial platform that supports WeChat Pay, Alipay, QR checkout, overseas Stripe payments, complete order center and refund closure, strong account history, finance reconciliation, flow control, failure recovery, monitoring, and controlled provider failover.

## Executive Decision

The repository should not keep extending the current commerce checkout seam as a thin `manual_lab` payment facade.

The correct target state is:

- keep the existing modular crate split across domain, app, storage, interface, and portal or admin packages
- keep `ai_account`, `ai_account_benefit_lot`, `ai_account_hold`, `ai_account_ledger_entry`, and `ai_request_settlement` as the canonical credit and quota kernel
- add a first-class payment orchestration subsystem for provider checkout, callbacks, refunds, disputes, and reconciliation
- add a separate immutable finance journal for cash and accounting posture instead of overloading account ledger history
- treat provider callbacks and verified provider queries as the only payment truth for final settlement
- connect confirmed payment events to account grants and order fulfillment through durable transactional boundaries and replay-safe outbox processing

This design intentionally separates four things that must be linked but must not collapse into one table:

- business order truth
- provider payment truth
- customer account and credit truth
- finance journal truth

That separation is the main difference between a lab-grade checkout flow and a commercial-grade platform.

## Verified Current Baseline

The local repository already contains a strong commercial kernel, but the money-collection layer is still incomplete.

### Strengths that already exist

- `sdkwork-api-app-billing` already implements canonical account, benefit-lot, hold, settlement, refund, and ledger history behavior.
- `sdkwork-api-interface-admin` and `sdkwork-api-interface-portal` already expose account balance, benefit lots, holds, request settlements, pricing plans, pricing rates, and account ledger history.
- `sdkwork-api-app-commerce` already supports commerce orders, quote generation, manual settlement, refund side effects, and payment-event auditing with dedupe.
- `ai_commerce_payment_events` already gives a useful replay-safe audit seam for the existing portal settlement flow.

### Gaps that block commercial-grade payment closure

The current payment flow is still demo-grade:

- `build_checkout_session(...)` returns `provider = "manual_lab"` and `mode = "operator_settlement"`
- the only actionable methods are `manual_settlement` and `cancel_order`
- `provider_handoff` is still marked `planned`
- order fulfillment still depends on internal payment events instead of a canonical provider payment order model
- there is no first-class payment attempt model, provider transaction model, refund attempt model, dispute case model, or reconciliation batch model
- there is no immutable finance journal for cash, liability, revenue, refund, and fee accounting

The practical conclusion is:

- the repository already has a solid account kernel
- it does not yet have a real payment gateway kernel
- it does not yet have a finance accounting kernel

## Industry Reference Baseline

The target design is grounded in four reference classes.

### Stripe

Stripe is the best reference for global hosted or API-driven payment control:

- idempotent request posture at the API boundary
- separation between checkout session, payment intent, charge, refund, and dispute
- webhook-first finality instead of client redirect finality
- hosted checkout for PCI scope reduction
- mature refund, dispute, invoice, subscription, and tax-adjacent extension points

What to borrow:

- explicit payment intent or order lifecycle
- strong idempotency on create, capture, refund, and replay
- event-first closure through webhooks
- operational read models for payment, refund, and dispute history

### Adyen

Adyen is the best reference for payment operations rigor:

- event-driven lifecycle and rich webhook posture
- refunds that may stay pending and may fail later
- reconciliation and finance operations as first-class control-plane behavior
- payment operations designed around retries, asynchronous provider outcomes, and exception handling

What to borrow:

- refund is a lifecycle, not a synchronous boolean
- reconciliation and operations must be built into the system, not bolted on
- callback processing must support delayed and non-terminal results safely

### WeChat Pay

WeChat Pay is the main domestic reference for QR and in-app payment:

- Native payment for QR code flows
- H5 and JSAPI support for mobile and WeChat contexts
- callback verification and encrypted notification handling
- refund callback and provider-side query flows
- bill download and reconciliation requirements

What to borrow:

- scan-code checkout as a first-class session type
- callback verification before any balance grant
- reconciliation imports and provider status query for ambiguous results

### Alipay

Alipay is the other main domestic reference:

- page payment, WAP payment, and precreate QR flows
- asynchronous merchant notification plus active query model
- refund and refund-status query support
- merchant account and channel configuration requirements

What to borrow:

- QR precreate support for desktop and operator-assisted payment
- notify plus query confirmation model
- merchant and application configuration as first-class data

## Distilled Design Rules

Across these references, the correct design rules are stable:

1. payment collection must be callback-first, not browser-first
2. payment order, attempt, transaction, refund, and reconciliation must be modeled separately
3. all money amounts must use integer minor units plus currency code, never floating point
4. provider raw events must be persisted before business mutation
5. account ledger and finance journal must be linked but separate
6. refunds must support partial, asynchronous, failed, and replayed outcomes safely
7. provider failover is valid for new sessions and pre-handoff failures, but not for silently mutating an already customer-visible in-flight payment

## Product Scope

The target system must support the following product capabilities.

### Portal product capabilities

- create quotes and orders for recharge packs, custom recharge, subscription plans, invoice collection, and future operator-issued payment links
- choose payment method by country, currency, and client context
- render QR checkout for WeChat Native and Alipay Precreate
- redirect to hosted Stripe Checkout for overseas card or wallet payment
- poll or subscribe to payment session state
- view order history, payment history, refund status, and account history
- request refunds where policy allows
- show account benefit-lot posture and ledger lineage for successful recharge and refund events

### Admin product capabilities

- order center with filters by status, provider, amount, workspace, user, and failure reason
- payment operations workbench for attempts, callbacks, transaction evidence, and retry actions
- refund review and approval workbench
- account history and grant lineage explorer
- finance reconciliation console
- gateway health, rate-limit posture, and failover posture
- exception queue for stuck callbacks, ambiguous provider results, and mismatched reconciliation lines

### Finance and operations capabilities

- immutable cash journal and line explorer
- provider reconciliation import and automated matching
- operator workflow for mismatch resolution
- dispute or chargeback tracking for Stripe and future gateways
- settlement drift alarms between provider success and account grant completion

## Canonical Architecture

The target architecture should be organized into seven layers.

### Layer 1: Commerce order layer

Responsibilities:

- quote snapshot
- business order creation
- offer and coupon application
- product entitlement intent
- order center read models

This layer answers:

- what the customer is trying to buy
- what price was promised
- what fulfillment should happen after confirmed payment

### Layer 2: Payment orchestration layer

Responsibilities:

- payment order creation
- payment channel selection
- payment attempt creation
- checkout session creation
- refund request orchestration
- provider callback processing
- provider query fallback

This layer answers:

- how money is collected
- what provider is responsible
- what lifecycle state the payment is currently in

### Layer 3: Provider gateway adapter layer

Responsibilities:

- Stripe adapter
- WeChat Pay adapter
- Alipay adapter
- signature or certificate verification
- request or response normalization
- provider-specific reconciliation artifact download

This layer answers:

- how to talk to one provider correctly and safely

### Layer 4: Finance journal layer

Responsibilities:

- immutable cash and liability journal entries
- fees, refunds, subsidy, and chargeback evidence
- journal line generation from confirmed provider transactions

This layer answers:

- what happened financially
- what the accounting posture is

### Layer 5: Account grant and quota closure layer

Responsibilities:

- grant benefit lots for successful recharge
- reverse or reduce benefit lots for successful refund
- link payment order and refund order to account history
- preserve request-settlement and account-ledger closure

This layer answers:

- what usable balance or credit the customer has

### Layer 6: Read models and operations layer

Responsibilities:

- admin order center projections
- portal checkout and history projections
- refund queue
- reconciliation dashboard
- order timeline view

### Layer 7: Observability, control, and recovery layer

Responsibilities:

- callback inbox and outbox replay
- rate limiting and traffic shaping
- circuit breakers and provider health scoring
- dead-letter queues
- metrics, tracing, and structured audit logs

## Canonical Domain Model

The following model keeps the new payment system aligned with the existing crate architecture.

### Existing records that remain authoritative

- `CommerceOrderRecord`
- `CommercePaymentEventRecord` as a compatibility audit projection during migration
- `AccountRecord`
- `AccountBenefitLotRecord`
- `AccountHoldRecord`
- `RequestSettlementRecord`
- `AccountLedgerEntryRecord`
- `AccountLedgerAllocationRecord`

### New canonical payment records

#### `PaymentGatewayAccountRecord`

Purpose:

- merchant account or application configuration for one provider and environment

Key fields:

- `gateway_account_id TEXT`
- `provider_code TEXT`
- `environment TEXT`
- `merchant_id TEXT`
- `app_id TEXT`
- `currency_codes_json TEXT`
- `country_codes_json TEXT`
- `capabilities_json TEXT`
- `status TEXT`
- `secret_ref TEXT`
- `certificate_ref TEXT`
- `notify_config_json TEXT`
- `priority INTEGER`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `PaymentChannelPolicyRecord`

Purpose:

- routing policy for choosing a provider by country, currency, order kind, and client context

Key fields:

- `channel_policy_id TEXT`
- `scene_code TEXT`
- `country_code TEXT`
- `currency_code TEXT`
- `client_kind TEXT`
- `provider_code TEXT`
- `method_code TEXT`
- `priority INTEGER`
- `degrade_to_provider_code TEXT NULL`
- `status TEXT`

#### `PaymentOrderRecord`

Purpose:

- canonical payable order for one business order

Key fields:

- `payment_order_id TEXT`
- `commerce_order_id TEXT`
- `tenant_id BIGINT`
- `organization_id BIGINT`
- `user_id BIGINT`
- `project_id TEXT`
- `order_kind TEXT`
- `subject_type TEXT`
- `subject_id TEXT`
- `currency_code TEXT`
- `amount_minor BIGINT`
- `discount_minor BIGINT`
- `subsidy_minor BIGINT`
- `payable_minor BIGINT`
- `provider_code TEXT`
- `method_code TEXT`
- `payment_status TEXT`
- `fulfillment_status TEXT`
- `refund_status TEXT`
- `quote_snapshot_json TEXT`
- `metadata_json TEXT`
- `version BIGINT`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `PaymentAttemptRecord`

Purpose:

- one provider-side initiation attempt for a payment order

Key fields:

- `payment_attempt_id TEXT`
- `payment_order_id TEXT`
- `attempt_no INTEGER`
- `gateway_account_id TEXT`
- `provider_code TEXT`
- `method_code TEXT`
- `client_kind TEXT`
- `idempotency_key TEXT`
- `provider_request_id TEXT NULL`
- `provider_payment_intent_id TEXT NULL`
- `provider_trade_no TEXT NULL`
- `attempt_status TEXT`
- `failure_code TEXT NULL`
- `failure_message TEXT NULL`
- `request_payload_hash TEXT`
- `response_payload_json TEXT NULL`
- `expires_at_ms BIGINT NULL`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `PaymentSessionRecord`

Purpose:

- customer-facing checkout artifact such as QR code, redirect URL, or hosted checkout URL

Key fields:

- `payment_session_id TEXT`
- `payment_attempt_id TEXT`
- `session_kind TEXT`
- `session_status TEXT`
- `display_reference TEXT`
- `qr_code_url TEXT NULL`
- `qr_payload TEXT NULL`
- `redirect_url TEXT NULL`
- `client_token TEXT NULL`
- `expires_at_ms BIGINT`
- `presented_at_ms BIGINT NULL`
- `closed_at_ms BIGINT NULL`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `PaymentTransactionRecord`

Purpose:

- immutable normalized provider transaction evidence

Key fields:

- `payment_transaction_id TEXT`
- `payment_order_id TEXT`
- `payment_attempt_id TEXT NULL`
- `transaction_kind TEXT`
- `provider_code TEXT`
- `provider_transaction_id TEXT`
- `provider_reference TEXT NULL`
- `currency_code TEXT`
- `amount_minor BIGINT`
- `fee_minor BIGINT NULL`
- `net_amount_minor BIGINT NULL`
- `provider_status TEXT`
- `occurred_at_ms BIGINT`
- `raw_event_id TEXT NULL`
- `created_at_ms BIGINT`

Transaction kinds:

- `authorization`
- `capture`
- `sale`
- `void`
- `refund`
- `refund_failure`
- `chargeback`
- `adjustment`

#### `PaymentCallbackEventRecord`

Purpose:

- raw callback storage with verification and replay posture

Key fields:

- `callback_event_id TEXT`
- `provider_code TEXT`
- `gateway_account_id TEXT`
- `event_type TEXT`
- `event_identity TEXT`
- `payment_order_id TEXT NULL`
- `payment_attempt_id TEXT NULL`
- `provider_transaction_id TEXT NULL`
- `signature_status TEXT`
- `processing_status TEXT`
- `dedupe_key TEXT`
- `headers_json TEXT`
- `payload_json TEXT`
- `received_at_ms BIGINT`
- `processed_at_ms BIGINT NULL`
- `error_message TEXT NULL`

#### `RefundOrderRecord`

Purpose:

- business refund request and approval lifecycle

Key fields:

- `refund_order_id TEXT`
- `payment_order_id TEXT`
- `commerce_order_id TEXT`
- `refund_reason_code TEXT`
- `refund_reason_note TEXT NULL`
- `requested_by_type TEXT`
- `requested_by_id TEXT`
- `approved_by_id BIGINT NULL`
- `currency_code TEXT`
- `requested_amount_minor BIGINT`
- `approved_amount_minor BIGINT NULL`
- `refunded_amount_minor BIGINT`
- `refund_status TEXT`
- `refund_policy_snapshot_json TEXT`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `RefundAttemptRecord`

Purpose:

- one provider-side refund execution attempt

Key fields:

- `refund_attempt_id TEXT`
- `refund_order_id TEXT`
- `attempt_no INTEGER`
- `provider_code TEXT`
- `gateway_account_id TEXT`
- `provider_refund_id TEXT NULL`
- `attempt_status TEXT`
- `provider_status TEXT NULL`
- `failure_code TEXT NULL`
- `failure_message TEXT NULL`
- `created_at_ms BIGINT`
- `updated_at_ms BIGINT`

#### `DisputeCaseRecord`

Purpose:

- chargeback or dispute tracking

Key fields:

- `dispute_case_id TEXT`
- `payment_order_id TEXT`
- `provider_code TEXT`
- `provider_dispute_id TEXT`
- `dispute_status TEXT`
- `reason_code TEXT NULL`
- `amount_minor BIGINT`
- `currency_code TEXT`
- `opened_at_ms BIGINT`
- `closed_at_ms BIGINT NULL`

### New canonical finance journal records

#### `FinanceJournalEntryRecord`

Purpose:

- immutable accounting event header

Key fields:

- `finance_journal_entry_id TEXT`
- `source_kind TEXT`
- `source_id TEXT`
- `entry_code TEXT`
- `currency_code TEXT`
- `entry_status TEXT`
- `occurred_at_ms BIGINT`
- `created_at_ms BIGINT`

#### `FinanceJournalLineRecord`

Purpose:

- immutable accounting line entries

Key fields:

- `finance_journal_line_id TEXT`
- `finance_journal_entry_id TEXT`
- `line_no INTEGER`
- `account_code TEXT`
- `direction TEXT`
- `amount_minor BIGINT`
- `party_type TEXT NULL`
- `party_id TEXT NULL`
- `metadata_json TEXT NULL`

### New canonical reconciliation records

#### `PaymentReconciliationBatchRecord`

Purpose:

- provider reconciliation run header

Key fields:

- `reconciliation_batch_id TEXT`
- `provider_code TEXT`
- `gateway_account_id TEXT`
- `artifact_date TEXT`
- `artifact_kind TEXT`
- `import_status TEXT`
- `matched_count BIGINT`
- `mismatch_count BIGINT`
- `missing_local_count BIGINT`
- `missing_provider_count BIGINT`
- `created_at_ms BIGINT`
- `completed_at_ms BIGINT NULL`

#### `PaymentReconciliationLineRecord`

Purpose:

- one matched or mismatched provider record

Key fields:

- `reconciliation_line_id TEXT`
- `reconciliation_batch_id TEXT`
- `provider_transaction_id TEXT`
- `payment_order_id TEXT NULL`
- `refund_order_id TEXT NULL`
- `provider_amount_minor BIGINT`
- `local_amount_minor BIGINT NULL`
- `provider_status TEXT`
- `local_status TEXT NULL`
- `match_status TEXT`
- `reason_code TEXT NULL`
- `created_at_ms BIGINT`

### Read-model projection

#### `OrderTimelineEventRecord`

Purpose:

- operator-friendly merged timeline across order, payment, refund, ledger, journal, and reconciliation evidence

This is a projection only, never a source of truth.

## State Machines

### Commerce order state

- `draft`: optional future server-side editable order state
- `pending_payment`: payable order exists but no terminal payment outcome yet
- `payment_in_progress`: customer-visible session exists and provider handoff succeeded
- `paid_unfulfilled`: provider has confirmed success but grant or entitlement fulfillment has not completed yet
- `fulfilled`: account grant or entitlement activation completed
- `partially_refunded`: one or more successful refunds exist but the order is not fully reversed
- `refunded`: full refundable value has been reversed
- `canceled`: order closed before successful payment
- `closed`: terminal archival state if needed for future operational closure

### Payment order state

- `created`
- `session_open`
- `awaiting_customer`
- `processing`
- `authorized`
- `captured`
- `failed`
- `expired`
- `canceled`
- `partially_refunded`
- `fully_refunded`
- `chargeback_open`

### Payment attempt state

- `initiated`
- `request_sent`
- `handoff_ready`
- `customer_action_required`
- `provider_processing`
- `succeeded`
- `failed`
- `expired`
- `canceled`

### Refund order state

- `requested`
- `awaiting_approval`
- `approved`
- `submitted`
- `processing`
- `succeeded`
- `partially_succeeded`
- `failed`
- `canceled`

### Reconciliation line state

- `matched`
- `mismatch_amount`
- `mismatch_status`
- `missing_local`
- `missing_provider`
- `resolved`
- `waived`

## Money and Credit Model

This system must use two separate numeric systems.

### Cash and provider money

Rules:

- use `BIGINT amount_minor`
- always store `currency_code`
- never use `f64`
- provider fee, refund amount, tax amount, subsidy amount, and settlement amount are all minor-unit integers

### Internal credits and quota units

Rules:

- keep the existing account kernel for credit quantity and benefit-lot allocation
- if the current `f64`-based quantity model remains, confine it to internal credit and quota quantities
- do not use the internal credit quantity model as the finance money source of truth

## Finance Journal Rules

The finance journal must model money posture, not usage posture.

### Recharge success journal

When payment capture is confirmed:

- debit `gateway_clearing_asset`
- credit `customer_prepaid_liability`

When the payment-funded credit is granted to the customer account:

- no extra cash movement is required
- the grant is represented in the account kernel
- the finance posture remains prepaid liability until usage consumption recognizes revenue

### Usage settlement journal

When prepaid credit is consumed by API usage:

- debit `customer_prepaid_liability`
- credit `usage_revenue`

Provider cost recognition can remain in the existing billing cost system in phase 1, then expand into full expense and payable journal lines later.

### Refund success journal

When a provider refund succeeds:

- debit `customer_prepaid_liability` or `refund_reserve`
- credit `gateway_clearing_asset`

If a partial refund is funded by platform subsidy or promotional grants, journal lines must split:

- customer cash refund amount
- platform subsidy reversal amount
- coupon expense reversal amount

## Functional Flows

### Flow 1: Quote and order creation

1. Portal requests a priced quote from commerce and pricing.
2. Commerce persists a quote snapshot used for reproducibility.
3. Commerce creates `ai_commerce_orders` with `pending_payment`.
4. If `payable_minor = 0`, the order bypasses external payment and fulfills immediately.
5. Otherwise payment orchestration creates or reuses one `PaymentOrderRecord`.

### Flow 2: Checkout session creation

1. Payment orchestration resolves eligible channels by:
   - country
   - currency
   - order kind
   - client kind such as web, mobile web, desktop web, operator, or WeChat browser
2. Routing chooses a `PaymentGatewayAccountRecord` and `method_code`.
3. A new `PaymentAttemptRecord` is created with an idempotency key.
4. The provider adapter creates a provider-side checkout or pay action.
5. A `PaymentSessionRecord` is stored with:
   - QR payload for WeChat Native or Alipay Precreate
   - redirect URL for Stripe Checkout or Alipay page payment
   - future client token for app-based flows
6. The order state moves to `payment_in_progress`.

### Flow 3: Callback-first payment finalization

1. Callback endpoint receives provider event.
2. The raw event is persisted into `PaymentCallbackEventRecord` before any business mutation.
3. Signature or certificate verification is performed.
4. Dedupe is checked on provider event identity.
5. If the provider event is ambiguous or non-terminal, the adapter performs provider query.
6. A normalized `PaymentTransactionRecord` is written.
7. Payment orchestration transitions `PaymentOrderRecord`.
8. If the transaction represents confirmed success:
   - write finance journal entry and lines
   - emit outbox event for account grant or membership activation
   - move commerce order to `paid_unfulfilled`
9. Fulfillment consumer grants benefit lots or activates subscription.
10. After grant success, order moves to `fulfilled`.

### Flow 4: Refund lifecycle

1. Portal or admin creates `RefundOrderRecord`.
2. Refund policy validates refundable amount and refundable asset posture.
3. High-risk or large refunds enter `awaiting_approval`.
4. `RefundAttemptRecord` is submitted to the provider.
5. Callback or query updates refund outcome.
6. On successful refund:
   - write refund transaction evidence
   - write finance journal reversal lines
   - reverse or reduce account benefit lots only for unused or policy-eligible value
   - update order and payment refund status
7. On failed refund:
   - preserve failure evidence
   - allow operator retry if policy allows

### Flow 5: Reconciliation

1. Scheduler downloads reconciliation artifacts by provider and merchant account.
2. Each imported row becomes a `PaymentReconciliationLineRecord`.
3. Matching compares provider transaction, local payment transaction, refund attempt, and finance journal.
4. Mismatches enter operator queue.
5. Resolved lines are annotated but raw imported evidence stays immutable.

### Flow 6: Failure recovery

If any post-payment side effect fails:

- payment success is never discarded
- account grant or order fulfillment is retried from outbox
- unresolved drift between provider success and local fulfillment raises a P0 alert after five minutes
- operator workbench must show the exact failed step

## Refund and Account History Closure Rules

The payment platform must define safe refund rules for different product types.

### Recharge packs and custom recharge

Rules:

- full refund is allowed only when the granted value is still unused or policy-eligible
- partial refund is calculated against remaining refundable granted value
- already consumed prepaid credits are not blindly refundable without explicit policy override
- refund reversal must update both finance journal and account ledger lineage

### Subscription plans

Rules:

- subscription refund uses proration policy
- entitlement rollback must be explicit
- membership activation and included quota grant must be reversible by policy
- already consumed subscription value may require partial refund or no-cash service credit

### Coupon and subsidy interactions

Rules:

- customer cash, platform subsidy, and coupon-funded value must be split
- the refund engine must not refund subsidy as customer cash
- refund journal and order detail must show the split clearly

### Account history linkage

To support commercial-grade account history, extend account ledger evidence with business references:

- `commerce_order_id TEXT NULL`
- `payment_order_id TEXT NULL`
- `refund_order_id TEXT NULL`
- `finance_journal_entry_id TEXT NULL`
- `evidence_code TEXT NULL`

This gives portal and admin history views a complete lineage:

- order
- payment
- refund
- account grant
- journal evidence

## API Design

The API surface should evolve in a way that preserves the current portal and admin style while adding a real payment subsystem.

### Portal APIs

#### Order and checkout

- `POST /portal/commerce/orders`
  - create business order from quote snapshot
- `GET /portal/commerce/orders`
  - list order center items
- `GET /portal/commerce/orders/{order_id}`
  - get order detail
- `POST /portal/commerce/orders/{order_id}/checkout-sessions`
  - create a provider checkout session
- `GET /portal/commerce/orders/{order_id}/checkout-sessions/{payment_session_id}`
  - read session detail, QR payload, redirect posture, and expiry
- `POST /portal/commerce/orders/{order_id}/cancel`
  - cancel order before successful payment

#### Payment status

- `GET /portal/payments/orders/{payment_order_id}`
  - read normalized payment order detail
- `GET /portal/payments/orders/{payment_order_id}/attempts`
  - read attempts
- `GET /portal/payments/orders/{payment_order_id}/timeline`
  - read merged order timeline

#### Refunds

- `POST /portal/refunds`
  - create refund request
- `GET /portal/refunds`
  - list workspace refund requests
- `GET /portal/refunds/{refund_order_id}`
  - get refund detail

#### Account and history

- `GET /portal/billing/account/ledger`
  - existing route expanded with order, payment, and refund linkage
- `GET /portal/billing/payments`
  - tenant-facing payment order list
- `GET /portal/billing/refunds`
  - tenant-facing refund list

### Admin APIs

#### Order center

- `GET /admin/commerce/orders`
- `GET /admin/commerce/orders/{order_id}`
- `GET /admin/commerce/orders/{order_id}/timeline`

#### Payment operations

- `GET /admin/payments/orders`
- `GET /admin/payments/orders/{payment_order_id}`
- `GET /admin/payments/orders/{payment_order_id}/attempts`
- `GET /admin/payments/callback-events`
- `GET /admin/payments/callback-events/{callback_event_id}`
- `POST /admin/payments/callback-events/{callback_event_id}/replay`
- `POST /admin/payments/orders/{payment_order_id}/query-provider`
- `POST /admin/payments/orders/{payment_order_id}/expire-session`

#### Refund operations

- `GET /admin/refunds`
- `GET /admin/refunds/{refund_order_id}`
- `POST /admin/refunds/{refund_order_id}/approve`
- `POST /admin/refunds/{refund_order_id}/reject`
- `POST /admin/refunds/{refund_order_id}/submit`
- `POST /admin/refunds/{refund_order_id}/retry`

#### Finance and reconciliation

- `GET /admin/finance/journal`
- `GET /admin/finance/journal/{finance_journal_entry_id}`
- `POST /admin/payments/reconciliation/runs`
- `GET /admin/payments/reconciliation/runs`
- `GET /admin/payments/reconciliation/runs/{reconciliation_batch_id}`
- `POST /admin/payments/reconciliation/lines/{reconciliation_line_id}/resolve`

### Callback endpoints

- `POST /callbacks/payments/stripe/{gateway_account_id}`
- `POST /callbacks/payments/wechatpay/{gateway_account_id}`
- `POST /callbacks/payments/alipay/{gateway_account_id}`

Each callback endpoint must:

- persist raw body and headers first
- verify provider signature or certificate envelope
- return fast acknowledgement after durable persistence
- continue business mutation asynchronously if needed

## Database Design

All new physical tables follow the current schema rules:

- every table name starts with `ai_`
- every table except `ai_tenant` carries `tenant_id` and `organization_id`
- `tenant_id`, `organization_id`, and `user_id` are `BIGINT`

### New payment tables

#### `ai_payment_gateway_account`

Purpose:

- canonical gateway or merchant configuration

Indexes:

- unique `(provider_code, environment, merchant_id, app_id)`
- `(tenant_id, organization_id, provider_code, status, priority)`

#### `ai_payment_channel_policy`

Purpose:

- runtime payment routing rules

Indexes:

- `(tenant_id, organization_id, scene_code, currency_code, country_code, client_kind, priority)`

#### `ai_payment_order`

Purpose:

- canonical payable order

Indexes:

- unique `(payment_order_id)`
- unique `(tenant_id, organization_id, commerce_order_id)`
- `(tenant_id, organization_id, user_id, created_at_ms DESC)`
- `(tenant_id, organization_id, project_id, created_at_ms DESC)`
- `(tenant_id, organization_id, provider_code, payment_status, updated_at_ms DESC)`

#### `ai_payment_attempt`

Purpose:

- one initiation attempt

Indexes:

- unique `(idempotency_key)`
- `(tenant_id, organization_id, payment_order_id, attempt_no DESC)`
- `(tenant_id, organization_id, provider_code, attempt_status, updated_at_ms DESC)`
- `(provider_code, provider_request_id)`
- `(provider_code, provider_trade_no)`

#### `ai_payment_session`

Purpose:

- customer-facing QR or redirect session

Indexes:

- unique `(payment_session_id)`
- `(tenant_id, organization_id, payment_attempt_id)`
- `(tenant_id, organization_id, session_status, expires_at_ms)`

#### `ai_payment_transaction`

Purpose:

- immutable transaction evidence

Indexes:

- unique `(provider_code, provider_transaction_id, transaction_kind)`
- `(tenant_id, organization_id, payment_order_id, occurred_at_ms DESC)`
- `(tenant_id, organization_id, provider_code, occurred_at_ms DESC)`

#### `ai_payment_callback_event`

Purpose:

- raw callback inbox

Indexes:

- unique `(dedupe_key)`
- `(tenant_id, organization_id, provider_code, received_at_ms DESC)`
- `(tenant_id, organization_id, processing_status, received_at_ms DESC)`
- `(provider_code, event_identity)`

### New refund and dispute tables

#### `ai_refund_order`

Indexes:

- unique `(refund_order_id)`
- `(tenant_id, organization_id, payment_order_id, created_at_ms DESC)`
- `(tenant_id, organization_id, refund_status, updated_at_ms DESC)`

#### `ai_refund_attempt`

Indexes:

- `(tenant_id, organization_id, refund_order_id, attempt_no DESC)`
- `(provider_code, provider_refund_id)`

#### `ai_dispute_case`

Indexes:

- unique `(provider_code, provider_dispute_id)`
- `(tenant_id, organization_id, dispute_status, opened_at_ms DESC)`

### New finance tables

#### `ai_finance_journal_entry`

Indexes:

- unique `(finance_journal_entry_id)`
- `(tenant_id, organization_id, source_kind, source_id)`
- `(tenant_id, organization_id, occurred_at_ms DESC)`

#### `ai_finance_journal_line`

Indexes:

- `(tenant_id, organization_id, finance_journal_entry_id, line_no)`
- `(tenant_id, organization_id, account_code, created_at_ms DESC)`

### New reconciliation tables

#### `ai_payment_reconciliation_batch`

Indexes:

- `(tenant_id, organization_id, provider_code, artifact_date DESC)`
- `(tenant_id, organization_id, import_status, created_at_ms DESC)`

#### `ai_payment_reconciliation_line`

Indexes:

- `(tenant_id, organization_id, reconciliation_batch_id, match_status)`
- `(provider_transaction_id)`
- `(payment_order_id)`
- `(refund_order_id)`

### Projection table

#### `ai_order_timeline_event`

Purpose:

- read-optimized merged event timeline

Indexes:

- `(tenant_id, organization_id, commerce_order_id, created_at_ms DESC)`
- `(tenant_id, organization_id, payment_order_id, created_at_ms DESC)`

### Existing table changes

#### `ai_commerce_orders`

Add:

- `currency_code TEXT`
- `amount_minor BIGINT`
- `discount_minor BIGINT`
- `subsidy_minor BIGINT`
- `payable_minor BIGINT`
- `payment_order_id TEXT NULL`
- `payment_status TEXT`
- `fulfillment_status TEXT`
- `refund_status TEXT`
- `quote_snapshot_json TEXT`
- `version BIGINT NOT NULL DEFAULT 0`

#### `ai_account_ledger_entry`

Add:

- `commerce_order_id TEXT NULL`
- `payment_order_id TEXT NULL`
- `refund_order_id TEXT NULL`
- `finance_journal_entry_id TEXT NULL`
- `evidence_code TEXT NULL`

#### `ai_commerce_payment_events`

Keep as compatibility history during migration, but new canonical callback truth must move to `ai_payment_callback_event`.

## Transactional and Idempotency Guarantees

### Order creation boundary

- quote snapshot and commerce order creation happen atomically
- repeated create requests with the same idempotency key must return the same order if the payload fingerprint matches
- conflicting payload reuse returns `409`

### Payment attempt boundary

- one `payment_attempt_id` per unique provider creation request
- repeated create attempt calls reuse the same attempt when the request fingerprint matches
- provider-side idempotency key must be derived from `payment_attempt_id`

### Callback boundary

- raw callback persistence happens first
- dedupe happens on normalized provider event identity
- business mutation happens only after verification and dedupe
- callback replay never creates duplicate grant, duplicate refund, or duplicate journal entries

### Outbox boundary

The following side effects must run through durable outbox events:

- account grant for successful payment
- entitlement activation
- refund reversal of credits
- order timeline projection updates
- notification and email hooks

### Optimistic locking

Use versioned updates for:

- `ai_commerce_orders`
- `ai_payment_order`
- `ai_refund_order`

Use a version column and compare-and-swap update on state transitions to prevent concurrent processors from racing a single order.

## Provider Routing, Traffic Control, and Failover

The payment subsystem must support controlled routing instead of hardcoded provider selection.

### Routing policy

Default routing should be:

- mainland China plus CNY web checkout:
  - WeChat Native or H5
  - Alipay Precreate or page payment
- overseas or non-CNY checkout:
  - Stripe Checkout

Policy inputs:

- country code
- currency code
- order kind
- client kind
- operator overrides
- gateway health score

### Traffic control

The payment subsystem must add explicit budgets for:

- per-provider checkout creation QPS
- per-provider callback processing concurrency
- per-workspace refund request rate
- per-operator manual override rate
- reconciliation import concurrency

### Circuit breaker rules

For each provider and method:

- open circuit when recent error rate exceeds threshold
- half-open after cool-down
- restore after successful probe window

### Controlled failover rules

Failover is allowed only in these cases:

- provider session creation failed before any customer-visible artifact was issued
- provider channel is marked unavailable before the customer starts payment
- provider session expired without successful payment and the customer explicitly chooses another method

Failover is not allowed silently in these cases:

- an active QR code has already been shown to the customer
- a Stripe hosted checkout session has already been issued
- provider-side payment may already have been submitted

In those cases the system must:

- preserve the first attempt
- expire or cancel it explicitly if supported
- create a new payment attempt under a new session

## Security and Risk Controls

### Core security rules

- never treat browser return or polling as final payment truth
- verify every callback signature, certificate, or encryption envelope
- store secrets in the existing secret abstraction, never inline provider secrets in app config files
- keep raw callback payload and headers for audit, but redact or hash high-risk sensitive fields where required
- every high-risk action must be RBAC-protected, including refund approval, refund retry, reconciliation override, and manual journal adjustments

### Data protection rules

- payment tables store normalized provider identifiers, not full card data
- prefer hosted payment or provider tokenization to reduce PCI scope
- store only the minimum payer data needed for business evidence and customer support
- encrypt sensitive provider configuration and certificate material through the secret layer

### Replay and fraud controls

- dedupe callback events by normalized provider event identity
- rate-limit checkout creation per subject
- rate-limit refund creation per workspace and operator
- record IP, device fingerprint hash, and user agent hash for suspicious flows where policy allows
- flag mismatched country, currency, and payment method combinations

## Performance and Reliability Targets

### Request-path targets

- quote and order creation: p95 <= 150 ms
- checkout session creation excluding provider latency: p95 <= 80 ms
- callback acknowledgement after durable persistence: p95 <= 200 ms
- admin or portal payment detail reads: p95 <= 250 ms

### Integrity targets

- duplicate callback replay must never duplicate credits, refunds, or journal entries
- provider success without local fulfillment older than 5 minutes is a P0 alert
- reconciliation drift older than T+1 is a release blocker
- refund requested versus refund provider result drift older than 30 minutes is a P1 alert

### Recovery posture

- callback failure enters retry queue
- irrecoverable callback enters dead-letter queue
- ambiguous provider state triggers scheduled provider query
- stuck `paid_unfulfilled` orders trigger outbox replay
- stuck `processing` refunds trigger provider status sync

## Monitoring and Telemetry

The system must expose first-class payment metrics and traces.

### Required metrics

- `sdkwork_payment_checkout_create_total{provider,method,outcome}`
- `sdkwork_payment_checkout_latency_ms{provider,method}`
- `sdkwork_payment_callback_total{provider,event_type,signature_status,processing_status}`
- `sdkwork_payment_callback_latency_ms{provider}`
- `sdkwork_payment_order_transition_total{from,to,provider}`
- `sdkwork_payment_refund_total{provider,outcome}`
- `sdkwork_payment_refund_latency_ms{provider}`
- `sdkwork_payment_drift_total{drift_type}`
- `sdkwork_payment_reconciliation_mismatch_total{provider,reason}`
- `sdkwork_payment_circuit_open{provider,method}`
- `sdkwork_payment_dead_letter_total{provider,event_type}`

### Required trace fields

- `commerce_order_id`
- `payment_order_id`
- `payment_attempt_id`
- `payment_session_id`
- `provider_code`
- `provider_transaction_id`
- `callback_event_id`
- `refund_order_id`
- `finance_journal_entry_id`

### Operator-visible queues

- callback retry queue
- payment fulfillment drift queue
- refund retry queue
- reconciliation mismatch queue

## Crate and Module Mapping

To stay aligned with the current repository structure, the best crate-level decomposition is:

### New backend crates

- `crates/sdkwork-api-domain-payment`
  - payment order, attempt, session, transaction, callback, refund, dispute, reconciliation models
- `crates/sdkwork-api-app-payment`
  - orchestration services, provider adapters, idempotency, callback handling, and outbox logic

### Existing crates to extend

- `crates/sdkwork-api-domain-billing`
  - extend account ledger references and credit reversal linkage
- `crates/sdkwork-api-app-billing`
  - grant and refund closure connected to payment success and refund success
- `crates/sdkwork-api-domain-commerce`
  - extend commerce order states and money snapshot fields
- `crates/sdkwork-api-app-commerce`
  - move current checkout seam to payment-orchestrated flow
- `crates/sdkwork-api-storage-core`
  - add store contracts for payment, refund, finance, reconciliation
- `crates/sdkwork-api-storage-sqlite`
  - add new canonical tables and indexes
- `crates/sdkwork-api-storage-postgres`
  - add parity schema and indexes
- `crates/sdkwork-api-interface-portal`
  - add portal order, payment, session, refund endpoints
- `crates/sdkwork-api-interface-admin`
  - add admin order center, payment operations, refund, reconciliation endpoints
- `crates/sdkwork-api-observability`
  - add payment metrics and structured events

### Portal packages to extend

- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-commerce`
  - order center and checkout initiation
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-recharge`
  - recharge-focused checkout UX
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-billing`
  - payment history, refund history, account lineage
- `apps/sdkwork-router-portal/packages/sdkwork-router-portal-settlements`
  - cross-link usage settlement with recharge history where useful

### Admin packages to extend

- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-commercial`
  - order center
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-operations`
  - payment callbacks, retry queues, provider health
- `apps/sdkwork-router-admin/packages/sdkwork-router-admin-pricing`
  - continue to own pricing and quote reproducibility
- add `sdkwork-router-admin-payments` if the current commercial package becomes too broad

## Rollout Plan

### Phase 1: Canonical payment kernel

- introduce payment domain and storage tables
- model payment order, attempt, session, transaction, and callback
- keep current `manual_lab` compatibility flow as a projection
- add portal and admin read APIs for the new records

### Phase 2: Stripe overseas closure

- implement Stripe Checkout adapter
- add webhook verification, refund flow, and dispute record ingestion
- grant account benefit lots only from verified Stripe success events

### Phase 3: WeChat Pay and Alipay domestic QR closure

- implement WeChat Native or H5 and Alipay Precreate or page adapters
- add QR session support in portal
- add callback verification, refund query, and reconciliation download flows

### Phase 4: Refund and finance closure

- add refund approval workflow
- add finance journal and line generation
- extend account ledger with payment and refund lineage
- implement safe partial refund rules by order kind

### Phase 5: Reconciliation, drift detection, and operator tooling

- add reconciliation import and automatic matching
- add operator queues for mismatches and stuck flows
- add provider circuit breakers, traffic controls, and failover policies

### Phase 6: Commercial hardening

- performance tuning
- PostgreSQL production parity validation
- dead-letter and replay operations
- full launch checklist for payment, refund, reconciliation, and monitoring

## Launch Gates

The platform should not be called commercially complete until all of the following are true:

- Stripe payment and refund closure is production-ready
- WeChat Pay payment and refund closure is production-ready
- Alipay payment and refund closure is production-ready
- callback persistence, verification, replay, and dead-letter processing are validated
- successful payment to account grant drift alarms are live
- refund to account reversal drift alarms are live
- reconciliation batch import and mismatch resolution are live
- admin and portal expose order, payment, refund, and account history lineage

## Final Recommendation

The best path for `sdkwork-api-router` is:

- preserve the current modular architecture
- add a first-class payment subsystem instead of stretching `manual_lab`
- keep provider callbacks as authoritative payment truth
- add an immutable finance journal beside the existing account ledger
- connect confirmed payment and refund outcomes to account grants and reversals through idempotent outbox workflows
- expose full operator and tenant history for order, payment, refund, account, and reconciliation evidence

This target state matches the behavior expected from serious payment platforms while staying aligned with the repository's current structure and strengths.

## Reference Baseline

Official references used to ground this design:

- Stripe idempotent requests: https://docs.stripe.com/api/idempotent_requests
- Stripe refunds: https://docs.stripe.com/refunds
- Stripe webhooks and signature verification: https://docs.stripe.com/webhooks
- Stripe Checkout: https://docs.stripe.com/payments/checkout
- Adyen refund lifecycle: https://docs.adyen.com/online-payments/refund
- Adyen webhooks: https://docs.adyen.com/development-resources/webhooks
- Alipay global developer portal: https://global.alipay.com/developer
- Alipay accept payment APIs: https://global.alipay.com/docs/ac/your_apis/accept_payment
- WeChat Pay official SDK and API v3 ecosystem: https://github.com/wechatpay-apiv3/wechatpay-java
- WeChat Pay merchant documentation: https://pay.wechatpay.cn/doc/v3/merchant/4012791877

Implementation note:

- Stripe and Adyen references are directly sufficient for architecture and operational posture
- WeChat Pay and Alipay live implementation details should be re-verified against the current provider documentation and sandbox behavior at implementation time because provider products and merchant qualification rules can change
