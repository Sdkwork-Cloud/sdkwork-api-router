use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json, Router,
    body::Bytes,
    extract::FromRequestParts,
    extract::Path,
    extract::Query,
    extract::State,
    http::HeaderMap,
    http::StatusCode,
    http::header,
    http::request::Parts,
    response::{Html, IntoResponse},
    routing::{delete, get, patch, post, put},
};
use sdkwork_api_app_billing::{
    AccountBalanceSnapshot, AccountLedgerHistoryEntry, CommercialBillingAdminKernel,
    PricingLifecycleSynchronizationReport, create_quota_policy, list_billing_events,
    list_ledger_entries, list_quota_policies, persist_quota_policy,
    summarize_billing_events_from_store, summarize_billing_from_store,
    synchronize_due_pricing_plan_lifecycle, synchronize_due_pricing_plan_lifecycle_with_report,
};
use sdkwork_api_app_catalog::{
    PersistProviderWithBindingsRequest, delete_channel as delete_catalog_channel,
    delete_channel_model as delete_catalog_channel_model,
    delete_model_price as delete_catalog_model_price, delete_model_variant,
    delete_provider as delete_catalog_provider, list_channel_models, list_channels,
    list_model_entries, list_model_prices, list_providers, persist_channel,
    persist_channel_model_with_metadata, persist_model_price_with_rates,
    persist_model_with_metadata, persist_provider_with_bindings_and_extension_id,
};
use sdkwork_api_app_coupon::{delete_coupon, list_coupons, persist_coupon};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_credential::{
    delete_credential_with_manager, delete_provider_credentials_with_manager,
    delete_tenant_credentials_with_manager, list_credentials,
    persist_credential_with_secret_and_manager,
};
use sdkwork_api_app_extension::{
    PersistExtensionInstanceInput, configured_extension_discovery_policy_from_env,
    list_discovered_extension_packages, list_extension_installations, list_extension_instances,
    list_extension_runtime_statuses, list_provider_health_snapshots,
    persist_extension_installation, persist_extension_instance,
};
use sdkwork_api_app_gateway::{
    ConfiguredExtensionHostReloadScope, invalidate_capability_catalog_cache,
    reload_extension_host_with_scope,
};
use sdkwork_api_app_identity::{
    AdminIdentityError, ApiKeyGroupInput, Claims, CreatedGatewayApiKey, PortalIdentityError,
    change_admin_password, create_api_key_group, delete_admin_user, delete_api_key_group,
    delete_gateway_api_key, delete_portal_user, list_admin_user_profiles, list_api_key_groups,
    list_gateway_api_keys, list_portal_user_profiles, load_admin_user_profile, login_admin_user,
    reset_admin_user_password, reset_portal_user_password, set_admin_user_active,
    set_api_key_group_active, set_gateway_api_key_active, set_portal_user_active,
    update_api_key_group, update_gateway_api_key_metadata, upsert_admin_user, upsert_portal_user,
    verify_jwt,
};
use sdkwork_api_app_jobs::{
    find_async_job, list_async_job_assets, list_async_job_attempts, list_async_job_callbacks,
    list_async_jobs,
};
use sdkwork_api_app_marketing::project_legacy_coupon_campaign;
use sdkwork_api_app_rate_limit::{
    create_rate_limit_policy, list_rate_limit_policies, persist_rate_limit_policy,
};
use sdkwork_api_app_routing::{
    CreateRoutingPolicyInput, CreateRoutingProfileInput, RouteSelectionContext,
    create_routing_policy, create_routing_profile, list_compiled_routing_snapshots,
    list_routing_decision_logs, list_routing_policies, list_routing_profiles,
    persist_routing_policy, persist_routing_profile, select_route_with_store_context,
};
use sdkwork_api_app_runtime::{
    CreateExtensionRuntimeRolloutRequest, CreateStandaloneConfigRolloutRequest,
    ExtensionRuntimeRolloutDetails, StandaloneConfigRolloutDetails,
    create_extension_runtime_rollout_with_request, create_standalone_config_rollout,
    find_extension_runtime_rollout, find_standalone_config_rollout,
    list_extension_runtime_rollouts, list_standalone_config_rollouts,
};
use sdkwork_api_app_tenant::{
    delete_project as delete_tenant_project, delete_tenant as delete_workspace_tenant,
    list_projects, list_tenants, persist_project, persist_tenant,
};
use sdkwork_api_app_usage::list_usage_records;
use sdkwork_api_app_usage::summarize_usage_records_from_store;
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountHoldRecord, AccountRecord, BillingEventRecord,
    BillingEventSummary, BillingSummary, LedgerEntry, PricingPlanRecord, PricingRateRecord,
    QuotaPolicy, RequestSettlementRecord,
};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCapability, ModelCatalogEntry, ModelPriceRecord,
    ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_commerce::{CommerceOrderRecord, CommercePaymentEventRecord};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{
    AdminUserProfile, ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserProfile,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord, CouponTemplateRecord,
    CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord, MarketingCampaignStatus,
    MarketingSubjectScope,
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
use sdkwork_api_observability::{HttpMetricsRegistry, observe_http_metrics, observe_http_tracing};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::openapi::Server;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::{Config as SwaggerUiConfig, SwaggerUi, Url as SwaggerUiUrl};

const DEFAULT_ADMIN_JWT_SIGNING_SECRET: &str = "local-dev-admin-jwt-secret";
static ADMIN_PRICING_ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Admin API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current admin router implementation."
    ),
    modifiers(&AdminApiDocModifier),
    tags(
        (name = "system", description = "Admin health and system-facing routes."),
        (name = "auth", description = "Admin authentication and session management routes."),
        (name = "marketing", description = "Coupon template, campaign, budget, and redemption administration routes."),
        (name = "tenants", description = "Tenant and project administration routes."),
        (name = "users", description = "Operator and portal user administration routes."),
        (name = "gateway", description = "Gateway API key and API key group administration routes."),
        (name = "billing", description = "Billing summary, event, and ledger administration routes."),
        (name = "commerce", description = "Recent order and payment callback audit routes.")
    )
)]
struct AdminApiDoc;

struct AdminApiDocModifier;

impl Modify for AdminApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/")]);
        openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
    }
}

mod openapi_paths {
    use super::*;

