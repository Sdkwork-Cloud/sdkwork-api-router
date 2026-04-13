use std::fmt::Write as _;

use super::fingerprint::{marketing_idempotency_fingerprint, marketing_subject_scope_token};
use sdkwork_api_domain_marketing::{CouponReservationRecord, MarketingSubjectScope};
use sha2::{Digest, Sha256};

pub fn derive_coupon_reservation_id(
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    target_kind: &str,
    idempotency_key: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update("reserve".as_bytes());
    hasher.update([0x1f]);
    hasher.update(marketing_subject_scope_token(subject_scope).as_bytes());
    hasher.update([0x1f]);
    hasher.update(subject_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(target_kind.as_bytes());
    hasher.update([0x1f]);
    hasher.update(idempotency_key.as_bytes());

    let digest = hasher.finalize();
    let mut fingerprint = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(&mut fingerprint, "{byte:02x}");
    }

    format!("coupon_reservation_{fingerprint}")
}

pub fn derive_coupon_redemption_id(
    reservation: &CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_redemption_{}",
        marketing_idempotency_fingerprint(
            "confirm",
            reservation.subject_scope,
            &reservation.subject_id,
            idempotency_key,
        )
    )
}

pub fn derive_coupon_rollback_id(
    reservation: &CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_rollback_{}",
        marketing_idempotency_fingerprint(
            "rollback",
            reservation.subject_scope,
            &reservation.subject_id,
            idempotency_key,
        )
    )
}
