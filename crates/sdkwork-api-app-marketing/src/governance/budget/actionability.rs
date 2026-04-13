use super::types::{
    CampaignBudgetActionDecision, CampaignBudgetActionability, CampaignBudgetDetail,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, MarketingCampaignRecord, MarketingCampaignStatus,
};

fn allowed_campaign_budget_action() -> CampaignBudgetActionDecision {
    CampaignBudgetActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_campaign_budget_action(reason: impl Into<String>) -> CampaignBudgetActionDecision {
    CampaignBudgetActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

pub(super) fn build_campaign_budget_actionability(
    budget: &CampaignBudgetRecord,
    campaign: &MarketingCampaignRecord,
) -> CampaignBudgetActionability {
    let campaign_closed = matches!(
        campaign.status,
        MarketingCampaignStatus::Ended | MarketingCampaignStatus::Archived
    );
    let available_headroom = budget.available_budget_minor();
    let activate = if budget.status == CampaignBudgetStatus::Closed {
        blocked_campaign_budget_action("campaign budget is already closed")
    } else if budget.status == CampaignBudgetStatus::Active {
        blocked_campaign_budget_action("campaign budget is already active")
    } else if campaign_closed {
        blocked_campaign_budget_action("linked marketing campaign is ended or archived")
    } else if available_headroom == 0 {
        blocked_campaign_budget_action("campaign budget has no available headroom")
    } else {
        allowed_campaign_budget_action()
    };
    let close = if budget.status == CampaignBudgetStatus::Closed {
        blocked_campaign_budget_action("campaign budget is already closed")
    } else {
        allowed_campaign_budget_action()
    };
    CampaignBudgetActionability { activate, close }
}

pub(super) fn build_campaign_budget_detail(
    budget: CampaignBudgetRecord,
    campaign: MarketingCampaignRecord,
) -> CampaignBudgetDetail {
    let actionability = build_campaign_budget_actionability(&budget, &campaign);
    CampaignBudgetDetail {
        budget,
        campaign,
        actionability,
    }
}
