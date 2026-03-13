use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCredential {
    pub tenant_id: String,
    pub provider_id: String,
    pub key_reference: String,
    pub secret_backend: String,
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
        }
    }
}
