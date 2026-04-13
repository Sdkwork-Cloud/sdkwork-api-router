use super::types::{CouponCodeActionDecision, CouponCodeActionability, CouponCodeDetail};
use sdkwork_api_domain_marketing::{CouponCodeRecord, CouponCodeStatus, CouponTemplateRecord};

fn allowed_coupon_code_action() -> CouponCodeActionDecision {
    CouponCodeActionDecision {
        allowed: true,
        reasons: Vec::new(),
    }
}

fn blocked_coupon_code_action(reason: impl Into<String>) -> CouponCodeActionDecision {
    CouponCodeActionDecision {
        allowed: false,
        reasons: vec![reason.into()],
    }
}

fn coupon_code_is_expired(coupon_code: &CouponCodeRecord, now_ms: u64) -> bool {
    coupon_code.status == CouponCodeStatus::Expired
        || coupon_code
            .expires_at_ms
            .is_some_and(|value| value <= now_ms)
}

pub(super) fn build_coupon_code_actionability(
    coupon_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeActionability {
    let expired = coupon_code_is_expired(coupon_code, now_ms);
    let disable = if expired {
        blocked_coupon_code_action("coupon code is expired and cannot be disabled")
    } else {
        match coupon_code.status {
            CouponCodeStatus::Available => allowed_coupon_code_action(),
            CouponCodeStatus::Disabled => {
                blocked_coupon_code_action("coupon code is already disabled")
            }
            CouponCodeStatus::Reserved => blocked_coupon_code_action(
                "reserved coupon code is governed by runtime and cannot be disabled",
            ),
            CouponCodeStatus::Redeemed => {
                blocked_coupon_code_action("redeemed coupon code cannot be disabled")
            }
            CouponCodeStatus::Expired => {
                blocked_coupon_code_action("coupon code is expired and cannot be disabled")
            }
        }
    };
    let restore = if expired {
        blocked_coupon_code_action("coupon code is expired and cannot be restored")
    } else {
        match coupon_code.status {
            CouponCodeStatus::Disabled => allowed_coupon_code_action(),
            CouponCodeStatus::Available => {
                blocked_coupon_code_action("coupon code is already available")
            }
            CouponCodeStatus::Reserved => blocked_coupon_code_action(
                "reserved coupon code is governed by runtime and cannot be restored",
            ),
            CouponCodeStatus::Redeemed => {
                blocked_coupon_code_action("redeemed coupon code cannot be restored")
            }
            CouponCodeStatus::Expired => {
                blocked_coupon_code_action("coupon code is expired and cannot be restored")
            }
        }
    };
    CouponCodeActionability { disable, restore }
}

pub(super) fn build_coupon_code_detail(
    coupon_code: CouponCodeRecord,
    coupon_template: CouponTemplateRecord,
    now_ms: u64,
) -> CouponCodeDetail {
    let actionability = build_coupon_code_actionability(&coupon_code, now_ms);
    CouponCodeDetail {
        coupon_code,
        coupon_template,
        actionability,
    }
}
