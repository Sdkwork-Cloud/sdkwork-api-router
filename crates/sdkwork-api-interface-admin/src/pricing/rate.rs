use super::*;

pub(crate) fn build_pricing_rate_with_status(
    rate: &PricingRateRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingRateRecord {
    PricingRateRecord::new(
        rate.pricing_rate_id,
        rate.tenant_id,
        rate.organization_id,
        rate.pricing_plan_id,
        rate.metric_code.clone(),
    )
    .with_capability_code(rate.capability_code.clone())
    .with_model_code(rate.model_code.clone())
    .with_provider_code(rate.provider_code.clone())
    .with_charge_unit(rate.charge_unit.clone())
    .with_pricing_method(rate.pricing_method.clone())
    .with_quantity_step(rate.quantity_step)
    .with_unit_price(rate.unit_price)
    .with_display_price_unit(rate.display_price_unit.clone())
    .with_minimum_billable_quantity(rate.minimum_billable_quantity)
    .with_minimum_charge(rate.minimum_charge)
    .with_rounding_increment(rate.rounding_increment)
    .with_rounding_mode(rate.rounding_mode.clone())
    .with_included_quantity(rate.included_quantity)
    .with_priority(rate.priority)
    .with_notes(rate.notes.clone())
    .with_status(status.to_owned())
    .with_created_at_ms(rate.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

pub(crate) async fn list_canonical_pricing_rates_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PricingRateRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    rates.sort_by_key(|rate| rate.pricing_rate_id);
    Ok(Json(rates))
}

fn build_canonical_pricing_rate_record(
    pricing_rate_id: u64,
    request: &CreateCommercialPricingRateRequest,
    created_at_ms: u64,
    updated_at_ms: u64,
) -> Result<PricingRateRecord, (StatusCode, Json<ErrorResponse>)> {
    let metric_code = request.metric_code.trim();
    let charge_unit = request.charge_unit.trim();
    let pricing_method = request.pricing_method.trim();
    let display_price_unit = request.display_price_unit.trim();
    let rounding_mode = request.rounding_mode.trim();
    let status = request.status.trim();

    let invalid = metric_code.is_empty()
        || charge_unit.is_empty()
        || pricing_method.is_empty()
        || display_price_unit.is_empty()
        || rounding_mode.is_empty()
        || status.is_empty()
        || request.quantity_step <= 0.0
        || request.unit_price < 0.0
        || request.minimum_billable_quantity < 0.0
        || request.minimum_charge < 0.0
        || request.rounding_increment <= 0.0
        || request.included_quantity < 0.0;

    if invalid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "pricing rate requires metric, charge unit, pricing method, display unit, positive quantity and rounding step, and non-negative commercial amounts",
        ));
    }

    Ok(PricingRateRecord::new(
        pricing_rate_id,
        request.tenant_id,
        request.organization_id,
        request.pricing_plan_id,
        metric_code.to_owned(),
    )
    .with_capability_code(normalize_optional_admin_text(
        request.capability_code.clone(),
    ))
    .with_model_code(normalize_optional_admin_text(request.model_code.clone()))
    .with_provider_code(normalize_optional_admin_text(request.provider_code.clone()))
    .with_charge_unit(charge_unit.to_owned())
    .with_pricing_method(pricing_method.to_owned())
    .with_quantity_step(request.quantity_step)
    .with_unit_price(request.unit_price)
    .with_display_price_unit(display_price_unit.to_owned())
    .with_minimum_billable_quantity(request.minimum_billable_quantity)
    .with_minimum_charge(request.minimum_charge)
    .with_rounding_increment(request.rounding_increment)
    .with_rounding_mode(rounding_mode.to_owned())
    .with_included_quantity(request.included_quantity)
    .with_priority(request.priority)
    .with_notes(normalize_optional_admin_text(request.notes.clone()))
    .with_status(status.to_owned())
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

pub(crate) async fn create_canonical_pricing_rate_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingRateRequest>,
) -> Result<(StatusCode, Json<PricingRateRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let pricing_plan_exists = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .any(|plan| plan.pricing_plan_id == request.pricing_plan_id);
    if !pricing_plan_exists {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {} does not exist", request.pricing_plan_id),
        ));
    }

    let now_ms = unix_timestamp_ms();
    let pricing_rate = build_canonical_pricing_rate_record(
        next_admin_pricing_record_id(now_ms),
        &request,
        now_ms,
        now_ms,
    )?;
    let rate = commercial_billing
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::CREATED, Json(rate)))
}

pub(crate) async fn update_canonical_pricing_rate_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_rate_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingRateRequest>,
) -> Result<(StatusCode, Json<PricingRateRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !plans
        .iter()
        .any(|plan| plan.pricing_plan_id == request.pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {} does not exist", request.pricing_plan_id),
        ));
    }

    let existing_rate = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .find(|rate| rate.pricing_rate_id == pricing_rate_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing rate {pricing_rate_id} does not exist"),
            )
        })?;

    let pricing_rate = build_canonical_pricing_rate_record(
        pricing_rate_id,
        &request,
        existing_rate.created_at_ms,
        unix_timestamp_ms(),
    )?;
    let rate = commercial_billing
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::OK, Json(rate)))
}