    #[utoipa::path(
        get,
        path = "/admin/health",
        tag = "system",
        responses((status = 200, description = "Admin health check response.", body = String))
    )]
    pub(super) async fn health() {}

    #[utoipa::path(
        post,
        path = "/admin/auth/login",
        tag = "auth",
        request_body = LoginRequest,
        responses(
            (status = 200, description = "Admin login session.", body = LoginResponse),
            (status = 401, description = "Invalid admin credentials.", body = ErrorResponse),
            (status = 500, description = "Admin authentication failed.", body = ErrorResponse)
        )
    )]
    pub(super) async fn auth_login() {}

    #[utoipa::path(
        post,
        path = "/admin/auth/change-password",
        tag = "auth",
        request_body = ChangePasswordRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated admin profile after password change.", body = AdminUserProfile),
            (status = 400, description = "Invalid password change request.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Password change failed.", body = ErrorResponse)
        )
    )]
    pub(super) async fn auth_change_password() {}

    #[utoipa::path(
        get,
        path = "/admin/tenants",
        tag = "tenants",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible tenant catalog.", body = [Tenant]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load tenants.")
        )
    )]
    pub(super) async fn tenants_list() {}

    #[utoipa::path(
        post,
        path = "/admin/tenants",
        tag = "tenants",
        request_body = CreateTenantRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created tenant.", body = Tenant),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create tenant.")
        )
    )]
    pub(super) async fn tenants_create() {}

    #[utoipa::path(
        get,
        path = "/admin/projects",
        tag = "tenants",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible project catalog.", body = [Project]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load projects.")
        )
    )]
    pub(super) async fn projects_list() {}

    #[utoipa::path(
        post,
        path = "/admin/projects",
        tag = "tenants",
        request_body = CreateProjectRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created project.", body = Project),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create project.")
        )
    )]
    pub(super) async fn projects_create() {}

    #[utoipa::path(
        get,
        path = "/admin/users/operators",
        tag = "users",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible operator users.", body = [AdminUserProfile]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load operator users.", body = ErrorResponse)
        )
    )]
    pub(super) async fn operator_users_list() {}

    #[utoipa::path(
        post,
        path = "/admin/users/operators",
        tag = "users",
        request_body = UpsertOperatorUserRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated operator user.", body = AdminUserProfile),
            (status = 400, description = "Invalid operator user payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist operator user.", body = ErrorResponse)
        )
    )]
    pub(super) async fn operator_users_upsert() {}

    #[utoipa::path(
        post,
        path = "/admin/users/operators/{user_id}/status",
        tag = "users",
        params(("user_id" = String, Path, description = "Operator user id.")),
        request_body = UpdateUserStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated operator user status.", body = AdminUserProfile),
            (status = 400, description = "Invalid operator user status payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to update operator user status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn operator_user_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/users/portal",
        tag = "users",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible portal users.", body = [PortalUserProfile]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load portal users.", body = ErrorResponse)
        )
    )]
    pub(super) async fn portal_users_list() {}

    #[utoipa::path(
        post,
        path = "/admin/users/portal",
        tag = "users",
        request_body = UpsertPortalUserRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated portal user.", body = PortalUserProfile),
            (status = 400, description = "Invalid portal user payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist portal user.", body = ErrorResponse)
        )
    )]
    pub(super) async fn portal_users_upsert() {}

    #[utoipa::path(
        post,
        path = "/admin/users/portal/{user_id}/status",
        tag = "users",
        params(("user_id" = String, Path, description = "Portal user id.")),
        request_body = UpdateUserStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated portal user status.", body = PortalUserProfile),
            (status = 400, description = "Invalid portal user status payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to update portal user status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn portal_user_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/coupon-templates",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon templates.", body = [CouponTemplateRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon templates.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates",
        tag = "marketing",
        request_body = CouponTemplateRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical coupon template.", body = CouponTemplateRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical coupon template.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/coupon-templates/{coupon_template_id}/status",
        tag = "marketing",
        params(("coupon_template_id" = String, Path, description = "Coupon template id")),
        request_body = UpdateCouponTemplateStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical coupon template status.", body = CouponTemplateRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical coupon template status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_coupon_templates_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/campaigns",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical marketing campaigns.", body = [MarketingCampaignRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical marketing campaigns.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns",
        tag = "marketing",
        request_body = MarketingCampaignRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical marketing campaign.", body = MarketingCampaignRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical marketing campaign.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/campaigns/{marketing_campaign_id}/status",
        tag = "marketing",
        params(("marketing_campaign_id" = String, Path, description = "Marketing campaign id")),
        request_body = UpdateMarketingCampaignStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical marketing campaign status.", body = MarketingCampaignRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical marketing campaign not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical marketing campaign status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_campaigns_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/budgets",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical campaign budgets.", body = [CampaignBudgetRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical campaign budgets.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/budgets",
        tag = "marketing",
        request_body = CampaignBudgetRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical campaign budget.", body = CampaignBudgetRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical campaign budget.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/budgets/{campaign_budget_id}/status",
        tag = "marketing",
        params(("campaign_budget_id" = String, Path, description = "Campaign budget id")),
        request_body = UpdateCampaignBudgetStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical campaign budget status.", body = CampaignBudgetRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical campaign budget not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical campaign budget status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_budgets_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/codes",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon codes.", body = [CouponCodeRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon codes.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_list() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/codes",
        tag = "marketing",
        request_body = CouponCodeRecord,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created or updated canonical coupon code.", body = CouponCodeRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to persist canonical coupon code.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_create() {}

    #[utoipa::path(
        post,
        path = "/admin/marketing/codes/{coupon_code_id}/status",
        tag = "marketing",
        params(("coupon_code_id" = String, Path, description = "Coupon code id")),
        request_body = UpdateCouponCodeStatusRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated canonical coupon code status.", body = CouponCodeRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update canonical coupon code status.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_codes_status_update() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/reservations",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon reservations.", body = [CouponReservationRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon reservations.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_reservations_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/redemptions",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon redemptions.", body = [CouponRedemptionRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon redemptions.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_redemptions_list() {}

    #[utoipa::path(
        get,
        path = "/admin/marketing/rollbacks",
        tag = "marketing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible canonical coupon rollback records.", body = [CouponRollbackRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load canonical coupon rollback records.", body = ErrorResponse)
        )
    )]
    pub(super) async fn marketing_rollbacks_list() {}

    #[utoipa::path(
        get,
        path = "/admin/api-keys",
        tag = "gateway",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible gateway API keys.", body = [GatewayApiKeyRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load gateway API keys.")
        )
    )]
    pub(super) async fn api_keys_list() {}

    #[utoipa::path(
        post,
        path = "/admin/api-keys",
        tag = "gateway",
        request_body = CreateApiKeyRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created gateway API key.", body = CreatedGatewayApiKey),
            (status = 400, description = "Invalid gateway API key payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create gateway API key.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_keys_create() {}

    #[utoipa::path(
        put,
        path = "/admin/api-keys/{hashed_key}",
        tag = "gateway",
        params(("hashed_key" = String, Path, description = "Hashed gateway API key identifier.")),
        request_body = UpdateApiKeyRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated gateway API key metadata.", body = GatewayApiKeyRecord),
            (status = 400, description = "Invalid gateway API key update payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Gateway API key not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update gateway API key.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_key_update() {}

    #[utoipa::path(
        get,
        path = "/admin/api-key-groups",
        tag = "gateway",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible gateway API key groups.", body = [ApiKeyGroupRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load gateway API key groups.")
        )
    )]
    pub(super) async fn api_key_groups_list() {}

    #[utoipa::path(
        post,
        path = "/admin/api-key-groups",
        tag = "gateway",
        request_body = CreateApiKeyGroupRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 201, description = "Created gateway API key group.", body = ApiKeyGroupRecord),
            (status = 400, description = "Invalid gateway API key group payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to create gateway API key group.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_key_groups_create() {}

    #[utoipa::path(
        patch,
        path = "/admin/api-key-groups/{group_id}",
        tag = "gateway",
        params(("group_id" = String, Path, description = "Gateway API key group identifier.")),
        request_body = UpdateApiKeyGroupRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated gateway API key group.", body = ApiKeyGroupRecord),
            (status = 400, description = "Invalid gateway API key group update payload.", body = ErrorResponse),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Gateway API key group not found.", body = ErrorResponse),
            (status = 500, description = "Failed to update gateway API key group.", body = ErrorResponse)
        )
    )]
    pub(super) async fn api_key_group_update() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/ledger",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible billing ledger entries.", body = [LedgerEntry]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing ledger.")
        )
    )]
    pub(super) async fn billing_ledger_list() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/events",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible billing events.", body = [BillingEventRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing events.")
        )
    )]
    pub(super) async fn billing_events_list() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/events/summary",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Billing events summary.", body = BillingEventSummary),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing event summary.")
        )
    )]
    pub(super) async fn billing_events_summary() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/summary",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Billing summary.", body = BillingSummary),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load billing summary.")
        )
    )]
    pub(super) async fn billing_summary() {}

    #[utoipa::path(
        post,
        path = "/admin/billing/pricing-lifecycle/synchronize",
        tag = "billing",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Synchronized due planned commercial pricing lifecycle state.", body = PricingLifecycleSynchronizationReport),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to synchronize commercial pricing lifecycle.", body = ErrorResponse)
        )
    )]
    pub(super) async fn billing_pricing_lifecycle_synchronize() {}

    #[utoipa::path(
        get,
        path = "/admin/billing/accounts/{account_id}/ledger",
        tag = "billing",
        security(("bearerAuth" = [])),
        params(("account_id" = u64, Path, description = "Canonical commercial account identifier.")),
        responses(
            (status = 200, description = "Canonical account ledger history.", body = [AccountLedgerHistoryEntry]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Account not found.", body = ErrorResponse),
            (status = 501, description = "Commercial billing kernel is not configured.", body = ErrorResponse),
            (status = 500, description = "Failed to load canonical account ledger history.", body = ErrorResponse)
        )
    )]
    pub(super) async fn billing_account_ledger() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(
            ("limit" = Option<usize>, Query, description = "Maximum number of recent commerce orders to return. Defaults to 24 and is capped at 100.")
        ),
        responses(
            (status = 200, description = "Recent commerce orders ordered by newest activity first.", body = [CommerceOrderRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load recent commerce orders.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_orders_recent() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders/{order_id}/payment-events",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Payment events recorded for the selected commerce order.", body = [CommercePaymentEventRecord]),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 500, description = "Failed to load commerce payment events.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_payment_events() {}

    #[utoipa::path(
        get,
        path = "/admin/commerce/orders/{order_id}/audit",
        tag = "commerce",
        security(("bearerAuth" = [])),
        params(("order_id" = String, Path, description = "Commerce order id.")),
        responses(
            (status = 200, description = "Aggregated payment and coupon evidence chain for the selected commerce order.", body = CommerceOrderAuditRecord),
            (status = 401, description = "Missing or invalid admin bearer token."),
            (status = 404, description = "Commerce order not found.", body = ErrorResponse),
            (status = 500, description = "Failed to load commerce order audit evidence.", body = ErrorResponse)
        )
    )]
    pub(super) async fn commerce_order_audit() {}
}

fn admin_openapi() -> utoipa::openapi::OpenApi {
    OpenApiRouter::<()>::with_openapi(AdminApiDoc::openapi())
        .routes(routes!(openapi_paths::health))
        .routes(routes!(openapi_paths::auth_login))
        .routes(routes!(openapi_paths::auth_change_password))
        .routes(routes!(openapi_paths::tenants_list))
        .routes(routes!(openapi_paths::tenants_create))
        .routes(routes!(openapi_paths::projects_list))
        .routes(routes!(openapi_paths::projects_create))
        .routes(routes!(openapi_paths::operator_users_list))
        .routes(routes!(openapi_paths::operator_users_upsert))
        .routes(routes!(openapi_paths::operator_user_status_update))
        .routes(routes!(openapi_paths::portal_users_list))
        .routes(routes!(openapi_paths::portal_users_upsert))
        .routes(routes!(openapi_paths::portal_user_status_update))
        .routes(routes!(openapi_paths::marketing_coupon_templates_list))
        .routes(routes!(openapi_paths::marketing_coupon_templates_create))
        .routes(routes!(
            openapi_paths::marketing_coupon_templates_status_update
        ))
        .routes(routes!(openapi_paths::marketing_campaigns_list))
        .routes(routes!(openapi_paths::marketing_campaigns_create))
        .routes(routes!(openapi_paths::marketing_campaigns_status_update))
        .routes(routes!(openapi_paths::marketing_budgets_list))
        .routes(routes!(openapi_paths::marketing_budgets_create))
        .routes(routes!(openapi_paths::marketing_budgets_status_update))
        .routes(routes!(openapi_paths::marketing_codes_list))
        .routes(routes!(openapi_paths::marketing_codes_create))
        .routes(routes!(openapi_paths::marketing_codes_status_update))
        .routes(routes!(openapi_paths::marketing_reservations_list))
        .routes(routes!(openapi_paths::marketing_redemptions_list))
        .routes(routes!(openapi_paths::marketing_rollbacks_list))
        .routes(routes!(openapi_paths::api_keys_list))
        .routes(routes!(openapi_paths::api_keys_create))
        .routes(routes!(openapi_paths::api_key_update))
        .routes(routes!(openapi_paths::api_key_groups_list))
        .routes(routes!(openapi_paths::api_key_groups_create))
        .routes(routes!(openapi_paths::api_key_group_update))
        .routes(routes!(openapi_paths::billing_ledger_list))
        .routes(routes!(openapi_paths::billing_events_list))
        .routes(routes!(openapi_paths::billing_events_summary))
        .routes(routes!(openapi_paths::billing_summary))
        .routes(routes!(
            openapi_paths::billing_pricing_lifecycle_synchronize
        ))
        .routes(routes!(openapi_paths::billing_account_ledger))
        .routes(routes!(openapi_paths::commerce_orders_recent))
        .routes(routes!(openapi_paths::commerce_order_payment_events))
        .routes(routes!(openapi_paths::commerce_order_audit))
        .into_openapi()
}

