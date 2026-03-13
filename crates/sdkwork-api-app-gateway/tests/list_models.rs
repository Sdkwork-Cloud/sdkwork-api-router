use sdkwork_api_app_gateway::{
    get_model, get_model_from_store, list_models, list_models_from_store,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn returns_platform_models() {
    let response = list_models("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
}

#[test]
fn returns_platform_model() {
    let response = get_model("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.id, "gpt-4.1");
    assert_eq!(response.object, "model");
}

#[tokio::test]
async fn returns_catalog_models_from_store() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let response = list_models_from_store(&store, "tenant-1", "project-1")
        .await
        .unwrap();
    assert_eq!(response.data[0].id, "gpt-4.1");
}

#[tokio::test]
async fn returns_catalog_model_from_store() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let response = get_model_from_store(&store, "tenant-1", "project-1", "gpt-4.1")
        .await
        .unwrap()
        .expect("catalog model");
    assert_eq!(response.id, "gpt-4.1");
    assert_eq!(response.owned_by, "provider-openai-official");
}
