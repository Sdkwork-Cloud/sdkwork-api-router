use super::errors::marketing_create_storage;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponDistributionKind, CouponTemplateRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub(crate) async fn load_coupon_template_record(
    store: &dyn AdminStore,
    coupon_template_id: &str,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    store
        .find_coupon_template_record(coupon_template_id)
        .await
        .map_err(marketing_create_storage)?
        .ok_or_else(|| {
            MarketingGovernanceError::NotFound(format!(
                "coupon template {coupon_template_id} not found"
            ))
        })
}

pub(crate) async fn require_marketing_campaign_record(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
) -> Result<(), MarketingGovernanceError> {
    if store
        .find_marketing_campaign_record(marketing_campaign_id)
        .await
        .map_err(marketing_create_storage)?
        .is_none()
    {
        return Err(MarketingGovernanceError::NotFound(format!(
            "marketing campaign {marketing_campaign_id} not found"
        )));
    }
    Ok(())
}

pub(crate) fn validate_coupon_code_template_compatibility(
    coupon_template: &CouponTemplateRecord,
    record: &CouponCodeRecord,
) -> Result<(), MarketingGovernanceError> {
    if matches!(
        coupon_template.distribution_kind,
        CouponDistributionKind::SharedCode
    ) && record.claimed_subject_scope.is_some()
    {
        return Err(MarketingGovernanceError::InvalidInput(
            "shared-code coupon template does not accept claimed coupon code ownership".to_owned(),
        ));
    }
    if let Some(scope) = record.claimed_subject_scope {
        if scope != coupon_template.restriction.subject_scope {
            return Err(MarketingGovernanceError::InvalidInput(format!(
                "coupon code claimed subject scope {:?} does not match template subject scope {:?}",
                scope, coupon_template.restriction.subject_scope
            )));
        }
    }
    Ok(())
}
