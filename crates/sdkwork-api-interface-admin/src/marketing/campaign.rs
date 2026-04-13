use super::support::{
    marketing_governance_error_response, normalized_marketing_lifecycle_reason,
    normalized_required_admin_identifier, update_marketing_campaign_status,
};
use super::*;

pub(crate) async fn list_marketing_campaigns_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<MarketingCampaignRecord>>, StatusCode> {
    list_marketing_campaigns(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_marketing_campaign_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<MarketingCampaignRecord>,
) -> Result<(StatusCode, Json<MarketingCampaignRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_marketing_campaign_record(state.store.as_ref(), record)
        .await
        .map(|record| (StatusCode::CREATED, Json(record)))
        .map_err(marketing_governance_error_response)
}

pub(crate) async fn update_marketing_campaign_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateMarketingCampaignStatusRequest>,
) -> Result<Json<MarketingCampaignRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_campaign_status(state.store.as_ref(), &marketing_campaign_id, request.status)
        .await
        .map(Json)
        .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn clone_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<CloneMarketingCampaignRequest>,
) -> Result<(StatusCode, Json<MarketingCampaignMutationResult>), (StatusCode, Json<ErrorResponse>)>
{
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    clone_marketing_campaign_revision(
        state.store.as_ref(),
        &marketing_campaign_id,
        CloneMarketingCampaignRevisionInput {
            marketing_campaign_id: normalized_required_admin_identifier(
                &request.marketing_campaign_id,
                "marketing_campaign_id",
            )?,
            display_name: request.display_name,
        },
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(|result| (StatusCode::CREATED, Json(result)))
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn compare_marketing_campaigns_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<CompareMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignComparisonResult>, (StatusCode, Json<ErrorResponse>)> {
    compare_marketing_campaign_revisions(
        state.store.as_ref(),
        &marketing_campaign_id,
        &request.target_marketing_campaign_id,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn submit_marketing_campaign_for_approval_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<SubmitMarketingCampaignForApprovalRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::SubmitForApproval,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn approve_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ApproveMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Approve,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn reject_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RejectMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Reject,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn publish_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<PublishMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Publish,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn schedule_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ScheduleMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Schedule,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn retire_marketing_campaign_handler(
    claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RetireMarketingCampaignRequest>,
) -> Result<Json<MarketingCampaignMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "campaign")?;
    mutate_marketing_campaign_lifecycle(
        state.store.as_ref(),
        &marketing_campaign_id,
        MarketingCampaignLifecycleAction::Retire,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn list_marketing_campaign_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<MarketingCampaignLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_marketing_campaign_lifecycle_audits(state.store.as_ref(), &marketing_campaign_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load marketing campaign lifecycle audits for {marketing_campaign_id}: {error}"
                ),
            )
        })
}
