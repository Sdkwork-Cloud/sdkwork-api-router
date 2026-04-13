use super::*;

pub(crate) async fn synchronize_canonical_pricing_lifecycle_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<PricingLifecycleSynchronizationReport>, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let report =
        synchronize_due_pricing_plan_lifecycle_with_report(commercial_billing.as_ref(), now_ms)
            .await
            .map_err(commercial_billing_error_response)?;
    Ok(Json(report))
}
