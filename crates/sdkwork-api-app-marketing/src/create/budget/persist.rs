use super::super::support::{
    marketing_create_invalid_input, marketing_create_storage, require_marketing_campaign_record,
    PersistMode,
};
use super::prepare::prepare_campaign_budget_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::CampaignBudgetRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_campaign_budget_record(
    store: &dyn AdminStore,
    record: CampaignBudgetRecord,
    mode: PersistMode,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    let record = prepare_campaign_budget_record_for_create(record)
        .map_err(marketing_create_invalid_input)?;

    if let Some(existing_budget) = store
        .find_campaign_budget_record(&record.campaign_budget_id)
        .await
        .map_err(marketing_create_storage)?
    {
        return mode.resolve_existing_primary(
            "campaign budget",
            &record.campaign_budget_id,
            existing_budget,
            &record,
        );
    }

    require_marketing_campaign_record(store, &record.marketing_campaign_id).await?;

    store
        .insert_campaign_budget_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_campaign_budget_record(
    store: &dyn AdminStore,
    record: CampaignBudgetRecord,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    persist_campaign_budget_record(store, record, PersistMode::Create).await
}

pub async fn ensure_campaign_budget_record(
    store: &dyn AdminStore,
    record: CampaignBudgetRecord,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    persist_campaign_budget_record(store, record, PersistMode::Ensure).await
}
