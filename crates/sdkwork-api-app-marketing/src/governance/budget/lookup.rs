use super::super::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{CampaignBudgetRecord, MarketingCampaignRecord};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn load_campaign_budget_context(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
) -> Result<(CampaignBudgetRecord, MarketingCampaignRecord), MarketingGovernanceError> {
    let budget = store
        .find_campaign_budget_record(campaign_budget_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!(
                "campaign budget {campaign_budget_id} not found"
            ))
        })?;
    let campaign = store
        .find_marketing_campaign_record(&budget.marketing_campaign_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!(
                "marketing campaign {} for campaign budget {} not found",
                budget.marketing_campaign_id, campaign_budget_id
            ))
        })?;
    Ok((budget, campaign))
}
