use super::decision::CouponValidationDecision;
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponTemplateRecord, CouponTemplateStatus,
    MarketingCampaignRecord,
};

pub fn validate_coupon_stack(
    template: &CouponTemplateRecord,
    campaign: &MarketingCampaignRecord,
    budget: &CampaignBudgetRecord,
    code: &CouponCodeRecord,
    now_ms: u64,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
) -> CouponValidationDecision {
    if template.status != CouponTemplateStatus::Active {
        return CouponValidationDecision::rejected("template_not_active");
    }
    if !campaign.is_effective_at(now_ms) {
        return CouponValidationDecision::rejected("campaign_not_effective");
    }
    if let Some(min_order_amount_minor) = template.restriction.min_order_amount_minor {
        if order_amount_minor < min_order_amount_minor {
            return CouponValidationDecision::rejected("order_amount_below_minimum");
        }
    }
    if !budget.can_reserve(reserve_amount_minor) {
        return CouponValidationDecision::rejected("budget_unavailable");
    }
    if !code.is_redeemable_at(now_ms) {
        return CouponValidationDecision::rejected("coupon_code_unavailable");
    }

    CouponValidationDecision::eligible(reserve_amount_minor)
}