async fn admin_openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(admin_openapi())
}

async fn admin_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Admin API</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      body {
        margin: 0;
        background: #f5f7fb;
        color: #101828;
      }

      .shell {
        display: grid;
        min-height: 100vh;
        grid-template-rows: auto 1fr;
      }

      .hero {
        padding: 20px 24px 16px;
        border-bottom: 1px solid rgba(15, 23, 42, 0.08);
        background: rgba(255, 255, 255, 0.96);
      }

      .eyebrow {
        margin: 0 0 8px;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #475467;
      }

      h1 {
        margin: 0 0 8px;
        font-size: 28px;
        line-height: 1.1;
      }

      p {
        margin: 0;
        font-size: 14px;
        line-height: 1.6;
        color: #475467;
      }

      code {
        padding: 2px 6px;
        border-radius: 999px;
        background: rgba(15, 23, 42, 0.06);
        font-size: 12px;
      }

      iframe {
        width: 100%;
        height: 100%;
        border: 0;
        background: white;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #09090b;
          color: #fafafa;
        }

        .hero {
          background: rgba(24, 24, 27, 0.96);
          border-bottom-color: rgba(255, 255, 255, 0.08);
        }

        .eyebrow,
        p {
          color: #a1a1aa;
        }

        code {
          background: rgba(255, 255, 255, 0.08);
        }
      }
    </style>
  </head>
  <body>
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">OpenAPI 3.1</p>
        <h1>SDKWORK Admin API</h1>
        <p>Interactive documentation is backed by the live schema endpoint <code>/admin/openapi.json</code>.</p>
      </section>
      <iframe src="/admin/docs/ui/" title="SDKWORK Admin API"></iframe>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

fn admin_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_index_handler))
        .merge(
            SwaggerUi::new("/admin/docs/ui/").config(SwaggerUiConfig::new([
                SwaggerUiUrl::with_primary("SDKWORK Admin API", "/admin/openapi.json", true),
            ])),
        )
}

