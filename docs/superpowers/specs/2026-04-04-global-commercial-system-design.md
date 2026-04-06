# Global Commercial System Design

**Status:** aligned to the local `sdkwork-api-router` repository and refreshed against official payment and routing reference material on 2026-04-04

**Goal:** define the clean-slate target state that turns `sdkwork-api-router` into a complete commercial API router platform with closed payment, finance, pricing, multimodal execution, control-plane, and performance loops.

## Executive Decision

The correct path is **not** a rewrite toward a `new-api`-style monolith.

The correct path is:

- keep the current plugin-first and package-split architecture
- finish the missing commercial closure around payment, finance, and async execution
- keep admin and portal as React package products
- make payment, finance, storage, cache, and runtime capabilities pluggable through explicit contracts

The repository already has a strong kernel:

- canonical commercial account primitives are present
- canonical pricing plans and pricing rates are present
- admin pricing governance is present
- admin and portal product modularity is present
- multimodal HTTP breadth is present
- the async job read model foundation is present

But the platform is still "strong kernel, weak closure" in the areas that determine whether revenue, balance, reconciliation, and customer trust are actually safe:

- payment collection is not yet a canonical subsystem
- finance accounting is not yet a first-class immutable journal
- recharge and subscription closure are not yet real
- refunds, disputes, invoices, and reconciliation are not yet complete
- async multimodal jobs are not yet a durable write-side execution kernel
- control-plane surfaces still over-index on configuration and under-index on financial truth

## Verified Current State

The following are already true in the local source tree.

### 1. The architecture direction is already correct

- backend boundaries are still meaningfully split across domain, app, storage, interface, runtime, and extension crates
- admin and portal remain React package products instead of one large frontend bundle
- storage, cache, runtime, policy, and product seams are already explicit enough to extend

### 2. Canonical billing and pricing work has progressed beyond the original baseline

- canonical account, hold, benefit-lot, and request-settlement records already exist
- admin and portal already expose canonical billing reads
- canonical pricing plans and rates already support rich commercial fields such as charge unit, pricing method, rounding, minimums, and lifecycle state
- admin already has a dedicated pricing governance module instead of burying pricing inside catalog UI only

### 3. Async jobs have started, but only as a foundation slice

- `sdkwork-api-domain-jobs` already defines job, attempt, asset, and callback records
- `sdkwork-api-app-jobs` already exposes read and insert kernels
- SQLite and PostgreSQL already have async job tables
- admin and portal already expose job read APIs

That is useful progress, but it is still not the full async commercial execution kernel.

### 4. The biggest remaining commercial weakness is payment and finance closure

The current `sdkwork-api-app-commerce` layer is still product-demo grade:

- it contains seeded plans, recharge packs, and coupon behavior
- it does not represent canonical payment orders, payment attempts, provider events, or reconciliation
- it does not provide immutable financial evidence
- it does not support global payment providers as pluggable gateways

That means the system can present commercial offers, but it cannot yet run a world-class commercial money flow.

## What To Borrow From `new-api` And OpenRouter

### Useful lessons from `new-api`

`new-api` remains useful as a product-density reference, not as an architecture reference.

The useful lessons are:

- denser provider capability coverage across image, audio, video, and music
- more operator-facing product depth for media workflows
- more direct presentation of provider-specific capability posture

The wrong lesson would be to collapse the codebase into a monolithic transport-first implementation.

### Useful lessons from OpenRouter

OpenRouter is the right inspiration for commercial routing posture:

- provider-routing as an explicit product feature
- model and provider capability metadata as first-class control-plane data
- prompt caching as visible economic behavior
- routing evidence that exposes cost, fallback, latency, and provider posture

The correct adaptation is:

- keep `sdkwork-api-router` modular
- expose OpenRouter-style cost and routing evidence through the admin and portal products
- ensure pricing and settlement remain canonical and auditable instead of purely market-table driven

## Target-State Commercial Architecture

The target platform should be organized into seven layers.

### Layer 1: Gateway and compatibility plane

Responsibilities:

- OpenAI-compatible and provider-compatible HTTP surfaces
- auth subject resolution
- routing compilation and execution
- synchronous request admission
- async job admission for long-running multimodal work

This layer must stay thin. It should never own payment truth.

### Layer 2: Commercial admission and settlement plane

Responsibilities:

- payable subject resolution
- pricing resolution
- account hold planning
- hold capture or release
- request settlement finalization
- immutable settlement evidence

This remains the commercial truth for API usage charging.

### Layer 3: Async multimodal execution plane

Responsibilities:

- enqueue long-running jobs
- claim and heartbeat attempts
- reconcile provider callbacks
- manage retries and timeout transitions
- manage generated assets
- finalize billing after actual output is known

This must become the canonical execution kernel for image, video, audio, and music generation workloads.

