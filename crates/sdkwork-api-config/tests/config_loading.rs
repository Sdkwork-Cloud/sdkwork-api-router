use sdkwork_api_config::RuntimeMode;
use sdkwork_api_config::SecretBackendKind;
use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_storage_core::StorageDialect;

#[test]
fn defaults_to_server_mode() {
    assert_eq!(RuntimeMode::default(), RuntimeMode::Server);
}

#[test]
fn standalone_defaults_are_local_friendly() {
    let config = StandaloneConfig::default();
    assert_eq!(config.gateway_bind, "127.0.0.1:8080");
    assert_eq!(config.admin_bind, "127.0.0.1:8081");
    assert_eq!(config.database_url, "sqlite://sdkwork-api-server.db");
    assert_eq!(config.secret_backend, SecretBackendKind::DatabaseEncrypted);
    assert_eq!(config.storage_dialect().unwrap(), StorageDialect::Sqlite);
}

#[test]
fn infers_postgres_dialect_from_database_url() {
    let config = StandaloneConfig {
        database_url: "postgres://sdkwork:secret@localhost/sdkwork".to_owned(),
        ..StandaloneConfig::default()
    };

    assert_eq!(config.storage_dialect().unwrap(), StorageDialect::Postgres);
}

#[test]
fn supports_three_secret_backend_strategies() {
    assert_eq!(
        SecretBackendKind::DatabaseEncrypted.as_str(),
        "database_encrypted"
    );
    assert_eq!(
        SecretBackendKind::LocalEncryptedFile.as_str(),
        "local_encrypted_file"
    );
    assert_eq!(SecretBackendKind::OsKeyring.as_str(), "os_keyring");
}
