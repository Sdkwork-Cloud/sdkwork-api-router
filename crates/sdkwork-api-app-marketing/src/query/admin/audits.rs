use anyhow::Result;
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAuditRecord, CouponCodeLifecycleAuditRecord,
    CouponTemplateLifecycleAuditRecord, MarketingCampaignLifecycleAuditRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn list_marketing_coupon_template_lifecycle_audits(
    store: &dyn AdminStore,
    coupon_template_id: &str,
) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
    let mut audits = store
        .list_coupon_template_lifecycle_audit_records_for_template(coupon_template_id)
        .await?;
    sort_audits_desc(&mut audits, |record| {
        (&record.requested_at_ms, &record.audit_id)
    });
    Ok(audits)
}

pub async fn list_marketing_campaign_lifecycle_audits(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
    let mut audits = store
        .list_marketing_campaign_lifecycle_audit_records_for_campaign(marketing_campaign_id)
        .await?;
    sort_audits_desc(&mut audits, |record| {
        (&record.requested_at_ms, &record.audit_id)
    });
    Ok(audits)
}

pub async fn list_marketing_campaign_budget_lifecycle_audits(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
    let mut audits = store
        .list_campaign_budget_lifecycle_audit_records_for_budget(campaign_budget_id)
        .await?;
    sort_audits_desc(&mut audits, |record| {
        (&record.requested_at_ms, &record.audit_id)
    });
    Ok(audits)
}

pub async fn list_marketing_coupon_code_lifecycle_audits(
    store: &dyn AdminStore,
    coupon_code_id: &str,
) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
    let mut audits = store
        .list_coupon_code_lifecycle_audit_records_for_code(coupon_code_id)
        .await?;
    sort_audits_desc(&mut audits, |record| {
        (&record.requested_at_ms, &record.audit_id)
    });
    Ok(audits)
}

fn sort_audits_desc<T>(audits: &mut [T], key: impl Fn(&T) -> (&u64, &String)) {
    audits.sort_by(|left, right| {
        let (left_requested_at_ms, left_audit_id) = key(left);
        let (right_requested_at_ms, right_audit_id) = key(right);
        right_requested_at_ms
            .cmp(left_requested_at_ms)
            .then_with(|| right_audit_id.cmp(left_audit_id))
    });
}
