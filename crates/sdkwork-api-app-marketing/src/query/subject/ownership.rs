use super::types::{
    MarketingRedemptionOwnershipView, MarketingReservationOwnershipView, MarketingSubjectSet,
};
use anyhow::Result;
use sdkwork_api_domain_marketing::{CouponRedemptionRecord, CouponReservationRecord};
use sdkwork_api_storage_core::AdminStore;

pub async fn load_coupon_reservation_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    reservation_id: &str,
) -> Result<Option<CouponReservationRecord>> {
    let Some(reservation) = store.find_coupon_reservation_record(reservation_id).await? else {
        return Ok(None);
    };
    if !subjects.matches(reservation.subject_scope, &reservation.subject_id) {
        return Ok(None);
    }
    Ok(Some(reservation))
}

pub async fn load_coupon_redemption_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    redemption_id: &str,
) -> Result<Option<CouponRedemptionRecord>> {
    let Some(redemption) = store.find_coupon_redemption_record(redemption_id).await? else {
        return Ok(None);
    };
    let Some(reservation) = load_coupon_reservation_owned_by_subject(
        store,
        subjects,
        &redemption.coupon_reservation_id,
    )
    .await?
    else {
        return Ok(None);
    };
    if reservation.coupon_reservation_id != redemption.coupon_reservation_id {
        return Ok(None);
    }
    Ok(Some(redemption))
}

pub async fn load_coupon_reservation_context_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    reservation_id: &str,
) -> Result<Option<MarketingReservationOwnershipView>> {
    let Some(reservation) =
        load_coupon_reservation_owned_by_subject(store, subjects, reservation_id).await?
    else {
        return Ok(None);
    };
    let Some(code) = store
        .find_coupon_code_record(&reservation.coupon_code_id)
        .await?
    else {
        return Ok(None);
    };
    Ok(Some(MarketingReservationOwnershipView {
        reservation,
        code,
    }))
}

pub async fn load_coupon_redemption_context_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
    redemption_id: &str,
) -> Result<Option<MarketingRedemptionOwnershipView>> {
    let Some(redemption) =
        load_coupon_redemption_owned_by_subject(store, subjects, redemption_id).await?
    else {
        return Ok(None);
    };
    let Some(reservation_view) = load_coupon_reservation_context_owned_by_subject(
        store,
        subjects,
        &redemption.coupon_reservation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    Ok(Some(MarketingRedemptionOwnershipView {
        reservation: reservation_view.reservation,
        redemption,
        code: reservation_view.code,
    }))
}
