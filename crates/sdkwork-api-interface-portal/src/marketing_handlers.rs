use super::*;

pub(crate) async fn validate_marketing_coupon_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCouponValidationRequest>,
) -> Result<Json<PortalCouponValidationResponse>, StatusCode> {
    let (workspace, subjects) =
        load_portal_marketing_workspace_and_subjects(&state, &claims).await?;
    let Some(subject_id) = subjects.subject_id_for_scope(request.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let target_kind = request.target_kind.trim();
    if target_kind.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Validate,
        request.subject_scope,
        &subject_id,
        &request.coupon_code,
    )
    .await?;

    let now_ms = current_time_millis();
    let ValidatedCouponResult { context, decision } = validate_coupon_for_subject(
        state.store.as_ref(),
        &request.coupon_code,
        request.subject_scope,
        &subject_id,
        target_kind,
        request.order_amount_minor,
        request.reserve_amount_minor,
        now_ms,
    )
    .await
    .map_err(|error| portal_marketing_operation_status(&error))?;

    Ok(Json(PortalCouponValidationResponse {
        decision: coupon_validation_decision_response(decision),
        template: context.template,
        campaign: context.campaign,
        budget: context.budget,
        code: context.code,
    }))
}

pub(crate) async fn reserve_marketing_coupon_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponReservationRequest>,
) -> Result<(StatusCode, Json<PortalCouponReservationResponse>), StatusCode> {
    let (workspace, subjects) =
        load_portal_marketing_workspace_and_subjects(&state, &claims).await?;
    let target_kind = request.target_kind.trim();
    let Some(subject_id) = subjects.subject_id_for_scope(request.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    if target_kind.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Reserve,
        request.subject_scope,
        &subject_id,
        &request.coupon_code,
    )
    .await?;
    let now_ms = current_time_millis();
    let result = reserve_coupon_for_subject(
        state.store.as_ref(),
        ReserveCouponInput {
            coupon_code: &request.coupon_code,
            subject_scope: request.subject_scope,
            subject_id: &subject_id,
            target_kind,
            order_amount_minor: request.order_amount_minor,
            reserve_amount_minor: request.reserve_amount_minor,
            ttl_ms: request.ttl_ms,
            idempotency_key: idempotency_key.as_deref(),
            now_ms,
        },
    )
    .await
    .map_err(|error| portal_marketing_operation_status(&error))?;

    Ok((
        if result.created {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(PortalCouponReservationResponse {
            reservation: result.reservation,
            template: result.context.template,
            campaign: result.context.campaign,
            budget: result.context.budget,
            code: result.context.code,
        }),
    ))
}

pub(crate) async fn confirm_marketing_coupon_redemption_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponRedemptionConfirmRequest>,
) -> Result<Json<PortalCouponRedemptionConfirmResponse>, StatusCode> {
    let (workspace, subjects) =
        load_portal_marketing_workspace_and_subjects(&state, &claims).await?;

    let reservation_view = portal_marketing_reservation_context_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &request.coupon_reservation_id,
    )
    .await?;
    let reservation_code_value = reservation_view.code.code_value.clone();
    let reservation = reservation_view.reservation;
    let Some(subject_id) = subjects.subject_id_for_scope(reservation.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Confirm,
        reservation.subject_scope,
        &subject_id,
        &reservation_code_value,
    )
    .await?;
    let result = confirm_coupon_for_subject(
        state.store.as_ref(),
        ConfirmCouponInput {
            coupon_reservation_id: &request.coupon_reservation_id,
            subject_scope: reservation.subject_scope,
            subject_id: &subject_id,
            subsidy_amount_minor: request.subsidy_amount_minor,
            order_id: request.order_id.clone(),
            payment_event_id: request.payment_event_id.clone(),
            idempotency_key: idempotency_key.as_deref(),
            now_ms,
        },
    )
    .await
    .map_err(|error| portal_marketing_operation_status(&error))?;

    Ok(Json(PortalCouponRedemptionConfirmResponse {
        reservation: result.reservation,
        redemption: result.redemption,
        budget: result.context.budget,
        code: result.context.code,
    }))
}

