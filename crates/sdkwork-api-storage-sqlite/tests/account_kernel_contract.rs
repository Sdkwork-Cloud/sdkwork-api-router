use sdkwork_api_domain_billing::{AccountRecord, AccountType};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_canonical_account_kernel_records_instead_of_exposing_only_a_seam() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = store
        .insert_account_record(&AccountRecord::new(
            7001,
            1001,
            2002,
            9001,
            AccountType::Primary,
        ))
        .await
        .unwrap();
    assert_eq!(account.account_id, 7001);
    assert_eq!(account.tenant_id, 1001);

    let accounts = store.list_account_records().await.unwrap();
    assert_eq!(accounts, vec![account]);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert!(request_facts.is_empty());
}
