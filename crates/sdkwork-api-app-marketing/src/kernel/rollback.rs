use super::error::MarketingServiceError;
use sdkwork_api_domain_marketing::{
    CouponRedemptionRecord, CouponRedemptionStatus, CouponRollbackRecord, CouponRollbackStatus,
    CouponRollbackType,
};

pub fn rollback_coupon_redemption(
    redemption: &CouponRedemptionRecord,
    coupon_rollback_id: impl Into<String>,
    rollback_type: CouponRollbackType,
    restored_budget_minor: u64,
    restored_inventory_count: u64,
    now_ms: u64,
) -> Result<(CouponRedemptionRecord, CouponRollbackRecord), MarketingServiceError> {
    if redemption.redemption_status != CouponRedemptionStatus::Redeemed {
        return Err(MarketingServiceError::invalid_state(
            "redemption is not in redeemed state",
        ));
    }
    if restored_budget_minor > redemption.budget_consumed_minor {
        return Err(MarketingServiceError::invalid_state(
            "restored budget exceeds consumed coupon budget",
        ));
    }

    let next_redemption_status = match rollback_type {
        CouponRollbackType::PartialRefund => CouponRedemptionStatus::PartiallyRolledBack,
        CouponRollbackType::Cancel | CouponRollbackType::Refund | CouponRollbackType::Manual => {
            CouponRedemptionStatus::RolledBack
        }
    };

    let rolled_back_redemption = redemption
        .clone()
        .with_status(next_redemption_status)
        .with_updated_at_ms(now_ms);
    let rollback = CouponRollbackRecord::new(
        coupon_rollback_id,
        rolled_back_redemption.coupon_redemption_id.clone(),
        rollback_type,
        now_ms,
    )
    .with_status(CouponRollbackStatus::Completed)
    .with_restored_budget_minor(restored_budget_minor)
    .with_restored_inventory_count(restored_inventory_count)
    .with_updated_at_ms(now_ms);

    Ok((rolled_back_redemption, rollback))
}
