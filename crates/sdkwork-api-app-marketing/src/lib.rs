mod context;
mod create;
mod governance;
mod idempotency;
mod kernel;
mod operations;
mod policy;
mod pricing;
mod query;
mod recovery;
mod state;

pub use context::{
    coupon_context_is_catalog_visible, load_marketing_coupon_context_by_value,
    load_marketing_coupon_context_for_code_id, load_marketing_coupon_context_for_reference,
    load_marketing_coupon_context_from_code_record, marketing_coupon_context_remaining_inventory,
    normalize_coupon_code, reclaim_expired_coupon_reservations_for_code_if_needed,
    reclaim_expired_coupon_reservations_for_code_record_if_needed,
    resolve_marketing_coupon_code_for_reference, select_campaign_budget_record,
    select_effective_marketing_campaign, MarketingCouponContext, MarketingCouponContextReference,
};
pub use create::{
    create_campaign_budget_record, create_coupon_code_record, create_coupon_template_record,
    create_marketing_campaign_record, ensure_campaign_budget_record, ensure_coupon_code_record,
    ensure_coupon_template_record, ensure_marketing_campaign_record,
};
pub use governance::{
    clone_marketing_campaign_revision, clone_marketing_coupon_template_revision,
    compare_marketing_campaign_revisions, compare_marketing_coupon_template_revisions,
    mutate_marketing_campaign_budget_lifecycle, mutate_marketing_campaign_lifecycle,
    mutate_marketing_coupon_code_lifecycle, mutate_marketing_coupon_template_lifecycle,
    CampaignBudgetActionDecision, CampaignBudgetActionability, CampaignBudgetDetail,
    CampaignBudgetMutationResult, CloneCouponTemplateRevisionInput,
    CloneMarketingCampaignRevisionInput, CouponCodeActionDecision, CouponCodeActionability,
    CouponCodeDetail, CouponCodeMutationResult, CouponTemplateActionDecision,
    CouponTemplateActionability, CouponTemplateComparisonFieldChange,
    CouponTemplateComparisonResult, CouponTemplateDetail, CouponTemplateMutationResult,
    MarketingCampaignActionDecision, MarketingCampaignActionability,
    MarketingCampaignComparisonFieldChange, MarketingCampaignComparisonResult,
    MarketingCampaignDetail, MarketingCampaignMutationResult, MarketingGovernanceError,
};
pub use idempotency::{
    derive_coupon_redemption_id, derive_coupon_reservation_id, derive_coupon_rollback_id,
    marketing_idempotency_fingerprint, marketing_subject_scope_token, normalize_idempotency_key,
    resolve_idempotency_key, MarketingIdempotencyError,
};
pub use kernel::{
    confirm_coupon_redemption, reserve_coupon_redemption, rollback_coupon_redemption,
    validate_coupon_stack, CouponValidationDecision, MarketingServiceError,
};
pub use operations::{
    confirm_coupon_for_subject, release_coupon_for_subject, reserve_coupon_for_subject,
    rollback_coupon_for_subject, validate_coupon_for_subject, ConfirmCouponInput,
    ConfirmCouponResult, MarketingOperationError, ReleaseCouponInput, ReleaseCouponResult,
    ReserveCouponInput, ReserveCouponResult, RollbackCouponInput, RollbackCouponResult,
    ValidatedCouponResult,
};
pub use policy::{marketing_target_kind_allowed, validate_marketing_coupon_context};
pub use pricing::{compute_coupon_reserve_amount_minor, compute_coupon_subsidy_minor};
pub use query::{
    list_catalog_visible_coupon_contexts, list_catalog_visible_coupon_views,
    list_coupon_code_views_for_subjects, list_coupon_redemptions_for_subjects,
    list_coupon_reward_history_views_for_subjects, list_marketing_campaign_budget_lifecycle_audits,
    list_marketing_campaign_budgets, list_marketing_campaign_lifecycle_audits,
    list_marketing_campaigns, list_marketing_coupon_code_lifecycle_audits,
    list_marketing_coupon_codes, list_marketing_coupon_redemptions,
    list_marketing_coupon_reservations, list_marketing_coupon_rollbacks,
    list_marketing_coupon_template_lifecycle_audits, list_marketing_coupon_templates,
    load_catalog_visible_coupon_resolution_by_value,
    load_coupon_redemption_context_owned_by_subject, load_coupon_redemption_owned_by_subject,
    load_coupon_reservation_context_owned_by_subject, load_coupon_reservation_owned_by_subject,
    load_marketing_order_evidence, marketing_catalog_coupon_view_from_context,
    summarize_coupon_code_views, summarize_coupon_codes, summarize_coupon_redemptions,
    MarketingCatalogCouponResolution, MarketingCatalogCouponView, MarketingCodeSummary,
    MarketingCodeView, MarketingOrderEvidenceView, MarketingRedemptionOwnershipView,
    MarketingRedemptionSummary, MarketingReservationOwnershipView, MarketingRewardHistoryView,
    MarketingSubjectSet,
};
pub use recovery::{recover_expired_coupon_reservations, MarketingRecoveryRunReport};
pub use state::{
    code_after_confirmation, code_after_release, code_after_reservation, code_after_rollback,
    confirm_campaign_budget, release_campaign_budget, reserve_campaign_budget,
    restore_coupon_code_availability, rollback_campaign_budget,
};
