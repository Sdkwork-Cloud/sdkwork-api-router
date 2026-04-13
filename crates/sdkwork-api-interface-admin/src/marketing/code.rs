use super::support::{
    marketing_governance_error_response, normalized_marketing_lifecycle_reason,
    update_marketing_coupon_code_status,
};
use super::*;

pub(crate) async fn list_marketing_coupon_codes_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCodeRecord>>, StatusCode> {
    list_marketing_coupon_codes(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_marketing_coupon_code_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CouponCodeRecord>,
) -> Result<(StatusCode, Json<CouponCodeRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_coupon_code_record(state.store.as_ref(), record)
        .await
        .map(|record| (StatusCode::CREATED, Json(record)))
        .map_err(marketing_governance_error_response)
}

pub(crate) async fn update_marketing_coupon_code_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCouponCodeStatusRequest>,
) -> Result<Json<CouponCodeRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_coupon_code_status(state.store.as_ref(), &coupon_code_id, request.status)
        .await
        .map(Json)
        .map_err(|(status, message)| error_response(status, message))
}

pub(crate) async fn disable_marketing_coupon_code_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<DisableCouponCodeRequest>,
) -> Result<Json<CouponCodeMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon code")?;
    mutate_marketing_coupon_code_lifecycle(
        state.store.as_ref(),
        &coupon_code_id,
        CouponCodeLifecycleAction::Disable,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn restore_marketing_coupon_code_handler(
    claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Extension(request_id): Extension<RequestId>,
    Json(request): Json<RestoreCouponCodeRequest>,
) -> Result<Json<CouponCodeMutationResult>, (StatusCode, Json<ErrorResponse>)> {
    let reason = normalized_marketing_lifecycle_reason(&request.reason, "coupon code")?;
    mutate_marketing_coupon_code_lifecycle(
        state.store.as_ref(),
        &coupon_code_id,
        CouponCodeLifecycleAction::Restore,
        &claims.claims().sub,
        request_id.as_str(),
        &reason,
    )
    .await
    .map(Json)
    .map_err(marketing_governance_error_response)
}

pub(crate) async fn list_marketing_coupon_code_lifecycle_audits_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCodeLifecycleAuditRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_marketing_coupon_code_lifecycle_audits(state.store.as_ref(), &coupon_code_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "failed to load coupon code lifecycle audits for {coupon_code_id}: {error}"
                ),
            )
        })
}
