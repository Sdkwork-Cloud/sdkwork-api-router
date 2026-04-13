use crate::{CouponValidationDecision, MarketingCouponContext};
use sdkwork_api_domain_marketing::{
    CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord, CouponRollbackType,
    MarketingSubjectScope,
};

#[derive(Debug, Clone)]
pub struct ValidatedCouponResult {
    pub context: MarketingCouponContext,
    pub decision: CouponValidationDecision,
}

#[derive(Debug, Clone)]
pub struct ReserveCouponInput<'a> {
    pub coupon_code: &'a str,
    pub subject_scope: MarketingSubjectScope,
    pub subject_id: &'a str,
    pub target_kind: &'a str,
    pub order_amount_minor: u64,
    pub reserve_amount_minor: u64,
    pub ttl_ms: u64,
    pub idempotency_key: Option<&'a str>,
    pub now_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ReserveCouponResult {
    pub context: MarketingCouponContext,
    pub reservation: CouponReservationRecord,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct ConfirmCouponInput<'a> {
    pub coupon_reservation_id: &'a str,
    pub subject_scope: MarketingSubjectScope,
    pub subject_id: &'a str,
    pub subsidy_amount_minor: u64,
    pub order_id: Option<String>,
    pub payment_event_id: Option<String>,
    pub idempotency_key: Option<&'a str>,
    pub now_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ConfirmCouponResult {
    pub context: MarketingCouponContext,
    pub reservation: CouponReservationRecord,
    pub redemption: CouponRedemptionRecord,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct ReleaseCouponInput<'a> {
    pub coupon_reservation_id: &'a str,
    pub subject_scope: MarketingSubjectScope,
    pub subject_id: &'a str,
    pub now_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ReleaseCouponResult {
    pub context: MarketingCouponContext,
    pub reservation: CouponReservationRecord,
    pub created: bool,
}

#[derive(Debug, Clone)]
pub struct RollbackCouponInput<'a> {
    pub coupon_redemption_id: &'a str,
    pub subject_scope: MarketingSubjectScope,
    pub subject_id: &'a str,
    pub rollback_type: CouponRollbackType,
    pub restored_budget_minor: u64,
    pub restored_inventory_count: u64,
    pub idempotency_key: Option<&'a str>,
    pub now_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RollbackCouponResult {
    pub context: MarketingCouponContext,
    pub redemption: CouponRedemptionRecord,
    pub rollback: CouponRollbackRecord,
    pub created: bool,
}
