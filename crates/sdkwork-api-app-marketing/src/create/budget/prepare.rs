use super::super::support::normalize_required_identifier;
use crate::MarketingServiceError;
use sdkwork_api_domain_marketing::{CampaignBudgetRecord, CampaignBudgetStatus};

pub(super) fn prepare_campaign_budget_record_for_create(
    mut record: CampaignBudgetRecord,
) -> Result<CampaignBudgetRecord, MarketingServiceError> {
    record.campaign_budget_id = normalize_required_identifier(
        &record.campaign_budget_id,
        "campaign budget",
        "campaign_budget_id",
    )?;
    record.marketing_campaign_id = normalize_required_identifier(
        &record.marketing_campaign_id,
        "campaign budget",
        "marketing_campaign_id",
    )?;

    if record.status != CampaignBudgetStatus::Draft {
        return Err(MarketingServiceError::invalid_state(
            "campaign budget create does not accept lifecycle status",
        ));
    }
    if record.reserved_budget_minor != 0 {
        return Err(MarketingServiceError::invalid_state(
            "campaign budget create does not accept reserved budget",
        ));
    }
    if record.consumed_budget_minor != 0 {
        return Err(MarketingServiceError::invalid_state(
            "campaign budget create does not accept consumed budget",
        ));
    }
    if record.total_budget_minor == 0 {
        return Err(MarketingServiceError::invalid_state(
            "campaign budget create requires positive total budget",
        ));
    }

    record.status = CampaignBudgetStatus::Draft;
    record.reserved_budget_minor = 0;
    record.consumed_budget_minor = 0;
    Ok(record)
}
