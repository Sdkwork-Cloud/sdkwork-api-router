use super::*;

pub(crate) async fn list_payment_methods_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PaymentMethodRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_payment_methods(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn put_payment_method_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(mut payment_method): Json<PaymentMethodRecord>,
) -> Result<Json<PaymentMethodRecord>, (StatusCode, Json<ErrorResponse>)> {
    let normalized_payment_method_id = payment_method_id.trim();
    if normalized_payment_method_id.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "payment_method_id is required",
        ));
    }
    if payment_method.payment_method_id.trim().is_empty() {
        payment_method.payment_method_id = normalized_payment_method_id.to_owned();
    } else if payment_method.payment_method_id != normalized_payment_method_id {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "payment_method_id mismatch between path {} and body {}",
                normalized_payment_method_id, payment_method.payment_method_id
            ),
        ));
    }

    persist_admin_payment_method(state.store.as_ref(), &payment_method)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn delete_payment_method_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_admin_payment_method(state.store.as_ref(), &payment_method_id)
        .await
        .map_err(admin_commerce_error_response)?
    {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(error_response(
            StatusCode::NOT_FOUND,
            format!("payment method {payment_method_id} not found"),
        )),
    }
}

pub(crate) async fn list_payment_method_credential_bindings_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PaymentMethodCredentialBindingRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_payment_method_credential_bindings(state.store.as_ref(), &payment_method_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn replace_payment_method_credential_bindings_handler(
    _claims: AuthenticatedAdminClaims,
    Path(payment_method_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(mut bindings): Json<Vec<PaymentMethodCredentialBindingRecord>>,
) -> Result<Json<Vec<PaymentMethodCredentialBindingRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let normalized_payment_method_id = payment_method_id.trim();
    if normalized_payment_method_id.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "payment_method_id is required",
        ));
    }
    for binding in &mut bindings {
        if binding.payment_method_id.trim().is_empty() {
            binding.payment_method_id = normalized_payment_method_id.to_owned();
        } else if binding.payment_method_id != normalized_payment_method_id {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "binding {} does not belong to payment method {}",
                    binding.binding_id, normalized_payment_method_id
                ),
            ));
        }
    }

    replace_admin_payment_method_credential_bindings(
        state.store.as_ref(),
        normalized_payment_method_id,
        &bindings,
    )
    .await
    .map(Json)
    .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_payment_events_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercePaymentEventRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce payment events for order {order_id}: {error}"),
            )
        })
}

pub(crate) async fn list_commerce_payment_attempts_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercePaymentAttemptRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_payment_attempts_for_order(state.store.as_ref(), &order_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn list_commerce_refunds_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceRefundRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_commerce_refunds_for_order(state.store.as_ref(), &order_id)
        .await
        .map(Json)
        .map_err(admin_commerce_error_response)
}

pub(crate) async fn create_commerce_refund_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<AdminCommerceRefundCreateRequest>,
) -> Result<Json<CommerceRefundRecord>, (StatusCode, Json<ErrorResponse>)> {
    create_admin_commerce_refund(
        state.store.as_ref(),
        state.commercial_billing.as_deref(),
        &state.secret_manager,
        &order_id,
        &request,
    )
    .await
    .map(Json)
    .map_err(admin_commerce_error_response)
}
