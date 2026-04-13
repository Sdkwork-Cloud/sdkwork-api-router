use super::super::types::{MarketingCodeSummary, MarketingCodeView, MarketingRedemptionSummary};
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponCodeStatus, CouponRedemptionRecord, CouponRedemptionStatus,
};

pub fn summarize_coupon_redemptions(
    items: &[CouponRedemptionRecord],
) -> MarketingRedemptionSummary {
    let mut summary = MarketingRedemptionSummary {
        total_count: items.len(),
        ..MarketingRedemptionSummary::default()
    };
    for item in items {
        match item.redemption_status {
            CouponRedemptionStatus::Redeemed => summary.redeemed_count += 1,
            CouponRedemptionStatus::PartiallyRolledBack => summary.partially_rolled_back_count += 1,
            CouponRedemptionStatus::RolledBack => summary.rolled_back_count += 1,
            CouponRedemptionStatus::Failed => summary.failed_count += 1,
            CouponRedemptionStatus::Pending => {}
        }
    }
    summary
}

pub fn summarize_coupon_code_views(items: &[MarketingCodeView]) -> MarketingCodeSummary {
    let codes = items
        .iter()
        .map(|item| item.context.code.clone())
        .collect::<Vec<_>>();
    summarize_coupon_codes(&codes)
}

pub fn summarize_coupon_codes(items: &[CouponCodeRecord]) -> MarketingCodeSummary {
    let mut summary = MarketingCodeSummary {
        total_count: items.len(),
        ..MarketingCodeSummary::default()
    };
    for item in items {
        match item.status {
            CouponCodeStatus::Available => summary.available_count += 1,
            CouponCodeStatus::Reserved => summary.reserved_count += 1,
            CouponCodeStatus::Redeemed => summary.redeemed_count += 1,
            CouponCodeStatus::Disabled => summary.disabled_count += 1,
            CouponCodeStatus::Expired => summary.expired_count += 1,
        }
    }
    summary
}
