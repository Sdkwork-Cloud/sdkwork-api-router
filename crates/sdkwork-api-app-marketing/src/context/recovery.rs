use super::resolution::load_marketing_coupon_context_from_code_record;
use crate::{code_after_release, release_campaign_budget};
use anyhow::Result;
use sdkwork_api_domain_marketing::{CouponCodeRecord, CouponReservationStatus};
use sdkwork_api_storage_core::{AdminStore, AtomicCouponReleaseCommand};

pub async fn reclaim_expired_coupon_reservations_for_code_record_if_needed(
    store: &dyn AdminStore,
    code_record: CouponCodeRecord,
    now_ms: u64,
) -> Result<u64> {
    let mut expired_reservation_ids = store
        .list_coupon_reservation_records_for_code(&code_record.coupon_code_id)
        .await?
        .into_iter()
        .filter(|reservation| {
            reservation.reservation_status == CouponReservationStatus::Reserved
                && reservation.expires_at_ms < now_ms
        })
        .map(|reservation| reservation.coupon_reservation_id)
        .collect::<Vec<_>>();
    expired_reservation_ids.sort();

    let mut reclaimed = 0;
    for reservation_id in expired_reservation_ids {
        let Some(reservation) = store
            .find_coupon_reservation_record(&reservation_id)
            .await?
        else {
            continue;
        };
        if reservation.reservation_status != CouponReservationStatus::Reserved
            || reservation.expires_at_ms >= now_ms
        {
            continue;
        }

        let Some(current_code) = store
            .find_coupon_code_record(&reservation.coupon_code_id)
            .await?
        else {
            continue;
        };
        let Some(context) =
            load_marketing_coupon_context_from_code_record(store, current_code, now_ms).await?
        else {
            continue;
        };

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
                    .with_status(CouponReservationStatus::Expired)
                    .with_updated_at_ms(now_ms),
            })
            .await?;
        reclaimed += 1;
    }

    Ok(reclaimed)
}

pub async fn reclaim_expired_coupon_reservations_for_code_if_needed(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> Result<u64> {
    let normalized = super::normalize_coupon_code(code);
    let Some(code_record) = store.find_coupon_code_record_by_value(&normalized).await? else {
        return Ok(0);
    };

    reclaim_expired_coupon_reservations_for_code_record_if_needed(store, code_record, now_ms).await
}
