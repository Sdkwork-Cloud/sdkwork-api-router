use super::*;

pub(crate) async fn load_coupon_catalog(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<PortalCommerceCoupon>> {
    Ok(load_coupon_definitions(store)
        .await?
        .into_iter()
        .map(|definition| definition.coupon)
        .collect())
}

async fn load_coupon_definitions(
    store: &dyn AdminStore,
) -> CommerceResult<Vec<CommerceCouponDefinition>> {
    let now_ms = current_time_ms()?;
    Ok(list_shared_catalog_visible_coupon_views(store, now_ms)
        .await?
        .into_iter()
        .map(commerce_coupon_definition_from_marketing_view)
        .collect())
}

pub(crate) async fn find_resolved_coupon_definition(
    store: &dyn AdminStore,
    code: &str,
) -> CommerceResult<ResolvedCouponDefinition> {
    let normalized = normalize_coupon_code(code);
    let now_ms = current_time_ms()?;
    load_shared_catalog_visible_coupon_resolution_by_value(store, &normalized, now_ms)
        .await?
        .map(|resolution| ResolvedCouponDefinition {
            definition: commerce_coupon_definition_from_marketing_view(resolution.view),
            marketing: Some(resolution.context),
        })
        .ok_or_else(|| CommerceError::NotFound(format!("coupon {normalized} not found")))
}

pub(crate) async fn load_optional_applied_coupon(
    store: &dyn AdminStore,
    coupon_code: Option<&str>,
    target_kind: &str,
    order_amount_cents: u64,
) -> CommerceResult<Option<ResolvedCouponDefinition>> {
    match coupon_code.map(str::trim).filter(|value| !value.is_empty()) {
        Some(code) => {
            let resolved = find_resolved_coupon_definition(store, code).await?;
            if let Some(context) = resolved.marketing.as_ref() {
                let reserve_amount_minor = compute_coupon_reserve_amount_minor(
                    order_amount_cents,
                    &context.template.benefit,
                );
                let decision = validate_shared_marketing_coupon_context(
                    context,
                    target_kind,
                    current_time_ms()?,
                    order_amount_cents,
                    reserve_amount_minor,
                    None,
                );
                if !decision.eligible {
                    return Err(CommerceError::InvalidInput(format!(
                        "coupon {} is not eligible: {}",
                        resolved.definition.coupon.code,
                        decision
                            .rejection_reason
                            .unwrap_or_else(|| "validation_failed".to_owned())
                    )));
                }
            }
            Ok(Some(resolved))
        }
        None => Ok(None),
    }
}

fn commerce_coupon_definition_from_marketing_view(
    view: MarketingCatalogCouponView,
) -> CommerceCouponDefinition {
    let benefit = CommerceCouponBenefit {
        discount_percent: view.discount_percent,
        bonus_units: view.bonus_units,
    };
    CommerceCouponDefinition {
        coupon: PortalCommerceCoupon {
            id: view.id,
            code: view.code,
            discount_label: view.discount_label,
            audience: view.audience,
            remaining: view.remaining,
            active: view.active,
            note: view.note,
            expires_on: view.expires_on,
            source: view.source,
            discount_percent: benefit.discount_percent,
            bonus_units: benefit.bonus_units,
        },
        benefit,
    }
}
