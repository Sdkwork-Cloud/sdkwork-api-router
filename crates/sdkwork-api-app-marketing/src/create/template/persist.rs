use super::super::support::{
    marketing_create_invalid_input, marketing_create_storage, PersistMode,
};
use super::prepare::prepare_coupon_template_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::CouponTemplateRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_coupon_template_record(
    store: &dyn AdminStore,
    record: CouponTemplateRecord,
    mode: PersistMode,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    let record = prepare_coupon_template_record_for_create(record)
        .map_err(marketing_create_invalid_input)?;

    if let Some(existing_template) = store
        .find_coupon_template_record(&record.coupon_template_id)
        .await
        .map_err(marketing_create_storage)?
    {
        return mode.resolve_existing_primary(
            "coupon template",
            &record.coupon_template_id,
            existing_template,
            &record,
        );
    }
    if let Some(existing_template) = store
        .find_coupon_template_record_by_template_key(&record.template_key)
        .await
        .map_err(marketing_create_storage)?
    {
        return mode.resolve_existing_unique(existing_template, &record, |existing_template| {
            MarketingGovernanceError::Conflict(format!(
                "coupon template key {} already exists on {}",
                record.template_key, existing_template.coupon_template_id
            ))
        });
    }

    store
        .insert_coupon_template_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_coupon_template_record(
    store: &dyn AdminStore,
    record: CouponTemplateRecord,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    persist_coupon_template_record(store, record, PersistMode::Create).await
}

pub async fn ensure_coupon_template_record(
    store: &dyn AdminStore,
    record: CouponTemplateRecord,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    persist_coupon_template_record(store, record, PersistMode::Ensure).await
}
