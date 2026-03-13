use sdkwork_api_app_identity::persist_gateway_api_key;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use sqlx::SqlitePool;

pub async fn issue_gateway_api_key(pool: &SqlitePool, tenant_id: &str, project_id: &str) -> String {
    let store = SqliteAdminStore::new(pool.clone());
    persist_gateway_api_key(&store, tenant_id, project_id, "live")
        .await
        .unwrap()
        .plaintext
}