pub struct AdminApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for AdminApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            live_commercial_billing: self.live_commercial_billing.clone(),
            store: self.live_store.snapshot(),
            secret_manager: self.live_secret_manager.snapshot(),
            commercial_billing: self
                .live_commercial_billing
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.commercial_billing.clone()),
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
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_and_secret_manager_and_jwt_secret_and_commercial_billing(
            store.clone(),
            CredentialSecretManager::database_encrypted(credential_master_key),
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
            Some(store),
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
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_and_secret_manager_and_jwt_secret_and_commercial_billing(
            store.clone(),
            secret_manager,
            jwt_signing_secret,
            Some(store),
        )
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret_and_commercial_billing(
            store,
            secret_manager,
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
            None,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret_and_commercial_billing(
            store,
            secret_manager,
            jwt_signing_secret,
            None,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret_and_commercial_billing(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            commercial_billing.map(Reloadable::new),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_secret_manager_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_and_jwt_secret_handle(
            live_store,
            Reloadable::new(secret_manager),
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_and_jwt_secret_handle(
            live_store,
            live_secret_manager,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_commercial_billing_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            live_store,
            live_secret_manager,
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_commercial_billing,
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

#[derive(Debug, Deserialize, ToSchema)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct LoginResponse {
    token: String,
    claims: Claims,
    user: AdminUserProfile,
}

#[derive(Debug, Deserialize, ToSchema)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

fn default_user_active() -> bool {
    true
}

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateUserStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateCouponTemplateStatusRequest {
    status: CouponTemplateStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateMarketingCampaignStatusRequest {
    status: MarketingCampaignStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateCampaignBudgetStatusRequest {
    status: CampaignBudgetStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateCouponCodeStatusRequest {
    status: CouponCodeStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
struct ResetUserPasswordRequest {
    new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorBody {
    message: String,
}

#[derive(Debug, Deserialize)]
struct RecentCommerceOrdersQuery {
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct CommerceOrderAuditRecord {
    order: CommerceOrderRecord,
    payment_events: Vec<CommercePaymentEventRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coupon_reservation: Option<CouponReservationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coupon_redemption: Option<CouponRedemptionRecord>,
    #[serde(default)]
    coupon_rollbacks: Vec<CouponRollbackRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coupon_code: Option<CouponCodeRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coupon_template: Option<CouponTemplateRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    marketing_campaign: Option<MarketingCampaignRecord>,
}

#[derive(Debug, Serialize)]
struct CommercialAccountSummaryResponse {
    account: AccountRecord,
    available_balance: f64,
    held_balance: f64,
    consumed_balance: f64,
    grant_balance: f64,
    active_lot_count: u64,
}

impl CommercialAccountSummaryResponse {
    fn from_balance(account: AccountRecord, balance: &AccountBalanceSnapshot) -> Self {
        Self {
            account,
            available_balance: balance.available_balance,
            held_balance: balance.held_balance,
            consumed_balance: balance.consumed_balance,
            grant_balance: balance.grant_balance,
            active_lot_count: balance.active_lot_count,
        }
    }
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

fn default_credit_unit_code() -> String {
    "credit".to_owned()
}

fn default_price_unit() -> String {
    "per_1m_tokens".to_owned()
}

fn default_charge_unit() -> String {
    "unit".to_owned()
}

fn default_pricing_method() -> String {
    "per_unit".to_owned()
}

fn default_rounding_increment() -> f64 {
    1.0
}

fn default_rounding_mode() -> String {
    "none".to_owned()
}

fn default_pricing_status() -> String {
    "draft".to_owned()
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

#[derive(Debug, Deserialize, ToSchema)]
struct CreateTenantRequest {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
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
    #[serde(default = "default_true")]
    execution_failover_enabled: bool,
    #[serde(default)]
    upstream_retry_max_attempts: Option<u32>,
    #[serde(default)]
    upstream_retry_base_delay_ms: Option<u64>,
    #[serde(default)]
    upstream_retry_max_delay_ms: Option<u64>,
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
struct CreateCommercialPricingPlanRequest {
    tenant_id: u64,
    #[serde(default)]
    organization_id: u64,
    plan_code: String,
    plan_version: u64,
    display_name: String,
    #[serde(default = "default_currency_code")]
    currency_code: String,
    #[serde(default = "default_credit_unit_code")]
    credit_unit_code: String,
    #[serde(default = "default_pricing_status")]
    status: String,
    #[serde(default)]
    effective_from_ms: u64,
    #[serde(default)]
    effective_to_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CloneCommercialPricingPlanRequest {
    #[serde(default)]
    plan_version: Option<u64>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default = "default_pricing_status")]
    status: String,
}

#[derive(Debug, Deserialize, Default)]
struct PublishCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize, Default)]
struct ScheduleCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize, Default)]
struct RetireCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize)]
struct CreateCommercialPricingRateRequest {
    tenant_id: u64,
    #[serde(default)]
    organization_id: u64,
    pricing_plan_id: u64,
    metric_code: String,
    capability_code: Option<String>,
    model_code: Option<String>,
    provider_code: Option<String>,
    #[serde(default = "default_charge_unit")]
    charge_unit: String,
    #[serde(default = "default_pricing_method")]
    pricing_method: String,
    #[serde(default = "default_rounding_increment")]
    quantity_step: f64,
    #[serde(default)]
    unit_price: f64,
    #[serde(default)]
    display_price_unit: String,
    #[serde(default)]
    minimum_billable_quantity: f64,
    #[serde(default)]
    minimum_charge: f64,
    #[serde(default = "default_rounding_increment")]
    rounding_increment: f64,
    #[serde(default = "default_rounding_mode")]
    rounding_mode: String,
    #[serde(default)]
    included_quantity: f64,
    #[serde(default)]
    priority: u64,
    notes: Option<String>,
    #[serde(default = "default_pricing_status")]
    status: String,
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

fn http_exposure_config() -> anyhow::Result<HttpExposureConfig> {
    HttpExposureConfig::from_env()
}

fn metrics_route<S>(
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: &HttpExposureConfig,
) -> axum::routing::MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    let expected_token: Arc<str> = Arc::from(http_exposure.metrics_bearer_token.clone());
    get(move |headers: HeaderMap| {
        let metrics = metrics.clone();
        let expected_token = expected_token.clone();
        async move {
            if !metrics_request_authorized(&headers, expected_token.as_ref()) {
                return (
                    StatusCode::UNAUTHORIZED,
                    [(header::WWW_AUTHENTICATE, "Bearer")],
                    "metrics bearer token required",
                )
                    .into_response();
            }

            (
                [(
                    header::CONTENT_TYPE,
                    "text/plain; version=0.0.4; charset=utf-8",
                )],
                metrics.render_prometheus(),
            )
                .into_response()
        }
    })
}

fn metrics_request_authorized(headers: &HeaderMap, expected_token: &str) -> bool {
    if expected_token.is_empty() {
        return false;
    }

    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, token)) = value.trim().split_once(' ') else {
        return false;
    };
    scheme.eq_ignore_ascii_case("Bearer") && token.trim() == expected_token
}

pub fn try_admin_router() -> anyhow::Result<Router> {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    let http_exposure = http_exposure_config()?;
    Ok(Router::new()
        .merge(admin_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
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
            "/admin/billing/accounts",
            get(|| async { "billing-accounts" }),
        )
        .route(
            "/admin/billing/accounts/{account_id}/balance",
            get(|| async { "billing-account-balance" }),
        )
        .route(
            "/admin/billing/accounts/{account_id}/benefit-lots",
            get(|| async { "billing-account-benefit-lots" }),
        )
        .route(
            "/admin/billing/accounts/{account_id}/ledger",
            get(|| async { "billing-account-ledger" }),
        )
        .route(
            "/admin/billing/account-holds",
            get(|| async { "billing-account-holds" }),
        )
        .route(
            "/admin/billing/request-settlements",
            get(|| async { "billing-request-settlements" }),
        )
        .route(
            "/admin/commerce/orders",
            get(|| async { "commerce-orders" }),
        )
        .route(
            "/admin/commerce/orders/{order_id}/payment-events",
            get(|| async { "commerce-order-payment-events" }),
        )
        .route(
            "/admin/commerce/orders/{order_id}/audit",
            get(|| async { "commerce-order-audit" }),
        )
        .route("/admin/async-jobs", get(|| async { "async-jobs" }))
        .route(
            "/admin/async-jobs/{job_id}/attempts",
            get(|| async { "async-job-attempts" }),
        )
        .route(
            "/admin/async-jobs/{job_id}/assets",
            get(|| async { "async-job-assets" }),
        )
        .route(
            "/admin/async-jobs/{job_id}/callbacks",
            get(|| async { "async-job-callbacks" }),
        )
        .route(
            "/admin/billing/pricing-lifecycle/synchronize",
            post(|| async { "billing-pricing-lifecycle-synchronize" }),
        )
        .route(
            "/admin/billing/pricing-plans",
            get(|| async { "billing-pricing-plans" })
                .post(|| async { "billing-pricing-plans-create" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}",
            put(|| async { "billing-pricing-plans-update" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/clone",
            post(|| async { "billing-pricing-plans-clone" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/schedule",
            post(|| async { "billing-pricing-plans-schedule" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/publish",
            post(|| async { "billing-pricing-plans-publish" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/retire",
            post(|| async { "billing-pricing-plans-retire" }),
        )
        .route(
            "/admin/billing/pricing-rates",
            get(|| async { "billing-pricing-rates" })
                .post(|| async { "billing-pricing-rates-create" }),
        )
        .route(
            "/admin/billing/pricing-rates/{pricing_rate_id}",
            put(|| async { "billing-pricing-rates-update" }),
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
        )))
}

pub fn admin_router() -> Router {
    try_admin_router().expect("http exposure config should load from process env")
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
    admin_router_with_state(AdminApiState::with_master_key(pool, credential_master_key))
}

pub fn admin_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    admin_router_with_state(AdminApiState::with_secret_manager(pool, secret_manager))
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

pub fn try_admin_router_with_state(state: AdminApiState) -> anyhow::Result<Router> {
    Ok(admin_router_with_state_and_http_exposure(
        state,
        http_exposure_config()?,
    ))
}

pub fn admin_router_with_state(state: AdminApiState) -> Router {
    try_admin_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn admin_router_with_state_and_http_exposure(
    state: AdminApiState,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    Router::new()
        .merge(admin_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
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
            "/admin/marketing/coupon-templates",
            get(list_marketing_coupon_templates_handler)
                .post(create_marketing_coupon_template_handler),
        )
        .route(
            "/admin/marketing/coupon-templates/{coupon_template_id}/status",
            post(update_marketing_coupon_template_status_handler),
        )
        .route(
            "/admin/marketing/campaigns",
            get(list_marketing_campaigns_handler).post(create_marketing_campaign_handler),
        )
        .route(
            "/admin/marketing/campaigns/{marketing_campaign_id}/status",
            post(update_marketing_campaign_status_handler),
        )
        .route(
            "/admin/marketing/budgets",
            get(list_marketing_budgets_handler).post(create_marketing_budget_handler),
        )
        .route(
            "/admin/marketing/budgets/{campaign_budget_id}/status",
            post(update_marketing_budget_status_handler),
        )
        .route(
            "/admin/marketing/codes",
            get(list_marketing_coupon_codes_handler).post(create_marketing_coupon_code_handler),
        )
        .route(
            "/admin/marketing/codes/{coupon_code_id}/status",
            post(update_marketing_coupon_code_status_handler),
        )
        .route(
            "/admin/marketing/reservations",
            get(list_marketing_coupon_reservations_handler),
        )
        .route(
            "/admin/marketing/redemptions",
            get(list_marketing_coupon_redemptions_handler),
        )
        .route(
            "/admin/marketing/rollbacks",
            get(list_marketing_coupon_rollbacks_handler),
        )
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
        .route(
            "/admin/billing/accounts",
            get(list_canonical_accounts_handler),
        )
        .route(
            "/admin/billing/accounts/{account_id}/balance",
            get(get_canonical_account_balance_handler),
        )
        .route(
            "/admin/billing/accounts/{account_id}/benefit-lots",
            get(list_canonical_account_benefit_lots_handler),
        )
        .route(
            "/admin/billing/accounts/{account_id}/ledger",
            get(list_canonical_account_ledger_handler),
        )
        .route(
            "/admin/billing/account-holds",
            get(list_canonical_account_holds_handler),
        )
        .route(
            "/admin/billing/request-settlements",
            get(list_canonical_request_settlements_handler),
        )
        .route(
            "/admin/commerce/orders",
            get(list_recent_commerce_orders_handler),
        )
        .route(
            "/admin/commerce/orders/{order_id}/payment-events",
            get(list_commerce_payment_events_handler),
        )
        .route(
            "/admin/commerce/orders/{order_id}/audit",
            get(get_commerce_order_audit_handler),
        )
        .route("/admin/async-jobs", get(list_async_jobs_handler))
        .route(
            "/admin/async-jobs/{job_id}/attempts",
            get(list_async_job_attempts_handler),
        )
        .route(
            "/admin/async-jobs/{job_id}/assets",
            get(list_async_job_assets_handler),
        )
        .route(
            "/admin/async-jobs/{job_id}/callbacks",
            get(list_async_job_callbacks_handler),
        )
        .route(
            "/admin/billing/pricing-lifecycle/synchronize",
            post(synchronize_canonical_pricing_lifecycle_handler),
        )
        .route(
            "/admin/billing/pricing-plans",
            get(list_canonical_pricing_plans_handler).post(create_canonical_pricing_plan_handler),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}",
            put(update_canonical_pricing_plan_handler),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/clone",
            post(clone_canonical_pricing_plan_handler),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/schedule",
            post(schedule_canonical_pricing_plan_handler),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/publish",
            post(publish_canonical_pricing_plan_handler),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/retire",
            post(retire_canonical_pricing_plan_handler),
        )
        .route(
            "/admin/billing/pricing-rates",
            get(list_canonical_pricing_rates_handler).post(create_canonical_pricing_rate_handler),
        )
        .route(
            "/admin/billing/pricing-rates/{pricing_rate_id}",
            put(update_canonical_pricing_rate_handler),
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
    let mut coupons = list_coupons(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let marketing_coupons = list_legacy_coupon_projections_from_marketing(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut seen_codes = coupons
        .iter()
        .map(|coupon| coupon.code.to_ascii_uppercase())
        .collect::<HashSet<_>>();
    for coupon in marketing_coupons {
        if seen_codes.insert(coupon.code.to_ascii_uppercase()) {
            coupons.push(coupon);
        }
    }
    coupons.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(Json(coupons))
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
    sync_legacy_coupon_marketing_projection(state.store.as_ref(), &coupon)
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

async fn list_marketing_coupon_templates_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponTemplateRecord>>, StatusCode> {
    let mut templates = state
        .store
        .list_coupon_template_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    templates.sort_by(|left, right| left.template_key.cmp(&right.template_key));
    Ok(Json(templates))
}

async fn create_marketing_coupon_template_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CouponTemplateRecord>,
) -> Result<(StatusCode, Json<CouponTemplateRecord>), StatusCode> {
    let record = state
        .store
        .insert_coupon_template_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn update_marketing_coupon_template_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_template_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCouponTemplateStatusRequest>,
) -> Result<Json<CouponTemplateRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_coupon_template_status(
        state.store.as_ref(),
        &coupon_template_id,
        request.status,
    )
    .await
    .map(Json)
    .map_err(|(status, message)| error_response(status, message))
}

async fn list_marketing_campaigns_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<MarketingCampaignRecord>>, StatusCode> {
    let mut campaigns = state
        .store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    campaigns.sort_by(|left, right| left.marketing_campaign_id.cmp(&right.marketing_campaign_id));
    Ok(Json(campaigns))
}

async fn create_marketing_campaign_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<MarketingCampaignRecord>,
) -> Result<(StatusCode, Json<MarketingCampaignRecord>), StatusCode> {
    let record = state
        .store
        .insert_marketing_campaign_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn update_marketing_campaign_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(marketing_campaign_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateMarketingCampaignStatusRequest>,
) -> Result<Json<MarketingCampaignRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_campaign_status(state.store.as_ref(), &marketing_campaign_id, request.status)
        .await
        .map(Json)
        .map_err(|(status, message)| error_response(status, message))
}

async fn list_marketing_budgets_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CampaignBudgetRecord>>, StatusCode> {
    let mut budgets = state
        .store
        .list_campaign_budget_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    budgets.sort_by(|left, right| left.campaign_budget_id.cmp(&right.campaign_budget_id));
    Ok(Json(budgets))
}

async fn create_marketing_budget_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CampaignBudgetRecord>,
) -> Result<(StatusCode, Json<CampaignBudgetRecord>), StatusCode> {
    let record = state
        .store
        .insert_campaign_budget_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn update_marketing_budget_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(campaign_budget_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCampaignBudgetStatusRequest>,
) -> Result<Json<CampaignBudgetRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_campaign_budget_status(
        state.store.as_ref(),
        &campaign_budget_id,
        request.status,
    )
    .await
    .map(Json)
    .map_err(|(status, message)| error_response(status, message))
}

async fn list_marketing_coupon_codes_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCodeRecord>>, StatusCode> {
    let mut codes = state
        .store
        .list_coupon_code_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    codes.sort_by(|left, right| left.code_value.cmp(&right.code_value));
    Ok(Json(codes))
}

async fn create_marketing_coupon_code_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(record): Json<CouponCodeRecord>,
) -> Result<(StatusCode, Json<CouponCodeRecord>), StatusCode> {
    let record = state
        .store
        .insert_coupon_code_record(&record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn update_marketing_coupon_code_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_code_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateCouponCodeStatusRequest>,
) -> Result<Json<CouponCodeRecord>, (StatusCode, Json<ErrorResponse>)> {
    update_marketing_coupon_code_status(state.store.as_ref(), &coupon_code_id, request.status)
        .await
        .map(Json)
        .map_err(|(status, message)| error_response(status, message))
}

async fn list_marketing_coupon_reservations_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponReservationRecord>>, StatusCode> {
    let mut reservations = state
        .store
        .list_coupon_reservation_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    reservations
        .sort_by(|left, right| left.coupon_reservation_id.cmp(&right.coupon_reservation_id));
    Ok(Json(reservations))
}

async fn list_marketing_coupon_redemptions_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponRedemptionRecord>>, StatusCode> {
    let mut redemptions = state
        .store
        .list_coupon_redemption_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    redemptions.sort_by(|left, right| left.coupon_redemption_id.cmp(&right.coupon_redemption_id));
    Ok(Json(redemptions))
}

async fn list_marketing_coupon_rollbacks_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponRollbackRecord>>, StatusCode> {
    let mut rollbacks = state
        .store
        .list_coupon_rollback_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    rollbacks.sort_by(|left, right| left.coupon_rollback_id.cmp(&right.coupon_rollback_id));
    Ok(Json(rollbacks))
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

async fn sync_legacy_coupon_marketing_projection(
    store: &dyn AdminStore,
    coupon: &CouponCampaign,
) -> anyhow::Result<()> {
    let (template, campaign, budget, code) = project_legacy_coupon_campaign(coupon);
    store.insert_coupon_template_record(&template).await?;
    store.insert_marketing_campaign_record(&campaign).await?;
    store.insert_campaign_budget_record(&budget).await?;
    store.insert_coupon_code_record(&code).await?;
    Ok(())
}

async fn update_marketing_coupon_template_status(
    store: &dyn AdminStore,
    coupon_template_id: &str,
    status: CouponTemplateStatus,
) -> Result<CouponTemplateRecord, (StatusCode, String)> {
    let record = store
        .find_coupon_template_record(coupon_template_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon template".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("coupon template {coupon_template_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_coupon_template_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon template status".to_owned(),
            )
        })
}

async fn update_marketing_campaign_status(
    store: &dyn AdminStore,
    marketing_campaign_id: &str,
    status: MarketingCampaignStatus,
) -> Result<MarketingCampaignRecord, (StatusCode, String)> {
    let record = store
        .list_marketing_campaign_records()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical marketing campaigns".to_owned(),
            )
        })?
        .into_iter()
        .find(|record| record.marketing_campaign_id == marketing_campaign_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("marketing campaign {marketing_campaign_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_marketing_campaign_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical marketing campaign status".to_owned(),
            )
        })
}

async fn update_marketing_campaign_budget_status(
    store: &dyn AdminStore,
    campaign_budget_id: &str,
    status: CampaignBudgetStatus,
) -> Result<CampaignBudgetRecord, (StatusCode, String)> {
    let record = store
        .list_campaign_budget_records()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical campaign budgets".to_owned(),
            )
        })?
        .into_iter()
        .find(|record| record.campaign_budget_id == campaign_budget_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("campaign budget {campaign_budget_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_campaign_budget_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical campaign budget status".to_owned(),
            )
        })
}

async fn update_marketing_coupon_code_status(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    status: CouponCodeStatus,
) -> Result<CouponCodeRecord, (StatusCode, String)> {
    let record = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load canonical coupon code".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("coupon code {coupon_code_id} not found"),
            )
        })?;
    let updated = record
        .with_status(status)
        .with_updated_at_ms(unix_timestamp_ms());
    store
        .insert_coupon_code_record(&updated)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist canonical coupon code status".to_owned(),
            )
        })
}

