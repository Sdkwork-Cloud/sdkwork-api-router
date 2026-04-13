use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateChannelRequest {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProviderRequest {
    pub(crate) id: String,
    pub(crate) channel_id: String,
    #[serde(default)]
    pub(crate) extension_id: Option<String>,
    #[serde(default)]
    pub(crate) protocol_kind: Option<String>,
    #[serde(default)]
    pub(crate) channel_bindings: Vec<CreateProviderChannelBindingRequest>,
    #[serde(default)]
    pub(crate) adapter_kind: Option<String>,
    #[serde(default)]
    pub(crate) default_plugin_family: Option<String>,
    pub(crate) base_url: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) supported_models: Option<Vec<CreateProviderModelRequest>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProviderChannelBindingRequest {
    pub(crate) channel_id: String,
    #[serde(default)]
    pub(crate) is_primary: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProviderModelRequest {
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    #[serde(default)]
    pub(crate) provider_model_id: Option<String>,
    #[serde(default)]
    pub(crate) provider_model_family: Option<String>,
    #[serde(default)]
    pub(crate) capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub(crate) streaming: Option<bool>,
    #[serde(default)]
    pub(crate) context_window: Option<u64>,
    #[serde(default)]
    pub(crate) max_output_tokens: Option<u64>,
    #[serde(default)]
    pub(crate) supports_prompt_caching: bool,
    #[serde(default)]
    pub(crate) supports_reasoning_usage: bool,
    #[serde(default)]
    pub(crate) supports_tool_usage_metrics: bool,
    #[serde(default)]
    pub(crate) is_default_route: bool,
    #[serde(default = "default_true")]
    pub(crate) is_active: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCredentialRequest {
    pub(crate) tenant_id: String,
    pub(crate) provider_id: String,
    pub(crate) key_reference: String,
    pub(crate) secret_value: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpsertOfficialProviderConfigRequest {
    pub(crate) provider_id: String,
    pub(crate) base_url: String,
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) api_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OfficialProviderConfigResponse {
    pub(crate) provider_id: String,
    pub(crate) base_url: String,
    pub(crate) enabled: bool,
    pub(crate) secret_configured: bool,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ListProvidersQuery {
    #[serde(default)]
    pub(crate) tenant_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProviderCatalogResponse {
    #[serde(flatten)]
    pub(crate) provider: ProxyProvider,
    pub(crate) integration: sdkwork_api_app_catalog::ProviderIntegrationView,
    pub(crate) execution: sdkwork_api_app_gateway::ProviderExecutionView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) credential_readiness:
        Option<sdkwork_api_app_credential::ProviderCredentialReadinessView>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProviderCreateResponse {
    #[serde(flatten)]
    pub(crate) provider: ProxyProvider,
    pub(crate) integration: sdkwork_api_app_catalog::ProviderIntegrationView,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct TenantProviderReadinessResponse {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) protocol_kind: String,
    pub(crate) integration: sdkwork_api_app_catalog::ProviderIntegrationView,
    pub(crate) credential_readiness: sdkwork_api_app_credential::ProviderCredentialReadinessView,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateModelRequest {
    pub(crate) external_name: String,
    pub(crate) provider_id: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub(crate) streaming: bool,
    pub(crate) context_window: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateChannelModelRequest {
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    pub(crate) model_display_name: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<ModelCapability>,
    #[serde(default)]
    pub(crate) streaming: bool,
    #[serde(default)]
    pub(crate) context_window: Option<u64>,
    #[serde(default)]
    pub(crate) description: Option<String>,
}

fn default_currency_code() -> String {
    "USD".to_owned()
}

fn default_price_unit() -> String {
    "per_1m_tokens".to_owned()
}

fn default_model_price_source_kind() -> String {
    "reference".to_owned()
}

fn default_provider_account_owner_scope() -> String {
    "platform".to_owned()
}

fn default_provider_account_weight() -> u32 {
    100
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateModelPriceRequest {
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    pub(crate) proxy_provider_id: String,
    #[serde(default = "default_currency_code")]
    pub(crate) currency_code: String,
    #[serde(default = "default_price_unit")]
    pub(crate) price_unit: String,
    #[serde(default)]
    pub(crate) input_price: f64,
    #[serde(default)]
    pub(crate) output_price: f64,
    #[serde(default)]
    pub(crate) cache_read_price: f64,
    #[serde(default)]
    pub(crate) cache_write_price: f64,
    #[serde(default)]
    pub(crate) request_price: f64,
    #[serde(default = "default_model_price_source_kind")]
    pub(crate) price_source_kind: String,
    #[serde(default)]
    pub(crate) billing_notes: Option<String>,
    #[serde(default)]
    pub(crate) pricing_tiers: Vec<ModelPriceTier>,
    #[serde(default = "default_true")]
    pub(crate) is_active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProviderAccountRequest {
    pub(crate) provider_account_id: String,
    pub(crate) provider_id: String,
    pub(crate) display_name: String,
    pub(crate) account_kind: String,
    #[serde(default = "default_provider_account_owner_scope")]
    pub(crate) owner_scope: String,
    #[serde(default)]
    pub(crate) owner_tenant_id: Option<String>,
    pub(crate) execution_instance_id: String,
    #[serde(default)]
    pub(crate) base_url_override: Option<String>,
    #[serde(default)]
    pub(crate) region: Option<String>,
    #[serde(default)]
    pub(crate) priority: i32,
    #[serde(default = "default_provider_account_weight")]
    pub(crate) weight: u32,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) routing_tags: Vec<String>,
    #[serde(default)]
    pub(crate) health_score_hint: Option<f64>,
    #[serde(default)]
    pub(crate) latency_ms_hint: Option<u64>,
    #[serde(default)]
    pub(crate) cost_hint: Option<f64>,
    #[serde(default)]
    pub(crate) success_rate_hint: Option<f64>,
    #[serde(default)]
    pub(crate) throughput_hint: Option<f64>,
    #[serde(default)]
    pub(crate) max_concurrency: Option<u32>,
    #[serde(default)]
    pub(crate) daily_budget: Option<f64>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
}
