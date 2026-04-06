use async_trait::async_trait;
use sdkwork_api_app_coupon::list_active_coupons;
use sdkwork_api_app_marketing::{
    confirm_coupon_redemption, project_legacy_coupon_campaign, reserve_coupon_redemption,
    rollback_coupon_redemption, validate_coupon_stack, CouponValidationDecision,
};
use sdkwork_api_domain_billing::QuotaPolicy;
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
    ProjectMembershipRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRedemptionStatus, CouponReservationStatus,
    CouponRollbackType, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingSubjectScope,
};
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_core::{
    AtomicCouponConfirmationCommand, AtomicCouponReleaseCommand, AtomicCouponReservationCommand,
    AtomicCouponRollbackCommand,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

pub use sdkwork_api_domain_commerce::CommerceOrderRecord as PortalCommerceOrderRecord;
pub use sdkwork_api_domain_commerce::CommercePaymentEventRecord as PortalCommercePaymentEventRecord;
pub use sdkwork_api_domain_commerce::ProjectMembershipRecord as PortalProjectMembershipRecord;

type CommerceResult<T> = std::result::Result<T, CommerceError>;
const DEFAULT_COUPON_RESERVATION_TTL_MS: u64 = 15 * 60 * 1_000;
const COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB: &str = "manual_lab";
const COMMERCE_PAYMENT_PROVIDER_STRIPE: &str = "stripe";
const COMMERCE_PAYMENT_PROVIDER_ALIPAY: &str = "alipay";
const COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY: &str = "wechat_pay";
const COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED: &str = "no_payment_required";
const COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT: &str = "operator_settlement";
const COMMERCE_PAYMENT_CHANNEL_HOSTED_CHECKOUT: &str = "hosted_checkout";
const COMMERCE_PAYMENT_CHANNEL_SCAN_QR: &str = "scan_qr";

#[derive(Debug)]
pub enum CommerceError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Storage(anyhow::Error),
}

impl std::fmt::Display for CommerceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Conflict(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CommerceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for CommerceError {
    fn from(value: anyhow::Error) -> Self {
        Self::Storage(value)
    }
}

fn commerce_atomic_coupon_error(error: anyhow::Error) -> CommerceError {
    let message = error.to_string();
    if message.contains("changed concurrently")
        || message.contains("already exists with different state")
        || message.contains(" is missing")
    {
        CommerceError::Conflict(message)
    } else {
        CommerceError::Storage(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalSubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price_label: String,
    pub cadence: String,
    pub included_units: u64,
    pub highlight: String,
    pub features: Vec<String>,
    pub cta: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalRechargePack {
    pub id: String,
    pub label: String,
    pub points: u64,
    pub price_label: String,
    pub note: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalRechargeOption {
    pub id: String,
    pub label: String,
    pub amount_cents: u64,
    pub amount_label: String,
    pub granted_units: u64,
    pub effective_ratio_label: String,
    pub note: String,
    pub recommended: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCustomRechargeRule {
    pub id: String,
    pub label: String,
    pub min_amount_cents: u64,
    pub max_amount_cents: u64,
    pub units_per_cent: u64,
    pub effective_ratio_label: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCustomRechargePolicy {
    pub enabled: bool,
    pub min_amount_cents: u64,
    pub max_amount_cents: u64,
    pub step_amount_cents: u64,
    pub suggested_amount_cents: u64,
    pub rules: Vec<PortalCustomRechargeRule>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCoupon {
    pub id: String,
    pub code: String,
    pub discount_label: String,
    pub audience: String,
    pub remaining: u64,
    pub active: bool,
    pub note: String,
    pub expires_on: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default)]
    pub bonus_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCatalog {
    pub plans: Vec<PortalSubscriptionPlan>,
    pub packs: Vec<PortalRechargePack>,
    pub recharge_options: Vec<PortalRechargeOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_recharge_policy: Option<PortalCustomRechargePolicy>,
    pub coupons: Vec<PortalCommerceCoupon>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceQuoteRequest {
    pub target_kind: String,
    pub target_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_amount_cents: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalAppliedCoupon {
    pub code: String,
    pub discount_label: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default)]
    pub bonus_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceQuote {
    pub target_kind: String,
    pub target_id: String,
    pub target_name: String,
    pub list_price_cents: u64,
    pub payable_price_cents: u64,
    pub list_price_label: String,
    pub payable_price_label: String,
    pub granted_units: u64,
    pub bonus_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_cents: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projected_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon: Option<PortalAppliedCoupon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_rule_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_ratio_label: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCheckoutSessionMethod {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub action: String,
    pub availability: String,
    pub provider: String,
    pub channel: String,
    pub session_kind: String,
    pub session_reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qr_code_payload: Option<String>,
    pub webhook_verification: String,
    pub supports_refund: bool,
    pub supports_partial_refund: bool,
    pub recommended: bool,
    pub supports_webhook: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCheckoutSession {
    pub order_id: String,
    pub order_status: String,
    pub session_status: String,
    pub provider: String,
    pub mode: String,
    pub reference: String,
    pub payable_price_label: String,
    pub guidance: String,
    pub methods: Vec<PortalCommerceCheckoutSessionMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommercePaymentEventRequest {
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout_method_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct CommerceCouponBenefit {
    discount_percent: Option<u8>,
    bonus_units: u64,
}

#[derive(Debug, Clone)]
struct CommerceCouponDefinition {
    coupon: PortalCommerceCoupon,
    benefit: CommerceCouponBenefit,
}

#[derive(Debug, Clone)]
struct ResolvedCouponDefinition {
    definition: CommerceCouponDefinition,
    marketing: Option<MarketingCouponContext>,
}

#[derive(Debug, Clone)]
struct MarketingCouponContext {
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
    source: String,
}

#[derive(Debug, Clone)]
struct ReservedMarketingCouponState {
    coupon_reservation_id: String,
    marketing_campaign_id: String,
    subsidy_amount_minor: u64,
}

#[derive(Debug, Clone, Copy)]
struct SubscriptionPlanSeed {
    id: &'static str,
    name: &'static str,
    price_cents: u64,
    cadence: &'static str,
    included_units: u64,
    highlight: &'static str,
    features: &'static [&'static str],
    cta: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct RechargePackSeed {
    id: &'static str,
    label: &'static str,
    points: u64,
    price_cents: u64,
    note: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct RechargeOptionSeed {
    id: &'static str,
    label: &'static str,
    amount_cents: u64,
    granted_units: u64,
    note: &'static str,
    recommended: bool,
}

#[derive(Debug, Clone, Copy)]
struct CustomRechargeRuleSeed {
    id: &'static str,
    label: &'static str,
    min_amount_cents: u64,
    max_amount_cents: u64,
    units_per_cent: u64,
    note: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct CouponSeed {
    id: &'static str,
    code: &'static str,
    discount_label: &'static str,
    audience: &'static str,
    remaining: u64,
    note: &'static str,
    expires_on: &'static str,
    discount_percent: Option<u8>,
    bonus_units: u64,
}

pub async fn load_portal_commerce_catalog(
    store: &dyn AdminStore,
) -> CommerceResult<PortalCommerceCatalog> {
    Ok(PortalCommerceCatalog {
        plans: subscription_plan_catalog(),
        packs: recharge_pack_catalog(),
        recharge_options: recharge_option_catalog(),
        custom_recharge_policy: Some(build_custom_recharge_policy()),
        coupons: load_coupon_catalog(store).await?,
    })
}

pub async fn preview_portal_commerce_quote(
    store: &dyn AdminStore,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<PortalCommerceQuote> {
    Ok(preview_portal_commerce_quote_internal(store, request)
        .await?
        .0)
}

async fn preview_portal_commerce_quote_internal(
    store: &dyn AdminStore,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<(PortalCommerceQuote, Option<ResolvedCouponDefinition>)> {
    let target_kind = request.target_kind.trim();
    let target_id = request.target_id.trim();

    if target_kind.is_empty() {
        return Err(CommerceError::InvalidInput(
            "target_kind is required".to_owned(),
        ));
    }
    if target_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "target_id is required".to_owned(),
        ));
    }

    match target_kind {
        "subscription_plan" => {
            let plan = subscription_plan_seeds()
                .into_iter()
                .find(|candidate| candidate.id.eq_ignore_ascii_case(target_id))
                .ok_or_else(|| CommerceError::NotFound("subscription plan not found".to_owned()))?;
            let applied_coupon = load_optional_applied_coupon(
                store,
                request.coupon_code.as_deref(),
                target_kind,
                plan.price_cents,
            )
            .await?;
            let quote = build_priced_quote(
                "subscription_plan",
                plan.id,
                plan.name,
                plan.price_cents,
                plan.included_units,
                "workspace_seed",
                request.current_remaining_units,
                applied_coupon.as_ref().map(|item| item.definition.clone()),
            );
            Ok((quote, applied_coupon))
        }
        "recharge_pack" => {
            let pack = recharge_pack_seeds()
                .into_iter()
                .find(|candidate| candidate.id.eq_ignore_ascii_case(target_id))
                .ok_or_else(|| CommerceError::NotFound("recharge pack not found".to_owned()))?;
            let applied_coupon = load_optional_applied_coupon(
                store,
                request.coupon_code.as_deref(),
                target_kind,
                pack.price_cents,
            )
            .await?;
            let quote = build_priced_quote(
                "recharge_pack",
                pack.id,
                pack.label,
                pack.price_cents,
                pack.points,
                "workspace_seed",
                request.current_remaining_units,
                applied_coupon.as_ref().map(|item| item.definition.clone()),
            );
            Ok((quote, applied_coupon))
        }
        "custom_recharge" => {
            let custom_amount_cents =
                resolve_custom_recharge_amount_cents(target_id, request.custom_amount_cents)?;
            let applied_coupon = load_optional_applied_coupon(
                store,
                request.coupon_code.as_deref(),
                target_kind,
                custom_amount_cents,
            )
            .await?;
            let quote = build_custom_recharge_quote(
                custom_amount_cents,
                request.current_remaining_units,
                applied_coupon.as_ref().map(|item| item.definition.clone()),
            )?;
            Ok((quote, applied_coupon))
        }
        "coupon_redemption" => {
            let coupon = find_resolved_coupon_definition(store, target_id).await?;
            if coupon.definition.benefit.bonus_units == 0 {
                return Err(CommerceError::InvalidInput(format!(
                    "coupon {} does not grant redeemable bonus units",
                    coupon.definition.coupon.code
                )));
            }
            let quote =
                build_redemption_quote(coupon.definition.clone(), request.current_remaining_units);
            Ok((quote, Some(coupon)))
        }
        _ => Err(CommerceError::InvalidInput(format!(
            "unsupported target_kind: {target_kind}"
        ))),
    }
}

pub async fn submit_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    request: &PortalCommerceQuoteRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let (quote, resolved_coupon) = preview_portal_commerce_quote_internal(store, request).await?;
    let status = initial_order_status(&quote);
    let order_id = generate_entity_id("commerce_order")?;
    let reserved_coupon = reserve_order_coupon_if_needed(
        store,
        &order_id,
        normalized_project_id,
        &quote,
        resolved_coupon.as_ref(),
    )
    .await?;

    let mut order = CommerceOrderRecord::new(
        order_id,
        normalized_project_id,
        normalized_user_id,
        quote.target_kind.clone(),
        quote.target_id.clone(),
        quote.target_name.clone(),
        quote.list_price_cents,
        quote.payable_price_cents,
        quote.list_price_label.clone(),
        quote.payable_price_label.clone(),
        quote.granted_units,
        quote.bonus_units,
        status,
        quote.source.clone(),
        current_time_ms()?,
    )
    .with_applied_coupon_code_option(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.code.clone()),
    )
    .with_coupon_reservation_id_option(
        reserved_coupon
            .as_ref()
            .map(|coupon| coupon.coupon_reservation_id.clone()),
    )
    .with_marketing_campaign_id_option(
        reserved_coupon
            .as_ref()
            .map(|coupon| coupon.marketing_campaign_id.clone()),
    )
    .with_subsidy_amount_minor(
        reserved_coupon
            .as_ref()
            .map(|coupon| coupon.subsidy_amount_minor)
            .unwrap_or(0),
    );

    if should_fulfill_on_order_create(&quote) {
        if let Err(error) = fulfill_order_on_create(
            store,
            normalized_user_id,
            normalized_project_id,
            &quote,
            &mut order,
        )
        .await
        {
            let _ = release_order_coupon_reservation_if_needed(store, &mut order).await;
            return Err(error);
        }
    }

    match store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
    {
        Ok(order) => Ok(order),
        Err(error) => {
            if order.coupon_redemption_id.is_some() {
                let _ = rollback_order_coupon_redemption_if_needed(
                    store,
                    &mut order,
                    CouponRollbackType::Manual,
                )
                .await;
            } else {
                let _ = release_order_coupon_reservation_if_needed(store, &mut order).await;
            }
            Err(error)
        }
    }
}

pub async fn settle_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    settle_portal_commerce_order_with_payment_event(store, user_id, project_id, order_id, None)
        .await
}

async fn settle_portal_commerce_order_with_payment_event(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    payment_event_id: Option<&str>,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "fulfilled" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be settled from status {other}"
            )))
        }
    }

    let settlement_quote = load_order_settlement_quote(store, &order).await?;
    apply_quote_to_project_quota(store, normalized_project_id, &settlement_quote).await?;
    activate_project_membership_if_needed(
        store,
        normalized_user_id,
        normalized_project_id,
        &settlement_quote,
    )
    .await?;
    confirm_order_coupon_if_needed(store, &mut order, payment_event_id).await?;

    order.status = "fulfilled".to_owned();
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub async fn cancel_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "canceled" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be canceled from status {other}"
            )))
        }
    }

    release_order_coupon_reservation_if_needed(store, &mut order).await?;
    order.status = "canceled".to_owned();
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

async fn refund_portal_commerce_order<T>(
    store: &T,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    refund_provider: &str,
) -> CommerceResult<CommerceOrderRecord>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;
    ensure_refund_provider_matches_order_settlement(store, &order, refund_provider).await?;

    if !supports_safe_order_refund(&order) {
        return Err(CommerceError::Conflict(format!(
            "order {normalized_order_id} target_kind {} cannot be refunded safely",
            order.target_kind
        )));
    }

    match order.status.as_str() {
        "refunded" => return Ok(order),
        "fulfilled" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be refunded from status {other}"
            )))
        }
    }

    reverse_order_quota_effect(store, normalized_project_id, &order).await?;
    rollback_order_coupon_redemption_if_needed(store, &mut order, CouponRollbackType::Refund)
        .await?;

    order.status = "refunded".to_owned();
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