async fn list_legacy_coupon_projections_from_marketing(
    store: &dyn AdminStore,
) -> anyhow::Result<Vec<CouponCampaign>> {
    let templates = store.list_coupon_template_records().await?;
    let campaigns = store.list_marketing_campaign_records().await?;
    let budgets = store.list_campaign_budget_records().await?;
    let codes = store.list_coupon_code_records().await?;

    let mut preferred_campaigns_by_template = HashMap::<String, MarketingCampaignRecord>::new();
    for campaign in campaigns {
        match preferred_campaigns_by_template.get_mut(&campaign.coupon_template_id) {
            Some(existing) if should_replace_marketing_campaign(existing, &campaign) => {
                *existing = campaign;
            }
            None => {
                preferred_campaigns_by_template
                    .insert(campaign.coupon_template_id.clone(), campaign);
            }
            Some(_) => {}
        }
    }

    let mut preferred_budgets_by_campaign = HashMap::<String, CampaignBudgetRecord>::new();
    for budget in budgets {
        match preferred_budgets_by_campaign.get_mut(&budget.marketing_campaign_id) {
            Some(existing) if should_replace_campaign_budget(existing, &budget) => {
                *existing = budget;
            }
            None => {
                preferred_budgets_by_campaign.insert(budget.marketing_campaign_id.clone(), budget);
            }
            Some(_) => {}
        }
    }

    let mut coupon_codes_by_template = HashMap::<String, Vec<CouponCodeRecord>>::new();
    for code in codes {
        coupon_codes_by_template
            .entry(code.coupon_template_id.clone())
            .or_default()
            .push(code);
    }
    for template_codes in coupon_codes_by_template.values_mut() {
        template_codes.sort_by(|left, right| left.code_value.cmp(&right.code_value));
    }

    let now_ms = unix_timestamp_ms();
    let mut coupons = Vec::new();
    for template in templates {
        if template.coupon_template_id.starts_with("legacy_tpl_") {
            continue;
        }
        let Some(template_codes) = coupon_codes_by_template.get(&template.coupon_template_id)
        else {
            continue;
        };
        let campaign = preferred_campaigns_by_template.get(&template.coupon_template_id);
        let budget = campaign
            .and_then(|record| preferred_budgets_by_campaign.get(&record.marketing_campaign_id));
        for code in template_codes {
            coupons.push(project_marketing_coupon_to_legacy(
                &template, campaign, budget, code, now_ms,
            ));
        }
    }

    coupons.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(coupons)
}

fn should_replace_marketing_campaign(
    existing: &MarketingCampaignRecord,
    candidate: &MarketingCampaignRecord,
) -> bool {
    marketing_campaign_priority(candidate.status) > marketing_campaign_priority(existing.status)
        || (marketing_campaign_priority(candidate.status)
            == marketing_campaign_priority(existing.status)
            && candidate.updated_at_ms > existing.updated_at_ms)
}

fn should_replace_campaign_budget(
    existing: &CampaignBudgetRecord,
    candidate: &CampaignBudgetRecord,
) -> bool {
    campaign_budget_priority(candidate.status) > campaign_budget_priority(existing.status)
        || (campaign_budget_priority(candidate.status) == campaign_budget_priority(existing.status)
            && candidate.updated_at_ms > existing.updated_at_ms)
}

fn marketing_campaign_priority(status: MarketingCampaignStatus) -> u8 {
    match status {
        MarketingCampaignStatus::Active => 5,
        MarketingCampaignStatus::Scheduled => 4,
        MarketingCampaignStatus::Paused => 3,
        MarketingCampaignStatus::Draft => 2,
        MarketingCampaignStatus::Ended => 1,
        MarketingCampaignStatus::Archived => 0,
    }
}

