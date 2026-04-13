use super::error::MarketingOperationError;
use crate::{
    load_coupon_redemption_owned_by_subject, load_coupon_reservation_owned_by_subject,
    MarketingSubjectSet,
};
use sdkwork_api_domain_marketing::{
    CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord, MarketingSubjectScope,
};
use sdkwork_api_storage_core::AdminStore;

pub(crate) async fn load_coupon_reservation_for_subject(
    store: &dyn AdminStore,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    reservation_id: &str,
) -> Result<CouponReservationRecord, MarketingOperationError> {
    let subjects = single_subject_set(subject_scope, subject_id);
    if let Some(reservation) =
        load_coupon_reservation_owned_by_subject(store, &subjects, reservation_id)
            .await
            .map_err(MarketingOperationError::storage)?
    {
        return Ok(reservation);
    }

    let reservation_exists = store
        .find_coupon_reservation_record(reservation_id)
        .await
        .map_err(MarketingOperationError::storage)?
        .is_some();
    if reservation_exists {
        return Err(MarketingOperationError::forbidden(
            "coupon reservation is not owned by the current subject",
        ));
    }
    Err(MarketingOperationError::not_found(
        "coupon reservation not found",
    ))
}

pub(crate) async fn load_coupon_redemption_for_subject(
    store: &dyn AdminStore,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    redemption_id: &str,
) -> Result<CouponRedemptionRecord, MarketingOperationError> {
    let subjects = single_subject_set(subject_scope, subject_id);
    if let Some(redemption) =
        load_coupon_redemption_owned_by_subject(store, &subjects, redemption_id)
            .await
            .map_err(MarketingOperationError::storage)?
    {
        return Ok(redemption);
    }

    let redemption_exists = store
        .find_coupon_redemption_record(redemption_id)
        .await
        .map_err(MarketingOperationError::storage)?
        .is_some();
    if redemption_exists {
        return Err(MarketingOperationError::forbidden(
            "coupon redemption is not owned by the current subject",
        ));
    }
    Err(MarketingOperationError::not_found(
        "coupon redemption not found",
    ))
}

pub(crate) async fn find_coupon_rollback_record(
    store: &dyn AdminStore,
    rollback_id: &str,
) -> Result<Option<CouponRollbackRecord>, MarketingOperationError> {
    store
        .find_coupon_rollback_record(rollback_id)
        .await
        .map_err(MarketingOperationError::storage)
}

fn single_subject_set(
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
) -> MarketingSubjectSet {
    match subject_scope {
        MarketingSubjectScope::User => {
            MarketingSubjectSet::new(Some(subject_id.to_owned()), None, None, None)
        }
        MarketingSubjectScope::Project => {
            MarketingSubjectSet::new(None, Some(subject_id.to_owned()), None, None)
        }
        MarketingSubjectScope::Workspace => {
            MarketingSubjectSet::new(None, None, Some(subject_id.to_owned()), None)
        }
        MarketingSubjectScope::Account => {
            MarketingSubjectSet::new(None, None, None, Some(subject_id.to_owned()))
        }
    }
}
