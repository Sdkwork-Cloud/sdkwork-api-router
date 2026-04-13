use std::fmt::Write as _;

use sdkwork_api_domain_marketing::MarketingSubjectScope;
use sha2::{Digest, Sha256};

pub fn marketing_subject_scope_token(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

pub fn marketing_idempotency_fingerprint(
    operation: &str,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    idempotency_key: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(operation.as_bytes());
    hasher.update([0x1f]);
    hasher.update(marketing_subject_scope_token(subject_scope).as_bytes());
    hasher.update([0x1f]);
    hasher.update(subject_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(idempotency_key.as_bytes());

    let digest = hasher.finalize();
    let mut fingerprint = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(&mut fingerprint, "{byte:02x}");
    }
    fingerprint
}
