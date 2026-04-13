use super::super::support::load_marketing_coupon_context_for_subject_code;
use super::super::types::{MarketingRewardHistoryView, MarketingSubjectSet};
use super::redemptions::list_coupon_redemptions_for_subjects;
use anyhow::Result;
use sdkwork_api_domain_marketing::CouponRollbackRecord;
use sdkwork_api_storage_core::AdminStore;
use std::collections::HashMap;

pub async fn list_coupon_reward_history_views_for_subjects(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    now_ms: u64,
) -> Result<Vec<MarketingRewardHistoryView>> {
    let redemptions = list_coupon_redemptions_for_subjects(store, subjects, None).await?;
    let codes = store
        .list_coupon_code_records_for_ids(
            &redemptions
                .iter()
                .map(|redemption| redemption.coupon_code_id.clone())
                .collect::<Vec<_>>(),
        )
        .await?
        .into_iter()
        .map(|code| (code.coupon_code_id.clone(), code))
        .collect::<HashMap<_, _>>();
    let mut rollbacks_by_redemption = HashMap::<String, Vec<CouponRollbackRecord>>::new();
    for rollback in store
        .list_coupon_rollback_records_for_redemption_ids(
            &redemptions
                .iter()
                .map(|redemption| redemption.coupon_redemption_id.clone())
                .collect::<Vec<_>>(),
        )
        .await?
    {
        rollbacks_by_redemption
            .entry(rollback.coupon_redemption_id.clone())
            .or_default()
            .push(rollback);
    }

    let mut views = Vec::new();
    for redemption in redemptions {
        let Some(code) = codes.get(&redemption.coupon_code_id).cloned() else {
            continue;
        };
        let Some(context) =
            load_marketing_coupon_context_for_subject_code(store, &code, now_ms).await?
        else {
            continue;
        };
        let mut rollbacks = rollbacks_by_redemption
            .remove(&redemption.coupon_redemption_id)
            .unwrap_or_default();
        rollbacks.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.coupon_rollback_id.cmp(&left.coupon_rollback_id))
        });
        views.push(MarketingRewardHistoryView {
            context,
            redemption,
            rollbacks,
        });
    }

    views.sort_by(|left, right| {
        right
            .redemption
            .redeemed_at_ms
            .cmp(&left.redemption.redeemed_at_ms)
            .then_with(|| {
                right
                    .redemption
                    .coupon_redemption_id
                    .cmp(&left.redemption.coupon_redemption_id)
            })
    });
    Ok(views)
}
