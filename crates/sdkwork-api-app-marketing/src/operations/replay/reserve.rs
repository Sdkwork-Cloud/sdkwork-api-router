use super::super::error::MarketingOperationError;
use super::super::types::{ReserveCouponInput, ReserveCouponResult};
use crate::{load_marketing_coupon_context_from_code_record, normalize_coupon_code};
use sdkwork_api_storage_core::AdminStore;

pub(crate) async fn try_replay_reserved_coupon(
    store: &dyn AdminStore,
    input: &ReserveCouponInput<'_>,
    coupon_reservation_id: &str,
) -> Result<Option<ReserveCouponResult>, MarketingOperationError> {
    if input.idempotency_key.is_none() {
        return Ok(None);
    }

    let Some(existing_reservation) = store
        .find_coupon_reservation_record(coupon_reservation_id)
        .await
        .map_err(MarketingOperationError::storage)?
    else {
        return Ok(None);
    };

    let existing_code = store
        .find_coupon_code_record(&existing_reservation.coupon_code_id)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| {
            MarketingOperationError::not_found(
                "coupon code not found for idempotent reservation replay",
            )
        })?;
    let existing_ttl_ms = existing_reservation
        .expires_at_ms
        .saturating_sub(existing_reservation.created_at_ms);
    if existing_reservation.subject_scope != input.subject_scope
        || existing_reservation.subject_id != input.subject_id
        || normalize_coupon_code(&existing_code.code_value)
            != normalize_coupon_code(input.coupon_code)
        || existing_reservation.budget_reserved_minor != input.reserve_amount_minor
        || existing_ttl_ms != input.ttl_ms
    {
        return Err(MarketingOperationError::conflict(
            "idempotent reservation replay does not match the original request",
        ));
    }

    let context =
        load_marketing_coupon_context_from_code_record(store, existing_code, input.now_ms)
            .await
            .map_err(MarketingOperationError::storage)?
            .ok_or_else(|| {
                MarketingOperationError::not_found(
                    "coupon context is unavailable for reservation replay",
                )
            })?;

    Ok(Some(ReserveCouponResult {
        context,
        reservation: existing_reservation,
        created: false,
    }))
}
