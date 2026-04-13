use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct RecentCommerceOrdersQuery {
    #[serde(default)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct CommerceOrderAuditRecord {
    pub(crate) order: CommerceOrderRecord,
    pub(crate) payment_events: Vec<CommercePaymentEventRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_reservation: Option<CouponReservationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_redemption: Option<CouponRedemptionRecord>,
    #[serde(default)]
    pub(crate) coupon_rollbacks: Vec<CouponRollbackRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_code: Option<CouponCodeRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) coupon_template: Option<CouponTemplateRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) marketing_campaign: Option<MarketingCampaignRecord>,
}

fn clamp_recent_commerce_orders_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(limit) if limit > 0 => limit.min(100),
        _ => 24,
    }
}

pub(crate) async fn list_recent_commerce_orders_handler(
    _claims: AuthenticatedAdminClaims,
    Query(query): Query<RecentCommerceOrdersQuery>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceOrderRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_recent_commerce_orders(clamp_recent_commerce_orders_limit(query.limit))
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load recent commerce orders: {error}"),
            )
        })
}

pub(crate) async fn get_commerce_order_audit_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<CommerceOrderAuditRecord>, (StatusCode, Json<ErrorResponse>)> {
    let order = state
        .store
        .list_commerce_orders()
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce order {order_id}: {error}"),
            )
        })?
        .into_iter()
        .find(|order| order.order_id == order_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("commerce order {order_id} not found"),
            )
        })?;

    let mut payment_events = state
        .store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce payment events for order {order_id}: {error}"),
            )
        })?;
    payment_events.sort_by(|left, right| {
        right
            .processed_at_ms
            .unwrap_or(right.received_at_ms)
            .cmp(&left.processed_at_ms.unwrap_or(left.received_at_ms))
            .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
    });

    let marketing_evidence = load_marketing_order_evidence(
        state.store.as_ref(),
        order.coupon_reservation_id.as_deref(),
        order.coupon_redemption_id.as_deref(),
        order.applied_coupon_code.as_deref(),
        order.marketing_campaign_id.as_deref(),
    )
    .await
    .map_err(|error| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to load marketing evidence for order {order_id}: {error}"),
        )
    })?;

    Ok(Json(CommerceOrderAuditRecord {
        order,
        payment_events,
        coupon_reservation: marketing_evidence.coupon_reservation,
        coupon_redemption: marketing_evidence.coupon_redemption,
        coupon_rollbacks: marketing_evidence.coupon_rollbacks,
        coupon_code: marketing_evidence.coupon_code,
        coupon_template: marketing_evidence.coupon_template,
        marketing_campaign: marketing_evidence.marketing_campaign,
    }))
}
