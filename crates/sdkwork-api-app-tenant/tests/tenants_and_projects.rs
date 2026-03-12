use sdkwork_api_app_tenant::{list_projects, list_tenants, persist_project, persist_tenant};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn persists_tenants_and_projects() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_tenant(&store, "tenant-1", "Tenant One")
        .await
        .unwrap();
    persist_project(&store, "tenant-1", "project-1", "Project One")
        .await
        .unwrap();

    let tenants = list_tenants(&store).await.unwrap();
    let projects = list_projects(&store).await.unwrap();

    assert_eq!(tenants[0].id, "tenant-1");
    assert_eq!(projects[0].tenant_id, "tenant-1");
}
