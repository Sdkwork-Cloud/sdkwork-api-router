use super::*;
use axum::Extension;
use sdkwork_api_app_marketing::{
    clone_marketing_campaign_revision, clone_marketing_coupon_template_revision,
    compare_marketing_campaign_revisions, compare_marketing_coupon_template_revisions,
    create_campaign_budget_record, create_coupon_code_record, create_coupon_template_record,
    create_marketing_campaign_record, list_marketing_campaign_budget_lifecycle_audits,
    list_marketing_campaign_budgets, list_marketing_campaign_lifecycle_audits,
    list_marketing_campaigns, list_marketing_coupon_code_lifecycle_audits,
    list_marketing_coupon_codes, list_marketing_coupon_redemptions,
    list_marketing_coupon_reservations, list_marketing_coupon_rollbacks,
    list_marketing_coupon_template_lifecycle_audits, list_marketing_coupon_templates,
    mutate_marketing_campaign_budget_lifecycle, mutate_marketing_campaign_lifecycle,
    mutate_marketing_coupon_code_lifecycle, mutate_marketing_coupon_template_lifecycle,
    CloneCouponTemplateRevisionInput, CloneMarketingCampaignRevisionInput,
    MarketingGovernanceError,
};
use sdkwork_api_observability::RequestId;

mod budget;
mod campaign;
mod code;
mod runtime;
mod support;
mod template;

pub(crate) use budget::{
    activate_marketing_campaign_budget_handler, close_marketing_campaign_budget_handler,
    create_marketing_budget_handler, list_marketing_budgets_handler,
    list_marketing_campaign_budget_lifecycle_audits_handler,
    update_marketing_budget_status_handler,
};
pub(crate) use campaign::{
    approve_marketing_campaign_handler, clone_marketing_campaign_handler,
    compare_marketing_campaigns_handler, create_marketing_campaign_handler,
    list_marketing_campaign_lifecycle_audits_handler, list_marketing_campaigns_handler,
    publish_marketing_campaign_handler, reject_marketing_campaign_handler,
    retire_marketing_campaign_handler, schedule_marketing_campaign_handler,
    submit_marketing_campaign_for_approval_handler, update_marketing_campaign_status_handler,
};
pub(crate) use code::{
    create_marketing_coupon_code_handler, disable_marketing_coupon_code_handler,
    list_marketing_coupon_code_lifecycle_audits_handler, list_marketing_coupon_codes_handler,
    restore_marketing_coupon_code_handler, update_marketing_coupon_code_status_handler,
};
pub(crate) use runtime::{
    list_marketing_coupon_redemptions_handler, list_marketing_coupon_reservations_handler,
    list_marketing_coupon_rollbacks_handler,
};
pub(crate) use sdkwork_api_app_marketing::{
    CampaignBudgetMutationResult, CouponCodeMutationResult, CouponTemplateComparisonResult,
    CouponTemplateMutationResult, MarketingCampaignComparisonResult,
    MarketingCampaignMutationResult,
};
pub(crate) use template::{
    approve_marketing_coupon_template_handler, clone_marketing_coupon_template_handler,
    compare_marketing_coupon_templates_handler, create_marketing_coupon_template_handler,
    list_marketing_coupon_template_lifecycle_audits_handler,
    list_marketing_coupon_templates_handler, publish_marketing_coupon_template_handler,
    reject_marketing_coupon_template_handler, retire_marketing_coupon_template_handler,
    schedule_marketing_coupon_template_handler,
    submit_marketing_coupon_template_for_approval_handler,
    update_marketing_coupon_template_status_handler,
};
