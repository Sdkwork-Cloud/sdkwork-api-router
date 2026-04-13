use super::super::super::super::{unix_timestamp_ms, MarketingGovernanceError};
use super::super::super::actionability::{
    build_marketing_campaign_actionability, build_marketing_campaign_detail,
};
use super::super::super::audit::{
    build_marketing_campaign_lifecycle_audit_record,
    persist_marketing_campaign_lifecycle_audit_record,
};
use super::super::super::lookup::load_marketing_campaign_context;
use super::super::super::types::MarketingCampaignMutationResult;
use super::transition::resolve_marketing_campaign_lifecycle_transition;
use super::validation::ensure_marketing_campaign_lifecycle_allowed;
use sdkwork_api_domain_marketing::{
    MarketingCampaignLifecycleAction, MarketingCampaignLifecycleAuditOutcome,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn mutate_marketing_campaign_lifecycle(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
    action: MarketingCampaignLifecycleAction,
    operator_id: &str,
    request_id: &str,
    reason: &str,
) -> Result<MarketingCampaignMutationResult, MarketingGovernanceError> {
    let now_ms = unix_timestamp_ms();
    let (campaign, coupon_template) =
        load_marketing_campaign_context(store, marketing_campaign_id).await?;
    let actionability = build_marketing_campaign_actionability(&campaign, &coupon_template, now_ms);
    let decision = match action {
        MarketingCampaignLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        MarketingCampaignLifecycleAction::SubmitForApproval => &actionability.submit_for_approval,
        MarketingCampaignLifecycleAction::Approve => &actionability.approve,
        MarketingCampaignLifecycleAction::Reject => &actionability.reject,
        MarketingCampaignLifecycleAction::Publish => &actionability.publish,
        MarketingCampaignLifecycleAction::Schedule => &actionability.schedule,
        MarketingCampaignLifecycleAction::Retire => &actionability.retire,
    };

    ensure_marketing_campaign_lifecycle_allowed(
        store,
        &campaign,
        action,
        decision,
        operator_id,
        request_id,
        reason,
        now_ms,
    )
    .await?;

    let (next_status, next_approval_state) =
        resolve_marketing_campaign_lifecycle_transition(&campaign, action);
    let updated_campaign = campaign
        .clone()
        .with_status(next_status)
        .with_approval_state(next_approval_state)
        .with_updated_at_ms(now_ms);
    let updated_campaign = store
        .insert_marketing_campaign_record(&updated_campaign)
        .await
        .map_err(MarketingGovernanceError::storage)?;

    let detail = build_marketing_campaign_detail(updated_campaign.clone(), coupon_template, now_ms);
    let audit = build_marketing_campaign_lifecycle_audit_record(
        &campaign,
        Some(&updated_campaign),
        None,
        action,
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
