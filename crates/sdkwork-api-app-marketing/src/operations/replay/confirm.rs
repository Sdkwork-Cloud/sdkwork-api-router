use super::super::error::MarketingOperationError;
use super::super::support::load_coupon_reservation_for_subject;
use super::super::types::{ConfirmCouponInput, ConfirmCouponResult};
use crate::load_marketing_coupon_context_for_code_id;
use sdkwork_api_domain_marketing::CouponReservationRecord;
use sdkwork_api_storage_core::AdminStore;

pub(crate) async fn try_replay_confirmed_coupon(
    store: &dyn AdminStore,
    input: &ConfirmCouponInput<'_>,
    reservation: &CouponReservationRecord,
    budget_consumed_minor: u64,
    coupon_redemption_id: &str,
) -> Result<Option<ConfirmCouponResult>, MarketingOperationError> {
    if input.idempotency_key.is_none() {
        return Ok(None);
    }

    let Some(existing_redemption) = store
        .find_coupon_redemption_record(coupon_redemption_id)
        .await
        .map_err(MarketingOperationError::storage)?
    else {
        return Ok(None);
    };

    if existing_redemption.coupon_reservation_id != reservation.coupon_reservation_id
        || existing_redemption.budget_consumed_minor != budget_consumed_minor
        || existing_redemption.subsidy_amount_minor != input.subsidy_amount_minor
        || existing_redemption.order_id.as_ref() != input.order_id.as_ref()
        || existing_redemption.payment_event_id.as_ref() != input.payment_event_id.as_ref()
    {
        return Err(MarketingOperationError::conflict(
            "idempotent redemption replay does not match the original request",
        ));
    }

    let current_reservation = load_coupon_reservation_for_subject(
        store,
        input.subject_scope,
        input.subject_id,
        &existing_redemption.coupon_reservation_id,
    )
    .await?;
    let context = load_marketing_coupon_context_for_code_id(
        store,
        &existing_redemption.coupon_code_id,
        input.now_ms,
    )
    .await
    .map_err(MarketingOperationError::storage)?
    .ok_or_else(|| {
        MarketingOperationError::not_found("coupon context is unavailable for redemption replay")
    })?;

    Ok(Some(ConfirmCouponResult {
        context,
        reservation: current_reservation,
        redemption: existing_redemption,
        created: false,
    }))
}