pub async fn apply_portal_commerce_payment_event(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let event_type = request.event_type.trim();
    if event_type.is_empty() {
        return Err(CommerceError::InvalidInput(
            "event_type is required".to_owned(),
        ));
    }

    if !matches!(event_type, "settled" | "canceled" | "failed" | "refunded") {
        return Err(CommerceError::InvalidInput(format!(
            "unsupported payment event_type: {event_type}"
        )));
    }

    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let current_order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;
    let checkout_method = resolve_checkout_method_for_payment_event(&current_order, request)?;
    let (provider, provider_event_id, dedupe_key) =
        resolve_commerce_payment_event_identity(&current_order.order_id, request, checkout_method)?;

    let existing_event = store
        .find_commerce_payment_event_by_dedupe_key(&dedupe_key)
        .await
        .map_err(CommerceError::from)?;
    if let Some(existing_event) = existing_event.as_ref() {
        if existing_event.order_id != current_order.order_id {
            return Err(CommerceError::Conflict(format!(
                "payment event {dedupe_key} already belongs to order {}",
                existing_event.order_id
            )));
        }
        if matches!(
            existing_event.processing_status,
            CommercePaymentEventProcessingStatus::Processed
                | CommercePaymentEventProcessingStatus::Ignored
        ) {
            return load_project_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
            )
            .await;
        }
    }

    let received_at_ms = current_time_ms()?;
    let mut payment_event = build_commerce_payment_event_record(
        &current_order,
        request,
        provider,
        provider_event_id,
        dedupe_key.clone(),
        received_at_ms,
        existing_event
            .as_ref()
            .map(|event| event.payment_event_id.as_str()),
    )?;
    payment_event = persist_commerce_payment_event(store, payment_event).await?;

    let order_result = match event_type {
        "settled" => {
            settle_portal_commerce_order_with_payment_event(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
                Some(payment_event.payment_event_id.as_str()),
            )
            .await
        }
        "canceled" => {
            cancel_portal_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
            )
            .await
        }
        "failed" => {
            fail_portal_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
            )
            .await
        }
        "refunded" => {
            refund_portal_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
                payment_event.provider.as_str(),
            )
            .await
        }
        _ => unreachable!(),
    };

    match order_result {
        Ok(order) => {
            let _ = persist_commerce_payment_event(
                store,
                finalize_commerce_payment_event(
                    payment_event,
                    CommercePaymentEventProcessingStatus::Processed,
                    None,
                    Some(order.status.clone()),
                    Some(current_time_ms()?),
                ),
            )
            .await;
            Ok(order)
        }
        Err(error) => {
            let rejection_status = commerce_payment_event_status_for_error(&error);
            let _ = persist_commerce_payment_event(
                store,
                finalize_commerce_payment_event(
                    payment_event,
                    rejection_status,
                    Some(error.to_string()),
                    Some(current_order.status.clone()),
                    Some(current_time_ms()?),
                ),
            )
            .await;
            Err(error)
        }
    }
}

pub async fn list_order_commerce_payment_events(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<Vec<CommercePaymentEventRecord>> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;
    let mut events = store
        .list_commerce_payment_events_for_order(&order.order_id)
        .await
        .map_err(CommerceError::from)?;
    events.sort_by(|left, right| {
        right
            .received_at_ms
            .cmp(&left.received_at_ms)
            .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
    });
    Ok(events)
}

pub async fn load_portal_commerce_checkout_session(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<PortalCommerceCheckoutSession> {
    let order = load_project_commerce_order(store, user_id, project_id, order_id).await?;
    Ok(build_checkout_session(&order))
}

pub async fn list_project_commerce_orders(
    store: &dyn AdminStore,
    project_id: &str,
) -> CommerceResult<Vec<CommerceOrderRecord>> {
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let mut orders = store
        .list_commerce_orders_for_project(normalized_project_id)
        .await
        .map_err(CommerceError::from)?;
    orders.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
            .then_with(|| right.order_id.cmp(&left.order_id))
    });
    Ok(orders)
}

pub async fn load_project_membership(
    store: &dyn AdminStore,
    project_id: &str,
) -> CommerceResult<Option<ProjectMembershipRecord>> {
    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    store
        .find_project_membership(normalized_project_id)
        .await
        .map_err(CommerceError::from)
}

fn subscription_plan_catalog() -> Vec<PortalSubscriptionPlan> {
    subscription_plan_seeds()
        .into_iter()
        .map(|seed| PortalSubscriptionPlan {
            id: seed.id.to_owned(),
            name: seed.name.to_owned(),
            price_label: format_catalog_price_label(seed.price_cents),
            cadence: seed.cadence.to_owned(),
            included_units: seed.included_units,
            highlight: seed.highlight.to_owned(),
            features: seed
                .features
                .iter()
                .map(|feature| (*feature).to_owned())
                .collect(),
            cta: seed.cta.to_owned(),
            source: "workspace_seed".to_owned(),
        })
        .collect()
}

fn recharge_pack_catalog() -> Vec<PortalRechargePack> {
    recharge_pack_seeds()
        .into_iter()
        .map(|seed| PortalRechargePack {
            id: seed.id.to_owned(),
            label: seed.label.to_owned(),
            points: seed.points,
            price_label: format_catalog_price_label(seed.price_cents),
            note: seed.note.to_owned(),
            source: "workspace_seed".to_owned(),
        })
        .collect()
}

fn recharge_option_catalog() -> Vec<PortalRechargeOption> {
    recharge_option_seeds()
        .into_iter()
        .map(|seed| PortalRechargeOption {
            id: seed.id.to_owned(),
            label: seed.label.to_owned(),
            amount_cents: seed.amount_cents,
            amount_label: format_quote_price_label(seed.amount_cents),
            granted_units: seed.granted_units,
            effective_ratio_label: format_effective_ratio_label(
                seed.granted_units / seed.amount_cents.max(1),
            ),
            note: seed.note.to_owned(),
            recommended: seed.recommended,
            source: "workspace_seed".to_owned(),
        })
        .collect()
}

