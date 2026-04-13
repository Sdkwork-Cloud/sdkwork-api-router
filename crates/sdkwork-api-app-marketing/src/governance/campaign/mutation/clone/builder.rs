use super::super::super::super::{normalize_optional_display_name, MarketingGovernanceError};
use super::super::super::lookup::{marketing_campaign_root_id, next_marketing_campaign_revision};
use super::super::super::types::CloneMarketingCampaignRevisionInput;
use sdkwork_api_domain_marketing::{
    MarketingCampaignApprovalState, MarketingCampaignRecord, MarketingCampaignStatus,
};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn build_cloned_marketing_campaign(
    store: &dyn AdminStore,
    source_campaign: &MarketingCampaignRecord,
    input: CloneMarketingCampaignRevisionInput,
    now_ms: u64,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    let cloned_display_name = input
        .display_name
        .and_then(normalize_optional_display_name)
        .unwrap_or_else(|| source_campaign.display_name.clone());
    let root_marketing_campaign_id = marketing_campaign_root_id(source_campaign);
    let cloned_campaign = source_campaign
        .clone()
        .with_status(MarketingCampaignStatus::Draft)
        .with_approval_state(MarketingCampaignApprovalState::Draft)
        .with_revision(next_marketing_campaign_revision(store, source_campaign).await?)
        .with_root_marketing_campaign_id(Some(root_marketing_campaign_id))
        .with_parent_marketing_campaign_id(Some(source_campaign.marketing_campaign_id.clone()))
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms);

    Ok(MarketingCampaignRecord {
        marketing_campaign_id: input.marketing_campaign_id,
        display_name: cloned_display_name,
        ..cloned_campaign
    })
}
