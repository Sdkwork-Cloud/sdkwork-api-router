use sdkwork_api_app_usage::record_usage;
use sdkwork_api_app_usage::{list_usage_records, persist_usage_record};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn usage_record_contains_model_and_provider() {
    let usage = record_usage("project-1", "gpt-4.1", "provider-openai-official").unwrap();
    assert_eq!(usage.model, "gpt-4.1");
}

#[tokio::test]
async fn persisted_usage_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_usage_record(&store, "project-1", "gpt-4.1", "provider-openai-official")
        .await
        .unwrap();

    let records = list_usage_records(&store).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].provider, "provider-openai-official");
}