fn build_custom_recharge_policy() -> PortalCustomRechargePolicy {
    PortalCustomRechargePolicy {
        enabled: true,
        min_amount_cents: custom_recharge_min_amount_cents(),
        max_amount_cents: custom_recharge_max_amount_cents(),
        step_amount_cents: custom_recharge_step_amount_cents(),
        suggested_amount_cents: custom_recharge_suggested_amount_cents(),
        rules: custom_recharge_rule_seeds()
            .into_iter()
            .map(|rule| PortalCustomRechargeRule {
                id: rule.id.to_owned(),
                label: rule.label.to_owned(),
                min_amount_cents: rule.min_amount_cents,
                max_amount_cents: rule.max_amount_cents,
                units_per_cent: rule.units_per_cent,
                effective_ratio_label: format_effective_ratio_label(rule.units_per_cent),
                note: rule.note.to_owned(),
            })
            .collect(),
        source: "workspace_seed".to_owned(),
    }
}

async fn load_coupon_catalog(store: &dyn AdminStore) -> CommerceResult<Vec<PortalCommerceCoupon>> {
    Ok(load_coupon_definitions(store)
        .await?
        .into_iter()
        .map(|definition| definition.coupon)
        .collect())
}

async fn load_coupon_definitions(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<CommerceCouponDefinition>> {
    let mut definitions = seed_coupon_definitions()
        .into_iter()
        .map(|definition| (normalize_coupon_code(&definition.coupon.code), definition))
        .collect::<BTreeMap<_, _>>();
    let now_ms = current_time_ms()?;

    for coupon in list_active_coupons(store).await? {
        let code = normalize_coupon_code(&coupon.code);
        let prior = definitions.get(&code).cloned();
        let parsed_benefit = CommerceCouponBenefit {
            discount_percent: parse_discount_percent(&coupon.discount_label),
            bonus_units: 0,
        };
        let benefit = merge_coupon_benefit(parsed_benefit, prior.as_ref().map(|item| item.benefit));

        definitions.insert(
            code.clone(),
            CommerceCouponDefinition {
                coupon: PortalCommerceCoupon {
                    id: coupon.id,
                    code,
                    discount_label: coupon.discount_label,
                    audience: coupon.audience,
                    remaining: coupon.remaining,
                    active: coupon.active,
                    note: coupon.note,
                    expires_on: coupon.expires_on,
                    source: "live".to_owned(),
                    discount_percent: benefit.discount_percent,
                    bonus_units: benefit.bonus_units,
                },
                benefit,
            },
        );
    }

    for definition in load_marketing_coupon_definitions(store, now_ms).await? {
        definitions.insert(normalize_coupon_code(&definition.coupon.code), definition);
    }

    Ok(definitions.into_values().collect())
}

async fn find_resolved_coupon_definition(
    store: &dyn AdminStore,
    code: &str,
) -> CommerceResult<ResolvedCouponDefinition> {
    let normalized = normalize_coupon_code(code);
    let now_ms = current_time_ms()?;
    if let Some(context) =
        load_marketing_coupon_context_by_value(store, &normalized, now_ms).await?
    {
        if !coupon_context_is_catalog_visible(&context, now_ms) {
            return Err(CommerceError::NotFound(format!(
                "coupon {normalized} not found"
            )));
        }
        return Ok(ResolvedCouponDefinition {
            definition: marketing_context_to_definition(&context, now_ms),
            marketing: Some(context),
        });
    }

    load_coupon_definitions(store)
        .await?
        .into_iter()
        .find(|definition| definition.coupon.code == normalized)
        .map(|definition| ResolvedCouponDefinition {
            definition,
            marketing: None,
        })
        .ok_or_else(|| CommerceError::NotFound(format!("coupon {normalized} not found")))
}

async fn load_optional_applied_coupon(
    store: &dyn AdminStore,
    coupon_code: Option<&str>,
    target_kind: &str,
    order_amount_cents: u64,
) -> CommerceResult<Option<ResolvedCouponDefinition>> {
    match coupon_code.map(str::trim).filter(|value| !value.is_empty()) {
        Some(code) => {
            let resolved = find_resolved_coupon_definition(store, code).await?;
            if let Some(context) = resolved.marketing.as_ref() {
                let reserve_amount_minor = compute_coupon_reserve_amount_minor(
                    order_amount_cents,
                    &context.template.benefit,
                );
                let decision = validate_marketing_coupon_context(
                    context,
                    target_kind,
                    current_time_ms()?,
                    order_amount_cents,
                    reserve_amount_minor,
                );
                if !decision.eligible {
                    return Err(CommerceError::InvalidInput(format!(
                        "coupon {} is not eligible: {}",
                        resolved.definition.coupon.code,
                        decision
                            .rejection_reason
                            .unwrap_or_else(|| "validation_failed".to_owned())
                    )));
                }
            }
            Ok(Some(resolved))
        }
        None => Ok(None),
    }
}

async fn load_marketing_coupon_definitions(
    store: &dyn AdminStore,
    now_ms: u64,
) -> CommerceResult<Vec<CommerceCouponDefinition>> {
    let mut definitions = Vec::new();
    for code_record in store
        .list_coupon_code_records()
        .await
        .map_err(CommerceError::from)?
    {
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, code_record, now_ms).await?
        {
            if coupon_context_is_catalog_visible(&context, now_ms) {
                definitions.push(marketing_context_to_definition(&context, now_ms));
            }
        }
    }
    Ok(definitions)
}

async fn load_marketing_coupon_context_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> CommerceResult<Option<MarketingCouponContext>> {
    let normalized = normalize_coupon_code(code);
    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized)
        .await
        .map_err(CommerceError::from)?
    {
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, code_record, now_ms).await?
        {
            return Ok(Some(context));
        }
    }

    load_compatibility_marketing_coupon_context(store, &normalized).await
}

async fn load_marketing_coupon_context_from_code_record(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    now_ms: u64,
) -> CommerceResult<Option<MarketingCouponContext>> {
    let Some(template) = store
        .find_coupon_template_record(&code.coupon_template_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(None);
    };

    let Some(campaign) = select_effective_marketing_campaign(
        store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .map_err(CommerceError::from)?,
        now_ms,
    ) else {
        return Ok(None);
    };

    let Some(budget) = select_campaign_budget_record(
        store
            .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
            .await
            .map_err(CommerceError::from)?,
    ) else {
        return Ok(None);
    };

    Ok(Some(MarketingCouponContext {
        template,
        campaign,
        budget,
        code,
        source: "marketing".to_owned(),
    }))
}

async fn load_compatibility_marketing_coupon_context<T>(
    store: &T,
    code: &str,
) -> CommerceResult<Option<MarketingCouponContext>>
where
    T: AdminStore + ?Sized,
{
    Ok(store
        .list_active_coupons()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|coupon| normalize_coupon_code(&coupon.code) == code)
        .map(|coupon| {
            let (template, campaign, budget, code_record) = project_legacy_coupon_campaign(&coupon);
            MarketingCouponContext {
                template,
                campaign,
                budget,
                code: code_record,
                source: "live".to_owned(),
            }
        }))
}

fn select_effective_marketing_campaign(
    campaigns: Vec<MarketingCampaignRecord>,
    now_ms: u64,
) -> Option<MarketingCampaignRecord> {
    campaigns
        .into_iter()
        .filter(|campaign| campaign.is_effective_at(now_ms))
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
        })
}

fn select_campaign_budget_record(
    budgets: Vec<CampaignBudgetRecord>,
) -> Option<CampaignBudgetRecord> {
    budgets.into_iter().max_by(|left, right| {
        left.updated_at_ms
            .cmp(&right.updated_at_ms)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    })
}

fn coupon_context_is_catalog_visible(context: &MarketingCouponContext, now_ms: u64) -> bool {
    context.template.status == CouponTemplateStatus::Active
        && context.campaign.is_effective_at(now_ms)
        && context.budget.available_budget_minor() > 0
        && coupon_code_is_available_for_template(&context.template, &context.code, now_ms)
}

fn coupon_code_is_available_for_template(
    template: &CouponTemplateRecord,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> bool {
    match template.distribution_kind {
        CouponDistributionKind::SharedCode => {
            !matches!(
                code.status,
                CouponCodeStatus::Disabled | CouponCodeStatus::Expired
            ) && code.expires_at_ms.is_none_or(|value| now_ms <= value)
        }
        CouponDistributionKind::UniqueCode | CouponDistributionKind::AutoClaim => {
            code.is_redeemable_at(now_ms)
        }
    }
}

fn marketing_context_to_definition(
    context: &MarketingCouponContext,
    now_ms: u64,
) -> CommerceCouponDefinition {
    let benefit = CommerceCouponBenefit {
        discount_percent: context.template.benefit.discount_percent,
        bonus_units: context.template.benefit.grant_units.unwrap_or(0),
    };
    let remaining = match context.template.distribution_kind {
        CouponDistributionKind::SharedCode => context.budget.available_budget_minor(),
        CouponDistributionKind::UniqueCode | CouponDistributionKind::AutoClaim => {
            if coupon_code_is_available_for_template(&context.template, &context.code, now_ms) {
                1
            } else {
                0
            }
        }
    };
    CommerceCouponDefinition {
        coupon: PortalCommerceCoupon {
            id: context.code.coupon_code_id.clone(),
            code: normalize_coupon_code(&context.code.code_value),
            discount_label: format_marketing_discount_label(&context.template.benefit),
            audience: format!("{:?}", context.template.restriction.subject_scope)
                .to_ascii_lowercase(),
            remaining,
            active: coupon_context_is_catalog_visible(context, now_ms),
            note: if context.template.display_name.trim().is_empty() {
                context.campaign.display_name.clone()
            } else {
                context.template.display_name.clone()
            },
            expires_on: format_marketing_expires_on(context),
            source: context.source.clone(),
            discount_percent: benefit.discount_percent,
            bonus_units: benefit.bonus_units,
        },
        benefit,
    }
}

fn format_marketing_discount_label(benefit: &CouponBenefitSpec) -> String {
    match benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => benefit
            .discount_percent
            .map(|percent| format!("{percent}% off"))
            .unwrap_or_else(|| "percentage off".to_owned()),
        MarketingBenefitKind::FixedAmountOff => benefit
            .discount_amount_minor
            .map(format_quote_price_label)
            .map(|label| format!("{label} off"))
            .unwrap_or_else(|| "fixed amount off".to_owned()),
        MarketingBenefitKind::GrantUnits => benefit
            .grant_units
            .map(|units| format!("+{} bonus units", format_integer_with_commas(units)))
            .unwrap_or_else(|| "bonus units".to_owned()),
    }
}

fn format_marketing_expires_on(context: &MarketingCouponContext) -> String {
    context
        .code
        .expires_at_ms
        .or(context.campaign.end_at_ms)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "rolling".to_owned())
}

