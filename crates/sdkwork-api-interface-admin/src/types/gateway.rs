use super::*;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateApiKeyRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    #[serde(default)]
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
    #[serde(default)]
    pub(crate) expires_at_ms: Option<u64>,
    #[serde(default)]
    pub(crate) plaintext_key: Option<String>,
    #[serde(default)]
    pub(crate) api_key_group_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateApiKeyRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) notes: Option<String>,
    #[serde(default)]
    pub(crate) expires_at_ms: Option<u64>,
    #[serde(default)]
    pub(crate) api_key_group_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateApiKeyGroupRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) slug: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) default_capability_scope: Option<String>,
    #[serde(default)]
    pub(crate) default_accounting_mode: Option<String>,
    #[serde(default)]
    pub(crate) default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateApiKeyGroupRequest {
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) environment: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) slug: Option<String>,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) color: Option<String>,
    #[serde(default)]
    pub(crate) default_capability_scope: Option<String>,
    #[serde(default)]
    pub(crate) default_accounting_mode: Option<String>,
    #[serde(default)]
    pub(crate) default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRateLimitPolicyRequest {
    pub(crate) policy_id: String,
    pub(crate) project_id: String,
    pub(crate) requests_per_window: u64,
    #[serde(default = "default_window_seconds")]
    pub(crate) window_seconds: u64,
    #[serde(default)]
    pub(crate) burst_requests: u64,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) route_key: Option<String>,
    #[serde(default)]
    pub(crate) api_key_hash: Option<String>,
    #[serde(default)]
    pub(crate) model_name: Option<String>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
}
