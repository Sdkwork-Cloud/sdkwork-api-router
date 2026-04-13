use super::super::super::super::{unix_timestamp_ms, MarketingGovernanceError};
use super::super::super::actionability::build_coupon_template_detail;
use super::super::super::audit::{
    build_coupon_template_lifecycle_audit_record, persist_coupon_template_lifecycle_audit_record,
};
use super::super::super::lookup::load_coupon_template_record;
use super::super::super::types::{CloneCouponTemplateRevisionInput, CouponTemplateMutationResult};
use super::builder::build_cloned_coupon_template;
use super::validation::ensure_coupon_template_clone_allowed;
use sdkwork_api_domain_marketing::{
    CouponTemplateLifecycleAction, CouponTemplateLifecycleAuditOutcome,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn clone_marketing_coupon_template_revision(
    store: &dyn AdminStore,
    source_coupon_template_id: &str,
    input: CloneCouponTemplateRevisionInput,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<CouponTemplateMutationResult, MarketingGovernanceError> {
    let now_ms = unix_timestamp_ms();
    let source_coupon_template =
        load_coupon_template_record(store, source_coupon_template_id).await?;

    ensure_coupon_template_clone_allowed(
        store,
        &source_coupon_template,
        &input,
        operator_id,
        request_id,
        reason,
        now_ms,
    )
    .await?;

    let cloned_coupon_template =
        build_cloned_coupon_template(store, &source_coupon_template, input, now_ms).await?;
    let cloned_coupon_template = store
        .insert_coupon_template_record(&cloned_coupon_template)
        .await
        .map_err(MarketingGovernanceError::storage)?;

    let detail = build_coupon_template_detail(cloned_coupon_template.clone(), now_ms);
    let audit = build_coupon_template_lifecycle_audit_record(
        &source_coupon_template,
        Some(&cloned_coupon_template),
        Some(source_coupon_template.coupon_template_id.clone()),
        CouponTemplateLifecycleAction::Clone,
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
