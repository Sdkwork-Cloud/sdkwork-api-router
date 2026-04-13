use crate::MarketingCouponContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketingCatalogCouponView {
    pub id: String,
    pub code: String,
    pub discount_label: String,
    pub audience: String,
    pub remaining: u64,
    pub active: bool,
    pub note: String,
    pub expires_on: String,
    pub source: String,
    pub discount_percent: Option<u8>,
    pub bonus_units: u64,
}

#[derive(Debug, Clone)]
pub struct MarketingCatalogCouponResolution {
    pub context: MarketingCouponContext,
    pub view: MarketingCatalogCouponView,
}
