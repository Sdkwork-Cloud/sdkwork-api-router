use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponCodeStatus, CouponReservationRecord, MarketingOutboxEventRecord,
};
use serde_json::json;

pub(super) fn expired_reservation_outbox_event(
    reservation: &CouponReservationRecord,
    code: &CouponCodeRecord,
    released_budget_minor: u64,
    now_ms: u64,
) -> MarketingOutboxEventRecord {
    MarketingOutboxEventRecord::new(
        format!(
            "recovery_coupon_reservation_expired_{}",
            reservation.coupon_reservation_id
        ),
        "coupon_reservation",
        reservation.coupon_reservation_id.clone(),
        "coupon.reservation.expired",
        json!({
            "coupon_reservation_id": reservation.coupon_reservation_id,
            "coupon_code_id": reservation.coupon_code_id,
            "coupon_code_status": coupon_code_status_label(code.status),
            "released_budget_minor": released_budget_minor,
            "expired_at_ms": now_ms,
        })
        .to_string(),
        now_ms,
    )
    .with_updated_at_ms(now_ms)
}

fn coupon_code_status_label(status: CouponCodeStatus) -> &'static str {
    match status {
        CouponCodeStatus::Available => "available",
        CouponCodeStatus::Reserved => "reserved",
        CouponCodeStatus::Redeemed => "redeemed",
        CouponCodeStatus::Expired => "expired",
        CouponCodeStatus::Disabled => "disabled",
    }
}
