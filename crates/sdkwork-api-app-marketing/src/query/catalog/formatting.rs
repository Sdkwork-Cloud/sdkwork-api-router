use super::types::MarketingCatalogCouponView;
use crate::{
    coupon_context_is_catalog_visible, marketing_coupon_context_remaining_inventory,
    normalize_coupon_code, MarketingCouponContext,
};
use sdkwork_api_domain_marketing::{
    CouponBenefitSpec, MarketingBenefitKind, MarketingSubjectScope,
};

pub fn marketing_catalog_coupon_view_from_context(
    context: &MarketingCouponContext,
    now_ms: u64,
) -> MarketingCatalogCouponView {
    MarketingCatalogCouponView {
        id: context.code.coupon_code_id.clone(),
        code: normalize_coupon_code(&context.code.code_value),
        discount_label: format_marketing_discount_label(&context.template.benefit),
        audience: marketing_subject_scope_label(context.template.restriction.subject_scope)
            .to_owned(),
        remaining: marketing_coupon_context_remaining_inventory(context, now_ms),
        active: coupon_context_is_catalog_visible(context, now_ms),
        note: if context.template.display_name.trim().is_empty() {
            context.campaign.display_name.clone()
        } else {
            context.template.display_name.clone()
        },
        expires_on: format_marketing_expires_on(context),
        source: "marketing".to_owned(),
        discount_percent: context.template.benefit.discount_percent,
        bonus_units: context.template.benefit.grant_units.unwrap_or(0),
    }
}

fn format_marketing_discount_label(benefit: &CouponBenefitSpec) -> String {
    match benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => benefit
            .discount_percent
            .map(|percent| format!("{percent}% off"))
            .unwrap_or_else(|| "percentage off".to_owned()),
        MarketingBenefitKind::FixedAmountOff => benefit
            .discount_amount_minor
            .map(format_minor_price_label)
            .map(|label| format!("{label} off"))
            .unwrap_or_else(|| "fixed amount off".to_owned()),
        MarketingBenefitKind::GrantUnits => benefit
            .grant_units
            .map(|units| format!("+{} bonus units", format_integer_with_commas(units)))
            .unwrap_or_else(|| "bonus units".to_owned()),
    }
}

fn format_marketing_expires_on(context: &MarketingCouponContext) -> String {
    context
        .code
        .expires_at_ms
        .or(context.campaign.end_at_ms)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "rolling".to_owned())
}

fn marketing_subject_scope_label(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

fn format_minor_price_label(price_minor: u64) -> String {
    format!("${}.{:02}", price_minor / 100, price_minor % 100)
}

fn format_integer_with_commas(value: u64) -> String {
    let digits = value.to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}
