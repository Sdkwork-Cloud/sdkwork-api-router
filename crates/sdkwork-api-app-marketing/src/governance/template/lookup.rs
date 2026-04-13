use super::super::MarketingGovernanceError;
use sdkwork_api_domain_marketing::CouponTemplateRecord;
use sdkwork_api_storage_core::AdminStore;

pub(super) fn coupon_template_root_id(record: &CouponTemplateRecord) -> String {
    record
        .root_coupon_template_id
        .clone()
        .unwrap_or_else(|| record.coupon_template_id.clone())
}

pub(super) fn coupon_template_revision(record: &CouponTemplateRecord) -> u32 {
    record.revision.max(1)
}

pub(super) async fn load_coupon_template_record(
    store: &dyn AdminStore,
    coupon_template_id: &str,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    store
        .find_coupon_template_record(coupon_template_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!(
                "coupon template {coupon_template_id} not found"
            ))
        })
}

pub(super) async fn next_coupon_template_revision(
    store: &dyn AdminStore,
    source_coupon_template: &CouponTemplateRecord,
) -> Result<u32, MarketingGovernanceError> {
    let root_coupon_template_id = coupon_template_root_id(source_coupon_template);
    let next_revision = store
        .list_coupon_template_records_for_root(&root_coupon_template_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .into_iter()
        .map(|record| coupon_template_revision(&record))
        .max()
        .unwrap_or_else(|| coupon_template_revision(source_coupon_template))
        .saturating_add(1);
    Ok(next_revision.max(coupon_template_revision(source_coupon_template) + 1))
}

pub(super) fn coupon_template_field_value(
    record: &CouponTemplateRecord,
    field: &str,
) -> Result<String, MarketingGovernanceError> {
    match field {
        "template_key" => Ok(record.template_key.clone()),
        "display_name" => Ok(record.display_name.clone()),
        "status" => Ok(serde_json::to_string(&record.status).unwrap_or_default()),
        "approval_state" => Ok(serde_json::to_string(&record.approval_state).unwrap_or_default()),
        "revision" => Ok(record.revision.to_string()),
        "distribution_kind" => {
            Ok(serde_json::to_string(&record.distribution_kind).unwrap_or_default())
        }
        "benefit" => {
            serde_json::to_string(&record.benefit).map_err(MarketingGovernanceError::storage)
        }
        "restriction" => {
            serde_json::to_string(&record.restriction).map_err(MarketingGovernanceError::storage)
        }
        "activation_at_ms" => Ok(record
            .activation_at_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_owned())),
        _ => Ok(String::new()),
    }
}
