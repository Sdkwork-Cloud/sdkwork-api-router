use super::support::{
    marketing_governance_error_response, normalized_marketing_lifecycle_reason,
    normalized_required_admin_identifier, update_marketing_coupon_template_status,
};
use super::*;

pub(crate) async fn list_marketing_coupon_templates_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponTemplateRecord>>, StatusCode> {
    list_marketing_coupon_templates(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_marketing_coupon_template_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CouponTemplateRecord>,
) -> Result<(StatusCode, Json<CouponTemplateRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_coupon_template_record(state.store.as_ref(), record)
        .await
        .map(|record| (StatusCode::CREATED, Json(record)))
        .map_err(marketing_governance_error_response)
}

pub(crate) async fn update_marketing_coupon_template_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCouponTemplateStatusRequest>,
) -> Result<Json<CouponTemplateRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_coupon_template_status(
        state.store.as_ref(),
        &coupon_template_id,
        request.status,
    )
    .await
    .map(Json)
    .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn clone_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<CloneCouponTemplateRequest>,
) -> Result<(StatusCode, Json<CouponTemplateMutationResult>), (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    clone_marketing_coupon_template_revision(
        state.store.as_ref(),
        &coupon_template_id,
        CloneCouponTemplateRevisionInput {
            coupon_template_id: normalized_required_admin_identifier(
                &request.coupon_template_id,
                "coupon_template_id",
            )?,
            template_key: normalized_required_admin_identifier(
                &request.template_key,
                "template_key",
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

pub(crate) async fn compare_marketing_coupon_templates_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<CompareCouponTemplateRequest>,
) -> Result<Json<CouponTemplateComparisonResult>, (StatusCode, Json<ErrorResponse>)> {
    compare_marketing_coupon_template_revisions(
        state.store.as_ref(),
        &coupon_template_id,
        &request.target_coupon_template_id,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn submit_marketing_coupon_template_for_approval_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<SubmitCouponTemplateForApprovalRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::SubmitForApproval,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn approve_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ApproveCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Approve,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn reject_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RejectCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Reject,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn publish_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<PublishCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Publish,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn schedule_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<ScheduleCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Schedule,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn retire_marketing_coupon_template_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RetireCouponTemplateRequest>,
) -> Result<Json<CouponTemplateMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon template")?;
    mutate_marketing_coupon_template_lifecycle(
        state.store.as_ref(),
        &coupon_template_id,
        CouponTemplateLifecycleAction::Retire,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn list_marketing_coupon_template_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponTemplateLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_marketing_coupon_template_lifecycle_audits(state.store.as_ref(), &coupon_template_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load coupon template lifecycle audits for {coupon_template_id}: {error}"
                ),
            )
        })
}
