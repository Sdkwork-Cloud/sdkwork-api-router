use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponTemplateRecord, MarketingCampaignRecord,
};

#[derive(Debug, Clone)]
pub struct MarketingCouponContext {
    pub template: CouponTemplateRecord,
    pub campaign: MarketingCampaignRecord,
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
}

#[derive(Debug, Clone, Copy)]
pub struct MarketingCouponContextReference<'a> {
    pub applied_coupon_code: Option<&'a str>,
    pub target_kind: &'a str,
    pub target_id: &'a str,
    pub preferred_marketing_campaign_id: Option<&'a str>,
}
