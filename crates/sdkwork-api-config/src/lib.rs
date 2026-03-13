use std::collections::HashMap;

use anyhow::{anyhow, Result};
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

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "database_encrypted" => Ok(Self::DatabaseEncrypted),
            "local_encrypted_file" => Ok(Self::LocalEncryptedFile),
            "os_keyring" => Ok(Self::OsKeyring),
            other => Err(anyhow!("unsupported secret backend: {other}")),
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
    pub fn from_env() -> Result<Self> {
        let pairs = std::env::vars();
        Self::from_pairs(pairs)
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values: HashMap<String, String> = pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        let default = Self::default();
        let secret_backend = match values.get("SDKWORK_SECRET_BACKEND") {
            Some(value) => SecretBackendKind::parse(value)?,
            None => default.secret_backend,
        };

        Ok(Self {
            gateway_bind: values
                .get("SDKWORK_GATEWAY_BIND")
                .cloned()
                .unwrap_or(default.gateway_bind),
            admin_bind: values
                .get("SDKWORK_ADMIN_BIND")
                .cloned()
                .unwrap_or(default.admin_bind),
            database_url: values
                .get("SDKWORK_DATABASE_URL")
                .cloned()
                .unwrap_or(default.database_url),
            secret_backend,
            credential_master_key: values
                .get("SDKWORK_CREDENTIAL_MASTER_KEY")
                .cloned()
                .unwrap_or(default.credential_master_key),
        })
    }

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
