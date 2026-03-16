use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCredential {
    pub tenant_id: String,
    pub provider_id: String,
    pub key_reference: String,
    pub secret_backend: String,
    #[serde(default)]
    pub secret_local_file: Option<String>,
    #[serde(default)]
    pub secret_keyring_service: Option<String>,
    #[serde(default)]
    pub secret_master_key_id: Option<String>,
}

impl UpstreamCredential {
    pub fn new(
        tenant_id: impl Into<String>,
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
    ) -> Self {
        Self::with_secret_backend(tenant_id, provider_id, key_reference, "database_encrypted")
    }

    pub fn with_secret_backend(
        tenant_id: impl Into<String>,
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
        secret_backend: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            provider_id: provider_id.into(),
            key_reference: key_reference.into(),
            secret_backend: secret_backend.into(),
            secret_local_file: None,
            secret_keyring_service: None,
            secret_master_key_id: None,
        }
    }

    pub fn with_secret_metadata(
        mut self,
        secret_local_file: Option<String>,
        secret_keyring_service: Option<String>,
        secret_master_key_id: Option<String>,
    ) -> Self {
        self.secret_local_file = secret_local_file;
        self.secret_keyring_service = secret_keyring_service;
        self.secret_master_key_id = secret_master_key_id;
        self
    }
}