### Layer 4: Payment and finance plane

Responsibilities:

- collect money from customers
- manage recharge, subscription, refund, and dispute lifecycles
- create immutable finance journal entries
- reconcile provider reports and platform cash records
- connect successful payment events to commercial balance creation

This is the missing closure that resolves the "strong kernel, weak closure" problem.

### Layer 5: Product control plane

Responsibilities:

- admin operator governance
- portal tenant self-service
- pricing, account, settlement, payment, finance, and job workbenches
- clear visibility into routing, cost, balance, and generated assets

This layer should read like a professional commercial platform, not like a configuration console with a few financial widgets.

### Layer 6: Plugin governance plane

Responsibilities:

- plugin inventory
- capability registry
- version compatibility
- feature flags
- staged rollout
- health and provenance

Payment gateways must become first-class plugins under this standard.

### Layer 7: Storage, cache, and runtime infrastructure plane

Responsibilities:

- relational persistence
- cache and idempotency coordination
- job queue and outbox dispatch
- secrets, telemetry, and audit storage
- dialect and driver selection

This layer remains pluggable and must not leak storage-specific behavior upward.

## Canonical Commercial Data Model

The current system already has a canonical usage-billing model. It now needs a canonical cash-finance model beside it.

### Existing records that remain authoritative

These remain part of the commercial kernel:

- commercial account
- benefit lot
- account hold
- request settlement
- pricing plan
- pricing rate
- billing events

### New canonical payment records

The platform should add a dedicated payment domain with at least the following records:

- `PaymentGatewayAccountRecord`
  - identifies the configured merchant account and gateway environment
- `PaymentOrderRecord`
  - the business order for recharge, subscription, invoice payment, or manual collection
- `PaymentAttemptRecord`
  - a single gateway initiation attempt for a payment order
- `PaymentTransactionRecord`
  - immutable provider-side movement evidence such as authorization, capture, refund, or adjustment
- `PaymentCallbackRecord`
  - raw callback event storage with dedupe and verification status
- `RefundOrderRecord`
  - the business refund request and operator approval lifecycle
- `DisputeCaseRecord`
  - chargeback or dispute tracking
- `InvoiceRecord`
  - invoice or tax-document header
- `InvoiceLineRecord`
  - invoice line items tied to recharge, subscription, or usage
- `ReconciliationBatchRecord`
  - provider or finance reconciliation run header
- `ReconciliationLineRecord`
  - per-transaction or per-order reconciliation evidence

### New canonical finance journal

The platform should introduce an immutable finance journal instead of overloading usage settlement records.

Recommended records:

- `FinanceJournalEntryRecord`
- `FinanceJournalLineRecord`
- `CashBalanceSnapshotRecord`
- `RevenueRecognitionSnapshotRecord`

This is the core rule:

- usage settlement tracks API consumption economics
- finance journal tracks money movement and accounting posture

Those two systems must be linked, but they must not be collapsed into one table.

### Commercial closure rule

A successful payment should not directly mutate arbitrary balance state from a UI callback.

The only correct path is:

1. payment callback or trusted provider query confirms success
2. payment kernel writes immutable provider transaction evidence
3. finance journal records the cash event
4. recharge allocator grants benefit lots or account balance
5. control plane surfaces the result to admin and portal

That is the minimum standard for a serious commercial system.

## Payment Architecture

### Core payment principles

- payment orders are idempotent
- client return pages are not the source of truth
- provider callbacks are persisted before business mutation
- callback processing is idempotent and replay-safe
- every provider event is linked to a business order and finance journal entry
- refunds and disputes are first-class lifecycles, not notes on an order
- successful payment does not bypass reconciliation and audit storage

### Stripe strategy

Stripe should be the default overseas gateway.

P0 capabilities:

- one-time recharge and manual top-up via Payment Intents
- optional hosted checkout for portal flows where the UX benefit outweighs inline control
- subscriptions and invoices for overseas recurring plans
- webhook verification, idempotency, refund lifecycle, and dispute tracking

P1 capabilities:

- Stripe Tax-backed tax calculation where the commercial footprint requires it
- deeper revenue-recognition and invoice workflows

### Alipay strategy

Alipay should be a first-class domestic gateway.

P0 capabilities:

- desktop web payment
- mobile web payment
- QR-native payment for portal and invoice scenarios
- refund initiation
- callback verification and idempotent processing
- bill download and reconciliation ingestion

P1 capabilities:

- advanced enterprise billing flows
- agreement or contract-style recurring payment only if the business model requires it and the merchant qualifications are in place

### WeChat Pay strategy

WeChat Pay should be the other first-class domestic gateway.

P0 capabilities:

- Native payment for QR flows
- H5 payment for mobile web
- JSAPI when the access context is inside WeChat
- refund initiation
- callback decryption and idempotent processing
- transaction and funds bill download plus reconciliation

