use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;
use sdkwork_api_secret_core::{CredentialSecretRef, SecretEnvelope};

pub fn backend_name() -> &'static str {
    "os_keyring"
}

pub trait KeyringBackend: Debug + Send + Sync {
    fn set_password(&self, service: &str, username: &str, secret: &str) -> Result<()>;
    fn get_password(&self, service: &str, username: &str) -> Result<Option<String>>;
}

#[derive(Debug, Default)]
pub struct OsKeyringBackend;

impl KeyringBackend for OsKeyringBackend {
    fn set_password(&self, service: &str, username: &str, secret: &str) -> Result<()> {
        let entry = keyring::Entry::new(service, username)?;
        entry.set_password(secret)?;
        Ok(())
    }

    fn get_password(&self, service: &str, username: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(service, username)?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyringSecretStore {
    service_name: String,
    backend: Arc<dyn KeyringBackend>,
}

impl KeyringSecretStore {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self::with_backend(service_name, Arc::new(OsKeyringBackend))
    }

    pub fn with_backend(service_name: impl Into<String>, backend: Arc<dyn KeyringBackend>) -> Self {
        Self {
            service_name: service_name.into(),
            backend,
        }
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn store_envelope(
        &self,
        secret_ref: &CredentialSecretRef,
        envelope: &SecretEnvelope,
    ) -> Result<()> {
        let payload = serde_json::to_string(envelope)?;
        self.backend
            .set_password(&self.service_name, &secret_ref.storage_key(), &payload)
    }

    pub fn load_envelope(
        &self,
        secret_ref: &CredentialSecretRef,
    ) -> Result<Option<SecretEnvelope>> {
        let Some(payload) = self
            .backend
            .get_password(&self.service_name, &secret_ref.storage_key())?
        else {
            return Ok(None);
        };

        Ok(Some(serde_json::from_str(&payload)?))
    }
}