fn campaign_budget_priority(status: CampaignBudgetStatus) -> u8 {
    match status {
        CampaignBudgetStatus::Active => 3,
        CampaignBudgetStatus::Exhausted => 2,
        CampaignBudgetStatus::Draft => 1,
        CampaignBudgetStatus::Closed => 0,
    }
}

fn project_marketing_coupon_to_legacy(
    template: &CouponTemplateRecord,
    campaign: Option<&MarketingCampaignRecord>,
    budget: Option<&CampaignBudgetRecord>,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCampaign {
    let active = marketing_coupon_is_active(template, campaign, budget, code, now_ms);
    let remaining = budget
        .map(CampaignBudgetRecord::available_budget_minor)
        .unwrap_or(u64::from(active));
    let expires_on = code
        .expires_at_ms
        .or_else(|| campaign.and_then(|record| record.end_at_ms))
        .map(|value| value.to_string())
        .unwrap_or_else(|| "2099-12-31".to_owned());
    let created_at_ms = code
        .created_at_ms
        .max(template.created_at_ms)
        .max(campaign.map_or(0, |record| record.created_at_ms))
        .max(budget.map_or(0, |record| record.created_at_ms));
    let note = marketing_coupon_note(template, campaign);

    CouponCampaign::new(
        code.coupon_code_id.clone(),
        code.code_value.clone(),
        marketing_coupon_discount_label(template),
        marketing_coupon_audience(template.restriction.subject_scope),
        remaining,
        active,
        note,
        expires_on,
    )
    .with_created_at_ms(created_at_ms)
}

fn marketing_coupon_discount_label(template: &CouponTemplateRecord) -> String {
    match template.benefit.benefit_kind {
        MarketingBenefitKind::PercentageOff => template
            .benefit
            .discount_percent
            .map(|value| format!("{value}% off"))
            .unwrap_or_else(|| marketing_coupon_note(template, None)),
        MarketingBenefitKind::FixedAmountOff => template
            .benefit
            .discount_amount_minor
            .map(|value| format!("{value} off"))
            .unwrap_or_else(|| marketing_coupon_note(template, None)),
        MarketingBenefitKind::GrantUnits => template
            .benefit
            .grant_units
            .map(|value| format!("{value} units"))
            .unwrap_or_else(|| marketing_coupon_note(template, None)),
    }
}

fn marketing_coupon_audience(scope: MarketingSubjectScope) -> String {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
    .to_owned()
}

fn marketing_coupon_note(
    template: &CouponTemplateRecord,
    campaign: Option<&MarketingCampaignRecord>,
) -> String {
    campaign
        .map(|record| record.display_name.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_owned())
        .or_else(|| {
            let display_name = template.display_name.trim();
            (!display_name.is_empty()).then(|| display_name.to_owned())
        })
        .unwrap_or_else(|| template.template_key.clone())
}

fn marketing_coupon_is_active(
    template: &CouponTemplateRecord,
    campaign: Option<&MarketingCampaignRecord>,
    budget: Option<&CampaignBudgetRecord>,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> bool {
    let template_active = template.status == CouponTemplateStatus::Active;
    let campaign_active = campaign.is_none_or(|record| record.is_effective_at(now_ms));
    let budget_active = budget.is_none_or(|record| {
        record.status == CampaignBudgetStatus::Active && record.available_budget_minor() > 0
    });
    let code_active = matches!(
        code.status,
        CouponCodeStatus::Available | CouponCodeStatus::Reserved
    ) && code.expires_at_ms.is_none_or(|value| now_ms <= value);

    template_active && campaign_active && budget_active && code_active
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

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody {
                message: message.into(),
            },
        }),
    )
}

fn commercial_billing_kernel(
    state: &AdminApiState,
) -> Result<&Arc<dyn CommercialBillingAdminKernel>, (StatusCode, Json<ErrorResponse>)> {
    state.commercial_billing.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "commercial billing control plane is unavailable for the current storage runtime",
        )
    })
}

fn commercial_billing_error_response(error: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = if message.starts_with("account ") && message.ends_with(" does not exist") {
        StatusCode::NOT_FOUND
    } else if message.contains("does not implement canonical account kernel method") {
        StatusCode::NOT_IMPLEMENTED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    error_response(status, message)
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

fn next_admin_pricing_record_id(now_ms: u64) -> u64 {
    let sequence = ADMIN_PRICING_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed) & 0x000f_ffff;
    (now_ms << 20) | sequence
}

fn normalize_optional_admin_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
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
        execution_failover_enabled: request.execution_failover_enabled,
        upstream_retry_max_attempts: request.upstream_retry_max_attempts,
        upstream_retry_base_delay_ms: request.upstream_retry_base_delay_ms,
        upstream_retry_max_delay_ms: request.upstream_retry_max_delay_ms,
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

async fn list_canonical_accounts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercialAccountSummaryResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let now_ms = unix_timestamp_ms();
    let mut accounts = commercial_billing
        .list_account_records()
        .await
        .map_err(commercial_billing_error_response)?;
    accounts.sort_by_key(|account| account.account_id);

    let mut response = Vec::with_capacity(accounts.len());
    for account in accounts {
        let balance = commercial_billing
            .summarize_account_balance(account.account_id, now_ms)
            .await
            .map_err(commercial_billing_error_response)?;
        response.push(CommercialAccountSummaryResponse::from_balance(
            account, &balance,
        ));
    }

    Ok(Json(response))
}

async fn get_canonical_account_balance_handler(
    _claims: AuthenticatedAdminClaims,
    Path(account_id): Path<u64>,
    State(state): State<AdminApiState>,
) -> Result<Json<AccountBalanceSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let balance = commercial_billing
        .summarize_account_balance(account_id, unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    Ok(Json(balance))
}

async fn list_canonical_account_benefit_lots_handler(
    _claims: AuthenticatedAdminClaims,
    Path(account_id): Path<u64>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AccountBenefitLotRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    commercial_billing
        .find_account_record(account_id)
        .await
        .map_err(commercial_billing_error_response)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("account {account_id} does not exist"),
            )
        })?;

    let mut lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    lots.sort_by_key(|lot| lot.lot_id);
    Ok(Json(lots))
}

async fn list_canonical_account_ledger_handler(
    _claims: AuthenticatedAdminClaims,
    Path(account_id): Path<u64>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AccountLedgerHistoryEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let history = commercial_billing
        .list_account_ledger_history(account_id)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok(Json(history))
}

async fn list_canonical_account_holds_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AccountHoldRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut holds = commercial_billing
        .list_account_holds()
        .await
        .map_err(commercial_billing_error_response)?;
    holds.sort_by_key(|hold| hold.hold_id);
    Ok(Json(holds))
}

async fn list_canonical_request_settlements_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RequestSettlementRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .map_err(commercial_billing_error_response)?;
    settlements.sort_by_key(|settlement| settlement.request_settlement_id);
    Ok(Json(settlements))
}

fn clamp_recent_commerce_orders_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(limit) if limit > 0 => limit.min(100),
        _ => 24,
    }
}

async fn list_recent_commerce_orders_handler(
    _claims: AuthenticatedAdminClaims,
    Query(query): Query<RecentCommerceOrdersQuery>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommerceOrderRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_recent_commerce_orders(clamp_recent_commerce_orders_limit(query.limit))
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load recent commerce orders: {error}"),
            )
        })
}

async fn list_commerce_payment_events_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CommercePaymentEventRecord>>, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .map(Json)
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce payment events for order {order_id}: {error}"),
            )
        })
}

async fn get_commerce_order_audit_handler(
    _claims: AuthenticatedAdminClaims,
    Path(order_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<CommerceOrderAuditRecord>, (StatusCode, Json<ErrorResponse>)> {
    let order = state
        .store
        .list_commerce_orders()
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce order {order_id}: {error}"),
            )
        })?
        .into_iter()
        .find(|order| order.order_id == order_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("commerce order {order_id} not found"),
            )
        })?;

    let mut payment_events = state
        .store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .map_err(|error| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load commerce payment events for order {order_id}: {error}"),
            )
        })?;
    payment_events.sort_by(|left, right| {
        right
            .processed_at_ms
            .unwrap_or(right.received_at_ms)
            .cmp(&left.processed_at_ms.unwrap_or(left.received_at_ms))
            .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
    });

    let coupon_reservation = match order.coupon_reservation_id.as_deref() {
        Some(coupon_reservation_id) => state
            .store
            .find_coupon_reservation_record(coupon_reservation_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon reservation {coupon_reservation_id} for order {order_id}: {error}"
                    ),
                )
            })?,
        None => None,
    };

    let coupon_redemption = match order.coupon_redemption_id.as_deref() {
        Some(coupon_redemption_id) => state
            .store
            .find_coupon_redemption_record(coupon_redemption_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon redemption {coupon_redemption_id} for order {order_id}: {error}"
                    ),
                )
            })?,
        None => None,
    };

    let mut coupon_rollbacks = match coupon_redemption.as_ref() {
        Some(redemption) => state
            .store
            .list_coupon_rollback_records()
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon rollback evidence for order {order_id}: {error}"
                    ),
                )
            })?
            .into_iter()
            .filter(|rollback| rollback.coupon_redemption_id == redemption.coupon_redemption_id)
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };
    coupon_rollbacks.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| right.coupon_rollback_id.cmp(&left.coupon_rollback_id))
    });

    let coupon_code = if let Some(redemption) = coupon_redemption.as_ref() {
        state
            .store
            .find_coupon_code_record(&redemption.coupon_code_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon code {} for order {order_id}: {error}",
                        redemption.coupon_code_id
                    ),
                )
            })?
    } else if let Some(reservation) = coupon_reservation.as_ref() {
        state
            .store
            .find_coupon_code_record(&reservation.coupon_code_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon code {} for order {order_id}: {error}",
                        reservation.coupon_code_id
                    ),
                )
            })?
    } else if let Some(applied_coupon_code) = order.applied_coupon_code.as_deref() {
        state
            .store
            .find_coupon_code_record_by_value(applied_coupon_code)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon code {applied_coupon_code} for order {order_id}: {error}"
                    ),
                )
            })?
    } else {
        None
    };

    let coupon_template_id = coupon_redemption
        .as_ref()
        .map(|redemption| redemption.coupon_template_id.as_str())
        .or_else(|| {
            coupon_code
                .as_ref()
                .map(|code| code.coupon_template_id.as_str())
        });
    let coupon_template = match coupon_template_id {
        Some(coupon_template_id) => state
            .store
            .find_coupon_template_record(coupon_template_id)
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load coupon template {coupon_template_id} for order {order_id}: {error}"
                    ),
                )
            })?,
        None => None,
    };

    let marketing_campaign = match order.marketing_campaign_id.as_deref() {
        Some(marketing_campaign_id) => state
            .store
            .list_marketing_campaign_records()
            .await
            .map_err(|error| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "failed to load marketing campaign evidence for order {order_id}: {error}"
                    ),
                )
            })?
            .into_iter()
            .find(|record| record.marketing_campaign_id == marketing_campaign_id),
        None => None,
    };

    Ok(Json(CommerceOrderAuditRecord {
        order,
        payment_events,
        coupon_reservation,
        coupon_redemption,
        coupon_rollbacks,
        coupon_code,
        coupon_template,
        marketing_campaign,
    }))
}

