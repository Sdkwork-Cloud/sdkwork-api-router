use super::super::types::MarketingSubjectSet;
use crate::{
    load_marketing_coupon_context_by_value, load_marketing_coupon_context_from_code_record,
    MarketingCouponContext,
};
use anyhow::Result;
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponReservationRecord, MarketingSubjectScope,
};
use sdkwork_api_storage_core::AdminStore;
use std::collections::HashMap;

pub(crate) async fn load_subject_reservations(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
) -> Result<Vec<CouponReservationRecord>> {
    let mut reservations = HashMap::<String, CouponReservationRecord>::new();

    for (scope, subject_id) in subject_scope_entries(subjects).into_iter().flatten() {
        for reservation in store
            .list_coupon_reservation_records_for_subject(scope, subject_id)
            .await?
        {
            reservations.insert(reservation.coupon_reservation_id.clone(), reservation);
        }
    }

    Ok(reservations.into_values().collect())
}

pub(crate) async fn load_claimed_subject_codes(
    store: &dyn AdminStore,
    subjects: &MarketingSubjectSet,
) -> Result<Vec<CouponCodeRecord>> {
    let mut codes = HashMap::<String, CouponCodeRecord>::new();

    for (scope, subject_id) in subject_scope_entries(subjects).into_iter().flatten() {
        for code in store
            .list_coupon_code_records_for_claimed_subject(scope, subject_id)
            .await?
        {
            codes.insert(code.coupon_code_id.clone(), code);
        }
    }

    Ok(codes.into_values().collect())
}

pub(crate) async fn load_marketing_coupon_context_for_subject_code(
    store: &dyn AdminStore,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>> {
    if let Some(context) =
        load_marketing_coupon_context_from_code_record(store, code.clone(), now_ms).await?
    {
        return Ok(Some(context));
    }

    load_marketing_coupon_context_by_value(store, &code.code_value, now_ms).await
}

fn subject_scope_entries(
    subjects: &MarketingSubjectSet,
) -> [Option<(MarketingSubjectScope, &str)>; 4] {
    [
        subjects
            .user_id
            .as_deref()
            .map(|value| (MarketingSubjectScope::User, value)),
        subjects
            .project_id
            .as_deref()
            .map(|value| (MarketingSubjectScope::Project, value)),
        subjects
            .workspace_id
            .as_deref()
            .map(|value| (MarketingSubjectScope::Workspace, value)),
        subjects
            .account_id
            .as_deref()
            .map(|value| (MarketingSubjectScope::Account, value)),
    ]
}
