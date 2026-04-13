use crate::{validate_coupon_stack, CouponValidationDecision, MarketingCouponContext};
use sdkwork_api_domain_marketing::{CouponCodeRecord, CouponTemplateRecord, MarketingSubjectScope};

pub fn marketing_target_kind_allowed(template: &CouponTemplateRecord, target_kind: &str) -> bool {
    template.restriction.eligible_target_kinds.is_empty()
        || template
            .restriction
            .eligible_target_kinds
            .iter()
            .any(|eligible| eligible == target_kind)
}

fn coupon_code_matches_subject(
    code: &CouponCodeRecord,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
) -> CouponValidationDecision {
    match (
        code.claimed_subject_scope,
        code.claimed_subject_id.as_deref(),
    ) {
        (None, None) => CouponValidationDecision::eligible(0),
        (Some(scope), Some(claimed_subject_id))
            if scope == subject_scope && claimed_subject_id == subject_id =>
        {
            CouponValidationDecision::eligible(0)
        }
        (Some(_), Some(_)) => {
            CouponValidationDecision::rejected("coupon_code_claimed_by_another_subject")
        }
        _ => CouponValidationDecision::rejected("coupon_code_claim_is_invalid"),
    }
}

pub fn validate_marketing_coupon_context(
    context: &MarketingCouponContext,
    target_kind: &str,
    now_ms: u64,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
    subject: Option<(MarketingSubjectScope, &str)>,
) -> CouponValidationDecision {
    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        order_amount_minor,
        reserve_amount_minor,
    );
    if !decision.eligible {
        return decision;
    }
    if !marketing_target_kind_allowed(&context.template, target_kind) {
        return CouponValidationDecision::rejected("target_kind_not_eligible");
    }
    if let Some((subject_scope, subject_id)) = subject {
        if context.template.restriction.subject_scope != subject_scope {
            return CouponValidationDecision::rejected("subject_scope_not_eligible");
        }
        let ownership_decision =
            coupon_code_matches_subject(&context.code, subject_scope, subject_id);
        if !ownership_decision.eligible {
            return ownership_decision;
        }
    }

    decision
}
