use super::*;
use crate::marketing::{MarketingCampaignComparisonResult, MarketingCampaignMutationResult};

#[utoipa::path(
    get,
    path = "/admin/marketing/campaigns",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical marketing campaigns.", body = [MarketingCampaignRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical marketing campaigns.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_list() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns",
    tag = "marketing",
    request_body = MarketingCampaignRecord,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created canonical marketing campaign.", body = MarketingCampaignRecord),
        (status = 400, description = "Marketing campaign create request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Referenced coupon template not found.", body = ErrorResponse),
        (status = 409, description = "Marketing campaign id already exists.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_create() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/status",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = UpdateMarketingCampaignStatusRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated canonical marketing campaign status.", body = MarketingCampaignRecord),
        (status = 400, description = "Marketing campaign status update request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical marketing campaign status.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_status_update() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/clone",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Source marketing campaign id")),
    request_body = CloneMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Cloned the selected canonical coupon campaign into a governed draft revision.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign clone request is invalid.", body = ErrorResponse),
        (status = 409, description = "Target marketing campaign id already exists.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to clone canonical marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_clone() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/compare",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Source marketing campaign id")),
    request_body = CompareMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Compared two coupon campaign revisions.", body = MarketingCampaignComparisonResult),
        (status = 400, description = "Campaign compare request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to compare canonical marketing campaigns.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_compare() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/submit-for-approval",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = SubmitMarketingCampaignForApprovalRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Submitted the selected coupon campaign revision for approval.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign cannot enter approval from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to submit marketing campaign for approval.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_submit_for_approval() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/approve",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = ApproveMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Approved the selected coupon campaign revision.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign cannot be approved from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to approve marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_approve() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/reject",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = RejectMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Rejected the selected coupon campaign revision.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign cannot be rejected from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to reject marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_reject() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/publish",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = PublishMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Published the selected canonical coupon campaign with semantic lifecycle evidence.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign cannot be published from the current coupon lifecycle state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to publish canonical marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_publish() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/schedule",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = ScheduleMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Scheduled the selected canonical coupon campaign with semantic lifecycle evidence.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign cannot be scheduled from the current coupon lifecycle state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to schedule canonical marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_schedule() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/retire",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    request_body = RetireMarketingCampaignRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Retired the selected canonical coupon campaign with semantic lifecycle evidence.", body = MarketingCampaignMutationResult),
        (status = 400, description = "Campaign cannot be retired from the current coupon lifecycle state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
        (status = 500, description = "Failed to retire canonical marketing campaign.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaigns_retire() {}

#[utoipa::path(
    get,
    path = "/admin/marketing/campaigns/{marketing_campaign_id}/lifecycle-audits",
    tag = "marketing",
    params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Lifecycle audit trail for the selected canonical coupon campaign.", body = [MarketingCampaignLifecycleAuditRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load marketing campaign lifecycle audit trail.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_campaign_lifecycle_audits_list() {}
