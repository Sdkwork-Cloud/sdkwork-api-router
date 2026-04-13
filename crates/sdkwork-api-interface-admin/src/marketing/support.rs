use super::*;

pub(super) fn normalized_required_admin_identifier(
    value: &str,
    field_label: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("{field_label} is required"),
        ));
    }
    Ok(normalized.to_owned())
}

pub(super) fn normalized_marketing_lifecycle_reason(
    reason: &str,
    aggregate_label: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let normalized = reason.trim();
    if normalized.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("{aggregate_label} lifecycle reason is required"),
        ));
    }
    Ok(normalized.to_owned())
}

pub(super) fn marketing_governance_error_response(
    error: MarketingGovernanceError,
) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        MarketingGovernanceError::InvalidInput(message) => {
            error_response(StatusCode::BAD_REQUEST, message)
        }
        MarketingGovernanceError::Conflict(message) => {
            error_response(StatusCode::CONFLICT, message)
        }
        MarketingGovernanceError::NotFound(message) => {
            error_response(StatusCode::NOT_FOUND, message)
        }
        MarketingGovernanceError::Storage(error) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
        }
    }
}

pub(super) async fn update_marketing_coupon_template_status(
    store: &dyn AdminStore,
    coupon_template_id: &str,
    status: CouponTemplateStatus,
) -> Result<CouponTemplateRecord, (StatusCode, String)> {
    let record = store
        .find_coupon_template_record(coupon_template_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("coupon template {coupon_template_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_coupon_template_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon template status".to_owned(),
            )
        })
}

pub(super) async fn update_marketing_campaign_status(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
    status: MarketingCampaignStatus,
) -> Result<MarketingCampaignRecord, (StatusCode, String)> {
    let record = store
        .find_marketing_campaign_record(marketing_campaign_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical marketing campaign".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("marketing campaign {marketing_campaign_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_marketing_campaign_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical marketing campaign status".to_owned(),
            )
        })
}

pub(super) async fn update_marketing_campaign_budget_status(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
    status: CampaignBudgetStatus,
) -> Result<CampaignBudgetRecord, (StatusCode, String)> {
    let record = store
        .find_campaign_budget_record(campaign_budget_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical campaign budget".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("campaign budget {campaign_budget_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_campaign_budget_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical campaign budget status".to_owned(),
            )
        })
}

pub(super) async fn update_marketing_coupon_code_status(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    status: CouponCodeStatus,
) -> Result<CouponCodeRecord, (StatusCode, String)> {
    let record = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon code".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("coupon code {coupon_code_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_coupon_code_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon code status".to_owned(),
            )
        })
}
