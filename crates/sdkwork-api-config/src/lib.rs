use sdkwork_api_storage_core::StorageDialect;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RuntimeMode {
    #[default]
    Server,
    Embedded,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SecretBackendKind {
    #[default]
    DatabaseEncrypted,
    LocalEncryptedFile,
    OsKeyring,
}

impl SecretBackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DatabaseEncrypted => "database_encrypted",
            Self::LocalEncryptedFile => "local_encrypted_file",
            Self::OsKeyring => "os_keyring",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfig {
    pub gateway_bind: String,
    pub admin_bind: String,
    pub database_url: String,
    pub secret_backend: SecretBackendKind,
    pub credential_master_key: String,
}

impl Default for StandaloneConfig {
    fn default() -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            database_url: "sqlite://sdkwork-api-server.db".to_owned(),
            secret_backend: SecretBackendKind::DatabaseEncrypted,
            credential_master_key: "local-dev-master-key".to_owned(),
        }
    }
}

impl StandaloneConfig {
    pub fn storage_dialect(&self) -> Option<StorageDialect> {
        let database_url = self.database_url.to_ascii_lowercase();

        if database_url.starts_with("sqlite:") {
            Some(StorageDialect::Sqlite)
        } else if database_url.starts_with("postgres://")
            || database_url.starts_with("postgresql://")
        {
            Some(StorageDialect::Postgres)
        } else if database_url.starts_with("mysql://") {
            Some(StorageDialect::Mysql)
        } else if database_url.starts_with("libsql://") || database_url.starts_with("turso://") {
            Some(StorageDialect::Libsql)
        } else {
            None
        }
    }
}
