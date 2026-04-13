use super::super::support::{normalize_optional_text, normalize_required_identifier};
use crate::MarketingServiceError;
use sdkwork_api_domain_marketing::{
    CouponTemplateApprovalState, CouponTemplateRecord, CouponTemplateStatus,
};

pub(super) fn prepare_coupon_template_record_for_create(
    mut record: CouponTemplateRecord,
) -> Result<CouponTemplateRecord, MarketingServiceError> {
    record.coupon_template_id = normalize_required_identifier(
        &record.coupon_template_id,
        "coupon template",
        "coupon_template_id",
    )?;
    record.template_key =
        normalize_required_identifier(&record.template_key, "coupon template", "template_key")?;
    record.display_name = normalize_optional_text(record.display_name);

    if record.status != CouponTemplateStatus::Draft {
        return Err(MarketingServiceError::invalid_state(
            "coupon template create does not accept lifecycle status",
        ));
    }
    if record.revision > 1 {
        return Err(MarketingServiceError::invalid_state(
            "coupon template create does not accept revision",
        ));
    }
    if record.parent_coupon_template_id.is_some() {
        return Err(MarketingServiceError::invalid_state(
            "coupon template create does not accept parent lineage",
        ));
    }
    if record
        .root_coupon_template_id
        .as_deref()
        .is_some_and(|root_id| root_id != record.coupon_template_id)
    {
        return Err(MarketingServiceError::invalid_state(
            "coupon template create root lineage must point to itself",
        ));
    }

    record.approval_state = CouponTemplateApprovalState::Draft;
    record.revision = 1;
    record.parent_coupon_template_id = None;
    record.root_coupon_template_id = Some(record.coupon_template_id.clone());
    Ok(record)
}
