use super::error::MarketingOperationError;
use super::replay::try_replay_rolled_back_coupon;
use super::support::{load_coupon_redemption_for_subject, load_coupon_reservation_for_subject};
use super::types::{RollbackCouponInput, RollbackCouponResult};
use crate::{
    code_after_rollback, derive_coupon_rollback_id, load_marketing_coupon_context_from_code_record,
    rollback_campaign_budget, rollback_coupon_redemption, MarketingCouponContext,
};
use sdkwork_api_storage_core::{AdminStore, AtomicCouponRollbackCommand};

pub async fn rollback_coupon_for_subject(
    store: &dyn AdminStore,
    input: RollbackCouponInput<'_>,
) -> Result<RollbackCouponResult, MarketingOperationError> {
    let redemption = load_coupon_redemption_for_subject(
        store,
        input.subject_scope,
        input.subject_id,
        input.coupon_redemption_id,
    )
    .await?;
    if input.restored_budget_minor > redemption.budget_consumed_minor {
        return Err(MarketingOperationError::invalid_input(
            "restored budget exceeds consumed coupon budget",
        ));
    }

    let reservation = load_coupon_reservation_for_subject(
        store,
        input.subject_scope,
        input.subject_id,
        &redemption.coupon_reservation_id,
    )
    .await?;
    let coupon_rollback_id = input
        .idempotency_key
        .map(|key| derive_coupon_rollback_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_rollback_{}_{}",
                redemption.coupon_redemption_id, input.now_ms
            )
        });

    if let Some(replay) = try_replay_rolled_back_coupon(store, &input, &coupon_rollback_id).await? {
        return Ok(replay);
    }

    let code = store
        .find_coupon_code_record(&redemption.coupon_code_id)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| MarketingOperationError::not_found("coupon code not found"))?;
    let context = load_marketing_coupon_context_from_code_record(store, code, input.now_ms)
        .await
        .map_err(MarketingOperationError::storage)?
        .ok_or_else(|| MarketingOperationError::not_found("coupon context is unavailable"))?;

    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        coupon_rollback_id,
        input.rollback_type,
        input.restored_budget_minor,
        input.restored_inventory_count,
        input.now_ms,
    )?;

    let atomic_result = store
        .rollback_coupon_redemption_atomic(&AtomicCouponRollbackCommand {
            expected_budget: context.budget.clone(),
            next_budget: rollback_campaign_budget(
                &context.budget,
                input.restored_budget_minor,
                input.now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_rollback(&context.template, &context.code, input.now_ms),
            expected_redemption: redemption,
            next_redemption: rolled_back_redemption,
            rollback,
        })
        .await
        .map_err(MarketingOperationError::storage)?;

    Ok(RollbackCouponResult {
        context: MarketingCouponContext {
            template: context.template,
            campaign: context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        },
        redemption: atomic_result.redemption,
        rollback: atomic_result.rollback,
        created: atomic_result.created,
    })
}
