use sdkwork_api_domain_marketing::{
    CouponTemplateRecord, MarketingCampaignLifecycleAuditRecord, MarketingCampaignRecord,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketingCampaignActionDecision {
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketingCampaignActionability {
    pub clone: MarketingCampaignActionDecision,
    pub submit_for_approval: MarketingCampaignActionDecision,
    pub approve: MarketingCampaignActionDecision,
    pub reject: MarketingCampaignActionDecision,
    pub publish: MarketingCampaignActionDecision,
    pub schedule: MarketingCampaignActionDecision,
    pub retire: MarketingCampaignActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketingCampaignDetail {
    pub campaign: MarketingCampaignRecord,
    pub coupon_template: CouponTemplateRecord,
    pub actionability: MarketingCampaignActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketingCampaignMutationResult {
    pub detail: MarketingCampaignDetail,
    pub audit: MarketingCampaignLifecycleAuditRecord,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketingCampaignComparisonFieldChange {
    pub field: String,
    pub source_value: String,
    pub target_value: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MarketingCampaignComparisonResult {
    pub source_marketing_campaign: MarketingCampaignRecord,
    pub target_marketing_campaign: MarketingCampaignRecord,
    pub same_lineage: bool,
    #[serde(default)]
    pub field_changes: Vec<MarketingCampaignComparisonFieldChange>,
}

#[derive(Debug, Clone)]
pub struct CloneMarketingCampaignRevisionInput {
    pub marketing_campaign_id: String,
    pub display_name: Option<String>,
}
