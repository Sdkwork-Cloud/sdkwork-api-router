use super::super::support::{
    latest_redemptions_by_code, latest_reservations_by_code, load_claimed_subject_codes,
    load_marketing_coupon_context_for_subject_code, load_subject_reservations,
};
use super::super::types::{MarketingCodeView, MarketingSubjectSet};
use anyhow::Result;
use sdkwork_api_storage_core::AdminStore;
use std::collections::HashMap;

pub async fn list_coupon_code_views_for_subjects(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    now_ms: u64,
) -> Result<Vec<MarketingCodeView>> {
    let reservations = load_subject_reservations(store, subjects).await?;

    let reservation_ids = reservations
        .iter()
        .map(|reservation| reservation.coupon_reservation_id.clone())
        .collect::<Vec<_>>();
    let redemptions = store
        .list_coupon_redemption_records_for_reservation_ids(&reservation_ids)
        .await?;

    let latest_reservations = latest_reservations_by_code(reservations);
    let latest_redemptions = latest_redemptions_by_code(redemptions);
    let mut code_map = store
        .list_coupon_code_records_for_ids(
            &latest_reservations
                .keys()
                .chain(latest_redemptions.keys())
                .cloned()
                .collect::<Vec<_>>(),
        )
        .await?
        .into_iter()
        .map(|code| (code.coupon_code_id.clone(), code))
        .collect::<HashMap<_, _>>();
    for code in load_claimed_subject_codes(store, subjects).await? {
        code_map.insert(code.coupon_code_id.clone(), code);
    }

    let mut views = Vec::new();
    for code in code_map.into_values() {
        let Some(context) =
            load_marketing_coupon_context_for_subject_code(store, &code, now_ms).await?
        else {
            continue;
        };
        views.push(MarketingCodeView {
            context,
            latest_reservation: latest_reservations.get(&code.coupon_code_id).cloned(),
            latest_redemption: latest_redemptions.get(&code.coupon_code_id).cloned(),
        });
    }

    views.sort_by(|left, right| {
        right
            .context
            .code
            .updated_at_ms
            .cmp(&left.context.code.updated_at_ms)
            .then_with(|| {
                right
                    .context
                    .code
                    .coupon_code_id
                    .cmp(&left.context.code.coupon_code_id)
            })
    });
    Ok(views)
}
