use super::super::super::types::MarketingCouponContext;
use super::super::selection::{
    select_campaign_budget_record, select_marketing_campaign_for_reference,
};
use anyhow::Result;
use sdkwork_api_domain_marketing::CouponCodeRecord;
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn load_marketing_coupon_context_for_loaded_code(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    preferred_marketing_campaign_id: Option<&str>,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>> {
    let Some(template) = store
        .find_coupon_template_record(&code.coupon_template_id)
        .await?
    else {
        return Ok(None);
    };

    let campaigns = store
        .list_marketing_campaign_records_for_template(&template.coupon_template_id)
        .await?;
    let Some(campaign) =
        select_marketing_campaign_for_reference(campaigns, preferred_marketing_campaign_id, now_ms)
    else {
        return Ok(None);
    };

    let Some(budget) = select_campaign_budget_record(
        store
            .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
            .await?,
    ) else {
        return Ok(None);
    };

    Ok(Some(MarketingCouponContext {
        template,
        campaign,
        budget,
        code,
    }))
}
