use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::ensure;
use axum::{
    body::Bytes,
    extract::FromRequestParts,
    extract::Path,
    extract::Query,
    extract::State,
    http::header,
    http::request::Parts,
    http::StatusCode,
    response::Html,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use sdkwork_api_app_billing::{
    create_quota_policy, list_billing_events, list_ledger_entries, list_quota_policies,
    persist_quota_policy, summarize_billing_events_from_store, summarize_billing_from_store,
};
use sdkwork_api_app_catalog::{
    delete_channel as delete_catalog_channel, delete_channel_model as delete_catalog_channel_model,
    delete_model_price as delete_catalog_model_price, delete_model_variant,
    delete_provider as delete_catalog_provider, list_channel_models, list_channels,
    list_model_entries, list_model_prices, list_providers, persist_channel,
    persist_channel_model_with_metadata, persist_model_price_with_rates,
    persist_model_with_metadata, persist_provider_with_bindings_and_extension_id,
    PersistProviderWithBindingsRequest,
};
use sdkwork_api_app_coupon::{delete_coupon, list_coupons, persist_coupon};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_credential::{
    delete_credential_with_manager, delete_provider_credentials_with_manager,
    delete_tenant_credentials_with_manager, list_credentials,
    persist_credential_with_secret_and_manager,
};
use sdkwork_api_app_extension::{
    configured_extension_discovery_policy_from_env, list_discovered_extension_packages,
    list_extension_installations, list_extension_instances, list_extension_runtime_statuses,
    list_provider_health_snapshots, persist_extension_installation, persist_extension_instance,
    PersistExtensionInstanceInput,
};
use sdkwork_api_app_gateway::{
    invalidate_capability_catalog_cache, reload_extension_host_with_scope,
    ConfiguredExtensionHostReloadScope,
};
use sdkwork_api_app_identity::{
    change_admin_password, create_api_key_group, delete_admin_user, delete_api_key_group,
    delete_gateway_api_key, delete_portal_user, list_admin_user_profiles, list_api_key_groups,
    list_gateway_api_keys, list_portal_user_profiles, load_admin_user_profile, login_admin_user,
    reset_admin_user_password, reset_portal_user_password, set_admin_user_active,
    set_api_key_group_active, set_gateway_api_key_active, set_portal_user_active,
    update_api_key_group, update_gateway_api_key_metadata, upsert_admin_user, upsert_portal_user,
    verify_jwt, AdminIdentityError, ApiKeyGroupInput, Claims, CreatedGatewayApiKey,
    PortalIdentityError,
};
use sdkwork_api_app_payment::{
    approve_refund_order_request, cancel_refund_order_request, load_admin_payment_order_dossier,
    start_refund_order_execution, AdminPaymentOrderDossier,
};
use sdkwork_api_app_rate_limit::{
    create_rate_limit_policy, list_rate_limit_policies, persist_rate_limit_policy,
};
use sdkwork_api_app_routing::{
    create_routing_policy, create_routing_profile, list_compiled_routing_snapshots,
    list_routing_decision_logs, list_routing_policies, list_routing_profiles,
    persist_routing_policy, persist_routing_profile, select_route_with_store_context,
    CreateRoutingPolicyInput, CreateRoutingProfileInput, RouteSelectionContext,
};
use sdkwork_api_app_runtime::{
    create_extension_runtime_rollout_with_request, create_standalone_config_rollout,
    find_extension_runtime_rollout, find_standalone_config_rollout,
    list_extension_runtime_rollouts, list_standalone_config_rollouts,
    CreateExtensionRuntimeRolloutRequest, CreateStandaloneConfigRolloutRequest,
    ExtensionRuntimeRolloutDetails, StandaloneConfigRolloutDetails,
};
use sdkwork_api_app_tenant::{
    delete_project as delete_tenant_project, delete_tenant as delete_workspace_tenant,
    list_projects, list_tenants, persist_project, persist_tenant,
};
use sdkwork_api_app_usage::list_usage_records;
use sdkwork_api_app_usage::summarize_usage_records_from_store;
use sdkwork_api_domain_billing::{
    BillingEventRecord, BillingEventSummary, BillingSummary, LedgerEntry, QuotaPolicy,
};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCapability, ModelCatalogEntry, ModelPriceRecord,
    ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{
    AdminUserProfile, ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserProfile,
};
use sdkwork_api_domain_payment::{
    PaymentChannelPolicyRecord, PaymentGatewayAccountRecord, PaymentOrderRecord,
    PaymentProviderCode, ReconciliationMatchStatus, ReconciliationMatchSummaryRecord,
    RefundOrderRecord, RefundOrderStatus,
};
use sdkwork_api_domain_rate_limit::{RateLimitPolicy, RateLimitWindowSnapshot};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProviderHealthSnapshot, RoutingCandidateAssessment,
    RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy, RoutingProfileRecord,
    RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_openapi::{
    build_openapi_document, extract_routes_from_function, render_docs_html, HttpMethod,
    OpenApiServiceSpec, RouteEntry,
};
use sdkwork_api_storage_core::{AdminStore, PaymentKernelStore, Reloadable};
use sdkwork_api_storage_postgres::PostgresAdminStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

const DEFAULT_ADMIN_JWT_SIGNING_SECRET: &str = "local-dev-admin-jwt-secret";
const ADMIN_OPENAPI_SPEC: OpenApiServiceSpec<'static> = OpenApiServiceSpec {
    title: "SDKWORK Admin API",
    version: env!("CARGO_PKG_VERSION"),
    description: "OpenAPI 3.1 inventory generated from the current admin router implementation.",
    openapi_path: "/admin/openapi.json",
    docs_path: "/admin/docs",
};

fn admin_route_inventory() -> &'static [RouteEntry] {
    static ROUTES: OnceLock<Vec<RouteEntry>> = OnceLock::new();
    ROUTES
        .get_or_init(|| {
            extract_routes_from_function(include_str!("lib.rs"), "admin_router_with_state")
                .expect("admin route inventory")
        })
        .as_slice()
}

fn admin_openapi_document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        build_openapi_document(
            &ADMIN_OPENAPI_SPEC,
            admin_route_inventory(),
            admin_tag_for_path,
            admin_route_requires_bearer_auth,
            admin_operation_summary,
        )
    })
}

fn admin_docs_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| render_docs_html(&ADMIN_OPENAPI_SPEC))
        .as_str()
}

async fn admin_openapi_handler() -> Json<Value> {
    Json(admin_openapi_document().clone())
}

async fn admin_docs_handler() -> Html<String> {
    Html(admin_docs_html().to_owned())
}

fn admin_tag_for_path(path: &str) -> String {
    match path {
        "/metrics" | "/admin/health" => "system".to_owned(),
        "/admin/docs" | "/admin/openapi.json" => "docs".to_owned(),
        _ if path.starts_with("/admin/") => path
            .trim_start_matches("/admin/")
            .split('/')
            .find(|segment| !segment.is_empty() && !segment.starts_with('{'))
            .unwrap_or("admin")
            .to_owned(),
        _ => "admin".to_owned(),
    }
}

fn admin_route_requires_bearer_auth(path: &str, _method: HttpMethod) -> bool {
    !matches!(
        path,
        "/metrics" | "/admin/health" | "/admin/openapi.json" | "/admin/docs" | "/admin/auth/login"
    )
}

fn admin_operation_summary(path: &str, method: HttpMethod) -> String {
    match path {
        "/metrics" => "Prometheus metrics".to_owned(),
        "/admin/health" => "Health check".to_owned(),
        "/admin/openapi.json" => "OpenAPI document".to_owned(),
        "/admin/docs" => "Interactive API inventory".to_owned(),
        _ => format!(
            "{} {}",
            method.display_name(),
            humanize_admin_route_path(path)
        ),
    }
}

fn humanize_admin_route_path(path: &str) -> String {
    let parts = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .filter(|segment| *segment != "admin")
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                format!(
                    "by {}",
                    segment
                        .trim_matches(|ch| ch == '{' || ch == '}')
                        .replace(['_', '-'], " ")
                )
            } else {
                segment.replace(['_', '-'], " ")
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "root".to_owned()
    } else {
        parts.join(" / ")
    }
}

pub struct AdminApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for AdminApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            store: self.live_store.snapshot(),
            secret_manager: self.live_secret_manager.snapshot(),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
        }
    }
}

impl AdminApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_master_key(pool, "local-dev-master-key")
    }

    pub fn with_master_key(pool: SqlitePool, credential_master_key: impl Into<String>) -> Self {
        Self::with_store_and_secret_manager(
            Arc::new(SqliteAdminStore::new(pool)),
            CredentialSecretManager::database_encrypted(credential_master_key),
        )
    }

    pub fn with_secret_manager(pool: SqlitePool, secret_manager: CredentialSecretManager) -> Self {
        Self::with_secret_manager_and_jwt_secret(
            pool,
            secret_manager,
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
        )
    }

    pub fn with_secret_manager_and_jwt_secret(
        pool: SqlitePool,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret(
            Arc::new(SqliteAdminStore::new(pool)),
            secret_manager,
            jwt_signing_secret,
        )
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret(
            store,
            secret_manager,
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_secret_manager_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
            live_store,
            Reloadable::new(secret_manager),
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            live_store,
            live_secret_manager,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
        }
    }
}

#[derive(Clone, Debug)]
struct AuthenticatedAdminClaims(Claims);

impl AuthenticatedAdminClaims {
    fn claims(&self) -> &Claims {
        &self.0
    }
}

impl FromRequestParts<AdminApiState> for AuthenticatedAdminClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AdminApiState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Ok(header_value) = header_value.to_str() else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Some(token) = header_value
            .strip_prefix("Bearer ")
            .or_else(|| header_value.strip_prefix("bearer "))
        else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        verify_jwt(token, &state.jwt_signing_secret)
            .map(Self)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    claims: Claims,
    user: AdminUserProfile,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

fn default_user_active() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct UpsertOperatorUserRequest {
    #[serde(default)]
    id: Option<String>,
    email: String,
    display_name: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default = "default_user_active")]
    active: bool,
}

#[derive(Debug, Deserialize)]
struct UpsertPortalUserRequest {
    #[serde(default)]
    id: Option<String>,
    email: String,
    display_name: String,
    #[serde(default)]
    password: Option<String>,
    workspace_tenant_id: String,
    workspace_project_id: String,
    #[serde(default = "default_user_active")]
    active: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateUserStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize)]
