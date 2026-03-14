use sdkwork_api_app_billing::{
    book_usage_cost, check_quota, create_quota_policy, list_ledger_entries, persist_ledger_entry,
    persist_quota_policy,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn booking_usage_creates_ledger_entry() {
    let ledger = book_usage_cost("project-1", 100, 0.25).unwrap();
    assert_eq!(ledger.project_id, "project-1");
}

#[tokio::test]
async fn persisted_ledger_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_ledger_entry(&store, "project-1", 100, 0.25)
        .await
        .unwrap();

    let ledger = list_ledger_entries(&store).await.unwrap();
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger[0].amount, 0.25);
}

#[tokio::test]
async fn quota_evaluation_rejects_requests_past_configured_limit() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = create_quota_policy("quota-project-1", "project-1", 100, true).unwrap();
    persist_quota_policy(&store, &policy).await.unwrap();
    persist_ledger_entry(&store, "project-1", 70, 0.25)
        .await
        .unwrap();

    let evaluation = check_quota(&store, "project-1", 40).await.unwrap();
    assert!(!evaluation.allowed);
    assert_eq!(evaluation.policy_id.as_deref(), Some("quota-project-1"));
    assert_eq!(evaluation.used_units, 70);
    assert_eq!(evaluation.limit_units, Some(100));
}
