mod recovery;
mod resolution;
mod types;
mod visibility;

pub use recovery::{
    reclaim_expired_coupon_reservations_for_code_if_needed,
    reclaim_expired_coupon_reservations_for_code_record_if_needed,
};
pub use resolution::{
    load_marketing_coupon_context_by_value, load_marketing_coupon_context_for_code_id,
    load_marketing_coupon_context_for_reference, load_marketing_coupon_context_from_code_record,
    resolve_marketing_coupon_code_for_reference, select_campaign_budget_record,
    select_effective_marketing_campaign,
};
pub use sdkwork_api_domain_marketing::normalize_coupon_code;
pub use types::{MarketingCouponContext, MarketingCouponContextReference};
pub use visibility::{
    coupon_context_is_catalog_visible, marketing_coupon_context_remaining_inventory,
};
