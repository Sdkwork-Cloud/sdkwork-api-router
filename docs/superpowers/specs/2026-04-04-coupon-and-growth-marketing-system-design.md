# Coupon And Growth Marketing System Design

**Status:** aligned to the local `sdkwork-api-router` repository and current coupon, commerce, admin, and portal implementations on 2026-04-04

**Goal:** replace the current lightweight coupon campaign model with a complete commercial marketing system that supports coupon templates, one-to-many coupon codes, code pools, redemption governance, attribution, referral growth, and finance-safe promotion accounting.

## Executive Decision

The current coupon model is not commercially complete.

Today the repository is effectively modeling:

- one coupon campaign
- one visible code
- one remaining counter
- one active flag

That is enough for a demo or a simple launch promotion, but it is not enough for a serious commercial system.

The correct architecture is:

- separate coupon definition from coupon distribution
- let one coupon template issue many coupon codes
- let redemptions create immutable finance and benefit evidence
- unify coupons, invite rewards, referral rewards, and channel codes under one marketing kernel

The system should not keep evolving around one table like `ai_coupon_campaigns`.

The correct target is a four-layer marketing model:

1. benefit and discount definition
2. campaign and budget governance
3. code issuance and claim lifecycle
4. redemption, fulfillment, and attribution evidence

## Current Gap Assessment

The local source tree shows that the current coupon stack is still minimal.

### Backend limitations

Current domain shape:

- `CouponCampaign { id, code, discount_label, audience, remaining, active, note, expires_on }`

Current system limitations:

- no coupon template versus issued-code separation
- no batch issuance
- no multi-code pool under one coupon
- no code claim lifecycle
- no stacking or exclusivity rules
- no per-user or per-account caps
- no budget or subsidy tracking
- no finance-grade redemption evidence
- no partner or channel attribution
- no referral program model

### Product limitations

Admin currently behaves like a campaign registry, not a marketing platform.

Portal currently behaves like:

- optional code input during checkout
- a simple redeem center
- invite copy tooling

Missing product behaviors:

- claimable coupon inventory
- code batch operations
- referral policy governance
- channel and partner attribution
- promotion budget tracking
- abuse and fraud review
- coupon analytics by source, campaign, and redemption cohort

## Design Principles

### 1. One template can issue many codes

This is the most important correction.

The platform must support:

- vanity shared codes such as `SPRING20`
- bulk unique codes under one template
- invite codes
- account-claimed but not yet redeemed coupons
- system-generated promotional grants without public code entry

### 2. Coupon is not always a discount

A coupon may result in:

- percentage discount
- fixed amount discount
- fixed credit grant
- meter allowance grant
- request allowance grant
- gift product or recharge pack
- trial activation reward

The benefit grammar must be explicit.

### 3. Marketing issuance and financial fulfillment are different

Marketing systems decide who is eligible and which code can be redeemed.

Finance and billing systems decide:

- what commercial value was granted
- how subsidy is accounted for
- which benefit lots were issued
- what order or settlement was affected

These must be linked, but not collapsed.

### 4. Codes must be secure and auditable

The platform should never rely on plaintext coupon storage alone.

Recommended standard:

- store normalized code hash for lookup
- store masked prefix and suffix for operator display
- allow raw export only for authorized batch-generation workflows
- keep issue source, batch source, claim source, and redemption source for audit

### 5. Referral and invite systems are part of marketing, not a side feature

Invite growth, affiliate growth, and activation rewards should be modeled inside the same marketing kernel so:

- attribution is consistent
- reward issuance is finance-safe
- fraud and abuse checks are shared

## Target Architecture

The marketing subsystem should be organized into five bounded components.

### Component 1: Coupon template kernel

Responsibilities:

- define coupon semantics
- define eligibility
- define benefit rules
- define stacking and exclusivity
- define lifecycle state

### Component 2: Campaign and budget kernel

Responsibilities:

- define campaign windows
- define target audience and channels
- define subsidy budget and redemption caps
- define referral or partner program context

### Component 3: Code issuance kernel

Responsibilities:

- create vanity codes
- create bulk code batches
- create invite codes
- reserve, claim, void, or expire codes

### Component 4: Redemption and fulfillment kernel

Responsibilities:

- validate code and context
- create immutable redemption records
- attach discount to order quote or order line
- issue benefit lots when grant-style coupon is redeemed
- prevent duplicate redemption and replay

