use super::super::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{CouponCodeRecord, CouponTemplateRecord};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn load_coupon_code_context(
    store: &dyn AdminStore,
    coupon_code_id: &str,
) -> Result<(CouponCodeRecord, CouponTemplateRecord), MarketingGovernanceError> {
    let coupon_code = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!("coupon code {coupon_code_id} not found"))
        })?;
    let coupon_template = store
        .find_coupon_template_record(&coupon_code.coupon_template_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::not_found(format!(
                "coupon template {} for coupon code {} not found",
                coupon_code.coupon_template_id, coupon_code_id
            ))
        })?;
    Ok((coupon_code, coupon_template))
}