P1 capabilities:

- App payment if a native client channel becomes a real launch requirement
- more advanced contract payment only when the business model truly needs it

### Gateway plugin contract

Payments should follow the plugin-first architecture, not bespoke conditionals in app logic.

Recommended first-class contract:

- `PaymentGatewayPlugin`

Required operations:

- `create_checkout_session(...)`
- `create_payment_attempt(...)`
- `query_payment_status(...)`
- `parse_and_verify_callback(...)`
- `refund_payment(...)`
- `query_refund_status(...)`
- `download_reconciliation_artifact(...)`
- `supported_methods()`
- `supported_regions()`
- `supported_currencies()`

Required metadata:

- `plugin_id`
- `plugin_kind = payment_gateway`
- `gateway_code`
- `environment`
- `capabilities`
- `currencies`
- `countries`
- `webhook_requirements`
- `reconciliation_modes`

This keeps Stripe, Alipay, and WeChat integrations aligned with the rest of the plugin system.

## Pricing, Offer, And Billing Model

The platform should keep three separate but linked planes.

### Plane 1: Capability and market catalog

This describes:

- providers
- models
- modalities
- market price references
- routing and availability posture

### Plane 2: Commercial pricing

This is already started and remains the authoritative source for API charging:

- pricing plans
- pricing rates
- charge units
- pricing methods
- lifecycle versions

### Plane 3: Commercial offers

This should be the product and commerce layer:

- recharge packs
- subscriptions
- prepaid and invoice-based offers
- enterprise contract offers
- coupons and promotions
- referral and invite rewards
- channel and partner campaign offers

The mistake would be to mix these into one schema.

The correct model is:

- pricing plans and rates decide API usage economics
- commercial offers decide how customers buy credits or plans
- payment orders decide how money is collected

## Coupon And Growth Marketing System

The current coupon model is still too thin for a commercial platform.

The target state should use a dedicated marketing kernel with:

- coupon templates
- campaign envelopes
- code batches
- one-to-many coupon codes under one template
- claim and redemption records
- referral and invite programs
- attribution and subsidy evidence

This is the key commercial rule:

- a coupon template defines the commercial rule
- a code batch defines issued inventory
- a coupon code is the redeemable unit
- a redemption record is the immutable business event
- finance and billing own the resulting subsidy and grant evidence

The platform should support both:

- discount coupons that adjust order pricing
- grant coupons that issue benefit lots or activation rewards

The platform should not continue using one campaign table as the only coupon model.

## Async Multimodal Execution Closure

The async job foundation now needs to become the full write-side kernel.

Required behaviors:

- enqueue and dedupe by idempotency key
- claim and lease attempts
- retry with bounded backoff and error classification
- callback ingestion with replay suppression
- asset lifecycle tracking
- progress updates
- timeout and abandonment recovery
- final billing settlement using actual output evidence

Required routing rule:

- text-first synchronous routes can remain sync where latency and provider behavior make sense
- long-running image, video, audio, and music routes must be able to enter the async job kernel by default

The best commercial behavior is hybrid:

- sync fast-path where truly short
- durable async kernel where not guaranteed to be short

## Admin Product Target

Admin should remain package-split, but its commercial posture needs to deepen substantially.

Required admin modules:

- `pricing`
  - already present and should remain canonical
- `commercial accounts`
  - balances, benefit lots, holds, settlements, recharge grants
- `payments`
  - payment orders, attempts, callbacks, refunds, disputes, gateway posture
- `finance`
  - reconciliation batches, journal explorer, invoice state, settlement drift
- `async jobs`
  - job queue, attempt visibility, callback failures, asset review
- `provider economics`
  - upstream cost versus retail price, routing fallback economics, prompt-cache effect
- `plugin governance`
  - payment gateway plugins, provider runtime plugins, storage and cache capability posture

Required operator powers:

- manually review and replay payment callbacks
- freeze or release payment orders
- trigger reconciliation runs
- inspect order-to-journal-to-balance lineage
- view provider routing and pricing evidence side by side

## Portal Product Target

Portal should remain package-split and self-service oriented.

Required portal surfaces:

- account balance and benefit-lot posture
- pricing and plan visibility
- recharge center with global and domestic payment choices
- order history
- invoice and tax-document center
- refund request status where policy allows
- settlement explorer
- async job and generated asset center
- API capability catalog and routing-cost transparency

The portal should never directly mutate balance from browser state. It can start checkout, but canonical state comes from the backend payment kernel only.

## Storage, Cache, And Database Strategy

### Databases

Target production support:

- SQLite for single-node and embedded modes
- PostgreSQL as the first full commercial production dialect
- MySQL and LibSQL later, after honest parity work

### Cache and coordination

Redis should become the default shared backend for:

