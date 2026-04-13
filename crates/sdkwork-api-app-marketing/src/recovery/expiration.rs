use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponCodeStatus, CouponReservationRecord, CouponReservationStatus,
};

pub(super) fn should_expire_reservation(
    reservation: &CouponReservationRecord,
    now_ms: u64,
) -> bool {
    reservation.reservation_status == CouponReservationStatus::Reserved
        && reservation.expires_at_ms < now_ms
}

pub(super) fn recovered_coupon_code(
    code: &CouponCodeRecord,
    now_ms: u64,
) -> Option<CouponCodeRecord> {
    if code.status != CouponCodeStatus::Reserved {
        return None;
    }

    let next_status = if code
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms < now_ms)
    {
        CouponCodeStatus::Expired
    } else {
        CouponCodeStatus::Available
    };

    Some(
        code.clone()
            .with_status(next_status)
            .with_updated_at_ms(now_ms),
    )
}
