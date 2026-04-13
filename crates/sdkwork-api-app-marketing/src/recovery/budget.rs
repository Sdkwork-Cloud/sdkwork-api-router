use anyhow::Result;
use sdkwork_api_domain_marketing::{CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord};
use sdkwork_api_storage_core::MarketingKernelTransaction;

pub(super) async fn plan_budget_release_updates(
    tx: &mut dyn MarketingKernelTransaction,
    code: &CouponCodeRecord,
    requested_release_minor: u64,
    now_ms: u64,
) -> Result<(Vec<CampaignBudgetRecord>, u64)> {
    if requested_release_minor == 0 {
        return Ok((Vec::new(), 0));
    }

    let campaigns = tx
        .list_marketing_campaign_records_for_template(&code.coupon_template_id)
        .await?;
    let mut budgets = Vec::new();
    for campaign in campaigns {
        budgets.extend(
            tx.list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
                .await?,
        );
    }
    budgets.sort_by(|left, right| {
        right
            .reserved_budget_minor
            .cmp(&left.reserved_budget_minor)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    });

    let mut remaining_release_minor = requested_release_minor;
    let mut updated_budgets = Vec::new();
    let mut released_budget_minor = 0;
    for mut budget in budgets {
        if remaining_release_minor == 0 {
            break;
        }
        let releasable_minor = budget.reserved_budget_minor.min(remaining_release_minor);
        if releasable_minor == 0 {
            continue;
        }

        budget.reserved_budget_minor = budget
            .reserved_budget_minor
            .saturating_sub(releasable_minor);
        if budget.status == CampaignBudgetStatus::Exhausted && budget.available_budget_minor() > 0 {
            budget.status = CampaignBudgetStatus::Active;
        }
        budget.updated_at_ms = now_ms;
        updated_budgets.push(budget);
        remaining_release_minor = remaining_release_minor.saturating_sub(releasable_minor);
        released_budget_minor += releasable_minor;
    }

    Ok((updated_budgets, released_budget_minor))
}
