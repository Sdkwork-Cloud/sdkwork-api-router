use super::super::super::super::{unix_timestamp_ms, MarketingGovernanceError};
use super::super::super::actionability::build_marketing_campaign_detail;
use super::super::super::audit::{
    build_marketing_campaign_lifecycle_audit_record,
    persist_marketing_campaign_lifecycle_audit_record,
};
use super::super::super::lookup::load_marketing_campaign_context;
use super::super::super::types::{
    CloneMarketingCampaignRevisionInput, MarketingCampaignMutationResult,
};
use super::builder::build_cloned_marketing_campaign;
use super::validation::ensure_marketing_campaign_clone_allowed;
use sdkwork_api_domain_marketing::{
    MarketingCampaignLifecycleAction, MarketingCampaignLifecycleAuditOutcome,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn clone_marketing_campaign_revision(
    store: &dyn AdminStore,
    source_marketing_campaign_id: &str,
    input: CloneMarketingCampaignRevisionInput,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<MarketingCampaignMutationResult, MarketingGovernanceError> {
    let now_ms = unix_timestamp_ms();
    let (source_campaign, coupon_template) =
        load_marketing_campaign_context(store, source_marketing_campaign_id).await?;

    ensure_marketing_campaign_clone_allowed(
        store,
        &source_campaign,
        &coupon_template,
        &input,
        operator_id,
        request_id,
        reason,
        now_ms,
    )
    .await?;

    let cloned_campaign =
        build_cloned_marketing_campaign(store, &source_campaign, input, now_ms).await?;
    let cloned_campaign = store
        .insert_marketing_campaign_record(&cloned_campaign)
        .await
        .map_err(MarketingGovernanceError::storage)?;

    let detail = build_marketing_campaign_detail(cloned_campaign.clone(), coupon_template, now_ms);
    let audit = build_marketing_campaign_lifecycle_audit_record(
        &source_campaign,
        Some(&cloned_campaign),
        Some(source_campaign.marketing_campaign_id.clone()),
        MarketingCampaignLifecycleAction::Clone,
        MarketingCampaignLifecycleAuditOutcome::Applied,
        operator_id,
        request_id,
        reason,
        now_ms,
        Vec::new(),
    );
    let audit = persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
    Ok(MarketingCampaignMutationResult { detail, audit })
}
