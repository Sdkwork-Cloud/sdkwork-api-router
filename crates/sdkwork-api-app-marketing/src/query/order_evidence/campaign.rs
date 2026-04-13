use anyhow::{Context, Result};
use sdkwork_api_domain_marketing::MarketingCampaignRecord;
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn load_marketing_campaign(
    store: &dyn AdminStore,
    marketing_campaign_id: Option<&str>,
) -> Result<Option<MarketingCampaignRecord>> {
    match marketing_campaign_id {
        Some(marketing_campaign_id) => store
            .find_marketing_campaign_record(marketing_campaign_id)
            .await
            .with_context(|| format!("failed to load marketing campaign {marketing_campaign_id}")),
        None => Ok(None),
    }
}