struct ResetUserPasswordRequest {
    new_password: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

#[derive(Debug, Deserialize)]
struct CreateChannelRequest {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateProviderRequest {
    id: String,
    channel_id: String,
    #[serde(default)]
    extension_id: Option<String>,
    #[serde(default)]
    channel_bindings: Vec<CreateProviderChannelBindingRequest>,
    adapter_kind: String,
    base_url: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct CreateProviderChannelBindingRequest {
    channel_id: String,
    #[serde(default)]
    is_primary: bool,
}

#[derive(Debug, Deserialize)]
struct CreateCredentialRequest {
    tenant_id: String,
    provider_id: String,
    key_reference: String,
    secret_value: String,
}

#[derive(Debug, Deserialize)]
struct CreateModelRequest {
    external_name: String,
    provider_id: String,
    #[serde(default)]
    capabilities: Vec<ModelCapability>,
    #[serde(default)]
    streaming: bool,
    context_window: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CreateChannelModelRequest {
    channel_id: String,
    model_id: String,
    model_display_name: String,
    #[serde(default)]
    capabilities: Vec<ModelCapability>,
    #[serde(default)]
    streaming: bool,
    #[serde(default)]
    context_window: Option<u64>,
    #[serde(default)]
    description: Option<String>,
}

fn default_currency_code() -> String {
    "USD".to_owned()
}

fn default_price_unit() -> String {
    "per_1m_tokens".to_owned()
}

#[derive(Debug, Deserialize)]
struct CreateModelPriceRequest {
    channel_id: String,
    model_id: String,
    proxy_provider_id: String,
    #[serde(default = "default_currency_code")]
    currency_code: String,
    #[serde(default = "default_price_unit")]
    price_unit: String,
    #[serde(default)]
    input_price: f64,
    #[serde(default)]
    output_price: f64,
    #[serde(default)]
    cache_read_price: f64,
    #[serde(default)]
    cache_write_price: f64,
    #[serde(default)]
    request_price: f64,
    #[serde(default = "default_true")]
    is_active: bool,
}

#[derive(Debug, Deserialize)]
struct CreateTenantRequest {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateProjectRequest {
    tenant_id: String,
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateCouponRequest {
    id: String,
    code: String,
    discount_label: String,
    audience: String,
    remaining: u64,
    active: bool,
    note: String,
    expires_on: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
    #[serde(default)]
    plaintext_key: Option<String>,
    #[serde(default)]
    api_key_group_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ResolveReconciliationLineRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolved_at_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
struct PaymentReconciliationReasonBreakdownItem {
    reason_code: String,
    count: usize,
    latest_updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
struct PaymentReconciliationSummaryResponse {
    total_count: usize,
    active_count: usize,
    resolved_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_updated_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldest_active_created_at_ms: Option<u64>,
    active_reason_breakdown: Vec<PaymentReconciliationReasonBreakdownItem>,
}

#[derive(Debug, Deserialize, Default)]
struct PaymentReconciliationListQuery {
    #[serde(default)]
    lifecycle: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct PaymentGatewayAccountListQuery {
    #[serde(default)]
    provider_code: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    tenant_id: Option<u64>,
    #[serde(default)]
    organization_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpsertPaymentGatewayAccountRequest {
    gateway_account_id: String,
    tenant_id: u64,
    organization_id: u64,
    provider_code: String,
    environment: String,
    merchant_id: String,
    app_id: String,
    status: String,
    priority: i32,
    #[serde(default)]
    created_at_ms: Option<u64>,
    #[serde(default)]
    updated_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct PaymentChannelPolicyListQuery {
    #[serde(default)]
    provider_code: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    tenant_id: Option<u64>,
    #[serde(default)]
    organization_id: Option<u64>,
    #[serde(default)]
    scene_code: Option<String>,
    #[serde(default)]
    currency_code: Option<String>,
    #[serde(default)]
    client_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpsertPaymentChannelPolicyRequest {
    channel_policy_id: String,
    tenant_id: u64,
    organization_id: u64,
    scene_code: String,
    country_code: String,
    currency_code: String,
    client_kind: String,
    provider_code: String,
    method_code: String,
    priority: i32,
    status: String,
    #[serde(default)]
    created_at_ms: Option<u64>,
    #[serde(default)]
    updated_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ApproveRefundOrderRequest {
    #[serde(default)]
    approved_amount_minor: Option<u64>,
    approved_at_ms: u64,
}

#[derive(Debug, Deserialize)]
struct CancelRefundOrderRequest {
    canceled_at_ms: u64,
}

#[derive(Debug, Deserialize)]
struct StartRefundOrderRequest {
    started_at_ms: u64,
}

#[derive(Debug, Deserialize, Default)]
struct RefundOrderListQuery {
    #[serde(default)]
    refund_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaymentReconciliationLifecycle {
    All,
    Active,
    Resolved,
}

impl PaymentReconciliationLifecycle {
    fn parse(raw: Option<&str>) -> anyhow::Result<Self> {
        match raw.unwrap_or("all") {
            "all" => Ok(Self::All),
            "active" => Ok(Self::Active),
            "resolved" => Ok(Self::Resolved),
            other => Err(anyhow::anyhow!(
                "unsupported reconciliation lifecycle filter: {other}"
            )),
        }
    }

    fn matches(self, line: &ReconciliationMatchSummaryRecord) -> bool {
        match self {
            Self::All => true,
            Self::Active => !matches!(line.match_status, ReconciliationMatchStatus::Resolved),
            Self::Resolved => matches!(line.match_status, ReconciliationMatchStatus::Resolved),
        }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
    label: String,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
    #[serde(default)]
    api_key_group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyGroupRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyGroupRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateExtensionInstallationRequest {
    installation_id: String,
    extension_id: String,
    runtime: ExtensionRuntime,
    enabled: bool,
    entrypoint: Option<String>,
    #[serde(default)]
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CreateExtensionInstanceRequest {
    instance_id: String,
    installation_id: String,
    extension_id: String,
    enabled: bool,
    base_url: Option<String>,
    credential_ref: Option<String>,
    #[serde(default)]
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CreateRoutingPolicyRequest {
    policy_id: String,
    capability: String,
    model_pattern: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    strategy: Option<RoutingStrategy>,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
}

#[derive(Debug, Deserialize)]
struct CreateRoutingProfileRequest {
    profile_id: String,
    tenant_id: String,
    project_id: String,
    name: String,
    slug: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_true")]
    active: bool,
    #[serde(default)]
    strategy: Option<RoutingStrategy>,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
    #[serde(default)]
    preferred_region: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateQuotaPolicyRequest {
    policy_id: String,
    project_id: String,
    max_units: u64,
    #[serde(default = "default_true")]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct CreateRateLimitPolicyRequest {
    policy_id: String,
    project_id: String,
    requests_per_window: u64,
    #[serde(default = "default_window_seconds")]
    window_seconds: u64,
    #[serde(default)]
    burst_requests: u64,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    route_key: Option<String>,
    #[serde(default)]
    api_key_hash: Option<String>,
    #[serde(default)]
    model_name: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoutingSimulationRequest {
    capability: String,
    model: String,
    #[serde(default)]
    tenant_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    api_key_group_id: Option<String>,
    #[serde(default)]
    requested_region: Option<String>,
    #[serde(default)]
    selection_seed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct RoutingSimulationResponse {
    selected_provider_id: String,
    candidate_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    applied_routing_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compiled_routing_snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_region: Option<String>,
    #[serde(default)]
    slo_applied: bool,
    #[serde(default)]
    slo_degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_candidate: Option<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    rejected_candidates: Vec<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assessments: Vec<RoutingCandidateAssessment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExtensionRuntimeReloadScope {
    All,
    Extension,
    Instance,
}

#[derive(Debug, Deserialize, Default)]
struct ExtensionRuntimeReloadRequest {
    #[serde(default)]
    extension_id: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ExtensionRuntimeRolloutCreateRequest {
    #[serde(default)]
    extension_id: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ExtensionRuntimeReloadResponse {
    scope: ExtensionRuntimeReloadScope,
    requested_extension_id: Option<String>,
    requested_instance_id: Option<String>,
    resolved_extension_id: Option<String>,
    discovered_package_count: usize,
    loadable_package_count: usize,
    active_runtime_count: usize,
    reloaded_at_ms: u64,
    runtime_statuses: Vec<sdkwork_api_app_extension::ExtensionRuntimeStatusRecord>,
}

struct ResolvedExtensionRuntimeReloadRequest {
    scope: ExtensionRuntimeReloadScope,
    requested_extension_id: Option<String>,
    requested_instance_id: Option<String>,
    resolved_extension_id: Option<String>,
    gateway_scope: ConfiguredExtensionHostReloadScope,
}

#[derive(Debug, Serialize)]
struct ExtensionRuntimeRolloutParticipantResponse {
    node_id: String,
    service_kind: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
struct ExtensionRuntimeRolloutResponse {
    rollout_id: String,
    status: String,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_extension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_extension_id: Option<String>,
    created_by: String,
    created_at_ms: u64,
    deadline_at_ms: u64,
    participant_count: usize,
    participants: Vec<ExtensionRuntimeRolloutParticipantResponse>,
}

impl From<ExtensionRuntimeRolloutDetails> for ExtensionRuntimeRolloutResponse {
    fn from(value: ExtensionRuntimeRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            scope: value.scope,
            requested_extension_id: value.requested_extension_id,
            requested_instance_id: value.requested_instance_id,
            resolved_extension_id: value.resolved_extension_id,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| ExtensionRuntimeRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct StandaloneConfigRolloutCreateRequest {
    #[serde(default)]
    service_kind: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
struct StandaloneConfigRolloutParticipantResponse {
    node_id: String,
    service_kind: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
struct StandaloneConfigRolloutResponse {
    rollout_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_service_kind: Option<String>,
    created_by: String,
    created_at_ms: u64,
    deadline_at_ms: u64,
    participant_count: usize,
    participants: Vec<StandaloneConfigRolloutParticipantResponse>,
}

impl From<StandaloneConfigRolloutDetails> for StandaloneConfigRolloutResponse {
    fn from(value: StandaloneConfigRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            requested_service_kind: value.requested_service_kind,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| StandaloneConfigRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

pub fn admin_router() -> Router {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_handler))
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                move || {
                    let metrics = metrics.clone();
                    async move {
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            metrics.render_prometheus(),
                        )
                    }
                }
            }),
        )
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(|| async { "login" }))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route(
            "/admin/auth/change-password",
            post(|| async { "change-password" }),
        )
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
        .route("/admin/api-key-groups", get(|| async { "api-key-groups" }))
        .route(
            "/admin/api-key-groups/{group_id}",
            patch(|| async { "api-key-groups" }).delete(|| async { "api-key-groups" }),
        )
        .route(
            "/admin/api-key-groups/{group_id}/status",
            post(|| async { "api-key-groups-status" }),
        )
        .route("/admin/channels", get(|| async { "channels" }))
        .route("/admin/providers", get(|| async { "providers" }))
        .route("/admin/credentials", get(|| async { "credentials" }))
        .route("/admin/channel-models", get(|| async { "channel-models" }))
        .route("/admin/models", get(|| async { "models" }))
        .route("/admin/model-prices", get(|| async { "model-prices" }))
        .route(
            "/admin/extensions/installations",
            get(|| async { "extension-installations" }),
        )
        .route(
            "/admin/extensions/packages",
            get(|| async { "extension-packages" }),
        )
        .route(
            "/admin/extensions/instances",
            get(|| async { "extension-instances" }),
        )
        .route(
            "/admin/extensions/runtime-statuses",
            get(|| async { "extension-runtime-statuses" }),
        )
        .route(
            "/admin/extensions/runtime-reloads",
            post(|| async { "extension-runtime-reloads" }),
        )
        .route(
            "/admin/runtime-config/rollouts",
            get(|| async { "runtime-config-rollouts" })
                .post(|| async { "runtime-config-rollouts-create" }),
        )
        .route(
            "/admin/runtime-config/rollouts/{rollout_id}",
            get(|| async { "runtime-config-rollout" }),
        )
        .route("/admin/usage/records", get(|| async { "usage-records" }))
        .route("/admin/usage/summary", get(|| async { "usage-summary" }))
        .route("/admin/billing/events", get(|| async { "billing-events" }))
        .route(
            "/admin/billing/events/summary",
            get(|| async { "billing-events-summary" }),
        )
        .route("/admin/billing/ledger", get(|| async { "billing-ledger" }))
        .route(
            "/admin/billing/summary",
            get(|| async { "billing-summary" }),
        )
        .route(
            "/admin/billing/quota-policies",
            get(|| async { "billing-quota-policies" }),
        )
        .route(
            "/admin/gateway/rate-limit-policies",
            get(|| async { "gateway-rate-limit-policies" }),
        )
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route("/admin/routing/profiles", get(|| async { "profiles" }))
        .route("/admin/routing/snapshots", get(|| async { "snapshots" }))
        .route(
            "/admin/routing/health-snapshots",
            get(|| async { "health-snapshots" }),
        )
        .route(
            "/admin/routing/decision-logs",
            get(|| async { "decision-logs" }),
        )
        .route(
            "/admin/routing/simulations",
            post(|| async { "simulations" }),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
}

pub fn admin_router_with_pool(pool: SqlitePool) -> Router {
    admin_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn admin_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    admin_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn admin_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    admin_router_with_store_and_secret_manager(
        Arc::new(SqliteAdminStore::new(pool)),
        CredentialSecretManager::database_encrypted(credential_master_key),
    )
}

pub fn admin_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    admin_router_with_store_and_secret_manager(
        Arc::new(SqliteAdminStore::new(pool)),
        secret_manager,
    )
}

pub fn admin_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> Router {
    admin_router_with_store_and_secret_manager_and_jwt_secret(
        store,
        secret_manager,
        DEFAULT_ADMIN_JWT_SIGNING_SECRET,
    )
}

pub fn admin_router_with_store_and_secret_manager_and_jwt_secret(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    admin_router_with_state(AdminApiState::with_store_and_secret_manager_and_jwt_secret(
        store,
        secret_manager,
        jwt_signing_secret,
    ))
}

pub fn admin_router_with_state(state: AdminApiState) -> Router {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    let metrics_store = state.store.clone();
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_handler))
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                let metrics_store = metrics_store.clone();
                move || {
                    let metrics = metrics.clone();
                    let metrics_store = metrics_store.clone();
                    async move {
                        let body =
                            render_admin_metrics_payload(metrics.as_ref(), metrics_store.as_ref())
                                .await;
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            body,
                        )
                    }
                }
            }),
        )
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/auth/me", get(me_handler))
        .route("/admin/auth/change-password", post(change_password_handler))
        .route(
            "/admin/users/operators",
            get(list_operator_users_handler).post(upsert_operator_user_handler),
        )
        .route(
            "/admin/users/operators/{user_id}",
            delete(delete_operator_user_handler),
        )
        .route(
            "/admin/users/operators/{user_id}/status",
            post(update_operator_user_status_handler),
        )
        .route(
            "/admin/users/operators/{user_id}/password",
            post(reset_operator_user_password_handler),
        )
        .route(
            "/admin/users/portal",
            get(list_portal_users_handler).post(upsert_portal_user_handler),
        )
        .route(
            "/admin/users/portal/{user_id}",
            delete(delete_portal_user_handler),
        )
        .route(
            "/admin/users/portal/{user_id}/status",
            post(update_portal_user_status_handler),
        )
        .route(
            "/admin/users/portal/{user_id}/password",
            post(reset_portal_user_password_handler),
        )
        .route(
            "/admin/coupons",
            get(list_coupons_handler).post(create_coupon_handler),
        )
        .route("/admin/coupons/{coupon_id}", delete(delete_coupon_handler))
        .route(
            "/admin/tenants",
            get(list_tenants_handler).post(create_tenant_handler),
        )
        .route("/admin/tenants/{tenant_id}", delete(delete_tenant_handler))
        .route(
            "/admin/projects",
            get(list_projects_handler).post(create_project_handler),
        )
        .route(
            "/admin/projects/{project_id}",
            delete(delete_project_handler),
        )
        .route(
            "/admin/api-key-groups",
            get(list_api_key_groups_handler).post(create_api_key_group_handler),
        )
        .route(
            "/admin/api-key-groups/{group_id}/status",
            post(update_api_key_group_status_handler),
        )
        .route(
            "/admin/api-key-groups/{group_id}",
            patch(update_api_key_group_handler).delete(delete_api_key_group_handler),
        )
        .route(
            "/admin/api-keys",
            get(list_api_keys_handler).post(create_api_key_handler),
        )
        .route(
            "/admin/api-keys/{hashed_key}/status",
            post(update_api_key_status_handler),
        )
        .route(
            "/admin/api-keys/{hashed_key}",
            put(update_api_key_handler).delete(delete_api_key_handler),
        )
        .route(
            "/admin/channels",
            get(list_channels_handler).post(create_channel_handler),
        )
        .route(
            "/admin/channels/{channel_id}",
            delete(delete_channel_handler),
        )
        .route(
            "/admin/providers",
            get(list_providers_handler).post(create_provider_handler),
        )
        .route(
            "/admin/providers/{provider_id}",
            delete(delete_provider_handler),
        )
        .route(
            "/admin/credentials",
            get(list_credentials_handler).post(create_credential_handler),
        )
        .route(
            "/admin/credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}",
            delete(delete_credential_handler),
        )
        .route(
            "/admin/channel-models",
            get(list_channel_models_handler).post(create_channel_model_handler),
        )
        .route(
            "/admin/channel-models/{channel_id}/models/{model_id}",
            delete(delete_channel_model_handler),
        )
        .route(
            "/admin/models",
            get(list_models_handler).post(create_model_handler),
        )
        .route(
            "/admin/models/{external_name}/providers/{provider_id}",
            delete(delete_model_handler),
        )
        .route(
            "/admin/model-prices",
            get(list_model_prices_handler).post(create_model_price_handler),
        )
        .route(
            "/admin/model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}",
            delete(delete_model_price_handler),
        )
        .route(
            "/admin/extensions/installations",
            get(list_extension_installations_handler).post(create_extension_installation_handler),
        )
        .route(
            "/admin/extensions/packages",
            get(list_extension_packages_handler),
        )
        .route(
            "/admin/extensions/instances",
            get(list_extension_instances_handler).post(create_extension_instance_handler),
        )
        .route(
            "/admin/extensions/runtime-statuses",
            get(list_extension_runtime_statuses_handler),
        )
        .route(
            "/admin/extensions/runtime-reloads",
            post(reload_extension_runtimes_handler),
        )
        .route(
            "/admin/extensions/runtime-rollouts",
            get(list_extension_runtime_rollouts_handler)
                .post(create_extension_runtime_rollout_handler),
        )
        .route(
            "/admin/extensions/runtime-rollouts/{rollout_id}",
            get(get_extension_runtime_rollout_handler),
        )
        .route(
            "/admin/runtime-config/rollouts",
            get(list_standalone_config_rollouts_handler)
                .post(create_standalone_config_rollout_handler),
        )
        .route(
            "/admin/runtime-config/rollouts/{rollout_id}",
            get(get_standalone_config_rollout_handler),
        )
        .route("/admin/usage/records", get(list_usage_records_handler))
        .route("/admin/usage/summary", get(usage_summary_handler))
        .route("/admin/billing/events", get(list_billing_events_handler))
        .route(
            "/admin/billing/events/summary",
            get(billing_events_summary_handler),
        )
        .route("/admin/billing/ledger", get(list_ledger_entries_handler))
        .route("/admin/billing/summary", get(billing_summary_handler))
        .route("/admin/payments/orders", get(list_payment_orders_handler))
        .route(
            "/admin/payments/orders/{payment_order_id}",
            get(get_payment_order_dossier_handler),
        )
        .route("/admin/payments/refunds", get(list_refund_orders_handler))
        .route(
            "/admin/payments/refunds/{refund_order_id}/approve",
            post(approve_refund_order_handler),
        )
        .route(
            "/admin/payments/refunds/{refund_order_id}/start",
            post(start_refund_order_handler),
        )
        .route(
            "/admin/payments/refunds/{refund_order_id}/cancel",
            post(cancel_refund_order_handler),
        )
        .route(
            "/admin/payments/gateway-accounts",
            get(list_payment_gateway_accounts_handler).post(upsert_payment_gateway_account_handler),
        )
        .route(
            "/admin/payments/channel-policies",
            get(list_payment_channel_policies_handler).post(upsert_payment_channel_policy_handler),
        )
        .route(
            "/admin/payments/reconciliation-summary",
            get(payment_reconciliation_summary_handler),
        )
        .route(
            "/admin/payments/reconciliation-lines",
            get(list_payment_reconciliation_lines_handler),
        )
        .route(
            "/admin/payments/reconciliation-lines/{reconciliation_line_id}/resolve",
            post(resolve_payment_reconciliation_line_handler),
        )
        .route(
            "/admin/billing/quota-policies",
            get(list_quota_policies_handler).post(create_quota_policy_handler),
        )
        .route(
            "/admin/gateway/rate-limit-policies",
            get(list_rate_limit_policies_handler).post(create_rate_limit_policy_handler),
        )
        .route(
            "/admin/gateway/rate-limit-windows",
            get(list_rate_limit_window_snapshots_handler),
        )
        .route(
            "/admin/routing/policies",
            get(list_routing_policies_handler).post(create_routing_policy_handler),
        )
        .route(
            "/admin/routing/profiles",
            get(list_routing_profiles_handler).post(create_routing_profile_handler),
        )
        .route(
            "/admin/routing/snapshots",
            get(list_compiled_routing_snapshots_handler),
        )
        .route(
            "/admin/routing/health-snapshots",
            get(list_provider_health_snapshots_handler),
        )
        .route(
            "/admin/routing/decision-logs",
            get(list_routing_decision_logs_handler),
        )
        .route("/admin/routing/simulations", post(simulate_routing_handler))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

async fn login_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session = login_admin_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map_err(admin_error_response)?;
    let token = session.token.clone();
    let claims = verify_jwt(&token, &state.jwt_signing_secret)
        .map_err(|error| admin_error_response(AdminIdentityError::Storage(error)))?;
    Ok(Json(LoginResponse {
        token,
        claims,
        user: session.user,
    }))
}

async fn me_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<AdminUserProfile>, StatusCode> {
    load_admin_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn change_password_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_admin_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(admin_error_response)
}

async fn list_operator_users_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AdminUserProfile>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_user_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_error_response)
}

async fn upsert_operator_user_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertOperatorUserRequest>,
) -> Result<(StatusCode, Json<AdminUserProfile>), (StatusCode, Json<ErrorResponse>)> {
    upsert_admin_user(
        state.store.as_ref(),
        request.id.as_deref(),
        &request.email,
        &request.display_name,
        request.password.as_deref(),
        request.active,
    )
    .await
    .map(|user| (StatusCode::CREATED, Json(user)))
    .map_err(admin_error_response)
}

async fn update_operator_user_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    set_admin_user_active(state.store.as_ref(), &user_id, request.active)
        .await
        .map(Json)
        .map_err(admin_error_response)
}

async fn reset_operator_user_password_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResetUserPasswordRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    reset_admin_user_password(state.store.as_ref(), &user_id, &request.new_password)
        .await
        .map(Json)
        .map_err(admin_error_response)
}

async fn delete_operator_user_handler(
    claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if claims.claims().sub == user_id {
        return Err(admin_error_response(AdminIdentityError::Protected(
            "current admin session cannot be deleted".to_owned(),
        )));
    }

    match delete_admin_user(state.store.as_ref(), &user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(admin_error_response(AdminIdentityError::NotFound(
            "admin user not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn list_portal_users_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PortalUserProfile>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_user_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

async fn upsert_portal_user_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPortalUserRequest>,
) -> Result<(StatusCode, Json<PortalUserProfile>), (StatusCode, Json<ErrorResponse>)> {
    upsert_portal_user(
        state.store.as_ref(),
        request.id.as_deref(),
        &request.email,
        &request.display_name,
        request.password.as_deref(),
        &request.workspace_tenant_id,
        &request.workspace_project_id,
        request.active,
    )
    .await
    .map(|user| (StatusCode::CREATED, Json(user)))
    .map_err(portal_admin_error_response)
}

async fn update_portal_user_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    set_portal_user_active(state.store.as_ref(), &user_id, request.active)
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

async fn reset_portal_user_password_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResetUserPasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    reset_portal_user_password(state.store.as_ref(), &user_id, &request.new_password)
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

async fn delete_portal_user_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_portal_user(state.store.as_ref(), &user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(portal_admin_error_response(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ))),
        Err(error) => Err(portal_admin_error_response(error)),
    }
}

async fn list_coupons_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCampaign>>, StatusCode> {
    list_coupons(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_coupon_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCouponRequest>,
) -> Result<(StatusCode, Json<CouponCampaign>), StatusCode> {
    let coupon = persist_coupon(
        state.store.as_ref(),
        &CouponCampaign::new(
            &request.id,
            &request.code,
            &request.discount_label,
            &request.audience,
            request.remaining,
            request.active,
            &request.note,
            &request.expires_on,
        ),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(coupon)))
}

async fn delete_coupon_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_coupon(state.store.as_ref(), &coupon_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_channels_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Channel>>, StatusCode> {
    list_channels(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn admin_error_response(error: AdminIdentityError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        AdminIdentityError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminIdentityError::DuplicateEmail => StatusCode::CONFLICT,
        AdminIdentityError::Protected(_) => StatusCode::CONFLICT,
        AdminIdentityError::InvalidCredentials | AdminIdentityError::InactiveUser => {
            StatusCode::UNAUTHORIZED
        }
        AdminIdentityError::NotFound(_) => StatusCode::NOT_FOUND,
        AdminIdentityError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

async fn invalidate_catalog_cache_after_mutation() {
    invalidate_capability_catalog_cache().await;
}

fn gateway_api_key_create_error_response(
    error: anyhow::Error,
) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = if looks_like_gateway_api_key_input_error(&message) {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    let body = ErrorResponse {
        error: ErrorBody { message },
    };
    (status, Json(body))
}

fn looks_like_gateway_api_key_input_error(message: &str) -> bool {
    matches!(
        message,
        "tenant_id is required"
            | "project_id is required"
            | "environment is required"
            | "label is required"
            | "expires_at_ms must be in the future"
            | "api key is required when custom key mode is selected"
            | "api_key is required when custom key mode is selected"
            | "api key group not found"
            | "api key group tenant does not match"
            | "api key group project does not match"
            | "api key group environment does not match"
            | "api key group is inactive"
    )
}

fn portal_admin_error_response(error: PortalIdentityError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        PortalIdentityError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        PortalIdentityError::DuplicateEmail => StatusCode::CONFLICT,
        PortalIdentityError::Protected(_) => StatusCode::CONFLICT,
        PortalIdentityError::InvalidCredentials | PortalIdentityError::InactiveUser => {
            StatusCode::UNAUTHORIZED
        }
        PortalIdentityError::NotFound(_) => StatusCode::NOT_FOUND,
        PortalIdentityError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

async fn create_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), StatusCode> {
    let channel = persist_channel(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(channel)))
}

async fn delete_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(channel_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_channel(state.store.as_ref(), &channel_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_providers_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProxyProvider>>, StatusCode> {
    list_providers(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_provider_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProxyProvider>), StatusCode> {
    let primary_channel_id = request
        .channel_bindings
        .iter()
        .find(|binding| binding.is_primary)
        .map(|binding| binding.channel_id.as_str())
        .unwrap_or(&request.channel_id);
    let bindings = provider_bindings_from_request(&request);
    let provider = persist_provider_with_bindings_and_extension_id(
        state.store.as_ref(),
        PersistProviderWithBindingsRequest {
            id: &request.id,
            channel_id: primary_channel_id,
            adapter_kind: &request.adapter_kind,
            extension_id: request.extension_id.as_deref(),
            base_url: &request.base_url,
            display_name: &request.display_name,
            channel_bindings: &bindings,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(provider)))
}

async fn delete_provider_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let provider_exists = state
        .store
        .find_provider(&provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    if !provider_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    delete_provider_credentials_with_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = delete_catalog_provider(state.store.as_ref(), &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_credentials_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UpstreamCredential>>, StatusCode> {
    list_credentials(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_credential_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<UpstreamCredential>), StatusCode> {
    let credential = persist_credential_with_secret_and_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &request.tenant_id,
        &request.provider_id,
        &request.key_reference,
        &request.secret_value,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(credential)))
}

async fn delete_credential_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((tenant_id, provider_id, key_reference)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_credential_with_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &tenant_id,
        &provider_id,
        &key_reference,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_channel_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ChannelModelRecord>>, StatusCode> {
    list_channel_models(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_channel_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelModelRequest>,
) -> Result<(StatusCode, Json<ChannelModelRecord>), StatusCode> {
    let record = persist_channel_model_with_metadata(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.model_display_name,
        &request.capabilities,
        request.streaming,
        request.context_window,
        request.description.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn delete_channel_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_channel_model(state.store.as_ref(), &channel_id, &model_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelCatalogEntry>>, StatusCode> {
    list_model_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<ModelCatalogEntry>), StatusCode> {
    let model = persist_model_with_metadata(
        state.store.as_ref(),
        &request.external_name,
        &request.provider_id,
        &request.capabilities,
        request.streaming,
        request.context_window,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(model)))
}

async fn delete_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((external_name, provider_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_model_variant(state.store.as_ref(), &external_name, &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_model_prices_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelPriceRecord>>, StatusCode> {
    list_model_prices(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_model_price_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelPriceRequest>,
) -> Result<(StatusCode, Json<ModelPriceRecord>), StatusCode> {
    let record = persist_model_price_with_rates(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.proxy_provider_id,
        &request.currency_code,
        &request.price_unit,
        request.input_price,
        request.output_price,
        request.cache_read_price,
        request.cache_write_price,
        request.request_price,
        request.is_active,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn delete_model_price_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id, proxy_provider_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_model_price(
        state.store.as_ref(),
        &channel_id,
        &model_id,
        &proxy_provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_tenants_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    list_tenants(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_tenant_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), StatusCode> {
    let tenant = persist_tenant(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tenant)))
}

async fn delete_tenant_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(tenant_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let tenant_exists = state
        .store
        .list_tenants()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .any(|tenant| tenant.id == tenant_id);
    if !tenant_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    delete_tenant_credentials_with_manager(state.store.as_ref(), &state.secret_manager, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = delete_workspace_tenant(state.store.as_ref(), &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_projects_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    list_projects(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_project_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    let project = persist_project(
        state.store.as_ref(),
        &request.tenant_id,
        &request.id,
        &request.name,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn delete_project_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_tenant_project(state.store.as_ref(), &project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_api_keys_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, StatusCode> {
    list_gateway_api_keys(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_api_key_groups_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ApiKeyGroupRecord>>, StatusCode> {
    list_api_key_groups(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_api_key_group_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyGroupRequest>,
) -> Result<(StatusCode, Json<ApiKeyGroupRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_api_key_group(
        state.store.as_ref(),
        ApiKeyGroupInput {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    .map(|group| (StatusCode::CREATED, Json(group)))
    .map_err(admin_error_response)
}

async fn update_api_key_group_handler(
    _claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateApiKeyGroupRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_api_key_group(
        state.store.as_ref(),
        &group_id,
        ApiKeyGroupInput {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn update_api_key_group_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_api_key_group_active(state.store.as_ref(), &group_id, request.active).await {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn delete_api_key_group_handler(
    _claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_api_key_group(state.store.as_ref(), &group_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn create_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    let metadata_label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{} gateway key", request.environment.trim()));
    let created = sdkwork_api_app_identity::persist_gateway_api_key_with_metadata(
        state.store.as_ref(),
        &request.tenant_id,
        &request.project_id,
        &request.environment,
        &metadata_label,
        request.expires_at_ms,
        request.plaintext_key.as_deref(),
        request.notes.as_deref(),
        request.api_key_group_id.as_deref(),
    )
    .await
    .map_err(gateway_api_key_create_error_response)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn update_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<Json<GatewayApiKeyRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_gateway_api_key_metadata(
        state.store.as_ref(),
        &hashed_key,
        &request.tenant_id,
        &request.project_id,
        &request.environment,
        &request.label,
        request.expires_at_ms,
        request.notes.as_deref(),
        request.api_key_group_id.as_deref(),
    )
    .await
    {
        Ok(Some(record)) => Ok(Json(record)),
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "gateway api key not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn update_api_key_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<GatewayApiKeyRecord>, StatusCode> {
    match set_gateway_api_key_active(state.store.as_ref(), &hashed_key, request.active)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(record) => Ok(Json(record)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_gateway_api_key(state.store.as_ref(), &hashed_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_extension_installations_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ExtensionInstallation>>, StatusCode> {
    list_extension_installations(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_extension_installation_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateExtensionInstallationRequest>,
) -> Result<(StatusCode, Json<ExtensionInstallation>), StatusCode> {
    let installation = persist_extension_installation(
        state.store.as_ref(),
        &request.installation_id,
        &request.extension_id,
        request.runtime,
        request.enabled,
        request.entrypoint.as_deref(),
        request.config,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(installation)))
}

async fn list_extension_instances_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ExtensionInstance>>, StatusCode> {
    list_extension_instances(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_extension_packages_handler(
    _claims: AuthenticatedAdminClaims,
    _state: State<AdminApiState>,
) -> Result<Json<Vec<sdkwork_api_app_extension::DiscoveredExtensionPackageRecord>>, StatusCode> {
    let policy = configured_extension_discovery_policy_from_env();
    list_discovered_extension_packages(&policy)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_extension_instance_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateExtensionInstanceRequest>,
) -> Result<(StatusCode, Json<ExtensionInstance>), StatusCode> {
    let instance = persist_extension_instance(
        state.store.as_ref(),
        PersistExtensionInstanceInput {
            instance_id: &request.instance_id,
            installation_id: &request.installation_id,
            extension_id: &request.extension_id,
            enabled: request.enabled,
            base_url: request.base_url.as_deref(),
            credential_ref: request.credential_ref.as_deref(),
            config: request.config,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(instance)))
}

async fn list_extension_runtime_statuses_handler(
    _claims: AuthenticatedAdminClaims,
    _state: State<AdminApiState>,
) -> Result<Json<Vec<sdkwork_api_app_extension::ExtensionRuntimeStatusRecord>>, StatusCode> {
    list_extension_runtime_statuses()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn reload_extension_runtimes_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    body: Bytes,
) -> Result<Json<ExtensionRuntimeReloadResponse>, StatusCode> {
    let request = parse_extension_runtime_reload_request(&body)?;
    let resolved = resolve_extension_runtime_reload_request(state.store.as_ref(), request).await?;
    let report = reload_extension_host_with_scope(&resolved.gateway_scope)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let runtime_statuses =
        list_extension_runtime_statuses().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ExtensionRuntimeReloadResponse {
        scope: resolved.scope,
        requested_extension_id: resolved.requested_extension_id,
        requested_instance_id: resolved.requested_instance_id,
        resolved_extension_id: resolved.resolved_extension_id,
        discovered_package_count: report.discovered_package_count,
        loadable_package_count: report.loadable_package_count,
        active_runtime_count: runtime_statuses.len(),
        reloaded_at_ms: unix_timestamp_ms(),
        runtime_statuses,
    }))
}

async fn create_extension_runtime_rollout_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    body: Bytes,
) -> Result<(StatusCode, Json<ExtensionRuntimeRolloutResponse>), StatusCode> {
    let request = parse_extension_runtime_rollout_create_request(&body)?;
    let resolved = resolve_extension_runtime_reload_request(
        state.store.as_ref(),
        ExtensionRuntimeReloadRequest {
            extension_id: request.extension_id,
            instance_id: request.instance_id,
        },
    )
    .await?;

    let rollout = create_extension_runtime_rollout_with_request(
        state.store.as_ref(),
        &claims.claims().sub,
        CreateExtensionRuntimeRolloutRequest {
            scope: resolved.gateway_scope,
            requested_extension_id: resolved.requested_extension_id,
            requested_instance_id: resolved.requested_instance_id,
            resolved_extension_id: resolved.resolved_extension_id,
            timeout_secs: request.timeout_secs.unwrap_or(30),
        },
    )
    .await
    .map_err(map_extension_runtime_rollout_creation_error)?;

    Ok((StatusCode::CREATED, Json(rollout.into())))
}

async fn create_standalone_config_rollout_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    body: Bytes,
) -> Result<(StatusCode, Json<StandaloneConfigRolloutResponse>), StatusCode> {
    let request = parse_standalone_config_rollout_create_request(&body)?;
    let requested_service_kind = validate_standalone_service_kind(request.service_kind)?;
    let rollout = create_standalone_config_rollout(
        state.store.as_ref(),
        &claims.claims().sub,
        CreateStandaloneConfigRolloutRequest::new(
            requested_service_kind,
            request.timeout_secs.unwrap_or(30),
        ),
    )
    .await
    .map_err(map_standalone_config_rollout_creation_error)?;

    Ok((StatusCode::CREATED, Json(rollout.into())))
}

async fn list_extension_runtime_rollouts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ExtensionRuntimeRolloutResponse>>, StatusCode> {
    list_extension_runtime_rollouts(state.store.as_ref())
        .await
        .map(|rollouts| {
            Json(
                rollouts
                    .into_iter()
                    .map(ExtensionRuntimeRolloutResponse::from)
                    .collect(),
            )
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_standalone_config_rollouts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<StandaloneConfigRolloutResponse>>, StatusCode> {
    list_standalone_config_rollouts(state.store.as_ref())
        .await
        .map(|rollouts| {
            Json(
                rollouts
                    .into_iter()
                    .map(StandaloneConfigRolloutResponse::from)
                    .collect(),
            )
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_extension_runtime_rollout_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(rollout_id): Path<String>,
) -> Result<Json<ExtensionRuntimeRolloutResponse>, StatusCode> {
    let rollout = find_extension_runtime_rollout(state.store.as_ref(), &rollout_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(rollout.into()))
}

async fn get_standalone_config_rollout_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(rollout_id): Path<String>,
) -> Result<Json<StandaloneConfigRolloutResponse>, StatusCode> {
    let rollout = find_standalone_config_rollout(state.store.as_ref(), &rollout_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(rollout.into()))
}

fn parse_extension_runtime_reload_request(
    body: &[u8],
) -> Result<ExtensionRuntimeReloadRequest, StatusCode> {
    if body.is_empty() {
        return Ok(ExtensionRuntimeReloadRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)
}

fn parse_extension_runtime_rollout_create_request(
    body: &[u8],
) -> Result<ExtensionRuntimeRolloutCreateRequest, StatusCode> {
    if body.is_empty() {
        return Ok(ExtensionRuntimeRolloutCreateRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)
}

fn parse_standalone_config_rollout_create_request(
    body: &[u8],
) -> Result<StandaloneConfigRolloutCreateRequest, StatusCode> {
    if body.is_empty() {
        return Ok(StandaloneConfigRolloutCreateRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)
}

fn map_extension_runtime_rollout_creation_error(error: anyhow::Error) -> StatusCode {
    if error
        .to_string()
        .contains("no active gateway or admin nodes available")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn map_standalone_config_rollout_creation_error(error: anyhow::Error) -> StatusCode {
    if error
        .to_string()
        .contains("no active standalone nodes available")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn resolve_extension_runtime_reload_request(
    store: &dyn AdminStore,
    request: ExtensionRuntimeReloadRequest,
) -> Result<ResolvedExtensionRuntimeReloadRequest, StatusCode> {
    let extension_id = validate_reload_identifier(request.extension_id)?;
    let instance_id = validate_reload_identifier(request.instance_id)?;

    match (extension_id, instance_id) {
        (Some(_), Some(_)) => Err(StatusCode::BAD_REQUEST),
        (Some(extension_id), None) => Ok(ResolvedExtensionRuntimeReloadRequest {
            scope: ExtensionRuntimeReloadScope::Extension,
            requested_extension_id: Some(extension_id.clone()),
            requested_instance_id: None,
            resolved_extension_id: Some(extension_id.clone()),
            gateway_scope: ConfiguredExtensionHostReloadScope::Extension { extension_id },
        }),
        (None, Some(instance_id)) => {
            let instance = list_extension_instances(store)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .into_iter()
                .find(|instance| instance.instance_id == instance_id)
                .ok_or(StatusCode::BAD_REQUEST)?;
            let installation = list_extension_installations(store)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .into_iter()
                .find(|installation| installation.installation_id == instance.installation_id)
                .ok_or(StatusCode::BAD_REQUEST)?;
            let resolved_extension_id = installation.extension_id.clone();

            let (scope, gateway_scope) = match installation.runtime {
                ExtensionRuntime::Connector => (
                    ExtensionRuntimeReloadScope::Instance,
                    ConfiguredExtensionHostReloadScope::Instance {
                        instance_id: instance_id.clone(),
                    },
                ),
                ExtensionRuntime::Builtin | ExtensionRuntime::NativeDynamic => (
                    ExtensionRuntimeReloadScope::Extension,
                    ConfiguredExtensionHostReloadScope::Extension {
                        extension_id: resolved_extension_id.clone(),
                    },
                ),
            };

            Ok(ResolvedExtensionRuntimeReloadRequest {
                scope,
                requested_extension_id: None,
                requested_instance_id: Some(instance_id),
                resolved_extension_id: Some(resolved_extension_id),
                gateway_scope,
            })
        }
        (None, None) => Ok(ResolvedExtensionRuntimeReloadRequest {
            scope: ExtensionRuntimeReloadScope::All,
            requested_extension_id: None,
            requested_instance_id: None,
            resolved_extension_id: None,
            gateway_scope: ConfiguredExtensionHostReloadScope::All,
        }),
    }
}

fn validate_reload_identifier(value: Option<String>) -> Result<Option<String>, StatusCode> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn validate_standalone_service_kind(value: Option<String>) -> Result<Option<String>, StatusCode> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Err(StatusCode::BAD_REQUEST);
            }

            match value {
                "gateway" | "admin" | "portal" => Ok(Some(value.to_owned())),
                _ => Err(StatusCode::BAD_REQUEST),
            }
        }
        None => Ok(None),
    }
}

async fn list_provider_health_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderHealthSnapshot>>, StatusCode> {
    list_provider_health_snapshots(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

async fn simulate_routing_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<RoutingSimulationRequest>,
) -> Result<Json<RoutingSimulationResponse>, StatusCode> {
    let decision = select_route_with_store_context(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        RouteSelectionContext::new(RoutingDecisionSource::AdminSimulation)
            .with_tenant_id_option(request.tenant_id.as_deref())
            .with_project_id_option(request.project_id.as_deref())
            .with_api_key_group_id_option(request.api_key_group_id.as_deref())
            .with_requested_region_option(request.requested_region.as_deref())
            .with_selection_seed_option(request.selection_seed),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (selected_candidate, rejected_candidates) =
        split_routing_assessments(&decision.selected_provider_id, &decision.assessments);
    Ok(Json(RoutingSimulationResponse {
        selected_provider_id: decision.selected_provider_id,
        candidate_ids: decision.candidate_ids,
        matched_policy_id: decision.matched_policy_id,
        applied_routing_profile_id: decision.applied_routing_profile_id,
        compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id,
        strategy: decision.strategy,
        selection_seed: decision.selection_seed,
        selection_reason: decision.selection_reason,
        fallback_reason: decision.fallback_reason,
        requested_region: decision.requested_region,
        slo_applied: decision.slo_applied,
        slo_degraded: decision.slo_degraded,
        selected_candidate,
        rejected_candidates,
        assessments: decision.assessments,
    }))
}

fn split_routing_assessments(
    selected_provider_id: &str,
    assessments: &[RoutingCandidateAssessment],
) -> (
    Option<RoutingCandidateAssessment>,
    Vec<RoutingCandidateAssessment>,
) {
    let mut selected_candidate = None;
    let mut rejected_candidates = Vec::new();
    for assessment in assessments {
        if assessment.provider_id == selected_provider_id && selected_candidate.is_none() {
            selected_candidate = Some(assessment.clone());
        } else {
            rejected_candidates.push(assessment.clone());
        }
    }
    (selected_candidate, rejected_candidates)
}

async fn list_compiled_routing_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CompiledRoutingSnapshotRecord>>, StatusCode> {
    list_compiled_routing_snapshots(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_routing_decision_logs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingDecisionLog>>, StatusCode> {
    list_routing_decision_logs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_routing_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingPolicy>>, StatusCode> {
    list_routing_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_routing_profiles_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingProfileRecord>>, StatusCode> {
    list_routing_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_routing_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRoutingPolicyRequest>,
) -> Result<(StatusCode, Json<RoutingPolicy>), StatusCode> {
    let policy = create_routing_policy(CreateRoutingPolicyInput {
        policy_id: &request.policy_id,
        capability: &request.capability,
        model_pattern: &request.model_pattern,
        enabled: request.enabled,
        priority: request.priority,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_routing_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

async fn create_routing_profile_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRoutingProfileRequest>,
) -> Result<(StatusCode, Json<RoutingProfileRecord>), StatusCode> {
    let profile = create_routing_profile(CreateRoutingProfileInput {
        profile_id: &request.profile_id,
        tenant_id: &request.tenant_id,
        project_id: &request.project_id,
        name: &request.name,
        slug: &request.slug,
        description: request.description.as_deref(),
        active: request.active,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
        preferred_region: request.preferred_region.as_deref(),
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let profile = persist_routing_profile(state.store.as_ref(), &profile)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(profile)))
}

async fn list_usage_records_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    list_usage_records(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn usage_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    summarize_usage_records_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_ledger_entries_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    list_ledger_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_billing_events_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<BillingEventRecord>>, StatusCode> {
    list_billing_events(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn billing_events_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<BillingEventSummary>, StatusCode> {
    summarize_billing_events_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn billing_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<BillingSummary>, StatusCode> {
    summarize_billing_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_payment_orders_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PaymentOrderRecord>>, StatusCode> {
    load_admin_payment_orders(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_payment_order_dossier_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(payment_order_id): Path<String>,
) -> Result<Json<AdminPaymentOrderDossier>, StatusCode> {
    if let Some(sqlite_store) = state.store.as_any().downcast_ref::<SqliteAdminStore>() {
        return load_admin_payment_order_dossier(sqlite_store, &payment_order_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(Json)
            .ok_or(StatusCode::NOT_FOUND);
    }
    if let Some(postgres_store) = state.store.as_any().downcast_ref::<PostgresAdminStore>() {
        return load_admin_payment_order_dossier(postgres_store, &payment_order_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(Json)
            .ok_or(StatusCode::NOT_FOUND);
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_refund_orders_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<RefundOrderListQuery>,
) -> Result<Json<Vec<RefundOrderRecord>>, StatusCode> {
    let refund_status_filter = query
        .refund_status
        .as_deref()
        .map(RefundOrderStatus::from_str)
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    load_admin_refund_orders(state.store.as_ref(), refund_status_filter)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn approve_refund_order_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(refund_order_id): Path<String>,
    Json(request): Json<ApproveRefundOrderRequest>,
) -> Result<Json<RefundOrderRecord>, StatusCode> {
    if let Some(sqlite_store) = state.store.as_any().downcast_ref::<SqliteAdminStore>() {
        return approve_refund_order_request(
            sqlite_store,
            &refund_order_id,
            request.approved_amount_minor,
            request.approved_at_ms,
        )
        .await
        .map(Json)
        .map_err(map_refund_request_action_error);
    }
    if let Some(postgres_store) = state.store.as_any().downcast_ref::<PostgresAdminStore>() {
        return approve_refund_order_request(
            postgres_store,
            &refund_order_id,
            request.approved_amount_minor,
            request.approved_at_ms,
        )
        .await
        .map(Json)
        .map_err(map_refund_request_action_error);
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn cancel_refund_order_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(refund_order_id): Path<String>,
    Json(request): Json<CancelRefundOrderRequest>,
) -> Result<Json<RefundOrderRecord>, StatusCode> {
    if let Some(sqlite_store) = state.store.as_any().downcast_ref::<SqliteAdminStore>() {
        return cancel_refund_order_request(sqlite_store, &refund_order_id, request.canceled_at_ms)
            .await
            .map(Json)
            .map_err(map_refund_request_action_error);
    }
    if let Some(postgres_store) = state.store.as_any().downcast_ref::<PostgresAdminStore>() {
        return cancel_refund_order_request(
            postgres_store,
            &refund_order_id,
            request.canceled_at_ms,
        )
        .await
        .map(Json)
        .map_err(map_refund_request_action_error);
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn start_refund_order_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(refund_order_id): Path<String>,
    Json(request): Json<StartRefundOrderRequest>,
) -> Result<Json<RefundOrderRecord>, StatusCode> {
    if let Some(sqlite_store) = state.store.as_any().downcast_ref::<SqliteAdminStore>() {
        return start_refund_order_execution(sqlite_store, &refund_order_id, request.started_at_ms)
            .await
            .map(Json)
            .map_err(map_refund_request_action_error);
    }
    if let Some(postgres_store) = state.store.as_any().downcast_ref::<PostgresAdminStore>() {
        return start_refund_order_execution(
            postgres_store,
            &refund_order_id,
            request.started_at_ms,
        )
        .await
        .map(Json)
        .map_err(map_refund_request_action_error);
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_payment_gateway_accounts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<PaymentGatewayAccountListQuery>,
) -> Result<Json<Vec<PaymentGatewayAccountRecord>>, StatusCode> {
    load_admin_payment_gateway_accounts(state.store.as_ref(), &query)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn upsert_payment_gateway_account_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPaymentGatewayAccountRequest>,
) -> Result<(StatusCode, Json<PaymentGatewayAccountRecord>), StatusCode> {
    let record = payment_gateway_account_record_from_request(request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let record = persist_admin_payment_gateway_account(state.store.as_ref(), &record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn list_payment_channel_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<PaymentChannelPolicyListQuery>,
) -> Result<Json<Vec<PaymentChannelPolicyRecord>>, StatusCode> {
    load_admin_payment_channel_policies(state.store.as_ref(), &query)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn upsert_payment_channel_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPaymentChannelPolicyRequest>,
) -> Result<(StatusCode, Json<PaymentChannelPolicyRecord>), StatusCode> {
    let record =
        payment_channel_policy_record_from_request(request).map_err(|_| StatusCode::BAD_REQUEST)?;
    let record = persist_admin_payment_channel_policy(state.store.as_ref(), &record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn list_payment_reconciliation_lines_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Query(query): Query<PaymentReconciliationListQuery>,
) -> Result<Json<Vec<ReconciliationMatchSummaryRecord>>, StatusCode> {
    let lifecycle = PaymentReconciliationLifecycle::parse(query.lifecycle.as_deref())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    load_admin_payment_reconciliation_lines(state.store.as_ref(), lifecycle)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn payment_reconciliation_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<PaymentReconciliationSummaryResponse>, StatusCode> {
    summarize_admin_payment_reconciliation(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn resolve_payment_reconciliation_line_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(reconciliation_line_id): Path<String>,
    Json(request): Json<ResolveReconciliationLineRequest>,
) -> Result<Json<ReconciliationMatchSummaryRecord>, StatusCode> {
    resolve_admin_payment_reconciliation_line(
        state.store.as_ref(),
        &reconciliation_line_id,
        request.resolved_at_ms.unwrap_or_else(unix_timestamp_ms),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map(Json)
    .ok_or(StatusCode::NOT_FOUND)
}

async fn list_quota_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<QuotaPolicy>>, StatusCode> {
    list_quota_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_quota_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateQuotaPolicyRequest>,
) -> Result<(StatusCode, Json<QuotaPolicy>), StatusCode> {
    let policy = create_quota_policy(
        &request.policy_id,
        &request.project_id,
        request.max_units,
        request.enabled,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_quota_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

async fn list_rate_limit_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RateLimitPolicy>>, StatusCode> {
    list_rate_limit_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_rate_limit_window_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RateLimitWindowSnapshot>>, StatusCode> {
    state
        .store
        .list_rate_limit_window_snapshots()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_rate_limit_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRateLimitPolicyRequest>,
) -> Result<(StatusCode, Json<RateLimitPolicy>), StatusCode> {
    let policy = create_rate_limit_policy(
        &request.policy_id,
        &request.project_id,
        request.requests_per_window,
        request.window_seconds,
        request.burst_requests,
        request.enabled,
        request.route_key.as_deref(),
        request.api_key_hash.as_deref(),
        request.model_name.as_deref(),
        request.notes.as_deref(),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_rate_limit_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

fn payment_gateway_account_record_from_request(
    request: UpsertPaymentGatewayAccountRequest,
) -> anyhow::Result<PaymentGatewayAccountRecord> {
    let provider_code = parse_admin_payment_provider_code(&request.provider_code)?;
    ensure!(
        !matches!(provider_code, PaymentProviderCode::Unspecified),
        "provider_code must be a concrete payment provider"
    );
    ensure!(
        !request.gateway_account_id.trim().is_empty(),
        "gateway_account_id must not be empty"
    );
    ensure!(
        !request.environment.trim().is_empty(),
        "environment must not be empty"
    );
    ensure!(
        !request.merchant_id.trim().is_empty(),
        "merchant_id must not be empty"
    );
    let status = normalize_admin_payment_route_status(&request.status)?;
    let updated_at_ms = request.updated_at_ms.unwrap_or_else(unix_timestamp_ms);
    let created_at_ms = request.created_at_ms.unwrap_or(updated_at_ms);

    Ok(PaymentGatewayAccountRecord::new(
        request.gateway_account_id,
        request.tenant_id,
        request.organization_id,
        provider_code,
    )
    .with_environment(request.environment.trim())
    .with_merchant_id(request.merchant_id.trim())
    .with_app_id(request.app_id.trim())
    .with_status(status)
    .with_priority(request.priority)
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

fn payment_channel_policy_record_from_request(
    request: UpsertPaymentChannelPolicyRequest,
) -> anyhow::Result<PaymentChannelPolicyRecord> {
    let provider_code = parse_admin_payment_provider_code(&request.provider_code)?;
    ensure!(
        !matches!(provider_code, PaymentProviderCode::Unspecified),
        "provider_code must be a concrete payment provider"
    );
    ensure!(
        !request.channel_policy_id.trim().is_empty(),
        "channel_policy_id must not be empty"
    );
    ensure!(
        !request.method_code.trim().is_empty(),
        "method_code must not be empty"
    );
    let status = normalize_admin_payment_route_status(&request.status)?;
    let updated_at_ms = request.updated_at_ms.unwrap_or_else(unix_timestamp_ms);
    let created_at_ms = request.created_at_ms.unwrap_or(updated_at_ms);

    Ok(PaymentChannelPolicyRecord::new(
        request.channel_policy_id,
        request.tenant_id,
        request.organization_id,
        provider_code,
        request.method_code.trim(),
    )
    .with_scene_code(request.scene_code.trim())
    .with_country_code(request.country_code.trim())
    .with_currency_code(request.currency_code.trim())
    .with_client_kind(request.client_kind.trim())
    .with_priority(request.priority)
    .with_status(status)
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

fn parse_admin_payment_provider_code(raw: &str) -> anyhow::Result<PaymentProviderCode> {
    PaymentProviderCode::from_str(raw.trim()).map_err(anyhow::Error::msg)
}

fn normalize_admin_payment_route_status(raw: &str) -> anyhow::Result<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    ensure!(
        matches!(normalized.as_str(), "active" | "inactive"),
        "unsupported payment route status"
    );
    Ok(normalized)
}

async fn persist_admin_payment_gateway_account(
    store: &dyn AdminStore,
    record: &PaymentGatewayAccountRecord,
) -> anyhow::Result<PaymentGatewayAccountRecord> {
    if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>() {
        return sqlite_store
            .insert_payment_gateway_account_record(record)
            .await;
    }
    if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        return postgres_store
            .insert_payment_gateway_account_record(record)
            .await;
    }

    Err(anyhow::anyhow!(
        "admin payment gateway account management is unavailable for the current store type"
    ))
}

async fn persist_admin_payment_channel_policy(
    store: &dyn AdminStore,
    record: &PaymentChannelPolicyRecord,
) -> anyhow::Result<PaymentChannelPolicyRecord> {
    if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>() {
        return sqlite_store
            .insert_payment_channel_policy_record(record)
            .await;
    }
    if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        return postgres_store
            .insert_payment_channel_policy_record(record)
            .await;
    }

    Err(anyhow::anyhow!(
        "admin payment channel policy management is unavailable for the current store type"
    ))
}

async fn load_admin_payment_gateway_accounts(
    store: &dyn AdminStore,
    query: &PaymentGatewayAccountListQuery,
) -> anyhow::Result<Vec<PaymentGatewayAccountRecord>> {
    let mut records = if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>()
    {
        sqlite_store.list_payment_gateway_account_records().await?
    } else if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        postgres_store
            .list_payment_gateway_account_records()
            .await?
    } else {
        return Err(anyhow::anyhow!(
            "admin payment gateway account inspection is unavailable for the current store type"
        ));
    };

    records.retain(|record| payment_gateway_account_matches_query(record, query));
    records.sort_by(compare_payment_gateway_accounts);
    Ok(records)
}

fn payment_gateway_account_matches_query(
    record: &PaymentGatewayAccountRecord,
    query: &PaymentGatewayAccountListQuery,
) -> bool {
    optional_string_filter_matches(
        record.provider_code.as_str(),
        query.provider_code.as_deref(),
    ) && optional_string_filter_matches(&record.status, query.status.as_deref())
        && optional_u64_filter_matches(record.tenant_id, query.tenant_id)
        && optional_u64_filter_matches(record.organization_id, query.organization_id)
}

fn compare_payment_gateway_accounts(
    left: &PaymentGatewayAccountRecord,
    right: &PaymentGatewayAccountRecord,
) -> std::cmp::Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| left.gateway_account_id.cmp(&right.gateway_account_id))
}

async fn load_admin_payment_channel_policies(
    store: &dyn AdminStore,
    query: &PaymentChannelPolicyListQuery,
) -> anyhow::Result<Vec<PaymentChannelPolicyRecord>> {
    let mut records = if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>()
    {
        sqlite_store.list_payment_channel_policy_records().await?
    } else if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        postgres_store.list_payment_channel_policy_records().await?
    } else {
        return Err(anyhow::anyhow!(
            "admin payment channel policy inspection is unavailable for the current store type"
        ));
    };

    records.retain(|record| payment_channel_policy_matches_query(record, query));
    records.sort_by(compare_payment_channel_policies);
    Ok(records)
}

fn payment_channel_policy_matches_query(
    record: &PaymentChannelPolicyRecord,
    query: &PaymentChannelPolicyListQuery,
) -> bool {
    optional_string_filter_matches(
        record.provider_code.as_str(),
        query.provider_code.as_deref(),
    ) && optional_string_filter_matches(&record.status, query.status.as_deref())
        && optional_u64_filter_matches(record.tenant_id, query.tenant_id)
        && optional_u64_filter_matches(record.organization_id, query.organization_id)
        && optional_string_filter_matches(&record.scene_code, query.scene_code.as_deref())
        && optional_string_filter_matches(&record.currency_code, query.currency_code.as_deref())
        && optional_string_filter_matches(&record.client_kind, query.client_kind.as_deref())
}

fn compare_payment_channel_policies(
    left: &PaymentChannelPolicyRecord,
    right: &PaymentChannelPolicyRecord,
) -> std::cmp::Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| left.channel_policy_id.cmp(&right.channel_policy_id))
}

fn optional_string_filter_matches(value: &str, expected: Option<&str>) -> bool {
    expected
        .map(|expected| value.eq_ignore_ascii_case(expected.trim()))
        .unwrap_or(true)
}

fn optional_u64_filter_matches(value: u64, expected: Option<u64>) -> bool {
    expected.map(|expected| value == expected).unwrap_or(true)
}

async fn load_admin_payment_orders(
    store: &dyn AdminStore,
) -> anyhow::Result<Vec<PaymentOrderRecord>> {
    if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>() {
        let mut orders = sqlite_store.list_payment_order_records().await?;
        orders.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.payment_order_id.cmp(&left.payment_order_id))
        });
        return Ok(orders);
    }
    if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        let mut orders = postgres_store.list_payment_order_records().await?;
        orders.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.payment_order_id.cmp(&left.payment_order_id))
        });
        return Ok(orders);
    }

    Err(anyhow::anyhow!(
        "admin payment order inspection is unavailable for the current store type"
    ))
}

async fn load_admin_refund_orders(
    store: &dyn AdminStore,
    refund_status_filter: Option<RefundOrderStatus>,
) -> anyhow::Result<Vec<RefundOrderRecord>> {
    let payment_orders = load_admin_payment_orders(store).await?;

    if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>() {
        let mut refunds = Vec::new();
        for payment_order in &payment_orders {
            let mut order_refunds = sqlite_store
                .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
                .await?;
            refunds.append(&mut order_refunds);
        }
        refunds.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
        });
        if let Some(refund_status_filter) = refund_status_filter {
            refunds.retain(|refund| refund.refund_status == refund_status_filter);
        }
        return Ok(refunds);
    }
    if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        let mut refunds = Vec::new();
        for payment_order in &payment_orders {
            let mut order_refunds = postgres_store
                .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
                .await?;
            refunds.append(&mut order_refunds);
        }
        refunds.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
        });
        if let Some(refund_status_filter) = refund_status_filter {
            refunds.retain(|refund| refund.refund_status == refund_status_filter);
        }
        return Ok(refunds);
    }

    Err(anyhow::anyhow!(
        "admin refund order inspection is unavailable for the current store type"
    ))
}

async fn load_admin_payment_reconciliation_lines(
    store: &dyn AdminStore,
    lifecycle: PaymentReconciliationLifecycle,
) -> anyhow::Result<Vec<ReconciliationMatchSummaryRecord>> {
    if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>() {
        let mut lines = sqlite_store
            .list_all_reconciliation_match_summary_records()
            .await?;
        apply_payment_reconciliation_queue_view(&mut lines, lifecycle);
        return Ok(lines);
    }
    if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        let mut lines = postgres_store
            .list_all_reconciliation_match_summary_records()
            .await?;
        apply_payment_reconciliation_queue_view(&mut lines, lifecycle);
        return Ok(lines);
    }

    Err(anyhow::anyhow!(
        "admin payment reconciliation inspection is unavailable for the current store type"
    ))
}

fn apply_payment_reconciliation_queue_view(
    lines: &mut Vec<ReconciliationMatchSummaryRecord>,
    lifecycle: PaymentReconciliationLifecycle,
) {
    lines.retain(|line| lifecycle.matches(line));
    lines.sort_by(compare_payment_reconciliation_lines);
}

fn compare_payment_reconciliation_lines(
    left: &ReconciliationMatchSummaryRecord,
    right: &ReconciliationMatchSummaryRecord,
) -> std::cmp::Ordering {
    matches!(left.match_status, ReconciliationMatchStatus::Resolved)
        .cmp(&matches!(
            right.match_status,
            ReconciliationMatchStatus::Resolved
        ))
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
        .then_with(|| {
            right
                .reconciliation_line_id
                .cmp(&left.reconciliation_line_id)
        })
}

async fn summarize_admin_payment_reconciliation(
    store: &dyn AdminStore,
) -> anyhow::Result<PaymentReconciliationSummaryResponse> {
    let lines =
        load_admin_payment_reconciliation_lines(store, PaymentReconciliationLifecycle::All).await?;
    let total_count = lines.len();
    let latest_updated_at_ms = lines.iter().map(|line| line.updated_at_ms).max();
    let active_lines = lines
        .iter()
        .filter(|line| !matches!(line.match_status, ReconciliationMatchStatus::Resolved))
        .collect::<Vec<_>>();
    let active_count = active_lines.len();
    let resolved_count = total_count.saturating_sub(active_count);
    let oldest_active_created_at_ms = active_lines.iter().map(|line| line.created_at_ms).min();

    let mut breakdown = BTreeMap::<String, (usize, u64)>::new();
    for line in active_lines {
        let reason_code = line
            .reason_code
            .clone()
            .unwrap_or_else(|| "unknown".to_owned());
        let entry = breakdown
            .entry(reason_code)
            .or_insert((0usize, line.updated_at_ms));
        entry.0 += 1;
        entry.1 = entry.1.max(line.updated_at_ms);
    }

    let mut active_reason_breakdown = breakdown
        .into_iter()
        .map(|(reason_code, (count, latest_updated_at_ms))| {
            PaymentReconciliationReasonBreakdownItem {
                reason_code,
                count,
                latest_updated_at_ms,
            }
        })
        .collect::<Vec<_>>();
    active_reason_breakdown.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| right.latest_updated_at_ms.cmp(&left.latest_updated_at_ms))
            .then_with(|| left.reason_code.cmp(&right.reason_code))
    });

    Ok(PaymentReconciliationSummaryResponse {
        total_count,
        active_count,
        resolved_count,
        latest_updated_at_ms,
        oldest_active_created_at_ms,
        active_reason_breakdown,
    })
}

async fn render_admin_metrics_payload(
    metrics: &HttpMetricsRegistry,
    store: &dyn AdminStore,
) -> String {
    let mut output = metrics.render_prometheus();
    match summarize_admin_payment_reconciliation(store).await {
        Ok(summary) => {
            output.push_str(&render_payment_reconciliation_prometheus(
                metrics.service(),
                &summary,
            ));
            output.push_str(
                "# HELP sdkwork_payment_reconciliation_metrics_scrape_error Whether reconciliation metric aggregation failed\n",
            );
            output.push_str("# TYPE sdkwork_payment_reconciliation_metrics_scrape_error gauge\n");
            output.push_str(&format!(
                "sdkwork_payment_reconciliation_metrics_scrape_error{{service=\"{}\"}} 0\n",
                escape_prometheus_label_value(metrics.service())
            ));
        }
        Err(_) => {
            output.push_str(
                "# HELP sdkwork_payment_reconciliation_metrics_scrape_error Whether reconciliation metric aggregation failed\n",
            );
            output.push_str("# TYPE sdkwork_payment_reconciliation_metrics_scrape_error gauge\n");
            output.push_str(&format!(
                "sdkwork_payment_reconciliation_metrics_scrape_error{{service=\"{}\"}} 1\n",
                escape_prometheus_label_value(metrics.service())
            ));
        }
    }
    output
}

fn render_payment_reconciliation_prometheus(
    service: &str,
    summary: &PaymentReconciliationSummaryResponse,
) -> String {
    let mut output = String::new();
    let service = escape_prometheus_label_value(service);

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_total Total reconciliation lines observed\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_total gauge\n");
    output.push_str(&format!(
        "sdkwork_payment_reconciliation_total{{service=\"{}\"}} {}\n",
        service, summary.total_count
    ));

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_active_total Active reconciliation lines observed\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_active_total gauge\n");
    output.push_str(&format!(
        "sdkwork_payment_reconciliation_active_total{{service=\"{}\"}} {}\n",
        service, summary.active_count
    ));

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_resolved_total Resolved reconciliation lines observed\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_resolved_total gauge\n");
    output.push_str(&format!(
        "sdkwork_payment_reconciliation_resolved_total{{service=\"{}\"}} {}\n",
        service, summary.resolved_count
    ));

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_latest_updated_at_ms Latest reconciliation update timestamp in milliseconds\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_latest_updated_at_ms gauge\n");
    if let Some(value) = summary.latest_updated_at_ms {
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_latest_updated_at_ms{{service=\"{}\"}} {}\n",
            service, value
        ));
    }

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_oldest_active_created_at_ms Oldest active reconciliation creation timestamp in milliseconds\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_oldest_active_created_at_ms gauge\n");
    if let Some(value) = summary.oldest_active_created_at_ms {
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_oldest_active_created_at_ms{{service=\"{}\"}} {}\n",
            service, value
        ));
    }

    output.push_str(
        "# HELP sdkwork_payment_reconciliation_active_reason_total Active reconciliation lines grouped by reason code\n",
    );
    output.push_str("# TYPE sdkwork_payment_reconciliation_active_reason_total gauge\n");
    for item in &summary.active_reason_breakdown {
        output.push_str(&format!(
            "sdkwork_payment_reconciliation_active_reason_total{{service=\"{}\",reason_code=\"{}\"}} {}\n",
            service,
            escape_prometheus_label_value(&item.reason_code),
            item.count
        ));
    }

    output
}

fn escape_prometheus_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn map_refund_request_action_error(error: anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("refund order not found") || message.contains("payment order not found") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn resolve_admin_payment_reconciliation_line(
    store: &dyn AdminStore,
    reconciliation_line_id: &str,
    resolved_at_ms: u64,
) -> anyhow::Result<Option<ReconciliationMatchSummaryRecord>> {
    if let Some(sqlite_store) = store.as_any().downcast_ref::<SqliteAdminStore>() {
        return resolve_payment_reconciliation_line_with_store(
            sqlite_store,
            reconciliation_line_id,
            resolved_at_ms,
        )
        .await;
    }
    if let Some(postgres_store) = store.as_any().downcast_ref::<PostgresAdminStore>() {
        return resolve_payment_reconciliation_line_with_store(
            postgres_store,
            reconciliation_line_id,
            resolved_at_ms,
        )
        .await;
    }

    Err(anyhow::anyhow!(
        "admin payment reconciliation resolution is unavailable for the current store type"
    ))
}

async fn resolve_payment_reconciliation_line_with_store<S>(
    store: &S,
    reconciliation_line_id: &str,
    resolved_at_ms: u64,
) -> anyhow::Result<Option<ReconciliationMatchSummaryRecord>>
where
    S: PaymentKernelStore + ?Sized,
{
    let Some(existing) = store
        .find_reconciliation_match_summary_record(reconciliation_line_id)
        .await?
    else {
        return Ok(None);
    };
    if matches!(existing.match_status, ReconciliationMatchStatus::Resolved) {
        return Ok(Some(existing));
    }

    let mut resolved = existing;
    resolved.match_status = ReconciliationMatchStatus::Resolved;
    resolved.updated_at_ms = resolved_at_ms.max(resolved.created_at_ms);
    store
        .insert_reconciliation_match_summary_record(&resolved)
        .await
        .map(Some)
}

fn provider_bindings_from_request(request: &CreateProviderRequest) -> Vec<ProviderChannelBinding> {
    let mut bindings = if request.channel_bindings.is_empty() {
        vec![ProviderChannelBinding::primary(
            &request.id,
            &request.channel_id,
        )]
    } else {
        request
            .channel_bindings
            .iter()
            .map(|binding| {
                let base = ProviderChannelBinding::new(&request.id, &binding.channel_id);
                if binding.is_primary {
                    ProviderChannelBinding::primary(&request.id, &binding.channel_id)
                } else {
                    base
                }
            })
            .collect::<Vec<_>>()
    };

    if !bindings
        .iter()
        .any(|binding| binding.channel_id == request.channel_id)
    {
        bindings.push(ProviderChannelBinding::primary(
            &request.id,
            &request.channel_id,
        ));
    }

    bindings
}

fn default_true() -> bool {
    true
}

fn default_window_seconds() -> u64 {
    60
}
