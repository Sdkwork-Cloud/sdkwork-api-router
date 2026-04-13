use super::error::MarketingOperationError;
use super::replay::try_replay_confirmed_coupon;
use super::support::load_coupon_reservation_for_subject;
use super::types::{ConfirmCouponInput, ConfirmCouponResult};
use crate::{
    code_after_confirmation, confirm_campaign_budget, confirm_coupon_redemption,
    derive_coupon_redemption_id, load_marketing_coupon_context_from_code_record,
    MarketingCouponContext,
};
use sdkwork_api_storage_core::{AdminStore, AtomicCouponConfirmationCommand};

pub async fn confirm_coupon_for_subject(
    store: &dyn AdminStore,
    input: ConfirmCouponInput<'_>,
) -> Result<ConfirmCouponResult, MarketingOperationError> {
    let reservation = load_coupon_reservation_for_subject(
        store,
        input.subject_scope,
        input.subject_id,
        input.coupon_reservation_id,
    )
    .await?;
    let budget_consumed_minor = if input.subsidy_amount_minor > 0 {
        input.subsidy_amount_minor
    } else {
        reservation.budget_reserved_minor
    };
    if budget_consumed_minor > reservation.budget_reserved_minor {
        return Err(MarketingOperationError::invalid_input(
            "budget consumption exceeds reserved coupon budget",
        ));
    }

    let coupon_redemption_id = input
        .idempotency_key
        .map(|key| derive_coupon_redemption_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_redemption_{}_{}",
                reservation.coupon_reservation_id, input.now_ms
            )
        });

    if let Some(replay) = try_replay_confirmed_coupon(
        store,
        &input,
        &reservation,
        budget_consumed_minor,
        &coupon_redemption_id,
    )
    .await?
    {
        return Ok(replay);
    }

    let code = store
        .find_coupon_code_record(&reservation.coupon_code_id)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| MarketingOperationError::not_found("coupon code not found"))?;
    let context = load_marketing_coupon_context_from_code_record(store, code, input.now_ms)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| MarketingOperationError::not_found("coupon context is unavailable"))?;

    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        coupon_redemption_id,
        context.code.coupon_code_id.clone(),
        context.template.coupon_template_id.clone(),
        budget_consumed_minor,
        input.subsidy_amount_minor,
        input.order_id,
        input.payment_event_id,
        input.now_ms,
    )?;

    let atomic_result = store
        .confirm_coupon_redemption_atomic(&AtomicCouponConfirmationCommand {
            expected_budget: context.budget.clone(),
            next_budget: confirm_campaign_budget(
                &context.budget,
                reservation.budget_reserved_minor,
                budget_consumed_minor,
                input.now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_confirmation(&context.template, &context.code, input.now_ms),
            expected_reservation: reservation,
            next_reservation: confirmed_reservation,
            redemption,
        })
        .await
        .map_err(MarketingOperationError::storage)?;

    Ok(ConfirmCouponResult {
        context: MarketingCouponContext {
            template: context.template,
            campaign: context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        },
        reservation: atomic_result.reservation,
        redemption: atomic_result.redemption,
        created: atomic_result.created,
    })
}
