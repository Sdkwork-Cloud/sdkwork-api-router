use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_billing::{list_ledger_entries, list_quota_policies, summarize_billing_snapshot};
use sdkwork_api_app_identity::{
    change_portal_password, create_portal_api_key, list_portal_api_keys, load_portal_user_profile,
    load_portal_workspace_summary, login_portal_user, register_portal_user, verify_portal_jwt,
    CreatedGatewayApiKey, PortalAuthSession, PortalClaims, PortalIdentityError,
    PortalWorkspaceSummary,
};
use sdkwork_api_app_usage::{list_usage_records, summarize_usage_records};
use sdkwork_api_domain_billing::{LedgerEntry, ProjectBillingSummary};
use sdkwork_api_domain_identity::{GatewayApiKeyRecord, PortalUserProfile};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const DEFAULT_PORTAL_JWT_SIGNING_SECRET: &str = "local-dev-portal-jwt-secret";

pub struct PortalApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    store: Arc<dyn AdminStore>,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for PortalApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            store: self.live_store.snapshot(),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
        }
    }
}

impl PortalApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
    }

    pub fn with_jwt_secret(pool: SqlitePool, jwt_signing_secret: impl Into<String>) -> Self {
        Self::with_store_and_jwt_secret(Arc::new(SqliteAdminStore::new(pool)), jwt_signing_secret)
    }

    pub fn with_store_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_live_store_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            live_store,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
        }
    }
}

#[derive(Clone, Debug)]
struct AuthenticatedPortalClaims(PortalClaims);

impl AuthenticatedPortalClaims {
    fn claims(&self) -> &PortalClaims {
        &self.0
    }
}

impl FromRequestParts<PortalApiState> for AuthenticatedPortalClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &PortalApiState,
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

        verify_portal_jwt(token, &state.jwt_signing_secret)
            .map(Self)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    environment: String,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
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

#[derive(Debug, Serialize)]
struct PortalDashboardSummary {
    workspace: PortalWorkspaceSummary,
    usage_summary: UsageSummary,
    billing_summary: ProjectBillingSummary,
    recent_requests: Vec<UsageRecord>,
    api_key_count: usize,
}

pub fn portal_router() -> Router {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    Router::new()
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
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(|| async { "register" }))
        .route("/portal/auth/login", post(|| async { "login" }))
        .route("/portal/auth/me", get(|| async { "me" }))
        .route(
            "/portal/auth/change-password",
            post(|| async { "change-password" }),
        )
        .route("/portal/dashboard", get(|| async { "dashboard" }))
        .route("/portal/workspace", get(|| async { "workspace" }))
        .route("/portal/api-keys", get(|| async { "api-keys" }))
        .route("/portal/usage/records", get(|| async { "usage-records" }))
        .route("/portal/usage/summary", get(|| async { "usage-summary" }))
        .route("/portal/billing/summary", get(|| async { "billing-summary" }))
        .route("/portal/billing/ledger", get(|| async { "billing-ledger" }))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
}

pub fn portal_router_with_pool(pool: SqlitePool) -> Router {
    portal_router_with_pool_and_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_pool_and_jwt_secret(
    pool: SqlitePool,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    portal_router_with_store_and_jwt_secret(
        Arc::new(SqliteAdminStore::new(pool)),
        jwt_signing_secret,
    )
}

pub fn portal_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    portal_router_with_store_and_jwt_secret(store, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_store_and_jwt_secret(
    store: Arc<dyn AdminStore>,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    portal_router_with_state(PortalApiState::with_store_and_jwt_secret(
        store,
        jwt_signing_secret,
    ))
}

pub fn portal_router_with_state(state: PortalApiState) -> Router {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    Router::new()
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
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(register_handler))
        .route("/portal/auth/login", post(login_handler))
        .route("/portal/auth/me", get(me_handler))
        .route(
            "/portal/auth/change-password",
            post(change_password_handler),
        )
        .route("/portal/dashboard", get(dashboard_handler))
        .route("/portal/workspace", get(workspace_handler))
        .route(
            "/portal/api-keys",
            get(list_api_keys_handler).post(create_api_key_handler),
        )
        .route("/portal/usage/records", get(list_usage_records_handler))
        .route("/portal/usage/summary", get(usage_summary_handler))
        .route("/portal/billing/summary", get(billing_summary_handler))
        .route("/portal/billing/ledger", get(list_billing_ledger_handler))
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

async fn register_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<PortalAuthSession>), (StatusCode, Json<ErrorResponse>)> {
    register_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &request.display_name,
        &state.jwt_signing_secret,
    )
    .await
    .map(|session| (StatusCode::CREATED, Json(session)))
    .map_err(portal_error_response)
}

async fn login_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<PortalAuthSession>, (StatusCode, Json<ErrorResponse>)> {
    login_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn me_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalUserProfile>, StatusCode> {
    load_portal_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn change_password_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_portal_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn workspace_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalWorkspaceSummary>, StatusCode> {
    load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
}

async fn list_api_keys_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

async fn create_api_key_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.environment,
    )
    .await
    .map(|created| (StatusCode::CREATED, Json(created)))
    .map_err(portal_error_response)
}

async fn dashboard_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalDashboardSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records = load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    let usage_summary = summarize_usage_records(&usage_records);
    let billing_summary =
        load_project_billing_summary(state.store.as_ref(), &workspace.project.id).await?;
    let api_key_count = list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    let recent_requests = usage_records.iter().take(10).cloned().collect();

    Ok(Json(PortalDashboardSummary {
        workspace,
        usage_summary,
        billing_summary,
        recent_requests,
        api_key_count,
    }))
}

async fn list_usage_records_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_usage_records(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn usage_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records = load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    Ok(Json(summarize_usage_records(&usage_records)))
}

async fn billing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectBillingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_summary(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn list_billing_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let ledger = list_ledger_entries(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|entry| entry.project_id == workspace.project.id)
        .collect();
    Ok(Json(ledger))
}

fn portal_error_response(error: PortalIdentityError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        PortalIdentityError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        PortalIdentityError::DuplicateEmail => StatusCode::CONFLICT,
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

async fn load_workspace_for_user(
    store: &dyn AdminStore,
    user_id: &str,
) -> Result<PortalWorkspaceSummary, StatusCode> {
    load_portal_workspace_summary(store, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn load_project_usage_records(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<Vec<UsageRecord>, StatusCode> {
    let mut usage_records = list_usage_records(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|record| record.project_id == project_id)
        .collect::<Vec<_>>();
    usage_records.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(usage_records)
}

async fn load_project_billing_summary(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectBillingSummary, StatusCode> {
    let ledger = list_ledger_entries(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|entry| entry.project_id == project_id)
        .collect::<Vec<_>>();
    let policies = list_quota_policies(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|policy| policy.project_id == project_id)
        .collect::<Vec<_>>();
    let billing = summarize_billing_snapshot(&ledger, &policies);

    Ok(billing
        .projects
        .into_iter()
        .next()
        .unwrap_or_else(|| ProjectBillingSummary::new(project_id.to_owned())))
}
