use super::super::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{CouponTemplateRecord, MarketingCampaignRecord};
use sdkwork_api_storage_core::AdminStore;

pub(super) fn marketing_campaign_root_id(record: &MarketingCampaignRecord) -> String {
    record
        .root_marketing_campaign_id
        .clone()
        .unwrap_or_else(|| record.marketing_campaign_id.clone())
}

pub(super) fn marketing_campaign_revision(record: &MarketingCampaignRecord) -> u32 {
    record.revision.max(1)
}

pub(super) async fn next_marketing_campaign_revision(
    store: &dyn AdminStore,
    source_campaign: &MarketingCampaignRecord,
) -> Result<u32, MarketingGovernanceError> {
    let root_marketing_campaign_id = marketing_campaign_root_id(source_campaign);
    let next_revision = store
        .list_marketing_campaign_records_for_root(&root_marketing_campaign_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .into_iter()
        .map(|record| marketing_campaign_revision(&record))
        .max()
        .unwrap_or_else(|| marketing_campaign_revision(source_campaign))
        .saturating_add(1);
    Ok(next_revision.max(marketing_campaign_revision(source_campaign) + 1))
}

pub(super) async fn load_marketing_campaign_context(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
) -> Result<(MarketingCampaignRecord, CouponTemplateRecord), MarketingGovernanceError> {
    let campaign = store
        .find_marketing_campaign_record(marketing_campaign_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!(
                "marketing campaign {marketing_campaign_id} not found"
            ))
        })?;
    let coupon_template = store
        .find_coupon_template_record(&campaign.coupon_template_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!(
                "coupon template {} for marketing campaign {} not found",
                campaign.coupon_template_id, marketing_campaign_id
            ))
        })?;
    Ok((campaign, coupon_template))
}

pub(super) fn marketing_campaign_field_value(
    record: &MarketingCampaignRecord,
    field: &str,
) -> Result<String, MarketingGovernanceError> {
    match field {
        "coupon_template_id" => Ok(record.coupon_template_id.clone()),
        "display_name" => Ok(record.display_name.clone()),
        "status" => Ok(serde_json::to_string(&record.status).unwrap_or_default()),
        "approval_state" => Ok(serde_json::to_string(&record.approval_state).unwrap_or_default()),
        "revision" => Ok(record.revision.to_string()),
        "start_at_ms" => Ok(record
            .start_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned())),
        "end_at_ms" => Ok(record
            .end_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned())),
        _ => Ok(String::new()),
    }
}
