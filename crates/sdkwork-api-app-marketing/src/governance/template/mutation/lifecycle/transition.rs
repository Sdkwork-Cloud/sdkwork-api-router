use sdkwork_api_domain_marketing::{
    CouponTemplateApprovalState, CouponTemplateLifecycleAction, CouponTemplateRecord,
    CouponTemplateStatus,
};

pub(super) fn resolve_coupon_template_lifecycle_transition(
    coupon_template: &CouponTemplateRecord,
    action: CouponTemplateLifecycleAction,
) -> (CouponTemplateStatus, CouponTemplateApprovalState) {
    match action {
        CouponTemplateLifecycleAction::Clone => unreachable!("clone uses dedicated helper"),
        CouponTemplateLifecycleAction::SubmitForApproval => (
            coupon_template.status,
            CouponTemplateApprovalState::InReview,
        ),
        CouponTemplateLifecycleAction::Approve => (
            coupon_template.status,
            CouponTemplateApprovalState::Approved,
        ),
        CouponTemplateLifecycleAction::Reject => (
            coupon_template.status,
            CouponTemplateApprovalState::Rejected,
        ),
        CouponTemplateLifecycleAction::Publish => {
            (CouponTemplateStatus::Active, coupon_template.approval_state)
        }
        CouponTemplateLifecycleAction::Schedule => (
            CouponTemplateStatus::Scheduled,
            coupon_template.approval_state,
        ),
        CouponTemplateLifecycleAction::Retire => (
            CouponTemplateStatus::Archived,
            coupon_template.approval_state,
        ),
    }
}
