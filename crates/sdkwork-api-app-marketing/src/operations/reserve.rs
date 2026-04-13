use super::error::MarketingOperationError;
use super::replay::try_replay_reserved_coupon;
use super::types::{ReserveCouponInput, ReserveCouponResult};
use super::validate::validate_coupon_for_subject;
use crate::{
    code_after_reservation, derive_coupon_reservation_id, normalize_coupon_code,
    reserve_campaign_budget, reserve_coupon_redemption, MarketingCouponContext,
};
use sdkwork_api_storage_core::{AdminStore, AtomicCouponReservationCommand};

pub async fn reserve_coupon_for_subject(
    store: &dyn AdminStore,
    input: ReserveCouponInput<'_>,
) -> Result<ReserveCouponResult, MarketingOperationError> {
    let target_kind = input.target_kind.trim();
    if target_kind.is_empty() {
        return Err(MarketingOperationError::invalid_input(
            "target_kind is required",
        ));
    }

    let coupon_reservation_id = input
        .idempotency_key
        .map(|key| {
            derive_coupon_reservation_id(input.subject_scope, input.subject_id, target_kind, key)
        })
        .unwrap_or_else(|| {
            format!(
                "coupon_reservation_{}_{}",
                normalize_coupon_code(input.coupon_code).to_ascii_lowercase(),
                input.now_ms
            )
        });

    if let Some(replay) = try_replay_reserved_coupon(store, &input, &coupon_reservation_id).await? {
        return Ok(replay);
    }

    let order_amount_minor = if input.order_amount_minor == 0 {
        input.reserve_amount_minor
    } else {
        input.order_amount_minor
    };
    let validation = validate_coupon_for_subject(
        store,
        input.coupon_code,
        input.subject_scope,
        input.subject_id,
        target_kind,
        order_amount_minor,
        input.reserve_amount_minor,
        input.now_ms,
    )
    .await?;
    if !validation.decision.eligible {
        return Err(MarketingOperationError::conflict(
            validation
                .decision
                .rejection_reason
                .unwrap_or_else(|| "coupon reservation rejected".to_owned()),
        ));
    }

    let (reserved_code, reservation) = reserve_coupon_redemption(
        &validation.context.code,
        coupon_reservation_id,
        input.subject_scope,
        input.subject_id.to_owned(),
        input.reserve_amount_minor,
        input.now_ms,
        input.ttl_ms,
    )?;

    let atomic_result = store
        .reserve_coupon_redemption_atomic(&AtomicCouponReservationCommand {
            template_to_persist: None,
            campaign_to_persist: None,
            expected_budget: validation.context.budget.clone(),
            next_budget: reserve_campaign_budget(
                &validation.context.budget,
                input.reserve_amount_minor,
                input.now_ms,
            ),
            expected_code: validation.context.code.clone(),
            next_code: code_after_reservation(
                &validation.context.template,
                &validation.context.code,
                &reserved_code,
                input.now_ms,
            ),
            reservation,
        })
        .await
        .map_err(MarketingOperationError::storage)?;

    Ok(ReserveCouponResult {
        context: MarketingCouponContext {
            template: validation.context.template,
            campaign: validation.context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        },
        reservation: atomic_result.reservation,
        created: atomic_result.created,
    })
}