async fn load_admin_async_job_or_404(
    state: &AdminApiState,
    job_id: &str,
) -> Result<AsyncJobRecord, StatusCode> {
    find_async_job(state.store.as_ref(), job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)
}

async fn list_async_jobs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobRecord>>, StatusCode> {
    list_async_jobs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_async_job_attempts_handler(
    _claims: AuthenticatedAdminClaims,
    Path(job_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobAttemptRecord>>, StatusCode> {
    let _job = load_admin_async_job_or_404(&state, &job_id).await?;
    list_async_job_attempts(state.store.as_ref(), &job_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_async_job_assets_handler(
    _claims: AuthenticatedAdminClaims,
    Path(job_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobAssetRecord>>, StatusCode> {
    let _job = load_admin_async_job_or_404(&state, &job_id).await?;
    list_async_job_assets(state.store.as_ref(), &job_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_async_job_callbacks_handler(
    _claims: AuthenticatedAdminClaims,
    Path(job_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AsyncJobCallbackRecord>>, StatusCode> {
    let _job = load_admin_async_job_or_404(&state, &job_id).await?;
    list_async_job_callbacks(state.store.as_ref(), &job_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn synchronize_canonical_pricing_lifecycle_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<PricingLifecycleSynchronizationReport>, (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let report =
        synchronize_due_pricing_plan_lifecycle_with_report(commercial_billing.as_ref(), now_ms)
            .await
            .map_err(commercial_billing_error_response)?;
    Ok(Json(report))
}

async fn list_canonical_pricing_plans_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PricingPlanRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    plans.sort_by_key(|plan| plan.pricing_plan_id);
    Ok(Json(plans))
}

fn build_canonical_pricing_plan_record(
    pricing_plan_id: u64,
    request: &CreateCommercialPricingPlanRequest,
    created_at_ms: u64,
    updated_at_ms: u64,
) -> Result<PricingPlanRecord, (StatusCode, Json<ErrorResponse>)> {
    let plan_code = request.plan_code.trim();
    let display_name = request.display_name.trim();
    let status = request.status.trim();

    if plan_code.is_empty()
        || display_name.is_empty()
        || status.is_empty()
        || request.plan_version == 0
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "pricing plan requires non-empty code, display name, status, and plan version",
        ));
    }

    if let Some(effective_to_ms) = request.effective_to_ms {
        if effective_to_ms < request.effective_from_ms {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "pricing plan effective_to_ms must be greater than or equal to effective_from_ms",
            ));
        }
    }

    Ok(PricingPlanRecord::new(
        pricing_plan_id,
        request.tenant_id,
        request.organization_id,
        plan_code.to_owned(),
        request.plan_version,
    )
    .with_display_name(display_name.to_owned())
    .with_currency_code(request.currency_code.trim())
    .with_credit_unit_code(request.credit_unit_code.trim())
    .with_status(status.to_owned())
    .with_effective_from_ms(request.effective_from_ms)
    .with_effective_to_ms(request.effective_to_ms)
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

fn resolve_cloned_pricing_plan_version(
    source_plan: &PricingPlanRecord,
    plans: &[PricingPlanRecord],
    requested_version: Option<u64>,
) -> Result<u64, (StatusCode, Json<ErrorResponse>)> {
    let plan_version = requested_version.unwrap_or_else(|| {
        plans
            .iter()
            .filter(|plan| {
                plan.tenant_id == source_plan.tenant_id
                    && plan.organization_id == source_plan.organization_id
                    && plan.plan_code == source_plan.plan_code
            })
            .map(|plan| plan.plan_version)
            .max()
            .unwrap_or(source_plan.plan_version)
            + 1
    });

    if plan_version == 0 {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "cloned pricing plan requires a positive plan version",
        ));
    }

    let version_exists = plans.iter().any(|plan| {
        plan.pricing_plan_id != source_plan.pricing_plan_id
            && plan.tenant_id == source_plan.tenant_id
            && plan.organization_id == source_plan.organization_id
            && plan.plan_code == source_plan.plan_code
            && plan.plan_version == plan_version
    });
    if version_exists {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "pricing plan {} version {} already exists",
                source_plan.plan_code, plan_version
            ),
        ));
    }

    Ok(plan_version)
}

fn resolve_cloned_pricing_plan_display_name(
    source_plan: &PricingPlanRecord,
    requested_display_name: Option<String>,
    plan_version: u64,
) -> String {
    normalize_optional_admin_text(requested_display_name).unwrap_or_else(|| {
        let base_name = if source_plan.display_name.trim().is_empty() {
            source_plan.plan_code.as_str()
        } else {
            source_plan.display_name.as_str()
        };
        format!("{base_name} v{plan_version}")
    })
}

fn build_pricing_plan_with_status(
    plan: &PricingPlanRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingPlanRecord {
    PricingPlanRecord::new(
        plan.pricing_plan_id,
        plan.tenant_id,
        plan.organization_id,
        plan.plan_code.clone(),
        plan.plan_version,
    )
    .with_display_name(plan.display_name.clone())
    .with_currency_code(plan.currency_code.clone())
    .with_credit_unit_code(plan.credit_unit_code.clone())
    .with_status(status.to_owned())
    .with_effective_from_ms(plan.effective_from_ms)
    .with_effective_to_ms(plan.effective_to_ms)
    .with_created_at_ms(plan.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

fn build_pricing_rate_with_status(
    rate: &PricingRateRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingRateRecord {
    PricingRateRecord::new(
        rate.pricing_rate_id,
        rate.tenant_id,
        rate.organization_id,
        rate.pricing_plan_id,
        rate.metric_code.clone(),
    )
    .with_capability_code(rate.capability_code.clone())
    .with_model_code(rate.model_code.clone())
    .with_provider_code(rate.provider_code.clone())
    .with_charge_unit(rate.charge_unit.clone())
    .with_pricing_method(rate.pricing_method.clone())
    .with_quantity_step(rate.quantity_step)
    .with_unit_price(rate.unit_price)
    .with_display_price_unit(rate.display_price_unit.clone())
    .with_minimum_billable_quantity(rate.minimum_billable_quantity)
    .with_minimum_charge(rate.minimum_charge)
    .with_rounding_increment(rate.rounding_increment)
    .with_rounding_mode(rate.rounding_mode.clone())
    .with_included_quantity(rate.included_quantity)
    .with_priority(rate.priority)
    .with_notes(rate.notes.clone())
    .with_status(status.to_owned())
    .with_created_at_ms(rate.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

async fn create_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let now_ms = unix_timestamp_ms();
    let pricing_plan = build_canonical_pricing_plan_record(
        next_admin_pricing_record_id(now_ms),
        &request,
        now_ms,
        now_ms,
    )?;

    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plan = commercial_billing
        .insert_pricing_plan_record(&pricing_plan)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::CREATED, Json(plan)))
}

async fn update_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let existing_plan = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    let pricing_plan = build_canonical_pricing_plan_record(
        pricing_plan_id,
        &request,
        existing_plan.created_at_ms,
        unix_timestamp_ms(),
    )?;
    let plan = commercial_billing
        .insert_pricing_plan_record(&pricing_plan)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::OK, Json(plan)))
}

async fn clone_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CloneCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let source_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    let cloned_plan_version =
        resolve_cloned_pricing_plan_version(&source_plan, &plans, request.plan_version)?;
    let cloned_status = {
        let status = request.status.trim();
        if status.is_empty() {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "cloned pricing plan requires a non-empty status",
            ));
        }
        status.to_owned()
    };
    let cloned_display_name = resolve_cloned_pricing_plan_display_name(
        &source_plan,
        request.display_name,
        cloned_plan_version,
    );
    let now_ms = unix_timestamp_ms();
    let cloned_plan = PricingPlanRecord::new(
        next_admin_pricing_record_id(now_ms),
        source_plan.tenant_id,
        source_plan.organization_id,
        source_plan.plan_code.clone(),
        cloned_plan_version,
    )
    .with_display_name(cloned_display_name)
    .with_currency_code(source_plan.currency_code.clone())
    .with_credit_unit_code(source_plan.credit_unit_code.clone())
    .with_status(cloned_status.clone())
    .with_effective_from_ms(source_plan.effective_from_ms)
    .with_effective_to_ms(source_plan.effective_to_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    let inserted_plan = commercial_billing
        .insert_pricing_plan_record(&cloned_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    let source_rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    for source_rate in source_rates
        .into_iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let cloned_rate = PricingRateRecord::new(
            next_admin_pricing_record_id(now_ms),
            source_rate.tenant_id,
            source_rate.organization_id,
            inserted_plan.pricing_plan_id,
            source_rate.metric_code.clone(),
        )
        .with_capability_code(source_rate.capability_code.clone())
        .with_model_code(source_rate.model_code.clone())
        .with_provider_code(source_rate.provider_code.clone())
        .with_charge_unit(source_rate.charge_unit.clone())
        .with_pricing_method(source_rate.pricing_method.clone())
        .with_quantity_step(source_rate.quantity_step)
        .with_unit_price(source_rate.unit_price)
        .with_display_price_unit(source_rate.display_price_unit.clone())
        .with_minimum_billable_quantity(source_rate.minimum_billable_quantity)
        .with_minimum_charge(source_rate.minimum_charge)
        .with_rounding_increment(source_rate.rounding_increment)
        .with_rounding_mode(source_rate.rounding_mode.clone())
        .with_included_quantity(source_rate.included_quantity)
        .with_priority(source_rate.priority)
        .with_notes(source_rate.notes.clone())
        .with_status(cloned_status.clone())
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms);
        commercial_billing
            .insert_pricing_rate_record(&cloned_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::CREATED, Json(inserted_plan)))
}

async fn publish_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(_request): Json<PublishCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let target_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    let rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !rates
        .iter()
        .any(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be published without rates"),
        ));
    }

    let now_ms = unix_timestamp_ms();
    if target_plan.effective_from_ms > now_ms {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be published before effective_from_ms"),
        ));
    }

    let active_sibling_plan_ids = plans
        .iter()
        .filter(|plan| {
            plan.pricing_plan_id != pricing_plan_id
                && plan.tenant_id == target_plan.tenant_id
                && plan.organization_id == target_plan.organization_id
                && plan.plan_code == target_plan.plan_code
                && plan.status == "active"
        })
        .map(|plan| plan.pricing_plan_id)
        .collect::<Vec<_>>();

    let published_plan = build_pricing_plan_with_status(&target_plan, "active", now_ms);
    let published_plan = commercial_billing
        .insert_pricing_plan_record(&published_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for archived_plan in plans.iter().filter(|plan| {
        active_sibling_plan_ids
            .iter()
            .any(|sibling_id| *sibling_id == plan.pricing_plan_id)
    }) {
        let archived_plan = build_pricing_plan_with_status(archived_plan, "archived", now_ms);
        commercial_billing
            .insert_pricing_plan_record(&archived_plan)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in rates
        .iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let published_rate = build_pricing_rate_with_status(rate, "active", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&published_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    for rate in rates.iter().filter(|rate| {
        active_sibling_plan_ids
            .iter()
            .any(|sibling_id| *sibling_id == rate.pricing_plan_id)
    }) {
        let archived_rate = build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&archived_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(published_plan)))
}

async fn schedule_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(_request): Json<ScheduleCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let target_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;

    if target_plan.status == "archived" {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be scheduled from archived status"),
        ));
    }

    let rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !rates
        .iter()
        .any(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {pricing_plan_id} cannot be scheduled without rates"),
        ));
    }

    let now_ms = unix_timestamp_ms();
    if target_plan.effective_from_ms <= now_ms {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!(
                "pricing plan {pricing_plan_id} can only be scheduled for a future effective_from_ms"
            ),
        ));
    }

    let scheduled_plan = build_pricing_plan_with_status(&target_plan, "planned", now_ms);
    let scheduled_plan = commercial_billing
        .insert_pricing_plan_record(&scheduled_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    for rate in rates
        .iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let scheduled_rate = build_pricing_rate_with_status(rate, "planned", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&scheduled_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(scheduled_plan)))
}

