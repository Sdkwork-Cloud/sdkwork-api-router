use super::super::support::{
    load_coupon_template_record, marketing_create_invalid_input, marketing_create_storage,
    PersistMode,
};
use super::prepare::prepare_marketing_campaign_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::MarketingCampaignRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_marketing_campaign_record(
    store: &dyn AdminStore,
    record: MarketingCampaignRecord,
    mode: PersistMode,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    let record = prepare_marketing_campaign_record_for_create(record)
        .map_err(marketing_create_invalid_input)?;

    if let Some(existing_campaign) = store
        .find_marketing_campaign_record(&record.marketing_campaign_id)
        .await
        .map_err(marketing_create_storage)?
    {
        return mode.resolve_existing_primary(
            "marketing campaign",
            &record.marketing_campaign_id,
            existing_campaign,
            &record,
        );
    }

    load_coupon_template_record(store, &record.coupon_template_id).await?;

    store
        .insert_marketing_campaign_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_marketing_campaign_record(
    store: &dyn AdminStore,
    record: MarketingCampaignRecord,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    persist_marketing_campaign_record(store, record, PersistMode::Create).await
}

pub async fn ensure_marketing_campaign_record(
    store: &dyn AdminStore,
    record: MarketingCampaignRecord,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    persist_marketing_campaign_record(store, record, PersistMode::Ensure).await
}
