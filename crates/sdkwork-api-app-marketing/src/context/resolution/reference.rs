use super::super::types::MarketingCouponContextReference;
use crate::context::normalize_coupon_code;

pub fn resolve_marketing_coupon_code_for_reference(
    reference: MarketingCouponContextReference<'_>,
) -> Option<String> {
    reference
        .applied_coupon_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_coupon_code)
        .or_else(|| {
            (reference.target_kind == "coupon_redemption")
                .then(|| normalize_coupon_code(reference.target_id))
        })
}
