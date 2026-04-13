use super::expiration::should_expire_reservation;
use super::reservation::recover_expired_coupon_reservation_in_tx;
use super::types::MarketingRecoveryRunReport;
use anyhow::Result;
use sdkwork_api_storage_core::{MarketingKernelTransactionExecutor, MarketingStore};

pub async fn recover_expired_coupon_reservations<S>(
    store: &S,
    now_ms: u64,
) -> Result<MarketingRecoveryRunReport>
where
    S: MarketingStore + MarketingKernelTransactionExecutor + ?Sized,
{
    let reservations = MarketingStore::list_coupon_reservation_records(store).await?;
    let mut report = MarketingRecoveryRunReport {
        scanned_reservations: reservations.len() as u64,
        ..MarketingRecoveryRunReport::default()
    };

    for reservation in reservations {
        if !should_expire_reservation(&reservation, now_ms) {
            continue;
        }

        let reservation_id = reservation.coupon_reservation_id.clone();
        if let Some(outcome) = store
            .with_marketing_kernel_transaction(|tx| {
                Box::pin(async move {
                    recover_expired_coupon_reservation_in_tx(tx, &reservation_id, now_ms).await
                })
            })
            .await?
        {
            report.expired_reservations += 1;
            report.released_codes += outcome.released_codes;
            report.released_budget_minor += outcome.released_budget_minor;
            report.outbox_events_created += 1;
        }
    }

    Ok(report)
}
