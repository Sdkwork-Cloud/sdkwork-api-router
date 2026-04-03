use sdkwork_api_storage_core::{
    AdminStore, BillingStore, CatalogStore, CredentialStore, ExtensionStore, IdentityStore,
    RoutingStore, StorageDialect, TenantStore, UsageStore,
};
use sdkwork_api_storage_postgres::PostgresAdminStore;

#[tokio::test]
async fn postgres_store_implements_admin_store_trait() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://sdkwork:secret@localhost/sdkwork")
        .unwrap();
    let store = PostgresAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;
    let _identity_store: &dyn IdentityStore = &store;
    let _tenant_store: &dyn TenantStore = &store;
    let _catalog_store: &dyn CatalogStore = &store;
    let _credential_store: &dyn CredentialStore = &store;
    let _routing_store: &dyn RoutingStore = &store;
    let _usage_store: &dyn UsageStore = &store;
    let _billing_store: &dyn BillingStore = &store;
    let _extension_store: &dyn ExtensionStore = &store;

    assert_eq!(trait_store.dialect(), StorageDialect::Postgres);
}