async fn retire_canonical_pricing_plan_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_plan_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(_request): Json<RetireCommercialPricingPlanRequest>,
) -> Result<(StatusCode, Json<PricingPlanRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    let target_plan = plans
        .iter()
        .find(|plan| plan.pricing_plan_id == pricing_plan_id)
        .cloned()
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing plan {pricing_plan_id} does not exist"),
            )
        })?;
    let now_ms = unix_timestamp_ms();
    let retired_plan = build_pricing_plan_with_status(&target_plan, "archived", now_ms);
    let retired_plan = commercial_billing
        .insert_pricing_plan_record(&retired_plan)
        .await
        .map_err(commercial_billing_error_response)?;

    let rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    for rate in rates
        .iter()
        .filter(|rate| rate.pricing_plan_id == pricing_plan_id)
    {
        let retired_rate = build_pricing_rate_with_status(rate, "archived", now_ms);
        commercial_billing
            .insert_pricing_rate_record(&retired_rate)
            .await
            .map_err(commercial_billing_error_response)?;
    }

    Ok((StatusCode::OK, Json(retired_plan)))
}

async fn list_canonical_pricing_rates_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PricingRateRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), unix_timestamp_ms())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?;
    rates.sort_by_key(|rate| rate.pricing_rate_id);
    Ok(Json(rates))
}

fn build_canonical_pricing_rate_record(
    pricing_rate_id: u64,
    request: &CreateCommercialPricingRateRequest,
    created_at_ms: u64,
    updated_at_ms: u64,
) -> Result<PricingRateRecord, (StatusCode, Json<ErrorResponse>)> {
    let metric_code = request.metric_code.trim();
    let charge_unit = request.charge_unit.trim();
    let pricing_method = request.pricing_method.trim();
    let display_price_unit = request.display_price_unit.trim();
    let rounding_mode = request.rounding_mode.trim();
    let status = request.status.trim();

    let invalid = metric_code.is_empty()
        || charge_unit.is_empty()
        || pricing_method.is_empty()
        || display_price_unit.is_empty()
        || rounding_mode.is_empty()
        || status.is_empty()
        || request.quantity_step <= 0.0
        || request.unit_price < 0.0
        || request.minimum_billable_quantity < 0.0
        || request.minimum_charge < 0.0
        || request.rounding_increment <= 0.0
        || request.included_quantity < 0.0;

    if invalid {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "pricing rate requires metric, charge unit, pricing method, display unit, positive quantity and rounding step, and non-negative commercial amounts",
        ));
    }

    Ok(PricingRateRecord::new(
        pricing_rate_id,
        request.tenant_id,
        request.organization_id,
        request.pricing_plan_id,
        metric_code.to_owned(),
    )
    .with_capability_code(normalize_optional_admin_text(
        request.capability_code.clone(),
    ))
    .with_model_code(normalize_optional_admin_text(request.model_code.clone()))
    .with_provider_code(normalize_optional_admin_text(request.provider_code.clone()))
    .with_charge_unit(charge_unit.to_owned())
    .with_pricing_method(pricing_method.to_owned())
    .with_quantity_step(request.quantity_step)
    .with_unit_price(request.unit_price)
    .with_display_price_unit(display_price_unit.to_owned())
    .with_minimum_billable_quantity(request.minimum_billable_quantity)
    .with_minimum_charge(request.minimum_charge)
    .with_rounding_increment(request.rounding_increment)
    .with_rounding_mode(rounding_mode.to_owned())
    .with_included_quantity(request.included_quantity)
    .with_priority(request.priority)
    .with_notes(normalize_optional_admin_text(request.notes.clone()))
    .with_status(status.to_owned())
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(updated_at_ms))
}

async fn create_canonical_pricing_rate_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingRateRequest>,
) -> Result<(StatusCode, Json<PricingRateRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let pricing_plan_exists = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .any(|plan| plan.pricing_plan_id == request.pricing_plan_id);
    if !pricing_plan_exists {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {} does not exist", request.pricing_plan_id),
        ));
    }

    let now_ms = unix_timestamp_ms();
    let pricing_rate = build_canonical_pricing_rate_record(
        next_admin_pricing_record_id(now_ms),
        &request,
        now_ms,
        now_ms,
    )?;
    let rate = commercial_billing
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::CREATED, Json(rate)))
}

async fn update_canonical_pricing_rate_handler(
    _claims: AuthenticatedAdminClaims,
    Path(pricing_rate_id): Path<u64>,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCommercialPricingRateRequest>,
) -> Result<(StatusCode, Json<PricingRateRecord>), (StatusCode, Json<ErrorResponse>)> {
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?;
    if !plans
        .iter()
        .any(|plan| plan.pricing_plan_id == request.pricing_plan_id)
    {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            format!("pricing plan {} does not exist", request.pricing_plan_id),
        ));
    }

    let existing_rate = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .find(|rate| rate.pricing_rate_id == pricing_rate_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                format!("pricing rate {pricing_rate_id} does not exist"),
            )
        })?;

    let pricing_rate = build_canonical_pricing_rate_record(
        pricing_rate_id,
        &request,
        existing_rate.created_at_ms,
        unix_timestamp_ms(),
    )?;
    let rate = commercial_billing
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((StatusCode::OK, Json(rate)))
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
