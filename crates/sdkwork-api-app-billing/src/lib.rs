use anyhow::{ensure, Result};
use sdkwork_api_domain_billing::LedgerEntry;
use sdkwork_api_storage_core::AdminStore;

pub use sdkwork_api_domain_billing::{QuotaCheckResult, QuotaPolicy};

pub fn service_name() -> &'static str {
    "billing-service"
}

pub fn create_quota_policy(
    policy_id: &str,
    project_id: &str,
    max_units: u64,
    enabled: bool,
) -> Result<QuotaPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(max_units > 0, "max_units must be greater than 0");

    Ok(QuotaPolicy::new(policy_id, project_id, max_units).with_enabled(enabled))
}

pub fn book_usage_cost(project_id: &str, units: u64, amount: f64) -> Result<LedgerEntry> {
    Ok(LedgerEntry::new(project_id, units, amount))
}

pub async fn persist_ledger_entry(
    store: &dyn AdminStore,
    project_id: &str,
    units: u64,
    amount: f64,
) -> Result<LedgerEntry> {
    let entry = book_usage_cost(project_id, units, amount)?;
    store.insert_ledger_entry(&entry).await
}

pub async fn list_ledger_entries(store: &dyn AdminStore) -> Result<Vec<LedgerEntry>> {
    store.list_ledger_entries().await
}

pub async fn persist_quota_policy(
    store: &dyn AdminStore,
    policy: &QuotaPolicy,
) -> Result<QuotaPolicy> {
    store.insert_quota_policy(policy).await
}

pub async fn list_quota_policies(store: &dyn AdminStore) -> Result<Vec<QuotaPolicy>> {
    store.list_quota_policies().await
}

pub async fn check_quota(
    store: &dyn AdminStore,
    project_id: &str,
    requested_units: u64,
) -> Result<QuotaCheckResult> {
    let used_units = store
        .list_ledger_entries()
        .await?
        .into_iter()
        .filter(|entry| entry.project_id == project_id)
        .map(|entry| entry.units)
        .sum();

    let effective_policy = store
        .list_quota_policies()
        .await?
        .into_iter()
        .filter(|policy| policy.enabled && policy.project_id == project_id)
        .min_by(|left, right| {
            left.max_units
                .cmp(&right.max_units)
                .then_with(|| left.policy_id.cmp(&right.policy_id))
        });

    Ok(match effective_policy {
        Some(policy) => QuotaCheckResult::from_policy(&policy, used_units, requested_units),
        None => QuotaCheckResult::allowed_without_policy(requested_units, used_units),
    })
}
