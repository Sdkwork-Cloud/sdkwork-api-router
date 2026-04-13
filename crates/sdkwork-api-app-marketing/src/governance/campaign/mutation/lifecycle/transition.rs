use sdkwork_api_domain_marketing::{
    MarketingCampaignApprovalState, MarketingCampaignLifecycleAction, MarketingCampaignRecord,
    MarketingCampaignStatus,
};

pub(super) fn resolve_marketing_campaign_lifecycle_transition(
    campaign: &MarketingCampaignRecord,
    action: MarketingCampaignLifecycleAction,
) -> (MarketingCampaignStatus, MarketingCampaignApprovalState) {
    match action {
        MarketingCampaignLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        MarketingCampaignLifecycleAction::SubmitForApproval => (
            MarketingCampaignStatus::Draft,
            MarketingCampaignApprovalState::InReview,
        ),
        MarketingCampaignLifecycleAction::Approve => (
            MarketingCampaignStatus::Draft,
            MarketingCampaignApprovalState::Approved,
        ),
        MarketingCampaignLifecycleAction::Reject => (
            MarketingCampaignStatus::Draft,
            MarketingCampaignApprovalState::Rejected,
        ),
        MarketingCampaignLifecycleAction::Publish => {
            (MarketingCampaignStatus::Active, campaign.approval_state)
        }
        MarketingCampaignLifecycleAction::Schedule => {
            (MarketingCampaignStatus::Scheduled, campaign.approval_state)
        }
        MarketingCampaignLifecycleAction::Retire => {
            (MarketingCampaignStatus::Ended, campaign.approval_state)
        }
    }
}
