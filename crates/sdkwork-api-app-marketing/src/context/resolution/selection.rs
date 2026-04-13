use sdkwork_api_domain_marketing::{CampaignBudgetRecord, MarketingCampaignRecord};

pub fn select_effective_marketing_campaign(
    campaigns: Vec<MarketingCampaignRecord>,
    now_ms: u64,
) -> Option<MarketingCampaignRecord> {
    campaigns
        .into_iter()
        .filter(|campaign| campaign.is_effective_at(now_ms))
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
        })
}

pub fn select_campaign_budget_record(
    budgets: Vec<CampaignBudgetRecord>,
) -> Option<CampaignBudgetRecord> {
    budgets.into_iter().max_by(|left, right| {
        left.updated_at_ms
            .cmp(&right.updated_at_ms)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    })
}

pub(super) fn select_marketing_campaign_for_reference(
    campaigns: Vec<MarketingCampaignRecord>,
    preferred_marketing_campaign_id: Option<&str>,
    now_ms: u64,
) -> Option<MarketingCampaignRecord> {
    if let Some(marketing_campaign_id) = preferred_marketing_campaign_id {
        if let Some(campaign) = campaigns
            .iter()
            .find(|campaign| campaign.marketing_campaign_id == marketing_campaign_id)
            .cloned()
        {
            return Some(campaign);
        }
    }

    select_effective_marketing_campaign(campaigns.clone(), now_ms).or_else(|| {
        campaigns.into_iter().max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
        })
    })
}