fn compute_coupon_subsidy_minor(list_price_cents: u64, benefit: &CouponBenefitSpec) -> u64 {
    let subsidy = match benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => benefit
            .discount_percent
            .map(|percent| list_price_cents.saturating_mul(percent as u64) / 100)
            .unwrap_or(0),
        MarketingBenefitKind::FixedAmountOff => benefit.discount_amount_minor.unwrap_or(0),
        MarketingBenefitKind::GrantUnits => 0,
    };

    subsidy
        .min(benefit.max_discount_minor.unwrap_or(u64::MAX))
        .min(list_price_cents)
}

fn compute_coupon_reserve_amount_minor(list_price_cents: u64, benefit: &CouponBenefitSpec) -> u64 {
    let subsidy_amount_minor = compute_coupon_subsidy_minor(list_price_cents, benefit);
    if subsidy_amount_minor > 0 {
        subsidy_amount_minor
    } else if matches!(benefit.benefit_kind, MarketingBenefitKind::GrantUnits) {
        1
    } else {
        0
    }
}

fn validate_marketing_coupon_context(
    context: &MarketingCouponContext,
    target_kind: &str,
    now_ms: u64,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
) -> CouponValidationDecision {
    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        order_amount_minor,
        reserve_amount_minor,
    );
    if !decision.eligible {
        return decision;
    }

    if marketing_coupon_target_kind_allowed(&context.template, target_kind) {
        decision
    } else {
        CouponValidationDecision::rejected("target_kind_not_eligible")
    }
}

fn marketing_coupon_target_kind_allowed(
    template: &CouponTemplateRecord,
    target_kind: &str,
) -> bool {
    template.restriction.eligible_target_kinds.is_empty()
        || template
            .restriction
            .eligible_target_kinds
            .iter()
            .any(|eligible| eligible == target_kind)
}

async fn reserve_order_coupon_if_needed<T>(
    store: &T,
    order_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
    resolved_coupon: Option<&ResolvedCouponDefinition>,
) -> CommerceResult<Option<ReservedMarketingCouponState>>
where
    T: AdminStore + ?Sized,
{
    let Some(resolved_coupon) = resolved_coupon else {
        return Ok(None);
    };
    let Some(context) = resolved_coupon.marketing.as_ref() else {
        return Ok(None);
    };

    let now_ms = current_time_ms()?;
    let reserve_amount_minor =
        compute_coupon_reserve_amount_minor(quote.list_price_cents, &context.template.benefit);
    let decision = validate_marketing_coupon_context(
        context,
        quote.target_kind.as_str(),
        now_ms,
        quote.list_price_cents,
        reserve_amount_minor,
    );
    if !decision.eligible {
        return Err(CommerceError::InvalidInput(format!(
            "coupon {} is not eligible: {}",
            resolved_coupon.definition.coupon.code,
            decision
                .rejection_reason
                .unwrap_or_else(|| "validation_failed".to_owned())
        )));
    }

    let coupon_reservation_id = format!("coupon_reservation_{order_id}");
    let (reserved_code, reservation) = reserve_coupon_redemption(
        &context.code,
        coupon_reservation_id.clone(),
        MarketingSubjectScope::Project,
        project_id.to_owned(),
        decision.reservable_budget_minor,
        now_ms,
        DEFAULT_COUPON_RESERVATION_TTL_MS,
    )
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    store
        .reserve_coupon_redemption_atomic(&AtomicCouponReservationCommand {
            template_to_persist: (context.source != "marketing")
                .then_some(context.template.clone()),
            campaign_to_persist: (context.source != "marketing")
                .then_some(context.campaign.clone()),
            expected_budget: context.budget.clone(),
            next_budget: reserve_campaign_budget(
                &context.budget,
                decision.reservable_budget_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_reservation(
                &context.template,
                &context.code,
                &reserved_code,
                now_ms,
            ),
            reservation,
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;

    Ok(Some(ReservedMarketingCouponState {
        coupon_reservation_id,
        marketing_campaign_id: context.campaign.marketing_campaign_id.clone(),
        subsidy_amount_minor: compute_coupon_subsidy_minor(
            quote.list_price_cents,
            &context.template.benefit,
        ),
    }))
}

async fn confirm_order_coupon_if_needed<T>(
    store: &T,
    order: &mut CommerceOrderRecord,
    payment_event_id: Option<&str>,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let Some(coupon_reservation_id) = order.coupon_reservation_id.clone() else {
        return Ok(());
    };
    if order.coupon_redemption_id.is_some() {
        return Ok(());
    }

    let coupon_redemption_id = format!("coupon_redemption_{}", order.order_id);
    let reservation = store
        .find_coupon_reservation_record(&coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "coupon reservation {} not found for order {}",
                coupon_reservation_id, order.order_id
            ))
        })?;
    let now_ms = current_time_ms()?;
    let context = load_order_marketing_context(store, order, now_ms).await?;
    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        coupon_redemption_id.clone(),
        context.code.coupon_code_id.clone(),
        context.template.coupon_template_id.clone(),
        order.subsidy_amount_minor,
        Some(order.order_id.clone()),
        payment_event_id.map(str::to_owned),
        now_ms,
    )
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    store
        .confirm_coupon_redemption_atomic(&AtomicCouponConfirmationCommand {
            expected_budget: context.budget.clone(),
            next_budget: confirm_campaign_budget(
                &context.budget,
                reservation.budget_reserved_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_confirmation(&context.template, &context.code, now_ms),
            expected_reservation: reservation,
            next_reservation: confirmed_reservation,
            redemption,
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    order.coupon_redemption_id = Some(coupon_redemption_id);
    Ok(())
}

async fn release_order_coupon_reservation_if_needed<T>(
    store: &T,
    order: &mut CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    if order.coupon_redemption_id.is_some() {
        return Ok(());
    }
    let Some(coupon_reservation_id) = order.coupon_reservation_id.clone() else {
        return Ok(());
    };

    let Some(reservation) = store
        .find_coupon_reservation_record(&coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(());
    };
    if reservation.reservation_status != CouponReservationStatus::Reserved {
        return Ok(());
    }

    let now_ms = current_time_ms()?;
    let context = load_order_marketing_context(store, order, now_ms).await?;
    store
        .release_coupon_reservation_atomic(&AtomicCouponReleaseCommand {
            expected_budget: context.budget.clone(),
            next_budget: release_campaign_budget(
                &context.budget,
                reservation.budget_reserved_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_release(&context.template, &context.code, now_ms),
            expected_reservation: reservation.clone(),
            next_reservation: reservation
                .with_status(CouponReservationStatus::Released)
                .with_updated_at_ms(now_ms),
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    Ok(())
}

async fn rollback_order_coupon_redemption_if_needed<T>(
    store: &T,
    order: &mut CommerceOrderRecord,
    rollback_type: CouponRollbackType,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let Some(coupon_redemption_id) = order.coupon_redemption_id.clone() else {
        return Ok(());
    };
    let Some(redemption) = store
        .find_coupon_redemption_record(&coupon_redemption_id)
        .await
        .map_err(CommerceError::from)?
    else {
        return Ok(());
    };
    if matches!(
        redemption.redemption_status,
        CouponRedemptionStatus::RolledBack | CouponRedemptionStatus::PartiallyRolledBack
    ) {
        return Ok(());
    }

    let reservation = store
        .find_coupon_reservation_record(&redemption.coupon_reservation_id)
        .await
        .map_err(CommerceError::from)?;
    let restored_budget_minor = reservation
        .as_ref()
        .map(|item| item.budget_reserved_minor)
        .unwrap_or(redemption.subsidy_amount_minor);
    let now_ms = current_time_ms()?;
    let context = load_order_marketing_context(store, order, now_ms).await?;
    let rollback_id = format!(
        "coupon_rollback_{}_{}",
        order.order_id,
        match rollback_type {
            CouponRollbackType::Cancel => "cancel",
            CouponRollbackType::Refund => "refund",
            CouponRollbackType::PartialRefund => "partial_refund",
            CouponRollbackType::Manual => "manual",
        }
    );
    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        rollback_id,
        rollback_type,
        restored_budget_minor,
        if coupon_code_is_exclusive(&context.template) {
            1
        } else {
            0
        },
        now_ms,
    )
    .map_err(|error| CommerceError::Conflict(error.to_string()))?;
    store
        .rollback_coupon_redemption_atomic(&AtomicCouponRollbackCommand {
            expected_budget: context.budget.clone(),
            next_budget: rollback_campaign_budget(&context.budget, restored_budget_minor, now_ms),
            expected_code: context.code.clone(),
            next_code: code_after_rollback(&context.template, &context.code, now_ms),
            expected_redemption: redemption,
            next_redemption: rolled_back_redemption,
            rollback,
        })
        .await
        .map_err(commerce_atomic_coupon_error)?;
    Ok(())
}

async fn load_order_marketing_context<T>(
    store: &T,
    order: &CommerceOrderRecord,
    now_ms: u64,
) -> CommerceResult<MarketingCouponContext>
where
    T: AdminStore + ?Sized,
{
    let code_value = order
        .applied_coupon_code
        .as_deref()
        .or_else(|| (order.target_kind == "coupon_redemption").then_some(order.target_id.as_str()))
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "order {} does not reference a marketing coupon",
                order.order_id
            ))
        })?;
    let normalized_code = normalize_coupon_code(code_value);
    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized_code)
        .await
        .map_err(CommerceError::from)?
    {
        let template = store
            .find_coupon_template_record(&code_record.coupon_template_id)
            .await
            .map_err(CommerceError::from)?
            .ok_or_else(|| {
                CommerceError::Conflict(format!(
                    "coupon template {} not found for order {}",
                    code_record.coupon_template_id, order.order_id
                ))
            })?;
        let campaigns = store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .map_err(CommerceError::from)?;
        let campaign = order
            .marketing_campaign_id
            .as_deref()
            .and_then(|marketing_campaign_id| {
                campaigns
                    .iter()
                    .find(|record| record.marketing_campaign_id == marketing_campaign_id)
                    .cloned()
            })
            .or_else(|| select_effective_marketing_campaign(campaigns.clone(), now_ms))
            .or_else(|| {
                campaigns.into_iter().max_by(|left, right| {
                    left.updated_at_ms
                        .cmp(&right.updated_at_ms)
                        .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
                })
            })
            .ok_or_else(|| {
                CommerceError::Conflict(format!(
                    "marketing campaign not found for order {}",
                    order.order_id
                ))
            })?;
        let budget = select_campaign_budget_record(
            store
                .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
                .await
                .map_err(CommerceError::from)?,
        )
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "campaign budget not found for order {}",
                order.order_id
            ))
        })?;
        return Ok(MarketingCouponContext {
            template,
            campaign,
            budget,
            code: code_record,
            source: "marketing".to_owned(),
        });
    }

    load_compatibility_marketing_coupon_context(store, &normalized_code)
        .await?
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "coupon {} no longer resolves to a marketing context",
                code_value
            ))
        })
}

