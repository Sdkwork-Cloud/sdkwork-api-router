use super::budget::plan_budget_release_updates;
use super::expiration::{recovered_coupon_code, should_expire_reservation};
use super::outbox::expired_reservation_outbox_event;
use super::types::MarketingRecoveryReservationOutcome;
use anyhow::Result;
use sdkwork_api_domain_marketing::CouponReservationStatus;
use sdkwork_api_storage_core::MarketingKernelTransaction;

pub(super) async fn recover_expired_coupon_reservation_in_tx(
    tx: &mut dyn MarketingKernelTransaction,
    reservation_id: &str,
    now_ms: u64,
) -> Result<Option<MarketingRecoveryReservationOutcome>> {
    let Some(persisted_reservation) = tx.find_coupon_reservation_record(reservation_id).await?
    else {
        anyhow::bail!("coupon reservation {} missing", reservation_id);
    };
    if !should_expire_reservation(&persisted_reservation, now_ms) {
        return Ok(None);
    }

    let Some(code) = tx
        .find_coupon_code_record(&persisted_reservation.coupon_code_id)
        .await?
    else {
        anyhow::bail!(
            "coupon code {} missing for reservation {}",
            persisted_reservation.coupon_code_id,
            persisted_reservation.coupon_reservation_id
        );
    };

    let recovered_code = recovered_coupon_code(&code, now_ms);
    let (updated_budgets, released_budget_minor) = plan_budget_release_updates(
        tx,
        &code,
        persisted_reservation.budget_reserved_minor,
        now_ms,
    )
    .await?;
    let outbox_event = expired_reservation_outbox_event(
        &persisted_reservation,
        recovered_code.as_ref().unwrap_or(&code),
        released_budget_minor,
        now_ms,
    );
    let expired_reservation = persisted_reservation
        .clone()
        .with_status(CouponReservationStatus::Expired)
        .with_updated_at_ms(now_ms);

    tx.upsert_coupon_reservation_record(&expired_reservation)
        .await?;
    let mut released_codes = 0;
    if let Some(code) = recovered_code.as_ref() {
        tx.upsert_coupon_code_record(code).await?;
        released_codes = 1;
    }
    for budget in &updated_budgets {
        tx.upsert_campaign_budget_record(budget).await?;
    }
    tx.upsert_marketing_outbox_event_record(&outbox_event)
        .await?;

    Ok(Some(MarketingRecoveryReservationOutcome {
        released_codes,
        released_budget_minor,
    }))
}
