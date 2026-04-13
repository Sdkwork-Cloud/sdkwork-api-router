use sdkwork_api_domain_marketing::{CouponRedemptionRecord, CouponReservationRecord};
use std::collections::HashMap;

pub(crate) fn latest_reservations_by_code(
    reservations: Vec<CouponReservationRecord>,
) -> HashMap<String, CouponReservationRecord> {
    let mut latest = HashMap::new();
    for reservation in reservations {
        latest
            .entry(reservation.coupon_code_id.clone())
            .and_modify(|current: &mut CouponReservationRecord| {
                if reservation.updated_at_ms > current.updated_at_ms
                    || (reservation.updated_at_ms == current.updated_at_ms
                        && reservation.coupon_reservation_id > current.coupon_reservation_id)
                {
                    *current = reservation.clone();
                }
            })
            .or_insert(reservation);
    }
    latest
}

pub(crate) fn latest_redemptions_by_code(
    redemptions: Vec<CouponRedemptionRecord>,
) -> HashMap<String, CouponRedemptionRecord> {
    let mut latest = HashMap::new();
    for redemption in redemptions {
        latest
            .entry(redemption.coupon_code_id.clone())
            .and_modify(|current: &mut CouponRedemptionRecord| {
                if redemption.updated_at_ms > current.updated_at_ms
                    || (redemption.updated_at_ms == current.updated_at_ms
                        && redemption.coupon_redemption_id > current.coupon_redemption_id)
                {
                    *current = redemption.clone();
                }
            })
            .or_insert(redemption);
    }
    latest
}