fn coupon_code_is_exclusive(template: &CouponTemplateRecord) -> bool {
    !matches!(
        template.distribution_kind,
        CouponDistributionKind::SharedCode
    )
}

fn code_after_reservation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    reserved_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        reserved_code.clone()
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_confirmation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        original_code
            .clone()
            .with_status(CouponCodeStatus::Redeemed)
            .with_updated_at_ms(now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_release(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        restore_coupon_code_availability(original_code, now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_rollback(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        restore_coupon_code_availability(original_code, now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn restore_coupon_code_availability(
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    let next_status = if original_code
        .expires_at_ms
        .is_some_and(|value| now_ms > value)
    {
        CouponCodeStatus::Expired
    } else {
        CouponCodeStatus::Available
    };
    original_code
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms)
}

fn reserve_campaign_budget(
    budget: &CampaignBudgetRecord,
    reserved_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_add(reserved_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            budget.consumed_budget_minor,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn release_campaign_budget(
    budget: &CampaignBudgetRecord,
    released_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_sub(released_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            budget.consumed_budget_minor,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn confirm_campaign_budget(
    budget: &CampaignBudgetRecord,
    consumed_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_sub(consumed_amount_minor);
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_add(consumed_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn rollback_campaign_budget(
    budget: &CampaignBudgetRecord,
    restored_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_sub(restored_amount_minor);
    budget
        .clone()
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            budget.reserved_budget_minor,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn campaign_budget_status_after_mutation(
    total_budget_minor: u64,
    reserved_budget_minor: u64,
    consumed_budget_minor: u64,
    prior_status: CampaignBudgetStatus,
) -> CampaignBudgetStatus {
    if matches!(
        prior_status,
        CampaignBudgetStatus::Closed | CampaignBudgetStatus::Draft
    ) {
        return prior_status;
    }

    let available_budget_minor = total_budget_minor
        .saturating_sub(reserved_budget_minor)
        .saturating_sub(consumed_budget_minor);
    if available_budget_minor == 0 {
        CampaignBudgetStatus::Exhausted
    } else {
        CampaignBudgetStatus::Active
    }
}

async fn fulfill_order_on_create<T>(
    store: &T,
    normalized_user_id: &str,
    normalized_project_id: &str,
    quote: &PortalCommerceQuote,
    order: &mut CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    apply_quote_to_project_quota(store, normalized_project_id, quote).await?;
    activate_project_membership_if_needed(store, normalized_user_id, normalized_project_id, quote)
        .await?;
    confirm_order_coupon_if_needed(store, order, None).await
}

async fn apply_quote_to_project_quota<T>(
    store: &T,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    let target_units = quote.granted_units.saturating_add(quote.bonus_units);
    if target_units == 0 {
        return Ok(());
    }

    let effective_policy = load_effective_quota_policy(store, project_id).await?;
    let current_limit = effective_policy
        .as_ref()
        .map(|policy| policy.max_units)
        .unwrap_or(0);
    let policy_id = effective_policy
        .as_ref()
        .map(|policy| policy.policy_id.clone())
        .unwrap_or_else(|| format!("portal_commerce_{project_id}"));
    let next_limit = match quote.target_kind.as_str() {
        "subscription_plan" => current_limit.max(target_units),
        "recharge_pack" | "custom_recharge" | "coupon_redemption" => {
            current_limit.saturating_add(target_units)
        }
        _ => current_limit,
    };

    if next_limit == current_limit {
        return Ok(());
    }

    let next_policy = QuotaPolicy::new(policy_id, project_id.to_owned(), next_limit);
    store
        .insert_quota_policy(&next_policy)
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

async fn reverse_order_quota_effect<T>(
    store: &T,
    project_id: &str,
    order: &CommerceOrderRecord,
) -> CommerceResult<()>
where
    T: AdminStore + CommerceQuotaStore + ?Sized,
{
    let target_units = order.granted_units.saturating_add(order.bonus_units);
    if target_units == 0 {
        return Ok(());
    }

    if let Some(membership) = store
        .find_project_membership(project_id)
        .await
        .map_err(CommerceError::from)?
    {
        if membership.updated_at_ms > order.created_at_ms {
            return Err(CommerceError::Conflict(format!(
                "order {} cannot be refunded safely after subscription changes",
                order.order_id
            )));
        }
    }

    let effective_policy = load_effective_quota_policy(store, project_id)
        .await?
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "order {} cannot be refunded because no active quota policy exists",
                order.order_id
            ))
        })?;

    if effective_policy.max_units < target_units {
        return Err(CommerceError::Conflict(format!(
            "order {} cannot be refunded because quota baseline drifted",
            order.order_id
        )));
    }

    let used_units = store
        .list_ledger_entries_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .map(|entry| entry.units)
        .sum::<u64>();
    let remaining_units = effective_policy.max_units.saturating_sub(used_units);
    if remaining_units < target_units {
        return Err(CommerceError::Conflict(format!(
            "order {} cannot be refunded because recharge headroom has already been consumed",
            order.order_id
        )));
    }

    let next_policy = QuotaPolicy::new(
        effective_policy.policy_id,
        project_id.to_owned(),
        effective_policy.max_units.saturating_sub(target_units),
    );
    store
        .insert_quota_policy(&next_policy)
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

async fn load_effective_quota_policy<T>(
    store: &T,
    project_id: &str,
) -> CommerceResult<Option<QuotaPolicy>>
where
    T: CommerceQuotaStore + ?Sized,
{
    Ok(store
        .list_quota_policies_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|policy| policy.enabled)
        .min_by(|left, right| {
            left.max_units
                .cmp(&right.max_units)
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        }))
}

async fn activate_project_membership_if_needed<T>(
    store: &T,
    user_id: &str,
    project_id: &str,
    quote: &PortalCommerceQuote,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    if quote.target_kind != "subscription_plan" {
        return Ok(());
    }

    let plan = subscription_plan_seeds()
        .into_iter()
        .find(|candidate| candidate.id.eq_ignore_ascii_case(&quote.target_id))
        .ok_or_else(|| CommerceError::NotFound("subscription plan not found".to_owned()))?;
    let activated_at_ms = current_time_ms()?;

    store
        .upsert_project_membership(&ProjectMembershipRecord::new(
            generate_entity_id("membership")?,
            project_id,
            user_id,
            plan.id,
            plan.name,
            quote.payable_price_cents,
            quote.payable_price_label.clone(),
            plan.cadence,
            plan.included_units,
            "active",
            quote.source.clone(),
            activated_at_ms,
            activated_at_ms,
        ))
        .await
        .map_err(CommerceError::from)?;
    Ok(())
}

fn should_fulfill_on_order_create(quote: &PortalCommerceQuote) -> bool {
    quote.target_kind == "coupon_redemption"
}

fn supports_safe_order_refund(order: &CommerceOrderRecord) -> bool {
    matches!(
        order.target_kind.as_str(),
        "recharge_pack" | "custom_recharge"
    ) && order.payable_price_cents > 0
}

fn initial_order_status(quote: &PortalCommerceQuote) -> &'static str {
    if should_fulfill_on_order_create(quote) {
        "fulfilled"
    } else {
        "pending_payment"
    }
}

async fn load_project_commerce_order<T>(
    store: &T,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord>
where
    T: AdminStore + ?Sized,
{
    let order = store
        .list_commerce_orders_for_project(project_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|candidate| candidate.order_id == order_id)
        .ok_or_else(|| CommerceError::NotFound(format!("order {order_id} not found")))?;

    if order.user_id != user_id {
        return Err(CommerceError::NotFound(format!(
            "order {order_id} not found"
        )));
    }

    Ok(order)
}

fn resolve_checkout_method_for_payment_event(
    order: &CommerceOrderRecord,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<Option<PortalCommerceCheckoutSessionMethod>> {
    let Some(checkout_method_id) = request
        .checkout_method_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    build_supported_checkout_methods(order)
        .into_iter()
        .find(|candidate| candidate.id == checkout_method_id)
        .map(Some)
        .ok_or_else(|| {
            CommerceError::InvalidInput(format!(
                "checkout_method_id {checkout_method_id} is not available for order {}",
                order.order_id
            ))
        })
}

fn resolve_commerce_payment_event_identity(
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
    checkout_method: Option<PortalCommerceCheckoutSessionMethod>,
) -> CommerceResult<(String, Option<String>, String)> {
    let event_type = request.event_type.trim();
    let provider = request
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_commerce_payment_provider)
        .transpose()?;
    let provider_event_id = request
        .provider_event_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let checkout_method_provider = checkout_method
        .as_ref()
        .map(|method| method.provider.clone());

    if checkout_method
        .as_ref()
        .is_some_and(|method| method.supports_webhook)
        && provider_event_id.is_none()
    {
        return Err(CommerceError::InvalidInput(
            "provider_event_id is required for webhook-backed checkout methods".to_owned(),
        ));
    }

    let provider = match (provider, checkout_method_provider) {
        (Some(provider), Some(method_provider)) => {
            if provider != method_provider {
                return Err(CommerceError::InvalidInput(format!(
                    "checkout_method_id belongs to provider {method_provider}, but request provider is {provider}"
                )));
            }
            Some(provider)
        }
        (Some(provider), None) => Some(provider),
        (None, Some(method_provider)) => Some(method_provider),
        (None, None) => None,
    };

    if provider.as_ref().is_some_and(|provider| {
        provider != COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB
            && provider != COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED
    }) && provider_event_id.is_none()
    {
        return Err(CommerceError::InvalidInput(
            "provider_event_id is required for provider-backed payment events".to_owned(),
        ));
    }

    match (provider, provider_event_id) {
        (Some(provider), Some(provider_event_id)) => Ok((
            provider.clone(),
            Some(provider_event_id.clone()),
            format!("{provider}:{provider_event_id}"),
        )),
        (Some(provider), None) => Ok((
            provider.clone(),
            None,
            format!("{provider}:{order_id}:{event_type}"),
        )),
        (None, Some(_)) => Err(CommerceError::InvalidInput(
            "provider is required when provider_event_id is set".to_owned(),
        )),
        (None, None) => Ok((
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB.to_owned(),
            None,
            format!("{COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB}:{order_id}:{event_type}"),
        )),
    }
}

fn normalize_commerce_payment_provider(value: &str) -> CommerceResult<String> {
    let provider = match value.trim().to_ascii_lowercase().as_str() {
        "manual" | "manual_lab" | "operator_settlement" => COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
        "stripe" | "stripe_checkout" | "stripe_hosted" => COMMERCE_PAYMENT_PROVIDER_STRIPE,
        "alipay" | "alipay_qr" | "alipay_scan" => COMMERCE_PAYMENT_PROVIDER_ALIPAY,
        "wechat" | "wechat_pay" | "wechatpay" | "wxpay" | "wechat_qr" | "wechat_pay_qr" => {
            COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY
        }
        _ => {
            return Err(CommerceError::InvalidInput(format!(
                "unsupported commerce payment provider: {value}"
            )));
        }
    };

    Ok(provider.to_owned())
}

async fn ensure_refund_provider_matches_order_settlement<T>(
    store: &T,
    order: &CommerceOrderRecord,
    refund_provider: &str,
) -> CommerceResult<()>
where
    T: AdminStore + ?Sized,
{
    let settled_provider = resolve_processed_settlement_provider_for_order(store, order).await?;
    if refund_provider != settled_provider {
        return Err(CommerceError::Conflict(format!(
            "refund provider {refund_provider} does not match settled provider {settled_provider} for order {}",
            order.order_id
        )));
    }

    Ok(())
}

async fn resolve_processed_settlement_provider_for_order<T>(
    store: &T,
    order: &CommerceOrderRecord,
) -> CommerceResult<String>
where
    T: AdminStore + ?Sized,
{
    let settled_provider = store
        .list_commerce_payment_events_for_order(&order.order_id)
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .filter(|event| {
            event.event_type == "settled"
                && matches!(
                    event.processing_status,
                    CommercePaymentEventProcessingStatus::Processed
                )
                && event.order_status_after.as_deref() == Some("fulfilled")
        })
        .max_by_key(|event| event.processed_at_ms.unwrap_or(event.received_at_ms))
        .map(|event| event.provider)
        .unwrap_or_else(|| COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB.to_owned());

    Ok(settled_provider)
}

fn build_commerce_payment_event_record(
    order: &CommerceOrderRecord,
    request: &PortalCommercePaymentEventRequest,
    provider: String,
    provider_event_id: Option<String>,
    dedupe_key: String,
    received_at_ms: u64,
    existing_payment_event_id: Option<&str>,
) -> CommerceResult<CommercePaymentEventRecord> {
    let payment_event_id = match existing_payment_event_id {
        Some(payment_event_id) => payment_event_id.to_owned(),
        None => generate_entity_id("commerce_payment_event")?,
    };
    let payload_json =
        serde_json::to_string(request).map_err(|error| CommerceError::Storage(error.into()))?;

    Ok(CommercePaymentEventRecord::new(
        payment_event_id,
        order.order_id.clone(),
        order.project_id.clone(),
        order.user_id.clone(),
        provider,
        dedupe_key,
        request.event_type.trim(),
        payload_json,
        received_at_ms,
    )
    .with_provider_event_id(provider_event_id))
}

fn finalize_commerce_payment_event(
    event: CommercePaymentEventRecord,
    processing_status: CommercePaymentEventProcessingStatus,
    processing_message: Option<String>,
    order_status_after: Option<String>,
    processed_at_ms: Option<u64>,
) -> CommercePaymentEventRecord {
    event
        .with_processing_status(processing_status)
        .with_processing_message(processing_message)
        .with_order_status_after(order_status_after)
        .with_processed_at_ms(processed_at_ms)
}

fn commerce_payment_event_status_for_error(
    error: &CommerceError,
) -> CommercePaymentEventProcessingStatus {
    match error {
        CommerceError::Storage(_) => CommercePaymentEventProcessingStatus::Failed,
        CommerceError::InvalidInput(_)
        | CommerceError::NotFound(_)
        | CommerceError::Conflict(_) => CommercePaymentEventProcessingStatus::Rejected,
    }
}

async fn persist_commerce_payment_event(
    store: &dyn AdminStore,
    payment_event: CommercePaymentEventRecord,
) -> CommerceResult<CommercePaymentEventRecord> {
    match store.upsert_commerce_payment_event(&payment_event).await {
        Ok(event) => Ok(event),
        Err(error) => {
            if let Some(existing_event) = store
                .find_commerce_payment_event_by_dedupe_key(&payment_event.dedupe_key)
                .await
                .map_err(CommerceError::from)?
            {
                if existing_event.order_id != payment_event.order_id {
                    return Err(CommerceError::Conflict(format!(
                        "payment event {} already belongs to order {}",
                        payment_event.dedupe_key, existing_event.order_id
                    )));
                }
            }
            Err(CommerceError::Storage(error))
        }
    }
}

async fn fail_portal_commerce_order(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let mut order = load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;

    match order.status.as_str() {
        "failed" => return Ok(order),
        "pending_payment" => {}
        other => {
            return Err(CommerceError::Conflict(format!(
                "order {normalized_order_id} cannot be marked failed from status {other}"
            )))
        }
    }

    release_order_coupon_reservation_if_needed(store, &mut order).await?;
    order.status = "failed".to_owned();
    order.updated_at_ms = current_time_ms()?;
    store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)
}

async fn load_order_settlement_quote(
    _store: &dyn AdminStore,
    order: &CommerceOrderRecord,
) -> CommerceResult<PortalCommerceQuote> {
    Ok(PortalCommerceQuote {
        target_kind: order.target_kind.clone(),
        target_id: order.target_id.clone(),
        target_name: order.target_name.clone(),
        list_price_cents: order.list_price_cents,
        payable_price_cents: order.payable_price_cents,
        list_price_label: order.list_price_label.clone(),
        payable_price_label: order.payable_price_label.clone(),
        granted_units: order.granted_units,
        bonus_units: order.bonus_units,
        amount_cents: if order.target_kind == "custom_recharge" {
            Some(order.list_price_cents)
        } else {
            None
        },
        projected_remaining_units: None,
        applied_coupon: order
            .applied_coupon_code
            .as_ref()
            .map(|code| PortalAppliedCoupon {
                code: code.clone(),
                discount_label: code.clone(),
                source: "order_snapshot".to_owned(),
                discount_percent: None,
                bonus_units: order.bonus_units,
            }),
        pricing_rule_label: if order.target_kind == "custom_recharge" {
            Some("Tiered custom recharge".to_owned())
        } else {
            None
        },
        effective_ratio_label: if order.target_kind == "custom_recharge"
            && order.list_price_cents > 0
        {
            Some(format_effective_ratio_label(
                order.granted_units / order.list_price_cents.max(1),
            ))
        } else {
            None
        },
        source: order.source.clone(),
    })
}

fn build_checkout_session(order: &CommerceOrderRecord) -> PortalCommerceCheckoutSession {
    let reference = format!("PAY-{}", normalize_payment_reference(&order.order_id));
    let guidance = match (order.target_kind.as_str(), order.status.as_str()) {
        ("subscription_plan", "pending_payment") => {
            "Settle this checkout to activate the workspace membership and included monthly units."
        }
        ("recharge_pack", "pending_payment") => {
            "Settle this checkout to apply the recharge pack and restore workspace quota headroom."
        }
        ("custom_recharge", "pending_payment") => {
            "Settle this checkout to apply the custom recharge amount and restore workspace quota headroom."
        }
        ("coupon_redemption", "fulfilled") => {
            "This order required no external payment and was fulfilled immediately at redemption time."
        }
        (_, "fulfilled") => {
            "This checkout session is closed because the order has already been settled."
        }
        (_, "canceled") => {
            "This checkout session is closed because the order was canceled before settlement."
        }
        (_, "failed") => "This checkout session is closed because the payment flow failed.",
        (_, "refunded") => {
            "This checkout session is closed because the order was refunded and quota side effects were rolled back."
        }
        _ => "This checkout session describes how the current order can move through the payment rail.",
    };

    let (session_status, provider, mode, methods) = match order.status.as_str() {
        "pending_payment" => (
            "open",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
            build_supported_checkout_methods(order),
        ),
        "fulfilled"
            if order.target_kind == "coupon_redemption" || order.payable_price_cents == 0 =>
        {
            (
                "not_required",
                COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED,
                "instant_fulfillment",
                Vec::new(),
            )
        }
        "fulfilled" => ("settled", COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, "closed", Vec::new()),
        "canceled" => ("canceled", COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, "closed", Vec::new()),
        "failed" => ("failed", COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, "closed", Vec::new()),
        "refunded" => ("refunded", COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, "closed", Vec::new()),
        _ => ("closed", COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, "closed", Vec::new()),
    };

    PortalCommerceCheckoutSession {
        order_id: order.order_id.clone(),
        order_status: order.status.clone(),
        session_status: session_status.to_owned(),
        provider: provider.to_owned(),
        mode: mode.to_owned(),
        reference,
        payable_price_label: order.payable_price_label.clone(),
        guidance: guidance.to_owned(),
        methods,
    }
}

fn build_supported_checkout_methods(
    order: &CommerceOrderRecord,
) -> Vec<PortalCommerceCheckoutSessionMethod> {
    let mut methods = vec![
        build_checkout_session_method(
            order,
            "manual_settlement",
            "Manual settlement",
            "Use the portal settlement action in desktop or lab mode to finalize the order.",
            "settle_order",
            "available",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
            "operator_action",
            "MANUAL",
            None,
            "manual",
            true,
            false,
            false,
            false,
        ),
        build_checkout_session_method(
            order,
            "cancel_order",
            "Cancel checkout",
            "Close the pending order without applying quota or membership side effects.",
            "cancel_order",
            "available",
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
            COMMERCE_PAYMENT_CHANNEL_OPERATOR_SETTLEMENT,
            "operator_action",
            "CANCEL",
            None,
            "manual",
            false,
            false,
            false,
            false,
        ),
    ];

    if order.payable_price_cents > 0 {
        methods.extend([
            build_checkout_session_method(
                order,
                "stripe_checkout",
                "Stripe checkout",
                "Hosted card and wallet checkout rail for global business payments, subscriptions, and webhook-driven settlement.",
                "provider_handoff",
                "planned",
                COMMERCE_PAYMENT_PROVIDER_STRIPE,
                COMMERCE_PAYMENT_CHANNEL_HOSTED_CHECKOUT,
                "hosted_checkout",
                "STRIPE",
                None,
                "stripe_signature",
                true,
                true,
                true,
                true,
            ),
            build_checkout_session_method(
                order,
                "alipay_qr",
                "Alipay QR",
                "Mainland China scan-to-pay rail for consumer and enterprise Alipay settlement with callback confirmation.",
                "provider_handoff",
                "planned",
                COMMERCE_PAYMENT_PROVIDER_ALIPAY,
                COMMERCE_PAYMENT_CHANNEL_SCAN_QR,
                "qr_code",
                "ALIPAY",
                Some(build_checkout_qr_payload("alipay_qr", &order.order_id)),
                "alipay_rsa_sha256",
                true,
                false,
                false,
                true,
            ),
            build_checkout_session_method(
                order,
                "wechat_pay_qr",
                "WeChat Pay QR",
                "Native WeChat Pay scan rail for real-time QR settlement, webhook confirmation, and refund lifecycle callbacks.",
                "provider_handoff",
                "planned",
                COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY,
                COMMERCE_PAYMENT_CHANNEL_SCAN_QR,
                "qr_code",
                "WECHAT",
                Some(build_checkout_qr_payload("wechat_pay_qr", &order.order_id)),
                "wechatpay_rsa_sha256",
                true,
                false,
                false,
                true,
            ),
        ]);
    }

    methods
}

fn build_checkout_session_method(
    order: &CommerceOrderRecord,
    id: &str,
    label: &str,
    detail: &str,
    action: &str,
    availability: &str,
    provider: &str,
    channel: &str,
    session_kind: &str,
    session_reference_prefix: &str,
    qr_code_payload: Option<String>,
    webhook_verification: &str,
    supports_refund: bool,
    supports_partial_refund: bool,
    recommended: bool,
    supports_webhook: bool,
) -> PortalCommerceCheckoutSessionMethod {
    PortalCommerceCheckoutSessionMethod {
        id: id.to_owned(),
        label: label.to_owned(),
        detail: detail.to_owned(),
        action: action.to_owned(),
        availability: availability.to_owned(),
        provider: provider.to_owned(),
        channel: channel.to_owned(),
        session_kind: session_kind.to_owned(),
        session_reference: build_checkout_session_reference(
            session_reference_prefix,
            &order.order_id,
        ),
        qr_code_payload,
        webhook_verification: webhook_verification.to_owned(),
        supports_refund,
        supports_partial_refund,
        recommended,
        supports_webhook,
    }
}

fn build_checkout_session_reference(prefix: &str, order_id: &str) -> String {
    format!("{prefix}-{}", normalize_payment_reference(order_id))
}

fn build_checkout_qr_payload(method_id: &str, order_id: &str) -> String {
    format!(
        "sdkworkpay://{method_id}/{}",
        normalize_payment_reference(order_id)
    )
}

fn normalize_payment_reference(order_id: &str) -> String {
    order_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn build_priced_quote(
    target_kind: &str,
    target_id: &str,
    target_name: &str,
    list_price_cents: u64,
    granted_units: u64,
    source: &str,
    current_remaining_units: Option<u64>,
    applied_coupon: Option<CommerceCouponDefinition>,
) -> PortalCommerceQuote {
    let discount_percent = applied_coupon
        .as_ref()
        .and_then(|coupon| coupon.benefit.discount_percent)
        .unwrap_or(0);
    let bonus_units = applied_coupon
        .as_ref()
        .map(|coupon| coupon.benefit.bonus_units)
        .unwrap_or(0);
    let payable_cents =
        list_price_cents.saturating_mul(u64::from(100_u8.saturating_sub(discount_percent))) / 100;
    let projected_remaining_units = current_remaining_units.map(|units| {
        units
            .saturating_add(granted_units)
            .saturating_add(bonus_units)
    });

    PortalCommerceQuote {
        target_kind: target_kind.to_owned(),
        target_id: target_id.to_owned(),
        target_name: target_name.to_owned(),
        list_price_cents,
        payable_price_cents: payable_cents,
        list_price_label: format_quote_price_label(list_price_cents),
        payable_price_label: format_quote_price_label(payable_cents),
        granted_units,
        bonus_units,
        amount_cents: None,
        projected_remaining_units,
        applied_coupon: applied_coupon.map(|coupon| PortalAppliedCoupon {
            code: coupon.coupon.code,
            discount_label: coupon.coupon.discount_label,
            source: coupon.coupon.source,
            discount_percent: coupon.benefit.discount_percent,
            bonus_units: coupon.benefit.bonus_units,
        }),
        pricing_rule_label: None,
        effective_ratio_label: None,
        source: source.to_owned(),
    }
}

fn build_custom_recharge_quote(
    amount_cents: u64,
    current_remaining_units: Option<u64>,
    applied_coupon: Option<CommerceCouponDefinition>,
) -> CommerceResult<PortalCommerceQuote> {
    let rule = resolve_custom_recharge_rule(amount_cents)?;
    let mut quote = build_priced_quote(
        "custom_recharge",
        &custom_recharge_target_id(amount_cents),
        "Custom recharge",
        amount_cents,
        amount_cents.saturating_mul(rule.units_per_cent),
        "workspace_seed",
        current_remaining_units,
        applied_coupon,
    );
    quote.amount_cents = Some(amount_cents);
    quote.pricing_rule_label = Some("Tiered custom recharge".to_owned());
    quote.effective_ratio_label = Some(format_effective_ratio_label(rule.units_per_cent));
    Ok(quote)
}

fn build_redemption_quote(
    coupon: CommerceCouponDefinition,
    current_remaining_units: Option<u64>,
) -> PortalCommerceQuote {
    let source = coupon.coupon.source.clone();
    let projected_remaining_units =
        current_remaining_units.map(|units| units.saturating_add(coupon.benefit.bonus_units));

    PortalCommerceQuote {
        target_kind: "coupon_redemption".to_owned(),
        target_id: coupon.coupon.code.clone(),
        target_name: coupon.coupon.code.clone(),
        list_price_cents: 0,
        payable_price_cents: 0,
        list_price_label: "$0.00".to_owned(),
        payable_price_label: "$0.00".to_owned(),
        granted_units: 0,
        bonus_units: coupon.benefit.bonus_units,
        amount_cents: None,
        projected_remaining_units,
        applied_coupon: Some(PortalAppliedCoupon {
            code: coupon.coupon.code,
            discount_label: coupon.coupon.discount_label,
            source: source.clone(),
            discount_percent: coupon.benefit.discount_percent,
            bonus_units: coupon.benefit.bonus_units,
        }),
        pricing_rule_label: None,
        effective_ratio_label: None,
        source,
    }
}

fn merge_coupon_benefit(
    current: CommerceCouponBenefit,
    previous: Option<CommerceCouponBenefit>,
) -> CommerceCouponBenefit {
    let fallback = previous.unwrap_or_default();
    CommerceCouponBenefit {
        discount_percent: current.discount_percent.or(fallback.discount_percent),
        bonus_units: if current.bonus_units > 0 {
            current.bonus_units
        } else {
            fallback.bonus_units
        },
    }
}

fn normalize_coupon_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn format_catalog_price_label(price_cents: u64) -> String {
    if price_cents % 100 == 0 {
        return format!("${}", price_cents / 100);
    }

    format_quote_price_label(price_cents)
}

fn format_quote_price_label(price_cents: u64) -> String {
    format!("${:.2}", price_cents as f64 / 100.0)
}

fn format_integer_with_commas(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(character);
    }

    formatted
}

fn format_effective_ratio_label(units_per_cent: u64) -> String {
    format!(
        "{} units / $1",
        format_integer_with_commas(units_per_cent.saturating_mul(100))
    )
}

fn custom_recharge_min_amount_cents() -> u64 {
    1_000
}

fn custom_recharge_max_amount_cents() -> u64 {
    200_000
}

fn custom_recharge_step_amount_cents() -> u64 {
    500
}

fn custom_recharge_suggested_amount_cents() -> u64 {
    5_000
}

fn custom_recharge_target_id(amount_cents: u64) -> String {
    format!("custom-{amount_cents}")
}

fn parse_custom_recharge_target_amount(target_id: &str) -> Option<u64> {
    target_id
        .strip_prefix("custom-")
        .and_then(|value| value.parse::<u64>().ok())
}

fn resolve_custom_recharge_amount_cents(
    target_id: &str,
    request_amount_cents: Option<u64>,
) -> CommerceResult<u64> {
    let amount_from_target = parse_custom_recharge_target_amount(target_id);

    if let (Some(target_amount_cents), Some(request_amount_cents)) =
        (amount_from_target, request_amount_cents)
    {
        if target_amount_cents != request_amount_cents {
            return Err(CommerceError::InvalidInput(
                "custom recharge amount does not match target_id".to_owned(),
            ));
        }
    }

    let amount_cents = request_amount_cents.or(amount_from_target).ok_or_else(|| {
        CommerceError::InvalidInput(
            "custom_amount_cents is required for custom_recharge".to_owned(),
        )
    })?;

    validate_custom_recharge_amount_cents(amount_cents)?;
    Ok(amount_cents)
}

fn validate_custom_recharge_amount_cents(amount_cents: u64) -> CommerceResult<()> {
    let min_amount_cents = custom_recharge_min_amount_cents();
    let max_amount_cents = custom_recharge_max_amount_cents();
    let step_amount_cents = custom_recharge_step_amount_cents();

    if amount_cents < min_amount_cents || amount_cents > max_amount_cents {
        return Err(CommerceError::InvalidInput(format!(
            "custom_amount_cents must stay between {min_amount_cents} and {max_amount_cents}"
        )));
    }

    if amount_cents % step_amount_cents != 0 {
        return Err(CommerceError::InvalidInput(format!(
            "custom_amount_cents must increase in steps of {step_amount_cents}"
        )));
    }

    Ok(())
}

fn resolve_custom_recharge_rule(amount_cents: u64) -> CommerceResult<CustomRechargeRuleSeed> {
    custom_recharge_rule_seeds()
        .into_iter()
        .find(|rule| amount_cents >= rule.min_amount_cents && amount_cents <= rule.max_amount_cents)
        .ok_or_else(|| {
            CommerceError::InvalidInput(format!(
                "no custom recharge rule applies to amount {amount_cents}"
            ))
        })
}

fn parse_discount_percent(label: &str) -> Option<u8> {
    let percent_index = label.find('%')?;
    let digits = label[..percent_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }

    let value = digits.parse::<u8>().ok()?;
    Some(value.min(100))
}

fn subscription_plan_seeds() -> Vec<SubscriptionPlanSeed> {
    vec![
        SubscriptionPlanSeed {
            id: "starter",
            name: "Starter",
            price_cents: 1_900,
            cadence: "/month",
            included_units: 10_000,
            highlight: "For prototypes and lean internal tools",
            features: &[
                "10k token units included",
                "2 live API keys",
                "Email support",
            ],
            cta: "Start Starter",
        },
        SubscriptionPlanSeed {
            id: "growth",
            name: "Growth",
            price_cents: 7_900,
            cadence: "/month",
            included_units: 100_000,
            highlight: "For production workloads and multi-environment delivery",
            features: &[
                "100k token units included",
                "10 live API keys",
                "Priority support",
            ],
            cta: "Upgrade to Growth",
        },
        SubscriptionPlanSeed {
            id: "scale",
            name: "Scale",
            price_cents: 24_900,
            cadence: "/month",
            included_units: 500_000,
            highlight: "For platform teams optimizing predictable spend",
            features: &[
                "500k token units included",
                "Unlimited keys",
                "Architecture advisory",
            ],
            cta: "Talk to Sales",
        },
    ]
}

fn recharge_pack_seeds() -> Vec<RechargePackSeed> {
    vec![
        RechargePackSeed {
            id: "pack-25k",
            label: "Boost 25k",
            points: 25_000,
            price_cents: 1_200,
            note: "Best for launch spikes and testing windows.",
        },
        RechargePackSeed {
            id: "pack-100k",
            label: "Boost 100k",
            points: 100_000,
            price_cents: 4_000,
            note: "Designed for monthly usage expansion.",
        },
        RechargePackSeed {
            id: "pack-500k",
            label: "Boost 500k",
            points: 500_000,
            price_cents: 16_500,
            note: "For scheduled releases and campaign traffic.",
        },
    ]
}

fn recharge_option_seeds() -> Vec<RechargeOptionSeed> {
    vec![
        RechargeOptionSeed {
            id: "recharge-10",
            label: "Starter top-up",
            amount_cents: 1_000,
            granted_units: 25_000,
            note: "Fastest way to restore balance for prototyping and short tests.",
            recommended: false,
        },
        RechargeOptionSeed {
            id: "recharge-50",
            label: "Growth top-up",
            amount_cents: 5_000,
            granted_units: 140_000,
            note: "Best balance between instant headroom and effective recharge ratio.",
            recommended: true,
        },
        RechargeOptionSeed {
            id: "recharge-100",
            label: "Scale top-up",
            amount_cents: 10_000,
            granted_units: 300_000,
            note: "Designed for sustained production traffic and larger daily workloads.",
            recommended: false,
        },
        RechargeOptionSeed {
            id: "recharge-200",
            label: "Campaign top-up",
            amount_cents: 20_000,
            granted_units: 660_000,
            note: "Most efficient preset for launches, campaigns, and heavy concurrency windows.",
            recommended: false,
        },
    ]
}

fn custom_recharge_rule_seeds() -> Vec<CustomRechargeRuleSeed> {
    vec![
        CustomRechargeRuleSeed {
            id: "tier-entry",
            label: "Entry tier",
            min_amount_cents: 1_000,
            max_amount_cents: 4_500,
            units_per_cent: 25,
            note:
                "Entry custom recharges restore balance quickly while preserving the starter ratio.",
        },
        CustomRechargeRuleSeed {
            id: "tier-growth",
            label: "Growth tier",
            min_amount_cents: 5_000,
            max_amount_cents: 9_500,
            units_per_cent: 28,
            note: "Growth custom recharges match the recommended balance-to-headroom ratio.",
        },
        CustomRechargeRuleSeed {
            id: "tier-scale",
            label: "Scale tier",
            min_amount_cents: 10_000,
            max_amount_cents: 19_500,
            units_per_cent: 30,
            note: "Scale custom recharges keep larger recurring traffic windows cost-efficient.",
        },
        CustomRechargeRuleSeed {
            id: "tier-campaign",
            label: "Campaign tier",
            min_amount_cents: 20_000,
            max_amount_cents: 200_000,
            units_per_cent: 33,
            note: "Campaign custom recharges maximize the effective ratio for larger top-ups.",
        },
    ]
}

fn seed_coupon_definitions() -> Vec<CommerceCouponDefinition> {
    coupon_seeds()
        .into_iter()
        .map(|seed| CommerceCouponDefinition {
            coupon: PortalCommerceCoupon {
                id: seed.id.to_owned(),
                code: seed.code.to_owned(),
                discount_label: seed.discount_label.to_owned(),
                audience: seed.audience.to_owned(),
                remaining: seed.remaining,
                active: true,
                note: seed.note.to_owned(),
                expires_on: seed.expires_on.to_owned(),
                source: "workspace_seed".to_owned(),
                discount_percent: seed.discount_percent,
                bonus_units: seed.bonus_units,
            },
            benefit: CommerceCouponBenefit {
                discount_percent: seed.discount_percent,
                bonus_units: seed.bonus_units,
            },
        })
        .collect()
}

fn coupon_seeds() -> Vec<CouponSeed> {
    vec![
        CouponSeed {
            id: "seed_welcome100",
            code: "WELCOME100",
            discount_label: "+100 starter points",
            audience: "new_workspace",
            remaining: 100,
            note: "Apply during onboarding to offset initial exploration traffic.",
            expires_on: "rolling",
            discount_percent: None,
            bonus_units: 100,
        },
        CouponSeed {
            id: "seed_springboost",
            code: "SPRINGBOOST",
            discount_label: "10% off Growth",
            audience: "growth_upgrade",
            remaining: 10_000,
            note: "Use on the next subscription change for a temporary expansion window.",
            expires_on: "rolling",
            discount_percent: Some(10),
            bonus_units: 0,
        },
        CouponSeed {
            id: "seed_teamready",
            code: "TEAMREADY",
            discount_label: "Free staging credits",
            audience: "team_rollout",
            remaining: 25_000,
            note: "Unlocks extra staging budget for launch validation.",
            expires_on: "rolling",
            discount_percent: None,
            bonus_units: 25_000,
        },
    ]
}

fn generate_entity_id(prefix: &str) -> CommerceResult<String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CommerceError::Storage(anyhow::anyhow!("system clock error")))?
        .as_nanos();
    Ok(format!("{prefix}_{nonce:x}"))
}

fn current_time_ms() -> CommerceResult<u64> {
    Ok(u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommerceError::Storage(anyhow::anyhow!("system clock error")))?
            .as_millis(),
    )
    .map_err(|error| CommerceError::Storage(error.into()))?)
}

#[async_trait]
trait CommerceQuotaStore: Send + Sync {
    async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<QuotaPolicy>>;
}

#[async_trait]
impl<T> CommerceQuotaStore for T
where
    T: AdminStore + ?Sized,
{
    async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<QuotaPolicy>> {
        AdminStore::list_quota_policies_for_project(self, project_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[test]
    fn parses_percent_discount_suffixes() {
        assert_eq!(parse_discount_percent("20% launch discount"), Some(20));
        assert_eq!(parse_discount_percent("10% off Growth"), Some(10));
        assert_eq!(parse_discount_percent("Free staging credits"), None);
    }

    #[test]
    fn priced_quote_applies_discount_and_bonus_units() {
        let quote = build_priced_quote(
            "recharge_pack",
            "pack-100k",
            "Boost 100k",
            4_000,
            100_000,
            "workspace_seed",
            Some(5_000),
            Some(CommerceCouponDefinition {
                coupon: PortalCommerceCoupon {
                    id: "coupon_spring_launch".to_owned(),
                    code: "SPRING20".to_owned(),
                    discount_label: "20% launch discount".to_owned(),
                    audience: "new_signup".to_owned(),
                    remaining: 120,
                    active: true,
                    note: "Spring launch campaign".to_owned(),
                    expires_on: "2026-05-31".to_owned(),
                    source: "live".to_owned(),
                    discount_percent: Some(20),
                    bonus_units: 0,
                },
                benefit: CommerceCouponBenefit {
                    discount_percent: Some(20),
                    bonus_units: 0,
                },
            }),
        );

        assert_eq!(quote.payable_price_label, "$32.00");
        assert_eq!(quote.projected_remaining_units, Some(105_000));
        assert_eq!(quote.applied_coupon.unwrap().code, "SPRING20");
    }

    #[test]
    fn redemption_quote_uses_bonus_units() {
        let quote = build_redemption_quote(
            CommerceCouponDefinition {
                coupon: PortalCommerceCoupon {
                    id: "seed_welcome100".to_owned(),
                    code: "WELCOME100".to_owned(),
                    discount_label: "+100 starter points".to_owned(),
                    audience: "new_workspace".to_owned(),
                    remaining: 100,
                    active: true,
                    note: "Apply during onboarding to offset initial exploration traffic."
                        .to_owned(),
                    expires_on: "rolling".to_owned(),
                    source: "workspace_seed".to_owned(),
                    discount_percent: None,
                    bonus_units: 100,
                },
                benefit: CommerceCouponBenefit {
                    discount_percent: None,
                    bonus_units: 100,
                },
            },
            Some(5_000),
        );

        assert_eq!(quote.payable_price_label, "$0.00");
        assert_eq!(quote.bonus_units, 100);
        assert_eq!(quote.projected_remaining_units, Some(5_100));
    }

    #[tokio::test]
    async fn load_effective_quota_policy_reads_only_project_scope() {
        let store = RecordingCommerceQuotaStore::new(vec![
            QuotaPolicy::new("policy-project-1-a", "project-1", 300).with_enabled(true),
            QuotaPolicy::new("policy-project-1-b", "project-1", 200).with_enabled(true),
            QuotaPolicy::new("policy-project-2", "project-2", 1).with_enabled(true),
        ]);

        let policy = load_effective_quota_policy(&store, "project-1")
            .await
            .unwrap()
            .expect("expected project policy");

        assert_eq!(policy.policy_id, "policy-project-1-b");
        assert_eq!(
            store.last_project_id.lock().unwrap().as_deref(),
            Some("project-1")
        );
    }

    struct RecordingCommerceQuotaStore {
        policies: Vec<QuotaPolicy>,
        last_project_id: Mutex<Option<String>>,
    }

    impl RecordingCommerceQuotaStore {
        fn new(policies: Vec<QuotaPolicy>) -> Self {
            Self {
                policies,
                last_project_id: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl CommerceQuotaStore for RecordingCommerceQuotaStore {
        async fn list_quota_policies_for_project(
            &self,
            project_id: &str,
        ) -> anyhow::Result<Vec<QuotaPolicy>> {
            *self.last_project_id.lock().unwrap() = Some(project_id.to_owned());
            Ok(self
                .policies
                .iter()
                .filter(|policy| policy.project_id == project_id)
                .cloned()
                .collect())
        }
    }
}
