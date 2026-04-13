use super::support::{
    marketing_governance_error_response, normalized_marketing_lifecycle_reason,
    update_marketing_campaign_budget_status,
};
use super::*;

pub(crate) async fn list_marketing_budgets_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CampaignBudgetRecord>>, StatusCode> {
    list_marketing_campaign_budgets(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_marketing_budget_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CampaignBudgetRecord>,
) -> Result<(StatusCode, Json<CampaignBudgetRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_campaign_budget_record(state.store.as_ref(), record)
        .await
        .map(|record| (StatusCode::CREATED, Json(record)))
        .map_err(marketing_governance_error_response)
}

pub(crate) async fn update_marketing_budget_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCampaignBudgetStatusRequest>,
) -> Result<Json<CampaignBudgetRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_campaign_budget_status(
        state.store.as_ref(),
        &campaign_budget_id,
        request.status,
    )
    .await
    .map(Json)
    .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn activate_marketing_campaign_budget_handler(
    claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ActivateCampaignBudgetRequest>,
) -> Result<Json<CampaignBudgetMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign budget")?;
    mutate_marketing_campaign_budget_lifecycle(
        state.store.as_ref(),
        &campaign_budget_id,
        CampaignBudgetLifecycleAction::Activate,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn close_marketing_campaign_budget_handler(
    claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<CloseCampaignBudgetRequest>,
) -> Result<Json<CampaignBudgetMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign budget")?;
    mutate_marketing_campaign_budget_lifecycle(
        state.store.as_ref(),
        &campaign_budget_id,
        CampaignBudgetLifecycleAction::Close,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn list_marketing_campaign_budget_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CampaignBudgetLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_marketing_campaign_budget_lifecycle_audits(state.store.as_ref(), &campaign_budget_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load campaign budget lifecycle audits for {campaign_budget_id}: {error}"
                ),
            )
        })
}
