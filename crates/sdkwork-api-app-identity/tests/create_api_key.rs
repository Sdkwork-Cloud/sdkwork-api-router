use sdkwork_api_app_identity::CreateGatewayApiKey;
use sdkwork_api_app_identity::{list_gateway_api_keys, persist_gateway_api_key};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn generated_key_has_sdkwork_prefix() {
    let created = CreateGatewayApiKey::execute("tenant-1", "project-1", "live").unwrap();
    assert!(created.plaintext.starts_with("skw_live_"));
}

#[tokio::test]
async fn persisted_gateway_api_keys_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let created = persist_gateway_api_key(&store, "tenant-1", "project-1", "live")
        .await
        .unwrap();

    let keys = list_gateway_api_keys(&store).await.unwrap();

    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].hashed_key, created.hashed);
    assert!(keys[0].active);
}
