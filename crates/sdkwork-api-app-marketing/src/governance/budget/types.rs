use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAuditRecord, CampaignBudgetRecord, MarketingCampaignRecord,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CampaignBudgetActionDecision {
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CampaignBudgetActionability {
    pub activate: CampaignBudgetActionDecision,
    pub close: CampaignBudgetActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CampaignBudgetDetail {
    pub budget: CampaignBudgetRecord,
    pub campaign: MarketingCampaignRecord,
    pub actionability: CampaignBudgetActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CampaignBudgetMutationResult {
    pub detail: CampaignBudgetDetail,
    pub audit: CampaignBudgetLifecycleAuditRecord,
}
