use super::error::MarketingOperationError;
use super::support::load_coupon_reservation_for_subject;
use super::types::{ReleaseCouponInput, ReleaseCouponResult};
use crate::{
    load_marketing_coupon_context_from_code_record, release_campaign_budget, MarketingCouponContext,
};
use sdkwork_api_domain_marketing::CouponReservationStatus;
use sdkwork_api_storage_core::{AdminStore, AtomicCouponReleaseCommand};

pub async fn release_coupon_for_subject(
    store: &dyn AdminStore,
    input: ReleaseCouponInput<'_>,
) -> Result<ReleaseCouponResult, MarketingOperationError> {
    let reservation = load_coupon_reservation_for_subject(
        store,
        input.subject_scope,
        input.subject_id,
        input.coupon_reservation_id,
    )
    .await?;
    let code = store
        .find_coupon_code_record(&reservation.coupon_code_id)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| MarketingOperationError::not_found("coupon code not found"))?;
    let context = load_marketing_coupon_context_from_code_record(store, code, input.now_ms)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| MarketingOperationError::not_found("coupon context is unavailable"))?;

    if reservation.reservation_status != CouponReservationStatus::Reserved {
        return Ok(ReleaseCouponResult {
            context,
            reservation,
            created: false,
        });
    }

    let atomic_result = store
        .release_coupon_reservation_atomic(&AtomicCouponReleaseCommand {
            expected_budget: context.budget.clone(),
            next_budget: release_campaign_budget(
                &context.budget,
                reservation.budget_reserved_minor,
                input.now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: crate::code_after_release(&context.template, &context.code, input.now_ms),
            expected_reservation: reservation.clone(),
            next_reservation: reservation
                .clone()
                .with_status(CouponReservationStatus::Released)
                .with_updated_at_ms(input.now_ms),
        })
        .await
        .map_err(MarketingOperationError::storage)?;

    Ok(ReleaseCouponResult {
        context: MarketingCouponContext {
            template: context.template,
            campaign: context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        },
        reservation: atomic_result.reservation,
        created: atomic_result.created,
    })
}
