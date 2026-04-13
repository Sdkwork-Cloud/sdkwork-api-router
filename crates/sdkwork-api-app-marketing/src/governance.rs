mod budget;
mod campaign;
mod code;
mod template;

pub use budget::{
    mutate_marketing_campaign_budget_lifecycle, CampaignBudgetActionDecision,
    CampaignBudgetActionability, CampaignBudgetDetail, CampaignBudgetMutationResult,
};
pub use campaign::{
    clone_marketing_campaign_revision, compare_marketing_campaign_revisions,
    mutate_marketing_campaign_lifecycle, CloneMarketingCampaignRevisionInput,
    MarketingCampaignActionDecision, MarketingCampaignActionability,
    MarketingCampaignComparisonFieldChange, MarketingCampaignComparisonResult,
    MarketingCampaignDetail, MarketingCampaignMutationResult,
};
pub use code::{
    mutate_marketing_coupon_code_lifecycle, CouponCodeActionDecision, CouponCodeActionability,
    CouponCodeDetail, CouponCodeMutationResult,
};
pub use template::{
    clone_marketing_coupon_template_revision, compare_marketing_coupon_template_revisions,
    mutate_marketing_coupon_template_lifecycle, CloneCouponTemplateRevisionInput,
    CouponTemplateActionDecision, CouponTemplateActionability, CouponTemplateComparisonFieldChange,
    CouponTemplateComparisonResult, CouponTemplateDetail, CouponTemplateMutationResult,
};

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum MarketingGovernanceError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Storage(anyhow::Error),
}

impl MarketingGovernanceError {
    fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    fn storage(error: impl Into<anyhow::Error>) -> Self {
        Self::Storage(error.into())
    }
}

impl std::fmt::Display for MarketingGovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::NotFound(message) | Self::Conflict(message) => {
                write!(f, "{message}")
            }
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MarketingGovernanceError {}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn normalize_optional_display_name(value: String) -> Option<String> {
    let trimmed = value.trim().to_owned();
    (!trimmed.is_empty()).then_some(trimmed)
}
