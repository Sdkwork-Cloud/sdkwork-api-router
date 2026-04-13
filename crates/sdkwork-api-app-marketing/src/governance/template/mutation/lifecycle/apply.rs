use super::super::super::super::{unix_timestamp_ms, MarketingGovernanceError};
use super::super::super::actionability::{
    build_coupon_template_actionability, build_coupon_template_detail,
};
use super::super::super::audit::{
    build_coupon_template_lifecycle_audit_record, persist_coupon_template_lifecycle_audit_record,
};
use super::super::super::lookup::load_coupon_template_record;
use super::super::super::types::CouponTemplateMutationResult;
use super::transition::resolve_coupon_template_lifecycle_transition;
use super::validation::ensure_coupon_template_lifecycle_allowed;
use sdkwork_api_domain_marketing::{
    CouponTemplateLifecycleAction, CouponTemplateLifecycleAuditOutcome,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn mutate_marketing_coupon_template_lifecycle(
    store: &dyn AdminStore,
    coupon_template_id: &str,
    action: CouponTemplateLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CouponTemplateMutationResult, MarketingGovernanceError> {
    let now_ms = unix_timestamp_ms();
    let coupon_template = load_coupon_template_record(store, coupon_template_id).await?;
    let actionability = build_coupon_template_actionability(&coupon_template, now_ms);
    let decision = match action {
        CouponTemplateLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        CouponTemplateLifecycleAction::SubmitForApproval => &actionability.submit_for_approval,
        CouponTemplateLifecycleAction::Approve => &actionability.approve,
        CouponTemplateLifecycleAction::Reject => &actionability.reject,
        CouponTemplateLifecycleAction::Publish => &actionability.publish,
        CouponTemplateLifecycleAction::Schedule => &actionability.schedule,
        CouponTemplateLifecycleAction::Retire => &actionability.retire,
    };

    ensure_coupon_template_lifecycle_allowed(
        store,
        &coupon_template,
        action,
        decision,
        operator_id,
        request_id,
        reason,
        now_ms,
    )
    .await?;

    let (next_status, next_approval_state) =
        resolve_coupon_template_lifecycle_transition(&coupon_template, action);
    let updated_coupon_template = coupon_template
        .clone()
        .with_status(next_status)
        .with_approval_state(next_approval_state)
        .with_updated_at_ms(now_ms);
    let updated_coupon_template = store
        .insert_coupon_template_record(&updated_coupon_template)
        .await
        .map_err(MarketingGovernanceError::storage)?;

    let detail = build_coupon_template_detail(updated_coupon_template.clone(), now_ms);
    let audit = build_coupon_template_lifecycle_audit_record(
        &coupon_template,
        Some(&updated_coupon_template),
        None,
        action,
        CouponTemplateLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
    Ok(CouponTemplateMutationResult { detail, audit })
}
