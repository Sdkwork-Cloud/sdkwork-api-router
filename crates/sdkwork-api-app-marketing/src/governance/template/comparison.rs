use super::super::MarketingGovernanceError;
use super::lookup::{
    coupon_template_field_value, coupon_template_root_id, load_coupon_template_record,
};
use super::types::{CouponTemplateComparisonFieldChange, CouponTemplateComparisonResult};
use sdkwork_api_storage_core::AdminStore;

pub async fn compare_marketing_coupon_template_revisions(
    store: &dyn AdminStore,
    source_coupon_template_id: &str,
    target_coupon_template_id: &str,
) -> Result<CouponTemplateComparisonResult, MarketingGovernanceError> {
    let source_coupon_template =
        load_coupon_template_record(store, source_coupon_template_id).await?;
    let target_coupon_template =
        load_coupon_template_record(store, target_coupon_template_id).await?;
    let mut field_changes = Vec::new();
    for field in [
        "template_key",
        "display_name",
        "status",
        "approval_state",
        "revision",
        "distribution_kind",
        "benefit",
        "restriction",
        "activation_at_ms",
    ] {
        let source_value = coupon_template_field_value(&source_coupon_template, field)?;
        let target_value = coupon_template_field_value(&target_coupon_template, field)?;
        if source_value != target_value {
            field_changes.push(CouponTemplateComparisonFieldChange {
                field: field.to_owned(),
                source_value,
                target_value,
            });
        }
    }
    Ok(CouponTemplateComparisonResult {
        same_lineage: coupon_template_root_id(&source_coupon_template)
            == coupon_template_root_id(&target_coupon_template),
        source_coupon_template,
        target_coupon_template,
        field_changes,
    })
}