pub(crate) async fn rollback_marketing_coupon_redemption_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponRedemptionRollbackRequest>,
) -> Result<Json<PortalCouponRedemptionRollbackResponse>, StatusCode> {
    let (workspace, subjects) =
        load_portal_marketing_workspace_and_subjects(&state, &claims).await?;

    let redemption_view = portal_marketing_redemption_context_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &request.coupon_redemption_id,
    )
    .await?;
    let redemption_code_value = redemption_view.code.code_value.clone();
    let redemption = redemption_view.redemption;
    if request.restored_budget_minor > redemption.budget_consumed_minor {
        return Err(StatusCode::BAD_REQUEST);
    }

    let reservation = redemption_view.reservation;
    let Some(subject_id) = subjects.subject_id_for_scope(reservation.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Rollback,
        reservation.subject_scope,
        &subject_id,
        &redemption_code_value,
    )
    .await?;
    let result = rollback_coupon_for_subject(
        state.store.as_ref(),
        RollbackCouponInput {
            coupon_redemption_id: &request.coupon_redemption_id,
            subject_scope: reservation.subject_scope,
            subject_id: &subject_id,
            rollback_type: request.rollback_type,
            restored_budget_minor: request.restored_budget_minor,
            restored_inventory_count: request.restored_inventory_count,
            idempotency_key: idempotency_key.as_deref(),
            now_ms,
        },
    )
    .await
    .map_err(|error| portal_marketing_operation_status(&error))?;

    Ok(Json(PortalCouponRedemptionRollbackResponse {
        redemption: result.redemption,
        rollback: result.rollback,
        budget: result.context.budget,
        code: result.context.code,
    }))
}

pub(crate) async fn list_my_coupons_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalMarketingCodesResponse>, StatusCode> {
    let (_, subjects) = load_portal_marketing_workspace_and_subjects(&state, &claims).await?;
    let items = load_marketing_code_items(state.store.as_ref(), &subjects).await?;
    let summary = summarize_marketing_code_items(&items);
    Ok(Json(PortalMarketingCodesResponse { summary, items }))
}

pub(crate) async fn list_marketing_reward_history_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalMarketingRewardHistoryItem>>, StatusCode> {
    let (workspace, subjects) =
        load_portal_marketing_workspace_and_subjects(&state, &claims).await?;
    let account_arrival = load_portal_coupon_account_arrival_context(&state, &workspace).await?;
    load_marketing_reward_history_items(state.store.as_ref(), &subjects, Some(&account_arrival))
        .await
        .map(Json)
}

async fn load_portal_coupon_account_arrival_context(
    state: &PortalApiState,
    workspace: &PortalWorkspaceSummary,
) -> Result<PortalCouponAccountArrivalContext, StatusCode> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(PortalCouponAccountArrivalContext::default());
    };

    let account = commercial_billing
        .resolve_payable_account_for_gateway_request_context(&portal_workspace_request_context(
            workspace,
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(account) = account else {
        return Ok(PortalCouponAccountArrivalContext::default());
    };

    let lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();

    Ok(PortalCouponAccountArrivalContext::from_account_lots(
        account.account_id,
        lots,
    ))
}

pub(crate) async fn list_marketing_redemptions_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingRedemptionsQuery>,
) -> Result<Json<PortalMarketingRedemptionsResponse>, StatusCode> {
    let (_, subjects) = load_portal_marketing_workspace_and_subjects(&state, &claims).await?;
    let items =
        load_marketing_redemptions_for_subject(state.store.as_ref(), &subjects, query.status)
            .await?;
    let summary = summarize_marketing_redemptions(&items);
    Ok(Json(PortalMarketingRedemptionsResponse { summary, items }))
}

pub(crate) async fn list_marketing_codes_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingCodesQuery>,
) -> Result<Json<PortalMarketingCodesResponse>, StatusCode> {
    let (_, subjects) = load_portal_marketing_workspace_and_subjects(&state, &claims).await?;
    let mut items = load_marketing_code_items(state.store.as_ref(), &subjects).await?;
    if let Some(status) = query.status {
        items.retain(|item| item.code.status == status);
    }
    let summary = summarize_marketing_code_items(&items);
    Ok(Json(PortalMarketingCodesResponse { summary, items }))
}
