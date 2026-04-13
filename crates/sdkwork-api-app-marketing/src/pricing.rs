use sdkwork_api_domain_marketing::{CouponBenefitSpec, MarketingBenefitKind};

pub fn compute_coupon_subsidy_minor(list_price_minor: u64, benefit: &CouponBenefitSpec) -> u64 {
    let subsidy = match benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => benefit
            .discount_percent
            .map(|percent| list_price_minor.saturating_mul(percent as u64) / 100)
            .unwrap_or(0),
        MarketingBenefitKind::FixedAmountOff => benefit.discount_amount_minor.unwrap_or(0),
        MarketingBenefitKind::GrantUnits => 0,
    };

    subsidy
        .min(benefit.max_discount_minor.unwrap_or(u64::MAX))
        .min(list_price_minor)
}

pub fn compute_coupon_reserve_amount_minor(
    list_price_minor: u64,
    benefit: &CouponBenefitSpec,
) -> u64 {
    let subsidy_amount_minor = compute_coupon_subsidy_minor(list_price_minor, benefit);
    if subsidy_amount_minor > 0 {
        subsidy_amount_minor
    } else if matches!(benefit.benefit_kind, MarketingBenefitKind::GrantUnits) {
        1
    } else {
        0
    }
}
