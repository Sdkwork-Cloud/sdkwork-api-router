use super::super::support::{normalize_optional_text, normalize_required_identifier};
use crate::MarketingServiceError;
use sdkwork_api_domain_marketing::{
    MarketingCampaignApprovalState, MarketingCampaignRecord, MarketingCampaignStatus,
};

pub(super) fn prepare_marketing_campaign_record_for_create(
    mut record: MarketingCampaignRecord,
) -> Result<MarketingCampaignRecord, MarketingServiceError> {
    record.marketing_campaign_id = normalize_required_identifier(
        &record.marketing_campaign_id,
        "marketing campaign",
        "marketing_campaign_id",
    )?;
    record.coupon_template_id = normalize_required_identifier(
        &record.coupon_template_id,
        "marketing campaign",
        "coupon_template_id",
    )?;
    record.display_name = normalize_optional_text(record.display_name);

    if record.status != MarketingCampaignStatus::Draft {
        return Err(MarketingServiceError::invalid_state(
            "marketing campaign create does not accept lifecycle status",
        ));
    }
    if record.revision > 1 {
        return Err(MarketingServiceError::invalid_state(
            "marketing campaign create does not accept revision",
        ));
    }
    if record.parent_marketing_campaign_id.is_some() {
        return Err(MarketingServiceError::invalid_state(
            "marketing campaign create does not accept parent lineage",
        ));
    }
    if record
        .root_marketing_campaign_id
        .as_deref()
        .is_some_and(|root_id| root_id != record.marketing_campaign_id)
    {
        return Err(MarketingServiceError::invalid_state(
            "marketing campaign create root lineage must point to itself",
        ));
    }
    if let (Some(start_at_ms), Some(end_at_ms)) = (record.start_at_ms, record.end_at_ms) {
        if end_at_ms < start_at_ms {
            return Err(MarketingServiceError::invalid_state(
                "marketing campaign create requires end_at_ms >= start_at_ms",
            ));
        }
    }

    record.approval_state = MarketingCampaignApprovalState::Draft;
    record.revision = 1;
    record.parent_marketing_campaign_id = None;
    record.root_marketing_campaign_id = Some(record.marketing_campaign_id.clone());
    Ok(record)
}
