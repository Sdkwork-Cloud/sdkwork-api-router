use sdkwork_api_app_billing::{book_usage_cost, check_quota};
use sdkwork_api_app_billing::{list_ledger_entries, persist_ledger_entry};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn booking_usage_creates_ledger_entry() {
    assert!(check_quota("project-1", 100).unwrap());
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
