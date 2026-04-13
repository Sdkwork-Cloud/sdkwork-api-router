use super::*;
use serde::Deserialize;
use utoipa::ToSchema;

pub(crate) async fn list_channel_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ChannelModelRecord>>, StatusCode> {
    list_channel_models(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_channel_model_handler(
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
    .map_err(|error| super::catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_channel_model_handler(
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

pub(crate) async fn list_provider_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderModelRecord>>, StatusCode> {
    list_provider_models(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_provider_accounts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderAccountRecord>>, StatusCode> {
    list_catalog_provider_accounts(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_provider_account_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderAccountRequest>,
) -> Result<(StatusCode, Json<ProviderAccountRecord>), StatusCode> {
    let record = persist_provider_account(
        state.store.as_ref(),
        &request.provider_account_id,
        &request.provider_id,
        &request.display_name,
        &request.account_kind,
        &request.owner_scope,
        request.owner_tenant_id.as_deref(),
        &request.execution_instance_id,
        request.base_url_override.as_deref(),
        request.region.as_deref(),
        request.priority,
        request.weight,
        request.enabled,
        &request.routing_tags,
        request.health_score_hint,
        request.latency_ms_hint,
        request.cost_hint,
        request.success_rate_hint,
        request.throughput_hint,
        request.max_concurrency,
        request.daily_budget,
        request.notes.as_deref(),
    )
    .await
    .map_err(|error| super::catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_provider_account_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(provider_account_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_provider_account(state.store.as_ref(), &provider_account_id)
        .await
        .map_err(|error| super::catalog_write_error_status(&error))?;
    if deleted {
        invalidate_catalog_cache_after_mutation().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(crate) async fn create_provider_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderModelRequestWithProvider>,
) -> Result<(StatusCode, Json<ProviderModelRecord>), StatusCode> {
    let record = persist_provider_model_with_metadata(
        state.store.as_ref(),
        &request.proxy_provider_id,
        &request.channel_id,
        &request.model_id,
        request.provider_model_id.as_deref(),
        request.provider_model_family.as_deref(),
        (!request.capabilities.is_empty()).then_some(request.capabilities.as_slice()),
        request.streaming,
        request.context_window,
        request.max_output_tokens,
        request.supports_prompt_caching,
        request.supports_reasoning_usage,
        request.supports_tool_usage_metrics,
        request.is_default_route,
        request.is_active,
    )
    .await
    .map_err(|error| super::catalog_write_error_status(&error))?;
    invalidate_catalog_cache_after_mutation().await;
    Ok((StatusCode::CREATED, Json(record)))
}

pub(crate) async fn delete_provider_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((proxy_provider_id, channel_id, model_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_provider_model(
        state.store.as_ref(),
        &proxy_provider_id,
        &channel_id,
        &model_id,
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

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub(crate) struct CreateProviderModelRequestWithProvider {
    pub(crate) proxy_provider_id: String,
    pub(crate) channel_id: String,
    pub(crate) model_id: String,
    pub(crate) provider_model_id: Option<String>,
    pub(crate) provider_model_family: Option<String>,
    pub(crate) capabilities: Vec<ModelCapability>,
    pub(crate) streaming: Option<bool>,
    pub(crate) context_window: Option<u64>,
    pub(crate) max_output_tokens: Option<u64>,
    pub(crate) supports_prompt_caching: bool,
    pub(crate) supports_reasoning_usage: bool,
    pub(crate) supports_tool_usage_metrics: bool,
    pub(crate) is_default_route: bool,
    pub(crate) is_active: bool,
}

impl From<(String, CreateProviderModelRequest)> for CreateProviderModelRequestWithProvider {
    fn from((proxy_provider_id, request): (String, CreateProviderModelRequest)) -> Self {
        Self {
            proxy_provider_id,
            channel_id: request.channel_id,
            model_id: request.model_id,
            provider_model_id: request.provider_model_id,
            provider_model_family: request.provider_model_family,
            capabilities: request.capabilities,
            streaming: request.streaming,
            context_window: request.context_window,
            max_output_tokens: request.max_output_tokens,
            supports_prompt_caching: request.supports_prompt_caching,
            supports_reasoning_usage: request.supports_reasoning_usage,
            supports_tool_usage_metrics: request.supports_tool_usage_metrics,
            is_default_route: request.is_default_route,
            is_active: request.is_active,
        }
    }
}

pub(super) async fn sync_provider_models_from_request(
    store: &dyn AdminStore,
    proxy_provider_id: &str,
    supported_models: &[CreateProviderModelRequest],
) -> anyhow::Result<()> {
    let mut requested_keys = std::collections::HashSet::new();
    for record in supported_models {
        requested_keys.insert(format!("{}::{}", record.channel_id, record.model_id));
        persist_provider_model_with_metadata(
            store,
            proxy_provider_id,
            &record.channel_id,
            &record.model_id,
            record.provider_model_id.as_deref(),
            record.provider_model_family.as_deref(),
            (!record.capabilities.is_empty()).then_some(record.capabilities.as_slice()),
            record.streaming,
            record.context_window,
            record.max_output_tokens,
            record.supports_prompt_caching,
            record.supports_reasoning_usage,
            record.supports_tool_usage_metrics,
            record.is_default_route,
            record.is_active,
        )
        .await?;
    }
    for existing in store
        .list_provider_models_for_provider(proxy_provider_id)
        .await?
    {
        if !requested_keys.contains(&format!("{}::{}", existing.channel_id, existing.model_id)) {
            store
                .delete_provider_model(
                    &existing.proxy_provider_id,
                    &existing.channel_id,
                    &existing.model_id,
                )
                .await?;
        }
    }
    Ok(())
}
