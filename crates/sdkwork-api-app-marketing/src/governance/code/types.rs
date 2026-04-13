use sdkwork_api_domain_marketing::{
    CouponCodeLifecycleAuditRecord, CouponCodeRecord, CouponTemplateRecord,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponCodeActionDecision {
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponCodeActionability {
    pub disable: CouponCodeActionDecision,
    pub restore: CouponCodeActionDecision,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponCodeDetail {
    pub coupon_code: CouponCodeRecord,
    pub coupon_template: CouponTemplateRecord,
    pub actionability: CouponCodeActionability,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CouponCodeMutationResult {
    pub detail: CouponCodeDetail,
    pub audit: CouponCodeLifecycleAuditRecord,
}
