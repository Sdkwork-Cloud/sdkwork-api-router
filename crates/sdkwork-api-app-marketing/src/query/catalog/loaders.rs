use super::formatting::marketing_catalog_coupon_view_from_context;
use super::types::MarketingCatalogCouponResolution;
use crate::{
    coupon_context_is_catalog_visible, load_marketing_coupon_context_by_value,
    load_marketing_coupon_context_from_code_record, normalize_coupon_code,
    reclaim_expired_coupon_reservations_for_code_record_if_needed, MarketingCouponContext,
};
use anyhow::Result;
use sdkwork_api_storage_core::AdminStore;
use std::collections::BTreeMap;

pub async fn load_catalog_visible_coupon_resolution_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> Result<Option<MarketingCatalogCouponResolution>> {
    let resolution = load_marketing_coupon_context_by_value(store, code, now_ms)
        .await?
        .filter(|context| coupon_context_is_catalog_visible(context, now_ms))
        .map(|context| MarketingCatalogCouponResolution {
            view: marketing_catalog_coupon_view_from_context(&context, now_ms),
            context,
        });
    Ok(resolution)
}

pub async fn list_catalog_visible_coupon_contexts(
    store: &dyn AdminStore,
    now_ms: u64,
) -> Result<Vec<MarketingCouponContext>> {
    let mut contexts = BTreeMap::<String, MarketingCouponContext>::new();

    for code_record in store.list_coupon_code_records().await? {
        reclaim_expired_coupon_reservations_for_code_record_if_needed(
            store,
            code_record.clone(),
            now_ms,
        )
        .await?;

        let Some(current_code_record) = store
            .find_coupon_code_record(&code_record.coupon_code_id)
            .await?
        else {
            continue;
        };
        let Some(context) =
            load_marketing_coupon_context_from_code_record(store, current_code_record, now_ms)
                .await?
        else {
            continue;
        };
        if !coupon_context_is_catalog_visible(&context, now_ms) {
            continue;
        }

        let normalized_code = normalize_coupon_code(&context.code.code_value);
        if should_replace_catalog_context(contexts.get(&normalized_code), &context) {
            contexts.insert(normalized_code, context);
        }
    }

    Ok(contexts.into_values().collect())
}

pub async fn list_catalog_visible_coupon_views(
    store: &dyn AdminStore,
    now_ms: u64,
) -> Result<Vec<super::types::MarketingCatalogCouponView>> {
    Ok(list_catalog_visible_coupon_contexts(store, now_ms)
        .await?
        .into_iter()
        .map(|context| marketing_catalog_coupon_view_from_context(&context, now_ms))
        .collect())
}

fn should_replace_catalog_context(
    current: Option<&MarketingCouponContext>,
    candidate: &MarketingCouponContext,
) -> bool {
    match current {
        Some(current) => {
            candidate.code.updated_at_ms > current.code.updated_at_ms
                || (candidate.code.updated_at_ms == current.code.updated_at_ms
                    && candidate.code.coupon_code_id > current.code.coupon_code_id)
        }
        None => true,
    }
}
