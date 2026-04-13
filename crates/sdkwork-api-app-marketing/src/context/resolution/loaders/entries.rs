use super::super::super::recovery::reclaim_expired_coupon_reservations_for_code_record_if_needed;
use super::super::super::types::{MarketingCouponContext, MarketingCouponContextReference};
use super::super::reference::resolve_marketing_coupon_code_for_reference;
use super::assembly::load_marketing_coupon_context_for_loaded_code;
use crate::context::normalize_coupon_code;
use anyhow::Result;
use sdkwork_api_domain_marketing::CouponCodeRecord;
use sdkwork_api_storage_core::AdminStore;

pub async fn load_marketing_coupon_context_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>> {
    let normalized = normalize_coupon_code(code);
    let Some(code_record) = store.find_coupon_code_record_by_value(&normalized).await? else {
        return Ok(None);
    };

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
        return Ok(None);
    };

    load_marketing_coupon_context_from_code_record(store, current_code_record, now_ms).await
}

pub async fn load_marketing_coupon_context_from_code_record(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>> {
    load_marketing_coupon_context_for_loaded_code(store, code, None, now_ms).await
}

pub async fn load_marketing_coupon_context_for_code_id(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>> {
    let Some(code) = store.find_coupon_code_record(coupon_code_id).await? else {
        return Ok(None);
    };

    load_marketing_coupon_context_from_code_record(store, code, now_ms).await
}

pub async fn load_marketing_coupon_context_for_reference(
    store: &dyn AdminStore,
    reference: MarketingCouponContextReference<'_>,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>> {
    let Some(normalized_code) = resolve_marketing_coupon_code_for_reference(reference) else {
        return Ok(None);
    };
    let Some(code) = store
        .find_coupon_code_record_by_value(&normalized_code)
        .await?
    else {
        return Ok(None);
    };

    load_marketing_coupon_context_for_loaded_code(
        store,
        code,
        reference.preferred_marketing_campaign_id,
        now_ms,
    )
    .await
}
