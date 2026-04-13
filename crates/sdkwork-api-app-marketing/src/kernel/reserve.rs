use super::error::MarketingServiceError;
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponCodeStatus, CouponReservationRecord, MarketingSubjectScope,
};

pub fn reserve_coupon_redemption(
    code: &CouponCodeRecord,
    coupon_reservation_id: impl Into<String>,
    subject_scope: MarketingSubjectScope,
    subject_id: impl Into<String>,
    budget_reserved_minor: u64,
    now_ms: u64,
    ttl_ms: u64,
) -> Result<(CouponCodeRecord, CouponReservationRecord), MarketingServiceError> {
    if ttl_ms == 0 {
        return Err(MarketingServiceError::invalid_state(
            "reservation ttl must be positive",
        ));
    }
    if !code.is_redeemable_at(now_ms) {
        return Err(MarketingServiceError::invalid_state(
            "coupon code is not redeemable",
        ));
    }

    let reserved_code = code
        .clone()
        .with_status(CouponCodeStatus::Reserved)
        .with_updated_at_ms(now_ms);
    let reservation = CouponReservationRecord::new(
        coupon_reservation_id,
        reserved_code.coupon_code_id.clone(),
        subject_scope,
        subject_id,
        now_ms.saturating_add(ttl_ms),
    )
    .with_budget_reserved_minor(budget_reserved_minor)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    Ok((reserved_code, reservation))
}
