use sdkwork_api_domain_marketing::{CouponTemplateLifecycleAuditRecord, CouponTemplateRecord};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponTemplateActionDecision {
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponTemplateActionability {
    pub clone: CouponTemplateActionDecision,
    pub submit_for_approval: CouponTemplateActionDecision,
    pub approve: CouponTemplateActionDecision,
    pub reject: CouponTemplateActionDecision,
    pub publish: CouponTemplateActionDecision,
    pub schedule: CouponTemplateActionDecision,
    pub retire: CouponTemplateActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponTemplateDetail {
    pub coupon_template: CouponTemplateRecord,
    pub actionability: CouponTemplateActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponTemplateMutationResult {
    pub detail: CouponTemplateDetail,
    pub audit: CouponTemplateLifecycleAuditRecord,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponTemplateComparisonFieldChange {
    pub field: String,
    pub source_value: String,
    pub target_value: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponTemplateComparisonResult {
    pub source_coupon_template: CouponTemplateRecord,
    pub target_coupon_template: CouponTemplateRecord,
    pub same_lineage: bool,
    #[serde(default)]
    pub field_changes: Vec<CouponTemplateComparisonFieldChange>,
}

#[derive(Debug, Clone)]
pub struct CloneCouponTemplateRevisionInput {
    pub coupon_template_id: String,
    pub template_key: String,
    pub display_name: Option<String>,
}
