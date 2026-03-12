use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayApiKey {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    revoked: bool,
}

impl GatewayApiKey {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            revoked: false,
        }
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    pub fn is_active(&self) -> bool {
        !self.revoked
    }
}

pub trait GatewayApiKeyRepository: Send + Sync {
    fn save(&self, key: &GatewayApiKey) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayApiKeyRecord {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub hashed_key: String,
    pub active: bool,
}

impl GatewayApiKeyRecord {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
        hashed_key: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            hashed_key: hashed_key.into(),
            active: true,
        }
    }
}