### Component 5: Attribution and analytics kernel

Responsibilities:

- capture source channel, referral, partner, and campaign touch
- attribute redemption and purchase outcomes
- expose conversion, subsidy, and fraud signals

## Canonical Marketing Model

### Template level

The template defines reusable semantics.

Recommended record:

- `CouponTemplateRecord`

Key fields:

- `coupon_template_id`
- `tenant_id`
- `organization_id`
- `template_code`
- `display_name`
- `benefit_kind`
- `distribution_kind`
- `status`
- `stacking_policy`
- `exclusive_group`
- `starts_at_ms`
- `ends_at_ms`
- `max_total_redemptions`
- `max_redemptions_per_subject`
- `claim_required`
- `created_at_ms`
- `updated_at_ms`

### Benefit rule level

One template can carry multiple benefit rules.

Recommended record:

- `CouponBenefitRuleRecord`

Benefit kinds should support:

- `percentage_discount`
- `fixed_amount_discount`
- `credit_grant`
- `request_allowance_grant`
- `meter_allowance_grant`
- `gift_product`
- `activation_reward`

Benefit rule fields should support:

- target order kind
- target SKU or product scope
- target capability or model scope
- target pricing plan scope
- discount value or grant quantity
- maximum subsidy cap

### Campaign level

Campaign is the operator-facing business envelope.

Recommended record:

- `MarketingCampaignRecord`

Key fields:

- `marketing_campaign_id`
- `campaign_code`
- `display_name`
- `campaign_kind`
- `channel_source`
- `audience_rule_json`
- `budget_amount`
- `currency_code`
- `subsidy_cap_amount`
- `owner_user_id`
- `status`
- `starts_at_ms`
- `ends_at_ms`

Campaign kinds should support:

- `launch`
- `seasonal`
- `winback`
- `partner`
- `referral`
- `enterprise`
- `internal_test`

### Batch level

One template or campaign may issue many codes in batches.

Recommended record:

- `CouponCodeBatchRecord`

Key fields:

- `coupon_code_batch_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `batch_kind`
- `code_prefix`
- `generation_mode`
- `issued_count`
- `claimed_count`
- `redeemed_count`
- `voided_count`
- `expires_at_ms`
- `created_at_ms`

Generation modes should support:

- `manual_vanity`
- `bulk_random`
- `bulk_patterned`
- `invite_auto`
- `claim_wallet`

### Code level

This is the physical or logical redeemable unit.

Recommended record:

- `CouponCodeRecord`

Key fields:

- `coupon_code_id`
- `coupon_code_batch_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `normalized_code_hash`
- `display_code_prefix`
- `display_code_suffix`
- `code_kind`
- `status`
- `claim_subject_type`
- `claim_subject_id`
- `claimed_at_ms`
- `expires_at_ms`
- `issued_at_ms`
- `redeemed_at_ms`

Code kinds should support:

- `shared_vanity`
- `single_use_unique`
- `invite_code`
- `claimable_wallet_coupon`

Code statuses should support:

- `draft`
- `issued`
- `reserved`
- `claimed`
- `redeemed`
- `expired`
- `voided`
- `blocked`

### Claim level

Claim and redeem are different.

Recommended record:

- `CouponClaimRecord`

Use this when:

- a user saves a coupon to account inventory
- an invite code is bound to a subject before actual purchase or benefit redemption

### Redemption level

Redemption is the immutable business event.

Recommended record:

- `CouponRedemptionRecord`

Key fields:

- `coupon_redemption_id`
- `coupon_code_id`
- `coupon_template_id`
- `marketing_campaign_id`
- `tenant_id`
- `organization_id`
- `user_id`
- `account_id`
- `project_id`
- `order_id`
- `payment_order_id`
- `benefit_lot_id`
- `pricing_adjustment_id`
- `redemption_status`
- `subsidy_amount`
- `currency_code`
- `idempotency_key`
- `redeemed_at_ms`

### Attribution level

Recommended records:

- `MarketingAttributionTouchRecord`
- `ReferralProgramRecord`
- `ReferralInviteRecord`

Attribution fields should support:

- `source_kind`
- `source_code`
- `utm_source`
- `utm_campaign`
- `utm_medium`
- `partner_id`
- `referrer_user_id`
- `invite_code_id`
- `conversion_subject_id`
- `converted_at_ms`

## Eligibility And Guardrail Model

Templates need explicit eligibility rules instead of freeform notes.

