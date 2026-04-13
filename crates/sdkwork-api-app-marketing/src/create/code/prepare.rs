use super::super::support::{normalize_optional_identifier, normalize_required_identifier};
use crate::context::normalize_coupon_code;
use crate::MarketingServiceError;
use sdkwork_api_domain_marketing::{CouponCodeRecord, CouponCodeStatus};

pub(super) fn prepare_coupon_code_record_for_create(
    mut record: CouponCodeRecord,
) -> Result<CouponCodeRecord, MarketingServiceError> {
    record.coupon_code_id =
        normalize_required_identifier(&record.coupon_code_id, "coupon code", "coupon_code_id")?;
    record.coupon_template_id = normalize_required_identifier(
        &record.coupon_template_id,
        "coupon code",
        "coupon_template_id",
    )?;
    record.code_value = normalize_coupon_code(&record.code_value);
    if record.code_value.is_empty() {
        return Err(MarketingServiceError::invalid_state(
            "coupon code create requires code_value",
        ));
    }
    record.claimed_subject_id = normalize_optional_identifier(record.claimed_subject_id);

    if record.status != CouponCodeStatus::Available {
        return Err(MarketingServiceError::invalid_state(
            "coupon code create does not accept lifecycle status",
        ));
    }
    match (
        record.claimed_subject_scope,
        record.claimed_subject_id.as_deref(),
    ) {
        (None, None) | (Some(_), Some(_)) => {}
        _ => {
            return Err(MarketingServiceError::invalid_state(
                "coupon code claimed subject must include both scope and id",
            ));
        }
    }

    record.status = CouponCodeStatus::Available;
    Ok(record)
}