- payment callback dedupe
- idempotency key coordination
- short-lived checkout session cache
- pricing and capability hot reads
- async job lease coordination where appropriate

Memory cache can remain valid for embedded or development modes, but it is not enough for a serious distributed commercial deployment.

### Outbox and replay model

All external side effects should use durable replayable seams:

- payment callback intake writes raw callback records first
- finance side effects emit through an outbox
- recharge grants are idempotent by business key
- reconciliation writes immutable batch evidence

## Security And Compliance Principles

- keep card data and regulated payment instrument details out of platform scope whenever possible by using hosted or provider-tokenized flows
- verify every provider callback signature or encryption envelope
- never grant credits on client redirect alone
- keep immutable audit evidence for payment, refund, dispute, and reconciliation state changes
- isolate payment secrets behind the existing secrets abstraction
- add role-based admin permissions for high-risk operations such as refund approval, reconciliation override, and manual adjustment

## Performance And Reliability Targets

The platform should publish and enforce explicit budgets.

### Request-path targets

- gateway internal overhead excluding upstream provider latency: p95 <= 40 ms, p99 <= 100 ms
- pricing resolution and commercial account admission: p95 <= 10 ms on hot path
- canonical hold and settlement transaction: p95 <= 25 ms
- async job enqueue: p95 <= 80 ms

### Control-plane targets

- admin and portal read APIs: p95 <= 250 ms for common list and detail paths
- checkout session creation: p95 <= 300 ms
- payment callback acknowledgement: p95 <= 200 ms after raw event persistence

### Integrity targets

- duplicate callback replay must never create duplicate recharge grants
- unresolved payment-success versus balance-grant drift older than 5 minutes is a P0 alert
- daily reconciliation drift older than T+1 is a release blocker
- settlement evidence must reproduce retail charge and upstream cost deterministically

### Degraded-mode policy

- if a payment callback cannot be fully processed, persist it, mark it for replay, and do not grant balance optimistically
- if reconciliation fails, freeze affected payout or refund flows until operator review
- if async jobs backlog exceeds SLO, the control plane must surface visible degradation instead of silently continuing

## P0, P1, And P2 Priorities

### P0: must land before calling the platform commercially complete

- canonical payment and finance domain
- Stripe production-grade one-time payment and subscription support
- Alipay and WeChat Pay production-grade one-time payment support
- coupon template and code-pool separation
- one template to many coupon codes with batch management
- immutable claim and redemption records
- callback verification, idempotency, refunds, and reconciliation ingestion
- balance grant closure from confirmed payment into canonical account kernel
- async multimodal write-side execution kernel
- gateway cutover for long-running image, video, audio, and music flows
- admin and portal payment, finance, and job workbenches
- PostgreSQL commercial parity
- Redis-backed idempotency and callback replay suppression
- explicit launch gates for settlement drift, payment drift, and async job backlog

### P1: should land soon after P0 to reach an elite platform standard

- invoice and tax-document center
- Stripe Tax or equivalent overseas tax posture where needed
- dispute management and operator decision flows
- referral and invite program kernel
- campaign budget and subsidy-burn control plane
- advanced enterprise offers, approvals, and contract pricing
- plugin governance center with compatibility snapshots
- richer provider economics, cost analytics, and prompt-cache visibility
- advanced recurring billing where the business model requires it

### P2: valuable but not required for the first elite commercial release

- MySQL parity
- LibSQL parity
- automated FX posture and multi-currency treasury optimization
- revenue-sharing and reseller settlement
- warehouse-native finops exports and anomaly detection

## Final Recommendation

The best target state for `sdkwork-api-router` is:

- keep the current modular backend and package-split frontend architecture
- turn payments into a first-class plugin-governed subsystem
- add a real finance journal beside the existing usage-settlement kernel
- convert multimodal long-running operations onto the durable async job kernel
- make admin and portal speak the canonical commercial truth directly
- freeze performance, reconciliation, and settlement launch gates before commercial release

This design is stronger than a monolith, more commercially correct than a pricing-only router, and closer to the standard of an advanced global API router platform.

## Reference Baseline

The target state above is aligned to the following external product and integration baselines:

- Stripe Payment Intents, webhooks, and subscriptions documentation
- WeChat Pay official API v3 documentation for Native, H5, JSAPI, callback handling, and bill download
- Alipay Open Platform web and payment product surfaces
- OpenRouter documentation for provider routing and prompt caching posture

Implementation note:

- Stripe and WeChat Pay operational requirements are directly anchored in official documentation
- Alipay refund, callback, and reconciliation behavior should be finalized against live Open Platform documentation and sandbox verification during gateway implementation because portions of the official documentation are delivered through dynamic documentation pages
- coupon and marketing target-state details are further expanded in `docs/superpowers/specs/2026-04-04-coupon-and-growth-marketing-system-design.md`
