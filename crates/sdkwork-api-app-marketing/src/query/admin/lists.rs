use anyhow::Result;
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord,
    CouponRollbackRecord, CouponTemplateRecord, MarketingCampaignRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn list_marketing_coupon_templates(
    store: &dyn AdminStore,
) -> Result<Vec<CouponTemplateRecord>> {
    let mut templates = store.list_coupon_template_records().await?;
    templates.sort_by(|left, right| left.template_key.cmp(&right.template_key));
    Ok(templates)
}

pub async fn list_marketing_campaigns(
    store: &dyn AdminStore,
) -> Result<Vec<MarketingCampaignRecord>> {
    let mut campaigns = store.list_marketing_campaign_records().await?;
    campaigns.sort_by(|left, right| left.marketing_campaign_id.cmp(&right.marketing_campaign_id));
    Ok(campaigns)
}

pub async fn list_marketing_campaign_budgets(
    store: &dyn AdminStore,
) -> Result<Vec<CampaignBudgetRecord>> {
    let mut budgets = store.list_campaign_budget_records().await?;
    budgets.sort_by(|left, right| left.campaign_budget_id.cmp(&right.campaign_budget_id));
    Ok(budgets)
}

pub async fn list_marketing_coupon_codes(store: &dyn AdminStore) -> Result<Vec<CouponCodeRecord>> {
    let mut codes = store.list_coupon_code_records().await?;
    codes.sort_by(|left, right| left.code_value.cmp(&right.code_value));
    Ok(codes)
}

pub async fn list_marketing_coupon_reservations(
    store: &dyn AdminStore,
) -> Result<Vec<CouponReservationRecord>> {
    let mut reservations = store.list_coupon_reservation_records().await?;
    reservations
        .sort_by(|left, right| left.coupon_reservation_id.cmp(&right.coupon_reservation_id));
    Ok(reservations)
}

pub async fn list_marketing_coupon_redemptions(
    store: &dyn AdminStore,
) -> Result<Vec<CouponRedemptionRecord>> {
    let mut redemptions = store.list_coupon_redemption_records().await?;
    redemptions.sort_by(|left, right| left.coupon_redemption_id.cmp(&right.coupon_redemption_id));
    Ok(redemptions)
}

pub async fn list_marketing_coupon_rollbacks(
    store: &dyn AdminStore,
) -> Result<Vec<CouponRollbackRecord>> {
    let mut rollbacks = store.list_coupon_rollback_records().await?;
    rollbacks.sort_by(|left, right| left.coupon_rollback_id.cmp(&right.coupon_rollback_id));
    Ok(rollbacks)
}
