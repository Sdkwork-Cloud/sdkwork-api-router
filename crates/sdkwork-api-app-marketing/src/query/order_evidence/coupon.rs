use anyhow::{Context, Result};
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord,
    CouponTemplateRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn load_coupon_reservation(
    store: &dyn AdminStore,
    coupon_reservation_id: Option<&str>,
) -> Result<Option<CouponReservationRecord>> {
    match coupon_reservation_id {
        Some(coupon_reservation_id) => store
            .find_coupon_reservation_record(coupon_reservation_id)
            .await
            .with_context(|| format!("failed to load coupon reservation {coupon_reservation_id}")),
        None => Ok(None),
    }
}

pub(super) async fn load_coupon_redemption(
    store: &dyn AdminStore,
    coupon_redemption_id: Option<&str>,
) -> Result<Option<CouponRedemptionRecord>> {
    match coupon_redemption_id {
        Some(coupon_redemption_id) => store
            .find_coupon_redemption_record(coupon_redemption_id)
            .await
            .with_context(|| format!("failed to load coupon redemption {coupon_redemption_id}")),
        None => Ok(None),
    }
}

pub(super) async fn load_coupon_rollbacks(
    store: &dyn AdminStore,
    coupon_redemption: Option<&CouponRedemptionRecord>,
) -> Result<Vec<CouponRollbackRecord>> {
    let Some(redemption) = coupon_redemption else {
        return Ok(Vec::new());
    };

    let mut coupon_rollbacks = store
        .list_coupon_rollback_records_for_redemption(&redemption.coupon_redemption_id)
        .await
        .with_context(|| {
            format!(
                "failed to load coupon rollback records for redemption {}",
                redemption.coupon_redemption_id
            )
        })?;
    coupon_rollbacks.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| right.coupon_rollback_id.cmp(&left.coupon_rollback_id))
    });
    Ok(coupon_rollbacks)
}

pub(super) async fn load_coupon_code(
    store: &dyn AdminStore,
    coupon_reservation: Option<&CouponReservationRecord>,
    coupon_redemption: Option<&CouponRedemptionRecord>,
    applied_coupon_code: Option<&str>,
) -> Result<Option<CouponCodeRecord>> {
    if let Some(redemption) = coupon_redemption {
        return store
            .find_coupon_code_record(&redemption.coupon_code_id)
            .await
            .with_context(|| format!("failed to load coupon code {}", redemption.coupon_code_id));
    }

    if let Some(reservation) = coupon_reservation {
        return store
            .find_coupon_code_record(&reservation.coupon_code_id)
            .await
            .with_context(|| format!("failed to load coupon code {}", reservation.coupon_code_id));
    }

    if let Some(applied_coupon_code) = applied_coupon_code {
        return store
            .find_coupon_code_record_by_value(applied_coupon_code)
            .await
            .with_context(|| format!("failed to load coupon code {applied_coupon_code}"));
    }

    Ok(None)
}

pub(super) async fn load_coupon_template(
    store: &dyn AdminStore,
    coupon_redemption: Option<&CouponRedemptionRecord>,
    coupon_code: Option<&CouponCodeRecord>,
) -> Result<Option<CouponTemplateRecord>> {
    let coupon_template_id = coupon_redemption
        .map(|redemption| redemption.coupon_template_id.as_str())
        .or_else(|| coupon_code.map(|coupon_code| coupon_code.coupon_template_id.as_str()));

    match coupon_template_id {
        Some(coupon_template_id) => store
            .find_coupon_template_record(coupon_template_id)
            .await
            .with_context(|| format!("failed to load coupon template {coupon_template_id}")),
        None => Ok(None),
    }
}
