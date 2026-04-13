mod actionability;
mod audit;
mod comparison;
mod lookup;
mod mutation;
mod types;

pub use comparison::compare_marketing_campaign_revisions;
pub use mutation::{clone_marketing_campaign_revision, mutate_marketing_campaign_lifecycle};
pub use types::{
    CloneMarketingCampaignRevisionInput, MarketingCampaignActionDecision,
    MarketingCampaignActionability, MarketingCampaignComparisonFieldChange,
    MarketingCampaignComparisonResult, MarketingCampaignDetail, MarketingCampaignMutationResult,
};
