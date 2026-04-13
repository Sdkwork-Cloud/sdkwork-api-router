mod actionability;
mod audit;
mod comparison;
mod lookup;
mod mutation;
mod types;

pub use comparison::compare_marketing_coupon_template_revisions;
pub use mutation::{
    clone_marketing_coupon_template_revision, mutate_marketing_coupon_template_lifecycle,
};
pub use types::{
    CloneCouponTemplateRevisionInput, CouponTemplateActionDecision, CouponTemplateActionability,
    CouponTemplateComparisonFieldChange, CouponTemplateComparisonResult, CouponTemplateDetail,
    CouponTemplateMutationResult,
};
