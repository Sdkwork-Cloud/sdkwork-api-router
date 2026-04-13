use super::super::{unix_timestamp_ms, MarketingGovernanceError};
use super::actionability::{build_coupon_code_actionability, build_coupon_code_detail};
use super::audit::{
    build_coupon_code_lifecycle_audit_record, persist_coupon_code_lifecycle_audit_record,
};
use super::lookup::load_coupon_code_context;
use super::types::CouponCodeMutationResult;
use sdkwork_api_domain_marketing::{
    CouponCodeLifecycleAction, CouponCodeLifecycleAuditOutcome, CouponCodeStatus,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn mutate_marketing_coupon_code_lifecycle(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    action: CouponCodeLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CouponCodeMutationResult, MarketingGovernanceError> {
    let now_ms = unix_timestamp_ms();
    let (coupon_code, coupon_template) = load_coupon_code_context(store, coupon_code_id).await?;
    let actionability = build_coupon_code_actionability(&coupon_code, now_ms);
    let decision = match action {
        CouponCodeLifecycleAction::Disable => &actionability.disable,
        CouponCodeLifecycleAction::Restore => &actionability.restore,
    };
    if !decision.allowed {
        let audit = build_coupon_code_lifecycle_audit_record(
            &coupon_code,
            None,
            action,
            CouponCodeLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            decision.reasons.clone(),
        );
        persist_coupon_code_lifecycle_audit_record(store, &audit).await?;
        return Err(MarketingGovernanceError::invalid_input(
            decision
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "coupon code lifecycle action is not allowed".to_owned()),
        ));
    }

    let next_status = match action {
        CouponCodeLifecycleAction::Disable => CouponCodeStatus::Disabled,
        CouponCodeLifecycleAction::Restore => CouponCodeStatus::Available,
    };
    let updated_coupon_code = coupon_code
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms);
    let updated_coupon_code = store
        .insert_coupon_code_record(&updated_coupon_code)
        .await
        .map_err(MarketingGovernanceError::storage)?;

    let detail = build_coupon_code_detail(updated_coupon_code.clone(), coupon_template, now_ms);
    let audit = build_coupon_code_lifecycle_audit_record(
        &coupon_code,
        Some(&updated_coupon_code),
        action,
        CouponCodeLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_coupon_code_lifecycle_audit_record(store, &audit).await?;
    Ok(CouponCodeMutationResult { detail, audit })
}