Recommended first-class conditions:

- first recharge only
- first paid order only
- subscription only
- recharge pack only
- selected product or SKU only
- selected pricing plan only
- selected currency only
- selected country or region only
- selected tenant or organization only
- selected user segment only
- referral or invite required

### Cap and anti-abuse rules

Required first slice:

- total redemption cap
- per-user redemption cap
- per-account redemption cap
- per-project redemption cap
- daily campaign redemption cap
- daily subsidy budget cap
- minimum order amount
- cooldown window

### Stacking rules

Required first slice:

- stackable with order discount
- stackable with recharge gift
- mutually exclusive within group
- only one code per quote
- marketing-only and not combinable with contract pricing

## Relationship To Pricing, Commerce, Payments, And Finance

The coupon and marketing kernel must integrate with existing commercial planes cleanly.

### With pricing

- coupon does not replace pricing plans
- coupon adjusts or supplements pricing outcomes
- pricing remains canonical for standard charge semantics

### With commerce offers

- a coupon can discount an order
- a coupon can grant a product benefit
- a coupon can attach to a subscription or recharge pack

### With payments

- discount-style coupons reduce payable order amount before payment finalization
- grant-style coupons may redeem without payment when policy allows

### With finance

- subsidy and discount value must be journaled
- grant issuance must write benefit-lot evidence
- redeemed value must be attributable to template, code, and campaign

This is the accounting rule:

- marketing decides eligibility
- commerce decides offer shape
- payment decides cash collection
- finance decides accounting truth

## Admin Product Target

The admin product needs a dedicated marketing control plane, not just a coupon registry page.

Required admin modules or workbenches:

- coupon templates
- campaigns
- code batches
- code vault
- redemption explorer
- referral and invite programs
- subsidy budget and burn dashboard
- fraud and abuse review

Required operator actions:

- create one template with many codes
- generate and export code batches
- search masked codes by prefix or suffix
- void or block leaked codes
- inspect code claim and redemption lineage
- review subsidy burn by campaign
- replay fulfillment for stuck redemptions

## Portal Product Target

Portal should provide a clear self-service marketing experience.

Required portal surfaces:

- redeem center
- my coupons and claimed benefits
- eligible offers during checkout
- invite and referral center
- reward history
- applied promotion evidence in quotes and orders

Portal should distinguish:

- code claim
- code apply to quote
- code redeem for grant
- invite reward pending versus activated

## API Surface

### Admin APIs

Required first slice:

- `GET /admin/marketing/coupon-templates`
- `POST /admin/marketing/coupon-templates`
- `GET /admin/marketing/campaigns`
- `POST /admin/marketing/campaigns`
- `GET /admin/marketing/code-batches`
- `POST /admin/marketing/code-batches`
- `GET /admin/marketing/codes`
- `POST /admin/marketing/codes/{coupon_code_id}/void`
- `GET /admin/marketing/redemptions`
- `GET /admin/marketing/referrals`

### Portal APIs

Required first slice:

- `GET /portal/marketing/my-coupons`
- `POST /portal/marketing/claim`
- `POST /portal/marketing/redeem`
- `GET /portal/marketing/referral`
- `POST /portal/marketing/referral/invite`

### Commerce integration APIs

Quote and checkout APIs should support:

- explicit coupon application
- returned coupon validation diagnostics
- returned subsidy amount
- returned promotion source evidence

## P0, P1, And P2 Priorities

### P0

- coupon template versus coupon code separation
- one template to many codes
- batch code generation
- masked code storage and secure lookup
- immutable claim and redemption records
- finance-safe benefit or discount fulfillment
- admin code-batch and redemption management
- portal redeem center backed by canonical coupon records

### P1

- referral and invite program kernel
- campaign budget and burn controls
- audience segmentation and source attribution
- subsidy analytics and anti-abuse workflows
- channel and partner campaign reporting

### P2

- experimentation and A/B uplift tracking
- partner commission settlement
- advanced CRM and warehouse sync

## Final Recommendation

The best coupon architecture for `sdkwork-api-router` is not a bigger campaign table.

The best architecture is:

- coupon templates define rules
- campaigns define business context
- batches issue many codes
- codes move through claim and redemption lifecycles
- redemptions create immutable business and accounting evidence
- referral, invite, and attribution data live inside the same marketing kernel

That design is commercially complete, operationally governable, and compatible with the broader global payment and finance program already defined for this repository.
