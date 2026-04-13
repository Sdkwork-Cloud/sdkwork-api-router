use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord,
    CouponTemplateRecord, MarketingCampaignRecord,
};

#[derive(Debug, Clone, Default)]
pub struct MarketingOrderEvidenceView {
    pub coupon_reservation: Option<CouponReservationRecord>,
    pub coupon_redemption: Option<CouponRedemptionRecord>,
    pub coupon_rollbacks: Vec<CouponRollbackRecord>,
    pub coupon_code: Option<CouponCodeRecord>,
    pub coupon_template: Option<CouponTemplateRecord>,
    pub marketing_campaign: Option<MarketingCampaignRecord>,
}
