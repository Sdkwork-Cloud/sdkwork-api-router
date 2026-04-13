use super::*;
use crate::marketing::CampaignBudgetMutationResult;

#[utoipa::path(
    get,
    path = "/admin/marketing/budgets",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical campaign budgets.", body = [CampaignBudgetRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical campaign budgets.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_budgets_list() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/budgets",
    tag = "marketing",
    request_body = CampaignBudgetRecord,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created canonical campaign budget.", body = CampaignBudgetRecord),
        (status = 400, description = "Campaign budget create request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Referenced marketing campaign not found.", body = ErrorResponse),
        (status = 409, description = "Campaign budget id already exists.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical campaign budget.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_budgets_create() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/budgets/{campaign_budget_id}/status",
    tag = "marketing",
    params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
    request_body = UpdateCampaignBudgetStatusRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated canonical campaign budget status.", body = CampaignBudgetRecord),
        (status = 400, description = "Campaign budget status update request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical campaign budget status.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_budgets_status_update() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/budgets/{campaign_budget_id}/activate",
    tag = "marketing",
    params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
    request_body = ActivateCampaignBudgetRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Activated the selected canonical campaign budget with semantic lifecycle evidence.", body = CampaignBudgetMutationResult),
        (status = 400, description = "Campaign budget cannot be activated from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
        (status = 500, description = "Failed to activate canonical campaign budget.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_budgets_activate() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/budgets/{campaign_budget_id}/close",
    tag = "marketing",
    params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
    request_body = CloseCampaignBudgetRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Closed the selected canonical campaign budget with semantic lifecycle evidence.", body = CampaignBudgetMutationResult),
        (status = 400, description = "Campaign budget cannot be closed from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
        (status = 500, description = "Failed to close canonical campaign budget.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_budgets_close() {}

#[utoipa::path(
    get,
    path = "/admin/marketing/budgets/{campaign_budget_id}/lifecycle-audits",
    tag = "marketing",
    params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Lifecycle audit trail for the selected canonical campaign budget.", body = [CampaignBudgetLifecycleAuditRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load campaign budget lifecycle audit trail.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_budget_lifecycle_audits_list() {}
