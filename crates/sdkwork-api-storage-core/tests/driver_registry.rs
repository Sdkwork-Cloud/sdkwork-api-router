use sdkwork_api_storage_core::{StorageDialect, StorageDriverFactory, StorageDriverRegistry};

struct FakeDriverFactory {
    dialect: StorageDialect,
    driver_name: &'static str,
}

#[async_trait::async_trait]
impl StorageDriverFactory<String> for FakeDriverFactory {
    fn dialect(&self) -> StorageDialect {
        self.dialect
    }

    fn driver_name(&self) -> &'static str {
        self.driver_name
    }

    async fn build(&self, database_url: &str) -> anyhow::Result<String> {
        Ok(format!("{}:{database_url}", self.driver_name))
    }
}

#[tokio::test]
async fn storage_driver_registry_resolves_registered_factory_by_dialect() {
    let registry = StorageDriverRegistry::new()
        .with_factory(FakeDriverFactory {
            dialect: StorageDialect::Sqlite,
            driver_name: "sqlite-driver",
        })
        .with_factory(FakeDriverFactory {
            dialect: StorageDialect::Postgres,
            driver_name: "postgres-driver",
        });

    let driver = registry.resolve(StorageDialect::Sqlite).unwrap();
    assert_eq!(driver.driver_name(), "sqlite-driver");
    assert_eq!(
        driver.build("sqlite://router.db").await.unwrap(),
        "sqlite-driver:sqlite://router.db"
    );
}

#[test]
fn storage_driver_registry_returns_none_for_unregistered_dialect() {
    let registry = StorageDriverRegistry::<String>::new();
    assert!(registry.resolve(StorageDialect::Libsql).is_none());
}
