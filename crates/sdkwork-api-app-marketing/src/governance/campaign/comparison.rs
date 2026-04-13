use super::super::MarketingGovernanceError;
use super::lookup::{
    load_marketing_campaign_context, marketing_campaign_field_value, marketing_campaign_root_id,
};
use super::types::{MarketingCampaignComparisonFieldChange, MarketingCampaignComparisonResult};
use sdkwork_api_storage_core::AdminStore;

pub async fn compare_marketing_campaign_revisions(
    store: &dyn AdminStore,
    source_marketing_campaign_id: &str,
    target_marketing_campaign_id: &str,
) -> Result<MarketingCampaignComparisonResult, MarketingGovernanceError> {
    let (source_marketing_campaign, _) =
        load_marketing_campaign_context(store, source_marketing_campaign_id).await?;
    let (target_marketing_campaign, _) =
        load_marketing_campaign_context(store, target_marketing_campaign_id).await?;
    let mut field_changes = Vec::new();
    for field in [
        "coupon_template_id",
        "display_name",
        "status",
        "approval_state",
        "revision",
        "start_at_ms",
        "end_at_ms",
    ] {
        let source_value = marketing_campaign_field_value(&source_marketing_campaign, field)?;
        let target_value = marketing_campaign_field_value(&target_marketing_campaign, field)?;
        if source_value != target_value {
            field_changes.push(MarketingCampaignComparisonFieldChange {
                field: field.to_owned(),
                source_value,
                target_value,
            });
        }
    }
    Ok(MarketingCampaignComparisonResult {
        same_lineage: marketing_campaign_root_id(&source_marketing_campaign)
            == marketing_campaign_root_id(&target_marketing_campaign),
        source_marketing_campaign,
        target_marketing_campaign,
        field_changes,
    })
}
