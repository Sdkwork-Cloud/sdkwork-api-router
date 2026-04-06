use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sdkwork_api_app_credential::{CredentialSecretManager, resolve_provider_secret_with_manager};
use sdkwork_api_app_routing::{RouteSelectionContext, select_route_with_store_context};
use sdkwork_api_cache_core::{
    CacheStore, CacheTag, DistributedLockStore, cache_get_or_insert_with,
};
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_contract_openai::assistants::{
    AssistantObject, CreateAssistantRequest, DeleteAssistantResponse, ListAssistantsResponse,
    UpdateAssistantRequest,
};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest, ListVoicesResponse, SpeechResponse, TranscriptionObject,
    TranslationObject, VoiceConsentObject, VoiceObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::{
    ChatCompletionMessageObject, ChatCompletionResponse, CreateChatCompletionRequest,
    DeleteChatCompletionResponse, ListChatCompletionMessagesResponse, ListChatCompletionsResponse,
    UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::containers::{
    ContainerFileObject, ContainerObject, CreateContainerFileRequest, CreateContainerRequest,
    DeleteContainerFileResponse, DeleteContainerResponse, ListContainerFilesResponse,
    ListContainersResponse,
};
use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, CreateConversationItemsRequest,
    CreateConversationRequest, DeleteConversationItemResponse, DeleteConversationResponse,
    ListConversationItemsResponse, ListConversationsResponse, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, DeleteEvalResponse, DeleteEvalRunResponse, EvalObject,
    EvalRunObject, EvalRunOutputItemObject, ListEvalRunOutputItemsResponse, ListEvalRunsResponse,
    ListEvalsResponse, UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
    DeleteFineTuningCheckpointPermissionResponse, FineTuningCheckpointPermissionObject,
    FineTuningJobCheckpointObject, FineTuningJobEventObject, FineTuningJobObject,
    ListFineTuningCheckpointPermissionsResponse, ListFineTuningJobCheckpointsResponse,
    ListFineTuningJobEventsResponse, ListFineTuningJobsResponse,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest, ImageObject,
    ImagesResponse,
};
use sdkwork_api_contract_openai::models::{DeleteModelResponse, ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::moderations::{
    CreateModerationRequest, ModerationCategoryScores, ModerationResponse, ModerationResult,
};
use sdkwork_api_contract_openai::music::{
    CreateMusicLyricsRequest, CreateMusicRequest, DeleteMusicResponse, MusicLyricsObject,
    MusicObject, MusicTracksResponse,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseCompactionObject,
    ResponseInputItemObject, ResponseInputTokensObject, ResponseObject,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, ListRunStepsResponse, ListRunsResponse, RunObject,
    RunStepObject, SubmitToolOutputsRunRequest, UpdateRunRequest,
};
use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, DeleteThreadMessageResponse,
    DeleteThreadResponse, ListThreadMessagesResponse, ThreadMessageObject, ThreadObject,
    UpdateThreadMessageRequest, UpdateThreadRequest,
};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest, UploadObject,
    UploadPartObject,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    DeleteVectorStoreFileResponse, DeleteVectorStoreResponse, ListVectorStoreFilesResponse,
    ListVectorStoresResponse, SearchVectorStoreRequest, SearchVectorStoreResponse,
    UpdateVectorStoreRequest, VectorStoreFileBatchObject, VectorStoreFileObject, VectorStoreObject,
};
use sdkwork_api_contract_openai::videos::{
    CreateVideoCharacterRequest, CreateVideoRequest, DeleteVideoResponse, EditVideoRequest,
    ExtendVideoRequest, RemixVideoRequest, UpdateVideoCharacterRequest, VideoCharacterObject,
    VideoCharactersResponse, VideoObject, VideosResponse,
};
use sdkwork_api_contract_openai::webhooks::{
    CreateWebhookRequest, DeleteWebhookResponse, ListWebhooksResponse, UpdateWebhookRequest,
    WebhookObject,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingDecision, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy,
};
use sdkwork_api_extension_core::{
    ExtensionKind, ExtensionManifest, ExtensionModality, ExtensionProtocol, ExtensionRuntime,
};
use sdkwork_api_extension_host::{
    BuiltinProviderExtensionFactory, DiscoveredExtensionPackage, ExtensionDiscoveryPolicy,
    ExtensionHost, discover_extension_packages, ensure_connector_runtime_started,
    shutdown_all_connector_runtimes, shutdown_all_native_dynamic_runtimes,
    shutdown_connector_runtime, shutdown_connector_runtimes_for_extension,
    shutdown_native_dynamic_runtimes_for_extension, verify_discovered_extension_package_trust,
};
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_provider_core::{
    ProviderExecutionAdapter, ProviderHttpError, ProviderOutput, ProviderRequest,
    ProviderRequestOptions, ProviderRetryAfterSource, ProviderStreamOutput,
};
use sdkwork_api_provider_ollama::OllamaProviderAdapter;
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use sdkwork_api_provider_openrouter::OpenRouterProviderAdapter;
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, UNIX_EPOCH};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

pub const LOCAL_PROVIDER_ID: &str = "sdkwork.local";

tokio::task_local! {
    static REQUEST_ROUTING_REGION: Option<String>;
}

tokio::task_local! {
    static REQUEST_API_KEY_GROUP_ID: Option<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedExecutionUsageContext {
    pub provider_id: String,
    pub channel_id: Option<String>,
    pub api_key_group_id: Option<String>,
    pub applied_routing_profile_id: Option<String>,
    pub compiled_routing_snapshot_id: Option<String>,
    pub fallback_reason: Option<String>,
    pub latency_ms: Option<u64>,
    pub reference_amount: Option<f64>,
}

pub struct GatewayExecutionResult<T> {
    pub response: Option<T>,
    pub usage_context: Option<PlannedExecutionUsageContext>,
}

impl<T> GatewayExecutionResult<T> {
    fn new(response: Option<T>, usage_context: Option<PlannedExecutionUsageContext>) -> Self {
        Self {
            response,
            usage_context,
        }
    }
}

#[derive(Clone)]
struct CachedConfiguredExtensionHost {
    key: ConfiguredExtensionHostCacheKey,
    host: ExtensionHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfiguredExtensionHostReloadReport {
    pub discovered_package_count: usize,
    pub loadable_package_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfiguredExtensionHostReloadScope {
    All,
    Extension { extension_id: String },
    Instance { instance_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfiguredExtensionHostCacheKey {
    search_paths: Vec<PathBuf>,
    enable_connector_extensions: bool,
    enable_native_dynamic_extensions: bool,
    require_signed_connector_extensions: bool,
    require_signed_native_dynamic_extensions: bool,
    trusted_signers: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfiguredExtensionHostWatchState {
    key: ConfiguredExtensionHostCacheKey,
    fingerprint: Vec<String>,
}

impl From<&ExtensionDiscoveryPolicy> for ConfiguredExtensionHostCacheKey {
    fn from(policy: &ExtensionDiscoveryPolicy) -> Self {
        let mut trusted_signers = policy
            .trusted_signers
            .iter()
            .map(|(publisher, public_key)| (publisher.clone(), public_key.clone()))
            .collect::<Vec<_>>();
        trusted_signers.sort_unstable();

        Self {
            search_paths: policy.search_paths.clone(),
            enable_connector_extensions: policy.enable_connector_extensions,
            enable_native_dynamic_extensions: policy.enable_native_dynamic_extensions,
            require_signed_connector_extensions: policy.require_signed_connector_extensions,
            require_signed_native_dynamic_extensions: policy
                .require_signed_native_dynamic_extensions,
            trusted_signers,
        }
    }
}

static CONFIGURED_EXTENSION_HOST_CACHE: OnceLock<Mutex<Option<CachedConfiguredExtensionHost>>> =
    OnceLock::new();

struct BuiltConfiguredExtensionHost {
    host: ExtensionHost,
    discovered_package_count: usize,
    loadable_package_count: usize,
}

pub fn service_name() -> &'static str {
    "gateway-service"
}

pub fn list_models(_tenant_id: &str, _project_id: &str) -> Result<ListModelsResponse> {
    Ok(ListModelsResponse::new(vec![ModelObject::new(
        "gpt-4.1", "sdkwork",
    )]))
}

fn ensure_local_model_exists(model_id: &str) -> Result<()> {
    if model_id != "gpt-4.1" {
        bail!("model not found");
    }

    Ok(())
}

fn ensure_local_deletable_model_exists(model_id: &str) -> Result<()> {
    if model_id != "ft:gpt-4.1:sdkwork" {
        bail!("model not found");
    }

    Ok(())
}

pub fn get_model(_tenant_id: &str, _project_id: &str, model_id: &str) -> Result<ModelObject> {
    ensure_local_model_exists(model_id)?;
    Ok(ModelObject::new(model_id, "sdkwork"))
}

pub fn delete_model(
    _tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<DeleteModelResponse> {
    ensure_local_deletable_model_exists(model_id)?;
    Ok(DeleteModelResponse::deleted(model_id))
}

pub async fn list_models_from_store(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
) -> Result<ListModelsResponse> {
    let Some(cache_store) = capability_catalog_cache_store() else {
        let models = store.list_models().await?;
        return Ok(ListModelsResponse::new(
            models
                .into_iter()
                .map(|entry| ModelObject::new(entry.external_name, entry.provider_id))
                .collect(),
        ));
    };
    let cache_key = capability_catalog_list_cache_key(tenant_id, project_id);
    let payload = cache_get_or_insert_with(
        cache_store.as_ref(),
        CAPABILITY_CATALOG_CACHE_NAMESPACE,
        &cache_key,
        Some(CAPABILITY_CATALOG_CACHE_TTL_MS),
        &[CacheTag::new(CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS)],
        || async {
            let models = store.list_models().await?;
            let cached = CachedCapabilityCatalogList {
                models: models
                    .into_iter()
                    .map(|entry| CachedCapabilityCatalogModel {
                        id: entry.external_name,
                        owned_by: entry.provider_id,
                    })
                    .collect(),
            };
            Ok(serde_json::to_vec(&cached)?)
        },
    )
    .await?;
    let cached: CachedCapabilityCatalogList = serde_json::from_slice(&payload)
        .context("failed to decode capability catalog list cache payload")?;
    Ok(cached.into_response())
}

pub async fn get_model_from_store(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    model_id: &str,
) -> Result<Option<ModelObject>> {
    let Some(cache_store) = capability_catalog_cache_store() else {
        return Ok(store
            .find_model(model_id)
            .await?
            .map(|entry| ModelObject::new(entry.external_name, entry.provider_id)));
    };
    let cache_key = capability_catalog_model_cache_key(tenant_id, project_id, model_id);
    let payload = cache_get_or_insert_with(
        cache_store.as_ref(),
        CAPABILITY_CATALOG_CACHE_NAMESPACE,
        &cache_key,
        Some(CAPABILITY_CATALOG_CACHE_TTL_MS),
        &[
            CacheTag::new(CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS),
            CacheTag::new(format!("model:{model_id}")),
        ],
        || async {
            let cached =
                store
                    .find_model(model_id)
                    .await?
                    .map(|entry| CachedCapabilityCatalogModel {
                        id: entry.external_name,
                        owned_by: entry.provider_id,
                    });
            Ok(serde_json::to_vec(&cached)?)
        },
    )
    .await?;
    let cached: Option<CachedCapabilityCatalogModel> = serde_json::from_slice(&payload)
        .context("failed to decode capability catalog model cache payload")?;
    Ok(cached.map(CachedCapabilityCatalogModel::into_model_object))
}

pub async fn delete_model_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<Option<Value>> {
    let Some(model_entry) = store.find_model(model_id).await? else {
        return Ok(None);
    };

    if let Some(provider) = store.find_provider(&model_entry.provider_id).await? {
        if let Some(api_key) =
            resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
                .await?
        {
            let response = execute_json_provider_request_for_provider(
                store,
                &provider,
                &api_key,
                ProviderRequest::ModelsDelete(model_id),
            )
            .await?;

            if let Some(response) = response {
                let _ = store.delete_model(model_id).await?;
                invalidate_capability_catalog_cache().await;
                return Ok(Some(response));
            }
        }
    }

    if store.delete_model(model_id).await? {
        invalidate_capability_catalog_cache().await;
        return Ok(Some(serde_json::to_value(DeleteModelResponse::deleted(
            model_id,
        ))?));
    }

    Ok(None)
}

#[derive(Clone)]
struct ProviderExecutionTarget {
    provider_id: String,
    runtime_key: String,
    base_url: String,
    runtime: ExtensionRuntime,
    local_fallback: bool,
}

impl ProviderExecutionTarget {
    fn local() -> Self {
        Self {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            runtime_key: String::new(),
            base_url: String::new(),
            runtime: ExtensionRuntime::Builtin,
            local_fallback: true,
        }
    }

    fn upstream(
        provider_id: String,
        runtime_key: String,
        base_url: String,
        runtime: ExtensionRuntime,
    ) -> Self {
        Self {
            provider_id,
            runtime_key,
            base_url,
            runtime,
            local_fallback: false,
        }
    }
}

#[derive(Clone)]
struct ProviderExecutionDescriptor {
    provider_id: String,
    runtime_key: String,
    base_url: String,
    api_key: String,
    runtime: ExtensionRuntime,
    local_fallback: bool,
}

async fn build_extension_host_from_store(store: &dyn AdminStore) -> Result<ExtensionHost> {
    let mut host = configured_extension_host()?;

    let mut installations = store.list_extension_installations().await?;
    installations.sort_by(|left, right| left.installation_id.cmp(&right.installation_id));
    for installation in installations {
        match host.install(installation) {
            Ok(()) => {}
            Err(sdkwork_api_extension_host::ExtensionHostError::ManifestNotFound { .. }) => {}
            Err(error) => return Err(error.into()),
        }
    }

    let mut instances = store.list_extension_instances().await?;
    instances.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    for instance in instances {
        match host.mount_instance(instance) {
            Ok(()) => {}
            Err(sdkwork_api_extension_host::ExtensionHostError::InstallationNotFound {
                ..
            }) => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(host)
}

async fn provider_execution_target_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
) -> Result<ProviderExecutionTarget> {
    let host = build_extension_host_from_store(store).await?;

    match host.load_plan(&provider.id) {
        Ok(load_plan) => {
            if !load_plan.enabled {
                return Ok(ProviderExecutionTarget::local());
            }

            let resolved_base_url = load_plan
                .base_url
                .clone()
                .unwrap_or_else(|| provider.base_url.clone());
            if load_plan.runtime == ExtensionRuntime::Connector {
                ensure_connector_runtime_started(&load_plan, &resolved_base_url)
                    .map_err(anyhow::Error::new)?;
            }

            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                load_plan.extension_id,
                resolved_base_url,
                load_plan.runtime,
            ))
        }
        Err(sdkwork_api_extension_host::ExtensionHostError::InstanceNotFound { .. }) => {
            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                provider_runtime_key(provider).to_owned(),
                provider.base_url.clone(),
                ExtensionRuntime::Builtin,
            ))
        }
        Err(error) => Err(error.into()),
    }
}

async fn provider_execution_descriptor_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: String,
) -> Result<ProviderExecutionDescriptor> {
    let target = provider_execution_target_for_provider(store, provider).await?;
    Ok(ProviderExecutionDescriptor {
        provider_id: target.provider_id,
        runtime_key: target.runtime_key,
        base_url: target.base_url,
        api_key,
        runtime: target.runtime,
        local_fallback: target.local_fallback,
    })
}

pub const ROUTING_DECISION_CACHE_NAMESPACE: &str = "gateway_route_decisions";
const ROUTING_DECISION_CACHE_TTL_MS: u64 = 30_000;
static ROUTING_DECISION_CACHE_STORE: OnceLock<Reloadable<Arc<dyn CacheStore>>> = OnceLock::new();
static ROUTING_RECOVERY_PROBE_LOCK_STORE: OnceLock<Reloadable<Arc<dyn DistributedLockStore>>> =
    OnceLock::new();
static GATEWAY_PROVIDER_MAX_IN_FLIGHT_LIMIT: OnceLock<Reloadable<Option<usize>>> = OnceLock::new();
static GATEWAY_PROVIDER_IN_FLIGHT_COUNTERS: OnceLock<Mutex<HashMap<String, Arc<AtomicUsize>>>> =
    OnceLock::new();
pub const CAPABILITY_CATALOG_CACHE_NAMESPACE: &str = "gateway_capability_catalog";
const CAPABILITY_CATALOG_CACHE_TTL_MS: u64 = 30_000;
const CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS: &str = "models:all";
static CAPABILITY_CATALOG_CACHE_STORE: OnceLock<Reloadable<Option<Arc<dyn CacheStore>>>> =
    OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedCapabilityCatalogModel {
    id: String,
    owned_by: String,
}

impl CachedCapabilityCatalogModel {
    fn into_model_object(self) -> ModelObject {
        ModelObject::new(self.id, self.owned_by)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedCapabilityCatalogList {
    models: Vec<CachedCapabilityCatalogModel>,
}

impl CachedCapabilityCatalogList {
    fn into_response(self) -> ListModelsResponse {
        ListModelsResponse::new(
            self.models
                .into_iter()
                .map(CachedCapabilityCatalogModel::into_model_object)
                .collect(),
        )
    }
}

fn routing_decision_cache_store_handle() -> &'static Reloadable<Arc<dyn CacheStore>> {
    ROUTING_DECISION_CACHE_STORE.get_or_init(|| {
        Reloadable::new(Arc::new(MemoryCacheStore::default()) as Arc<dyn CacheStore>)
    })
}

fn routing_decision_cache_store() -> Arc<dyn CacheStore> {
    routing_decision_cache_store_handle().snapshot()
}

pub fn configure_route_decision_cache_store(cache_store: Arc<dyn CacheStore>) {
    routing_decision_cache_store_handle().replace(cache_store);
}

fn routing_recovery_probe_lock_store_handle() -> &'static Reloadable<Arc<dyn DistributedLockStore>>
{
    ROUTING_RECOVERY_PROBE_LOCK_STORE.get_or_init(|| {
        Reloadable::new(Arc::new(MemoryCacheStore::default()) as Arc<dyn DistributedLockStore>)
    })
}

fn routing_recovery_probe_lock_store() -> Arc<dyn DistributedLockStore> {
    routing_recovery_probe_lock_store_handle().snapshot()
}

pub fn configure_route_recovery_probe_lock_store(lock_store: Arc<dyn DistributedLockStore>) {
    routing_recovery_probe_lock_store_handle().replace(lock_store);
}

const GATEWAY_PROVIDER_MAX_IN_FLIGHT_ENV: &str = "SDKWORK_GATEWAY_PROVIDER_MAX_IN_FLIGHT";

fn gateway_provider_max_in_flight_limit_from_env(configured: Option<&str>) -> Option<usize> {
    configured
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|limit| *limit > 0)
}

fn gateway_provider_max_in_flight_limit_handle() -> &'static Reloadable<Option<usize>> {
    GATEWAY_PROVIDER_MAX_IN_FLIGHT_LIMIT.get_or_init(|| {
        Reloadable::new(gateway_provider_max_in_flight_limit_from_env(
            std::env::var(GATEWAY_PROVIDER_MAX_IN_FLIGHT_ENV)
                .ok()
                .as_deref(),
        ))
    })
}

fn gateway_provider_max_in_flight_limit() -> Option<usize> {
    gateway_provider_max_in_flight_limit_handle().snapshot()
}

pub fn configure_gateway_provider_max_in_flight_limit(limit: Option<usize>) {
    gateway_provider_max_in_flight_limit_handle().replace(limit);
}

fn gateway_provider_in_flight_counters_handle() -> &'static Mutex<HashMap<String, Arc<AtomicUsize>>>
{
    GATEWAY_PROVIDER_IN_FLIGHT_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn gateway_provider_in_flight_counter(provider_id: &str) -> Arc<AtomicUsize> {
    gateway_provider_in_flight_counters_handle()
        .lock()
        .expect("gateway provider in-flight counter lock")
        .entry(provider_id.to_owned())
        .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
        .clone()
}

fn capability_catalog_cache_store_handle() -> &'static Reloadable<Option<Arc<dyn CacheStore>>> {
    CAPABILITY_CATALOG_CACHE_STORE.get_or_init(|| Reloadable::new(None))
}

fn capability_catalog_cache_store() -> Option<Arc<dyn CacheStore>> {
    capability_catalog_cache_store_handle().snapshot()
}

pub fn configure_capability_catalog_cache_store(cache_store: Arc<dyn CacheStore>) {
    capability_catalog_cache_store_handle().replace(Some(cache_store));
}

pub fn clear_capability_catalog_cache_store() {
    capability_catalog_cache_store_handle().replace(None);
}

pub async fn invalidate_capability_catalog_cache() {
    if let Some(cache_store) = capability_catalog_cache_store() {
        if let Err(error) = cache_store
            .invalidate_tag(
                CAPABILITY_CATALOG_CACHE_NAMESPACE,
                CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS,
            )
            .await
        {
            eprintln!("capability catalog cache invalidation failed: {error}");
        }
    }
}

fn capability_catalog_list_cache_key(tenant_id: &str, project_id: &str) -> String {
    format!("{tenant_id}|{project_id}|list")
}

fn capability_catalog_model_cache_key(tenant_id: &str, project_id: &str, model_id: &str) -> String {
    format!("{tenant_id}|{project_id}|model|{model_id}")
}

fn routing_decision_cache_key(
    tenant_id: &str,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
    requested_region: Option<&str>,
) -> String {
    format!(
        "{tenant_id}|{}|{}|{capability}|{route_key}|{}",
        project_id.unwrap_or_default(),
        api_key_group_id.unwrap_or_default(),
        requested_region.unwrap_or_default()
    )
}

async fn cache_routing_decision(
    tenant_id: &str,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
    requested_region: Option<&str>,
    decision: &RoutingDecision,
) {
    let key = routing_decision_cache_key(
        tenant_id,
        project_id,
        api_key_group_id,
        capability,
        route_key,
        requested_region,
    );
    let payload = match serde_json::to_vec(decision) {
        Ok(payload) => payload,
        Err(error) => {
            eprintln!("routing decision cache serialization failed: {error}");
            return;
        }
    };
    if let Err(error) = routing_decision_cache_store()
        .put(
            ROUTING_DECISION_CACHE_NAMESPACE,
            &key,
            payload,
            Some(ROUTING_DECISION_CACHE_TTL_MS),
            &[],
        )
        .await
    {
        eprintln!("routing decision cache write failed: {error}");
    }
}

async fn take_cached_routing_decision(
    tenant_id: &str,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
    requested_region: Option<&str>,
) -> Option<RoutingDecision> {
    let key = routing_decision_cache_key(
        tenant_id,
        project_id,
        api_key_group_id,
        capability,
        route_key,
        requested_region,
    );
    let cache_store = routing_decision_cache_store();
    let entry = match cache_store
        .get(ROUTING_DECISION_CACHE_NAMESPACE, &key)
        .await
    {
        Ok(entry) => entry,
        Err(error) => {
            eprintln!("routing decision cache read failed: {error}");
            return None;
        }
    }?;
    if let Err(error) = cache_store
        .delete(ROUTING_DECISION_CACHE_NAMESPACE, &key)
        .await
    {
        eprintln!("routing decision cache delete failed: {error}");
    }
    match serde_json::from_slice(entry.value()) {
        Ok(decision) => Some(decision),
        Err(error) => {
            eprintln!("routing decision cache decode failed: {error}");
            None
        }
    }
}

async fn select_gateway_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> Result<RoutingDecision> {
    let requested_region = current_request_routing_region();
    let api_key_group_id = current_request_api_key_group_id();
    let recovery_probe_lock_store = routing_recovery_probe_lock_store();
    let decision = select_route_with_store_context(
        store,
        capability,
        route_key,
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_tenant_id_option(Some(tenant_id))
            .with_project_id_option(project_id)
            .with_api_key_group_id_option(api_key_group_id.as_deref())
            .with_requested_region_option(requested_region.as_deref())
            .with_recovery_probe_lock_store_option(Some(recovery_probe_lock_store.as_ref())),
    )
    .await?;
    record_gateway_recovery_probe_from_decision(&decision);
    cache_routing_decision(
        tenant_id,
        project_id,
        api_key_group_id.as_deref(),
        capability,
        route_key,
        requested_region.as_deref(),
        &decision,
    )
    .await;
    Ok(decision)
}

pub async fn planned_execution_provider_id_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<String> {
    Ok(planned_execution_usage_context_for_route(
        store, tenant_id, project_id, capability, route_key,
    )
    .await?
    .provider_id)
}

pub async fn planned_execution_usage_context_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<PlannedExecutionUsageContext> {
    let requested_region = current_request_routing_region();
    let api_key_group_id = current_request_api_key_group_id();
    let decision = match take_cached_routing_decision(
        tenant_id,
        Some(project_id),
        api_key_group_id.as_deref(),
        capability,
        route_key,
        requested_region.as_deref(),
    )
    .await
    {
        Some(decision) => decision,
        None => {
            let recovery_probe_lock_store = routing_recovery_probe_lock_store();
            let decision = select_route_with_store_context(
                store,
                capability,
                route_key,
                RouteSelectionContext::new(RoutingDecisionSource::Gateway)
                    .with_tenant_id_option(Some(tenant_id))
                    .with_project_id_option(Some(project_id))
                    .with_api_key_group_id_option(api_key_group_id.as_deref())
                    .with_requested_region_option(requested_region.as_deref())
                    .with_recovery_probe_lock_store_option(Some(
                        recovery_probe_lock_store.as_ref(),
                    )),
            )
            .await?;
            record_gateway_recovery_probe_from_decision(&decision);
            decision
        }
    };
    gateway_usage_context_for_decision_provider(
        store,
        tenant_id,
        &decision,
        &decision.selected_provider_id,
        api_key_group_id,
        decision.fallback_reason.clone(),
    )
    .await
}

async fn gateway_usage_context_for_decision_provider(
    store: &dyn AdminStore,
    tenant_id: &str,
    decision: &RoutingDecision,
    provider_id: &str,
    api_key_group_id: Option<String>,
    fallback_reason: Option<String>,
) -> Result<PlannedExecutionUsageContext> {
    let selected_assessment = decision
        .assessments
        .iter()
        .find(|assessment| assessment.provider_id == provider_id);
    let Some(provider) = store.find_provider(provider_id).await? else {
        return Ok(PlannedExecutionUsageContext {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            channel_id: None,
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        });
    };

    let target = provider_execution_target_for_provider(store, &provider).await?;
    if target.local_fallback {
        return Ok(PlannedExecutionUsageContext {
            provider_id: target.provider_id,
            channel_id: Some(provider.channel_id),
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        });
    }

    let has_credential = store
        .find_provider_credential(tenant_id, &provider.id)
        .await?
        .is_some();

    if has_credential {
        Ok(PlannedExecutionUsageContext {
            provider_id: target.provider_id,
            channel_id: Some(provider.channel_id),
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        })
    } else {
        Ok(PlannedExecutionUsageContext {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            channel_id: Some(provider.channel_id),
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        })
    }
}

fn gateway_execution_failover_fallback_reason(existing: Option<&str>) -> Option<String> {
    match existing {
        Some(existing)
            if existing
                .split(';')
                .any(|value| value == "gateway_execution_failover") =>
        {
            Some(existing.to_owned())
        }
        Some(existing) => Some(format!("{existing};gateway_execution_failover")),
        None => Some("gateway_execution_failover".to_owned()),
    }
}

fn gateway_execution_decision_id(provider_id: &str, created_at_ms: u64) -> String {
    format!("route_decision:gateway_execution:{provider_id}:{created_at_ms}")
}

async fn persist_gateway_execution_failover_decision_log(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    decision: &RoutingDecision,
    executed_provider_id: &str,
) -> Result<()> {
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;
    let requested_region =
        current_request_routing_region().or_else(|| decision.requested_region.clone());
    let api_key_group_id = current_request_api_key_group_id();
    let fallback_reason =
        gateway_execution_failover_fallback_reason(decision.fallback_reason.as_deref());
    let log = RoutingDecisionLog::new(
        gateway_execution_decision_id(executed_provider_id, created_at_ms),
        RoutingDecisionSource::Gateway,
        capability,
        route_key,
        executed_provider_id,
        decision
            .strategy
            .clone()
            .unwrap_or_else(|| "deterministic_priority".to_owned()),
        created_at_ms,
    )
    .with_tenant_id_option(Some(tenant_id.to_owned()))
    .with_project_id_option(Some(project_id.to_owned()))
    .with_api_key_group_id_option(api_key_group_id)
    .with_matched_policy_id_option(decision.matched_policy_id.clone())
    .with_applied_routing_profile_id_option(decision.applied_routing_profile_id.clone())
    .with_compiled_routing_snapshot_id_option(decision.compiled_routing_snapshot_id.clone())
    .with_selection_seed_option(decision.selection_seed)
    .with_selection_reason_option(decision.selection_reason.clone())
    .with_fallback_reason_option(fallback_reason)
    .with_requested_region_option(requested_region)
    .with_slo_state(decision.slo_applied, decision.slo_degraded)
    .with_assessments(decision.assessments.clone());
    store.insert_routing_decision_log(&log).await?;
    Ok(())
}

fn gateway_execution_observed_at_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn gateway_execution_health_message(
    capability: &str,
    provider_id: &str,
    healthy: bool,
    error: Option<&anyhow::Error>,
) -> String {
    match (healthy, error) {
        (true, _) => format!(
            "gateway execution succeeded for capability {capability} on provider {provider_id}"
        ),
        (false, Some(error)) => format!(
            "gateway execution failed for capability {capability} on provider {provider_id}: {error}"
        ),
        (false, None) => format!(
            "gateway execution failed for capability {capability} on provider {provider_id}"
        ),
    }
}

async fn persist_gateway_execution_health_snapshot(
    store: &dyn AdminStore,
    descriptor: &ProviderExecutionDescriptor,
    healthy: bool,
    capability: &str,
    error: Option<&anyhow::Error>,
) {
    if descriptor.local_fallback {
        return;
    }

    let observed_at_ms = gateway_execution_observed_at_ms();
    record_gateway_provider_health(
        &descriptor.provider_id,
        descriptor.runtime.as_str(),
        healthy,
        observed_at_ms,
    );

    let snapshot = ProviderHealthSnapshot::new(
        &descriptor.provider_id,
        &descriptor.runtime_key,
        descriptor.runtime.as_str(),
        observed_at_ms,
    )
    .with_running(true)
    .with_healthy(healthy)
    .with_message(gateway_execution_health_message(
        capability,
        &descriptor.provider_id,
        healthy,
        error,
    ));

    if let Err(persist_error) = store.insert_provider_health_snapshot(&snapshot).await {
        record_gateway_provider_health_persist_failure(
            &descriptor.provider_id,
            descriptor.runtime.as_str(),
        );
        eprintln!(
            "gateway execution health snapshot persistence failed for provider {}: {persist_error}",
            descriptor.provider_id
        );
    }
}

const GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS_ENV: &str = "SDKWORK_GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS";
const GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS_ENV: &str =
    "SDKWORK_GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS";
const GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS_ENV: &str = "SDKWORK_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS";
const DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS: usize = 2;
const DEFAULT_GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS: u64 = 25;
const DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GatewayUpstreamRetryPolicy {
    max_attempts: usize,
    base_delay_ms: u64,
    max_delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GatewayExecutionPolicy {
    failover_enabled: bool,
    retry_policy: GatewayUpstreamRetryPolicy,
}

impl GatewayUpstreamRetryPolicy {
    fn disabled() -> Self {
        Self {
            max_attempts: 1,
            base_delay_ms: 0,
            max_delay_ms: 0,
        }
    }

    fn enabled(&self) -> bool {
        self.max_attempts > 1
    }

    fn delay_before_next_attempt(&self, failed_attempt: usize) -> Duration {
        if failed_attempt == 0 || self.base_delay_ms == 0 {
            return Duration::from_millis(0);
        }

        let exponent = failed_attempt.saturating_sub(1).min(20) as u32;
        let multiplier = 1u64.checked_shl(exponent).unwrap_or(u64::MAX);
        let max_delay_ms = self.max_delay_ms.max(self.base_delay_ms);
        let delay_ms = self
            .base_delay_ms
            .saturating_mul(multiplier)
            .min(max_delay_ms);
        Duration::from_millis(delay_ms)
    }
}

async fn gateway_routing_policy_for_decision(
    store: &dyn AdminStore,
    decision: &RoutingDecision,
) -> Result<Option<RoutingPolicy>> {
    let Some(policy_id) = decision.matched_policy_id.as_deref() else {
        return Ok(None);
    };
    Ok(store
        .list_routing_policies()
        .await?
        .into_iter()
        .find(|policy| policy.policy_id == policy_id))
}

async fn gateway_execution_policy_for_decision(
    store: &dyn AdminStore,
    decision: &RoutingDecision,
    request: &ProviderRequest<'_>,
) -> Result<GatewayExecutionPolicy> {
    let routing_policy = gateway_routing_policy_for_decision(store, decision).await?;
    Ok(GatewayExecutionPolicy {
        failover_enabled: routing_policy
            .as_ref()
            .map(|policy| policy.execution_failover_enabled)
            .unwrap_or(true),
        retry_policy: gateway_upstream_retry_policy(request, routing_policy.as_ref()),
    })
}

fn gateway_upstream_retry_policy(
    request: &ProviderRequest<'_>,
    routing_policy: Option<&RoutingPolicy>,
) -> GatewayUpstreamRetryPolicy {
    if !gateway_request_supports_retry(request) {
        return GatewayUpstreamRetryPolicy::disabled();
    }

    let base_delay_ms = routing_policy
        .and_then(|policy| policy.upstream_retry_base_delay_ms)
        .unwrap_or_else(|| {
            std::env::var(GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS_ENV)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_GATEWAY_UPSTREAM_RETRY_BASE_DELAY_MS)
        });
    let max_delay_ms = routing_policy
        .and_then(|policy| policy.upstream_retry_max_delay_ms)
        .unwrap_or_else(|| {
            std::env::var(GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS_ENV)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS)
        })
        .max(base_delay_ms);

    GatewayUpstreamRetryPolicy {
        max_attempts: routing_policy
            .and_then(|policy| policy.upstream_retry_max_attempts)
            .map(|value| value.clamp(1, 4) as usize)
            .or_else(|| {
                std::env::var(GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS_ENV)
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok())
                    .map(|value| value.clamp(1, 4))
            })
            .unwrap_or(DEFAULT_GATEWAY_UPSTREAM_RETRY_MAX_ATTEMPTS),
        base_delay_ms,
        max_delay_ms,
    }
}

fn gateway_request_supports_retry(request: &ProviderRequest<'_>) -> bool {
    matches!(
        request,
        ProviderRequest::ChatCompletions(_)
            | ProviderRequest::ChatCompletionsStream(_)
            | ProviderRequest::Responses(_)
            | ProviderRequest::ResponsesStream(_)
    )
}

fn gateway_upstream_error_is_retryable(error: &anyhow::Error) -> bool {
    if let Some(error) = gateway_execution_context_error(error) {
        return matches!(
            error.kind(),
            GatewayExecutionContextErrorKind::RequestTimeout
        );
    }

    if let Some(error) = gateway_provider_http_error(error) {
        return matches!(
            error.status(),
            Some(
                reqwest::StatusCode::REQUEST_TIMEOUT
                    | reqwest::StatusCode::TOO_MANY_REQUESTS
                    | reqwest::StatusCode::INTERNAL_SERVER_ERROR
                    | reqwest::StatusCode::BAD_GATEWAY
                    | reqwest::StatusCode::SERVICE_UNAVAILABLE
                    | reqwest::StatusCode::GATEWAY_TIMEOUT
            )
        );
    }

    let Some(error) = gateway_reqwest_error(error) else {
        return false;
    };

    if error.is_timeout() || error.is_connect() {
        return true;
    }

    matches!(
        error.status(),
        Some(
            reqwest::StatusCode::REQUEST_TIMEOUT
                | reqwest::StatusCode::TOO_MANY_REQUESTS
                | reqwest::StatusCode::INTERNAL_SERVER_ERROR
                | reqwest::StatusCode::BAD_GATEWAY
                | reqwest::StatusCode::SERVICE_UNAVAILABLE
                | reqwest::StatusCode::GATEWAY_TIMEOUT
        )
    )
}

fn gateway_retry_reason_for_status(status: reqwest::StatusCode) -> &'static str {
    match status {
        reqwest::StatusCode::REQUEST_TIMEOUT => "status_408",
        reqwest::StatusCode::TOO_MANY_REQUESTS => "status_429",
        reqwest::StatusCode::INTERNAL_SERVER_ERROR => "status_500",
        reqwest::StatusCode::BAD_GATEWAY => "status_502",
        reqwest::StatusCode::SERVICE_UNAVAILABLE => "status_503",
        reqwest::StatusCode::GATEWAY_TIMEOUT => "status_504",
        _ => "status_other",
    }
}

fn gateway_retry_reason_for_error(error: &anyhow::Error) -> &'static str {
    if let Some(error) = gateway_execution_context_error(error) {
        return match error.kind() {
            GatewayExecutionContextErrorKind::RequestTimeout => "execution_timeout",
            GatewayExecutionContextErrorKind::DeadlineExceeded => "deadline_exceeded",
            GatewayExecutionContextErrorKind::ProviderOverloaded => "provider_overloaded",
        };
    }

    if let Some(error) = gateway_provider_http_error(error) {
        if let Some(status) = error.status() {
            return gateway_retry_reason_for_status(status);
        }
    }

    if let Some(error) = gateway_reqwest_error(error) {
        if error.is_timeout() {
            return "reqwest_timeout";
        }
        if error.is_connect() {
            return "reqwest_connect";
        }
        if let Some(status) = error.status() {
            return gateway_retry_reason_for_status(status);
        }
    }

    "unknown"
}

fn gateway_execution_context_metric_reason(error: &GatewayExecutionContextError) -> &'static str {
    match error.kind() {
        GatewayExecutionContextErrorKind::RequestTimeout => "request_timeout",
        GatewayExecutionContextErrorKind::DeadlineExceeded => "deadline_exceeded",
        GatewayExecutionContextErrorKind::ProviderOverloaded => "provider_overloaded",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewayExecutionContextErrorKind {
    RequestTimeout,
    DeadlineExceeded,
    ProviderOverloaded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GatewayExecutionContextError {
    kind: GatewayExecutionContextErrorKind,
    timeout_ms: Option<u64>,
    deadline_at_ms: Option<u64>,
    provider_id: Option<String>,
    max_in_flight: Option<usize>,
}

impl GatewayExecutionContextError {
    fn request_timeout(timeout_ms: u64, deadline_at_ms: Option<u64>) -> Self {
        Self {
            kind: GatewayExecutionContextErrorKind::RequestTimeout,
            timeout_ms: Some(timeout_ms),
            deadline_at_ms,
            provider_id: None,
            max_in_flight: None,
        }
    }

    fn deadline_exceeded(deadline_at_ms: Option<u64>) -> Self {
        Self {
            kind: GatewayExecutionContextErrorKind::DeadlineExceeded,
            timeout_ms: None,
            deadline_at_ms,
            provider_id: None,
            max_in_flight: None,
        }
    }

    fn provider_overloaded(provider_id: impl Into<String>, max_in_flight: usize) -> Self {
        Self {
            kind: GatewayExecutionContextErrorKind::ProviderOverloaded,
            timeout_ms: None,
            deadline_at_ms: None,
            provider_id: Some(provider_id.into()),
            max_in_flight: Some(max_in_flight),
        }
    }

    fn kind(&self) -> GatewayExecutionContextErrorKind {
        self.kind
    }
}

impl fmt::Display for GatewayExecutionContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.kind, self.timeout_ms, self.deadline_at_ms) {
            (
                GatewayExecutionContextErrorKind::RequestTimeout,
                Some(timeout_ms),
                Some(deadline),
            ) => {
                write!(
                    f,
                    "gateway upstream request timed out after {timeout_ms}ms before deadline {deadline}"
                )
            }
            (GatewayExecutionContextErrorKind::RequestTimeout, Some(timeout_ms), None) => {
                write!(f, "gateway upstream request timed out after {timeout_ms}ms")
            }
            (GatewayExecutionContextErrorKind::DeadlineExceeded, _, Some(deadline)) => {
                write!(
                    f,
                    "gateway upstream deadline {deadline} has already expired"
                )
            }
            (GatewayExecutionContextErrorKind::DeadlineExceeded, _, None) => {
                write!(f, "gateway upstream deadline has already expired")
            }
            (GatewayExecutionContextErrorKind::RequestTimeout, None, _) => {
                write!(f, "gateway upstream request timed out")
            }
            (GatewayExecutionContextErrorKind::ProviderOverloaded, _, _) => {
                match (self.provider_id.as_deref(), self.max_in_flight) {
                    (Some(provider_id), Some(max_in_flight)) => write!(
                        f,
                        "gateway provider {provider_id} is locally overloaded because in-flight requests reached {max_in_flight}"
                    ),
                    (Some(provider_id), None) => {
                        write!(f, "gateway provider {provider_id} is locally overloaded")
                    }
                    (None, Some(max_in_flight)) => write!(
                        f,
                        "gateway provider is locally overloaded because in-flight requests reached {max_in_flight}"
                    ),
                    (None, None) => write!(f, "gateway provider is locally overloaded"),
                }
            }
        }
    }
}

impl std::error::Error for GatewayExecutionContextError {}

struct GatewayProviderInFlightPermit {
    counter: Arc<AtomicUsize>,
}

impl Drop for GatewayProviderInFlightPermit {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GatewayRetryDelayDecision {
    delay: Duration,
    source: &'static str,
}

fn gateway_retry_delay_for_error(
    retry_policy: GatewayUpstreamRetryPolicy,
    failed_attempt: usize,
    error: &anyhow::Error,
) -> GatewayRetryDelayDecision {
    let base_delay = retry_policy.delay_before_next_attempt(failed_attempt);
    let retry_after = gateway_provider_http_error(error).and_then(|error| {
        Some((
            Duration::from_secs(error.retry_after_secs()?),
            error.retry_after_source(),
        ))
    });
    let retry_after_delay = retry_after
        .map(|(delay, _)| delay)
        .unwrap_or(Duration::from_millis(0));
    let capped_retry_after = if retry_after_delay.is_zero() {
        retry_after_delay
    } else {
        retry_after_delay.min(Duration::from_millis(retry_policy.max_delay_ms))
    };
    if !capped_retry_after.is_zero() && capped_retry_after > base_delay {
        let source = match retry_after.and_then(|(_, source)| source) {
            Some(ProviderRetryAfterSource::Seconds) => "retry_after_seconds",
            Some(ProviderRetryAfterSource::HttpDate) => "retry_after_http_date",
            None => "retry_after",
        };
        return GatewayRetryDelayDecision {
            delay: capped_retry_after,
            source,
        };
    }

    GatewayRetryDelayDecision {
        delay: base_delay.max(capped_retry_after),
        source: "backoff",
    }
}

fn gateway_provider_http_error(error: &anyhow::Error) -> Option<&ProviderHttpError> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<ProviderHttpError>())
}

fn gateway_execution_context_error(error: &anyhow::Error) -> Option<&GatewayExecutionContextError> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<GatewayExecutionContextError>())
}

fn gateway_execution_context_error_impacts_provider_health(
    error: &GatewayExecutionContextError,
) -> bool {
    matches!(
        error.kind(),
        GatewayExecutionContextErrorKind::RequestTimeout
    )
}

fn gateway_error_impacts_provider_health(error: &anyhow::Error) -> bool {
    if let Some(error) = gateway_execution_context_error(error) {
        return gateway_execution_context_error_impacts_provider_health(error);
    }
    true
}

fn gateway_reqwest_error(error: &anyhow::Error) -> Option<&reqwest::Error> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<reqwest::Error>())
}

fn try_acquire_gateway_provider_in_flight_permit(
    provider_id: Option<&str>,
) -> Result<Option<GatewayProviderInFlightPermit>> {
    let (Some(provider_id), Some(max_in_flight)) =
        (provider_id, gateway_provider_max_in_flight_limit())
    else {
        return Ok(None);
    };

    let counter = gateway_provider_in_flight_counter(provider_id);
    loop {
        let current = counter.load(Ordering::Acquire);
        if current >= max_in_flight {
            return Err(anyhow::Error::new(
                GatewayExecutionContextError::provider_overloaded(provider_id, max_in_flight),
            ));
        }
        if counter
            .compare_exchange(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return Ok(Some(GatewayProviderInFlightPermit { counter }));
        }
    }
}

async fn execute_provider_request_with_execution_context(
    adapter: &dyn ProviderExecutionAdapter,
    provider_id: Option<&str>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<ProviderOutput> {
    let now_ms = gateway_execution_observed_at_ms();
    if options.deadline_expired(now_ms) {
        return Err(anyhow::Error::new(
            GatewayExecutionContextError::deadline_exceeded(options.deadline_at_ms()),
        ));
    }

    let _in_flight_permit = try_acquire_gateway_provider_in_flight_permit(provider_id)?;

    if let Some(timeout_ms) = options.effective_timeout_ms(now_ms) {
        return match tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            adapter.execute_with_options(api_key, request, options),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(anyhow::Error::new(
                GatewayExecutionContextError::request_timeout(timeout_ms, options.deadline_at_ms()),
            )),
        };
    }

    adapter
        .execute_with_options(api_key, request, options)
        .await
}

pub async fn with_request_routing_region<T, F>(requested_region: Option<String>, future: F) -> T
where
    F: Future<Output = T>,
{
    REQUEST_ROUTING_REGION
        .scope(
            requested_region.and_then(|region| normalize_routing_region(&region)),
            future,
        )
        .await
}

pub async fn with_request_api_key_group_id<T, F>(api_key_group_id: Option<String>, future: F) -> T
where
    F: Future<Output = T>,
{
    REQUEST_API_KEY_GROUP_ID
        .scope(
            api_key_group_id.and_then(|group_id| normalize_api_key_group_id(&group_id)),
            future,
        )
        .await
}

pub fn current_request_routing_region() -> Option<String> {
    REQUEST_ROUTING_REGION.try_with(Clone::clone).ok().flatten()
}

pub fn current_request_api_key_group_id() -> Option<String> {
    REQUEST_API_KEY_GROUP_ID
        .try_with(Clone::clone)
        .ok()
        .flatten()
}

fn normalize_routing_region(region: &str) -> Option<String> {
    let normalized = region.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_api_key_group_id(api_key_group_id: &str) -> Option<String> {
    let normalized = api_key_group_id.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_owned())
    }
}

async fn resolve_non_model_provider(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<Option<(String, String, String)>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), capability, route_key).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    let descriptor = provider_execution_descriptor_for_provider(store, &provider, api_key).await?;
    if descriptor.local_fallback {
        return Ok(None);
    }

    Ok(Some((
        descriptor.runtime_key,
        descriptor.base_url,
        descriptor.api_key,
    )))
}

pub async fn relay_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    Ok(relay_chat_completion_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
        &options,
    )
    .await?
    .response)
}

pub async fn relay_chat_completion_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<GatewayExecutionResult<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "chat_completion",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &decision,
        &ProviderRequest::ChatCompletions(request),
    )
    .await?;
    let selected_provider_id = decision.selected_provider_id.clone();
    let Some(provider) = store.find_provider(&selected_provider_id).await? else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    match execute_json_provider_request_for_provider_with_options(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletions(request),
        options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_context = gateway_usage_context_for_decision_provider(
                store,
                tenant_id,
                &decision,
                &provider.id,
                current_request_api_key_group_id(),
                decision.fallback_reason.clone(),
            )
            .await?;
            Ok(GatewayExecutionResult::new(
                Some(response),
                Some(usage_context),
            ))
        }
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_api_key) = resolve_provider_secret_with_manager(
                    store,
                    secret_manager,
                    tenant_id,
                    &candidate_provider.id,
                )
                .await?
                else {
                    continue;
                };
                match execute_json_provider_request_for_provider_with_options(
                    store,
                    &candidate_provider,
                    &candidate_api_key,
                    ProviderRequest::ChatCompletions(request),
                    options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "chat_completion",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                decision.fallback_reason.as_deref(),
                            ),
                        )
                        .await?;
                        return Ok(GatewayExecutionResult::new(
                            Some(response),
                            Some(usage_context),
                        ));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_chat_completion_from_store_with_options(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    Ok(relay_chat_completion_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
        options,
    )
    .await?
    .response)
}

pub async fn relay_list_chat_completions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        "chat_completions",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsList,
    )
    .await
}

pub async fn relay_get_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsRetrieve(completion_id),
    )
    .await
}

pub async fn relay_update_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
    request: &UpdateChatCompletionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsUpdate(completion_id, request),
    )
    .await
}

pub async fn relay_delete_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsDelete(completion_id),
    )
    .await
}

pub async fn relay_list_chat_completion_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsMessagesList(completion_id),
    )
    .await
}

pub async fn relay_chat_completion_stream_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    Ok(
        relay_chat_completion_stream_from_store_with_execution_context(
            store,
            secret_manager,
            tenant_id,
            _project_id,
            request,
            &options,
        )
        .await?
        .response,
    )
}

pub async fn relay_chat_completion_stream_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<GatewayExecutionResult<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "chat_completion",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &decision,
        &ProviderRequest::ChatCompletionsStream(request),
    )
    .await?;
    let selected_provider_id = decision.selected_provider_id.clone();
    let Some(provider) = store.find_provider(&selected_provider_id).await? else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    match execute_stream_provider_request_for_provider_with_options(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsStream(request),
        options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_context = gateway_usage_context_for_decision_provider(
                store,
                tenant_id,
                &decision,
                &provider.id,
                current_request_api_key_group_id(),
                decision.fallback_reason.clone(),
            )
            .await?;
            Ok(GatewayExecutionResult::new(
                Some(response),
                Some(usage_context),
            ))
        }
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_api_key) = resolve_provider_secret_with_manager(
                    store,
                    secret_manager,
                    tenant_id,
                    &candidate_provider.id,
                )
                .await?
                else {
                    continue;
                };
                match execute_stream_provider_request_for_provider_with_options(
                    store,
                    &candidate_provider,
                    &candidate_api_key,
                    ProviderRequest::ChatCompletionsStream(request),
                    options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "chat_completion",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                decision.fallback_reason.as_deref(),
                            ),
                        )
                        .await?;
                        return Ok(GatewayExecutionResult::new(
                            Some(response),
                            Some(usage_context),
                        ));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "chat_completion",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_chat_completion_stream_from_store_with_options(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    Ok(
        relay_chat_completion_stream_from_store_with_execution_context(
            store,
            secret_manager,
            tenant_id,
            _project_id,
            request,
            options,
        )
        .await?
        .response,
    )
}

pub async fn relay_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateConversationRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        "conversations",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::Conversations(request),
    )
    .await
}

pub async fn relay_list_conversations_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        "conversations",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsList,
    )
    .await
}

pub async fn relay_get_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsRetrieve(conversation_id),
    )
    .await
}

pub async fn relay_update_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    request: &UpdateConversationRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsUpdate(conversation_id, request),
    )
    .await
}

pub async fn relay_delete_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsDelete(conversation_id),
    )
    .await
}

pub async fn relay_conversation_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    request: &CreateConversationItemsRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItems(conversation_id, request),
    )
    .await
}

pub async fn relay_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateThreadRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        "threads",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::Threads(request),
    )
    .await
}

pub async fn relay_get_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsRetrieve(thread_id),
    )
    .await
}

pub async fn relay_update_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &UpdateThreadRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsUpdate(thread_id, request),
    )
    .await
}

pub async fn relay_delete_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsDelete(thread_id),
    )
    .await
}

pub async fn relay_thread_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &CreateThreadMessageRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessages(thread_id, request),
    )
    .await
}

pub async fn relay_list_thread_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesList(thread_id),
    )
    .await
}

pub async fn relay_get_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesRetrieve(thread_id, message_id),
    )
    .await
}

pub async fn relay_update_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
    request: &UpdateThreadMessageRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesUpdate(thread_id, message_id, request),
    )
    .await
}

pub async fn relay_delete_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesDelete(thread_id, message_id),
    )
    .await
}

pub async fn relay_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &CreateRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRuns(thread_id, request),
    )
    .await
}

pub async fn relay_list_thread_runs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsList(thread_id),
    )
    .await
}

pub async fn relay_get_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsRetrieve(thread_id, run_id),
    )
    .await
}

pub async fn relay_update_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    request: &UpdateRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsUpdate(thread_id, run_id, request),
    )
    .await
}

pub async fn relay_cancel_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsCancel(thread_id, run_id),
    )
    .await
}

pub async fn relay_submit_thread_run_tool_outputs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    request: &SubmitToolOutputsRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsSubmitToolOutputs(thread_id, run_id, request),
    )
    .await
}

pub async fn relay_list_thread_run_steps_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunStepsList(thread_id, run_id),
    )
    .await
}

pub async fn relay_get_thread_run_step_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunStepsRetrieve(thread_id, run_id, step_id),
    )
    .await
}

pub async fn relay_thread_and_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateThreadAndRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        "threads/runs",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsRuns(request),
    )
    .await
}

pub async fn relay_list_conversation_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsList(conversation_id),
    )
    .await
}

pub async fn relay_get_conversation_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsRetrieve(conversation_id, item_id),
    )
    .await
}

pub async fn relay_delete_conversation_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsDelete(conversation_id, item_id),
    )
    .await
}

pub async fn relay_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<Value>> {
    Ok(relay_response_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
    )
    .await?
    .response)
}

pub async fn relay_response_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateResponseRequest,
) -> Result<GatewayExecutionResult<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "responses",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &decision,
        &ProviderRequest::Responses(request),
    )
    .await?;
    let selected_provider_id = decision.selected_provider_id.clone();
    let Some(provider) = store.find_provider(&selected_provider_id).await? else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let options = ProviderRequestOptions::default();

    match execute_json_provider_request_for_provider_with_options(
        store,
        &provider,
        &api_key,
        ProviderRequest::Responses(request),
        &options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_context = gateway_usage_context_for_decision_provider(
                store,
                tenant_id,
                &decision,
                &provider.id,
                current_request_api_key_group_id(),
                decision.fallback_reason.clone(),
            )
            .await?;
            Ok(GatewayExecutionResult::new(
                Some(response),
                Some(usage_context),
            ))
        }
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_api_key) = resolve_provider_secret_with_manager(
                    store,
                    secret_manager,
                    tenant_id,
                    &candidate_provider.id,
                )
                .await?
                else {
                    continue;
                };
                match execute_json_provider_request_for_provider_with_options(
                    store,
                    &candidate_provider,
                    &candidate_api_key,
                    ProviderRequest::Responses(request),
                    &options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "responses",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                decision.fallback_reason.as_deref(),
                            ),
                        )
                        .await?;
                        return Ok(GatewayExecutionResult::new(
                            Some(response),
                            Some(usage_context),
                        ));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_response_stream_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<ProviderStreamOutput>> {
    Ok(relay_response_stream_from_store_with_execution_context(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        request,
    )
    .await?
    .response)
}

pub async fn relay_response_stream_from_store_with_execution_context(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateResponseRequest,
) -> Result<GatewayExecutionResult<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "responses",
        &request.model,
    )
    .await?;
    let execution_policy = gateway_execution_policy_for_decision(
        store,
        &decision,
        &ProviderRequest::ResponsesStream(request),
    )
    .await?;
    let selected_provider_id = decision.selected_provider_id.clone();
    let Some(provider) = store.find_provider(&selected_provider_id).await? else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(GatewayExecutionResult::new(None, None));
    };
    let options = ProviderRequestOptions::default();

    match execute_stream_provider_request_for_provider_with_options(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesStream(request),
        &options,
        execution_policy.retry_policy,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_context = gateway_usage_context_for_decision_provider(
                store,
                tenant_id,
                &decision,
                &provider.id,
                current_request_api_key_group_id(),
                decision.fallback_reason.clone(),
            )
            .await?;
            Ok(GatewayExecutionResult::new(
                Some(response),
                Some(usage_context),
            ))
        }
        Ok(None) => Ok(GatewayExecutionResult::new(None, None)),
        Err(mut last_error) => {
            if !execution_policy.failover_enabled {
                return Err(last_error);
            }
            for candidate_provider_id in decision
                .candidate_ids
                .iter()
                .filter(|provider_id| provider_id.as_str() != selected_provider_id)
            {
                let Some(candidate_provider) = store.find_provider(candidate_provider_id).await?
                else {
                    continue;
                };
                let Some(candidate_api_key) = resolve_provider_secret_with_manager(
                    store,
                    secret_manager,
                    tenant_id,
                    &candidate_provider.id,
                )
                .await?
                else {
                    continue;
                };
                match execute_stream_provider_request_for_provider_with_options(
                    store,
                    &candidate_provider,
                    &candidate_api_key,
                    ProviderRequest::ResponsesStream(request),
                    &options,
                    execution_policy.retry_policy,
                )
                .await
                {
                    Ok(Some(response)) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "success",
                        );
                        persist_gateway_execution_failover_decision_log(
                            store,
                            tenant_id,
                            project_id,
                            "responses",
                            &request.model,
                            &decision,
                            &candidate_provider.id,
                        )
                        .await?;
                        let usage_context = gateway_usage_context_for_decision_provider(
                            store,
                            tenant_id,
                            &decision,
                            &candidate_provider.id,
                            current_request_api_key_group_id(),
                            gateway_execution_failover_fallback_reason(
                                decision.fallback_reason.as_deref(),
                            ),
                        )
                        .await?;
                        return Ok(GatewayExecutionResult::new(
                            Some(response),
                            Some(usage_context),
                        ));
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        record_gateway_execution_failover(
                            "responses",
                            &selected_provider_id,
                            &candidate_provider.id,
                            "failure",
                        );
                        last_error = error;
                    }
                }
            }
            Err(last_error)
        }
    }
}

pub async fn relay_count_response_input_tokens_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CountResponseInputTokensRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesInputTokens(request),
    )
    .await
}

pub async fn relay_get_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesRetrieve(response_id),
    )
    .await
}

pub async fn relay_cancel_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesCancel(response_id),
    )
    .await
}

pub async fn relay_delete_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesDelete(response_id),
    )
    .await
}

pub async fn relay_compact_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CompactResponseRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesCompact(request),
    )
    .await
}

pub async fn relay_list_response_input_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesInputItemsList(response_id),
    )
    .await
}

pub async fn relay_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateCompletionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "completions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Completions(request),
    )
    .await
}

pub async fn relay_embedding_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEmbeddingRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "embeddings",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Embeddings(request),
    )
    .await
}

pub async fn relay_moderation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateModerationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "moderations",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Moderations(request),
    )
    .await
}

pub async fn relay_image_generation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesGenerations(request),
    )
    .await
}

pub async fn relay_image_edit_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageEditRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        request.model_or_default(),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesEdits(request),
    )
    .await
}

pub async fn relay_image_variation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageVariationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        request.model_or_default(),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesVariations(request),
    )
    .await
}

pub async fn relay_transcription_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateTranscriptionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_transcriptions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioTranscriptions(request),
    )
    .await
}

pub async fn relay_translation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateTranslationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_translations",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioTranslations(request),
    )
    .await
}

pub async fn relay_speech_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateSpeechRequest,
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_speech",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioSpeech(request),
    )
    .await
}

pub async fn relay_audio_voices_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "audio", "voices").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioVoicesList,
    )
    .await
}

pub async fn relay_audio_voice_consent_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVoiceConsentRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "audio", &request.voice).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioVoiceConsents(request),
    )
    .await
}

pub async fn relay_container_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateContainerRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        &request.name,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Containers(request),
    )
    .await
}

pub async fn relay_list_containers_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        "containers",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainersList,
    )
    .await
}

pub async fn relay_get_container_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainersRetrieve(container_id),
    )
    .await
}

pub async fn relay_delete_container_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainersDelete(container_id),
    )
    .await
}

pub async fn relay_container_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    request: &CreateContainerFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainerFiles(container_id, request),
    )
    .await
}

pub async fn relay_list_container_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainerFilesList(container_id),
    )
    .await
}

pub async fn relay_get_container_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainerFilesRetrieve(container_id, file_id),
    )
    .await
}

pub async fn relay_delete_container_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainerFilesDelete(container_id, file_id),
    )
    .await
}

pub async fn relay_container_file_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ContainerFilesContent(container_id, file_id),
    )
    .await
}

pub async fn relay_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "files",
        &request.purpose,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Files(request),
    )
    .await
}

pub async fn relay_list_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", "files").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesList,
    )
    .await
}

pub async fn relay_get_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesRetrieve(file_id),
    )
    .await
}

pub async fn relay_delete_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesDelete(file_id),
    )
    .await
}

pub async fn relay_file_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesContent(file_id),
    )
    .await
}

pub async fn relay_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.purpose,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Uploads(request),
    )
    .await
}

pub async fn relay_upload_part_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.upload_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::UploadParts(request),
    )
    .await
}

pub async fn relay_complete_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CompleteUploadRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.upload_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::UploadComplete(request),
    )
    .await
}

pub async fn relay_cancel_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    upload_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "uploads", upload_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::UploadCancel(upload_id),
    )
    .await
}

pub async fn relay_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFineTuningJobRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "fine_tuning",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobs(request),
    )
    .await
}

pub async fn relay_list_fine_tuning_jobs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", "jobs").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsList,
    )
    .await
}

pub async fn relay_get_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsRetrieve(job_id),
    )
    .await
}

pub async fn relay_cancel_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsCancel(job_id),
    )
    .await
}

pub async fn relay_list_fine_tuning_job_events_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsEvents(job_id),
    )
    .await
}

pub async fn relay_list_fine_tuning_job_checkpoints_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsCheckpoints(job_id),
    )
    .await
}

pub async fn relay_pause_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsPause(job_id),
    )
    .await
}

pub async fn relay_resume_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsResume(job_id),
    )
    .await
}

pub async fn relay_fine_tuning_checkpoint_permissions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    checkpoint_id: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "fine_tuning",
        checkpoint_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningCheckpointPermissions(checkpoint_id, request),
    )
    .await
}

pub async fn relay_list_fine_tuning_checkpoint_permissions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    checkpoint_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "fine_tuning",
        checkpoint_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningCheckpointPermissionsList(checkpoint_id),
    )
    .await
}

pub async fn relay_delete_fine_tuning_checkpoint_permission_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "fine_tuning",
        checkpoint_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningCheckpointPermissionsDelete(checkpoint_id, permission_id),
    )
    .await
}

pub async fn relay_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateAssistantRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Assistants(request),
    )
    .await
}

pub async fn relay_list_assistants_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        "assistants",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsList,
    )
    .await
}

pub async fn relay_get_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsRetrieve(assistant_id),
    )
    .await
}

pub async fn relay_update_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    request: &UpdateAssistantRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsUpdate(assistant_id, request),
    )
    .await
}

pub async fn relay_delete_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsDelete(assistant_id),
    )
    .await
}

pub async fn relay_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateWebhookRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "webhooks",
        &request.url,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Webhooks(request),
    )
    .await
}

pub async fn relay_list_webhooks_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", "webhooks").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksList,
    )
    .await
}

pub async fn relay_get_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksRetrieve(webhook_id),
    )
    .await
}

pub async fn relay_update_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
    request: &UpdateWebhookRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksUpdate(webhook_id, request),
    )
    .await
}

pub async fn relay_delete_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksDelete(webhook_id),
    )
    .await
}

pub async fn relay_realtime_session_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateRealtimeSessionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "realtime_sessions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::RealtimeSessions(request),
    )
    .await
}

pub async fn relay_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEvalRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", &request.name).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Evals(request),
    )
    .await
}

pub async fn relay_list_evals_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", "evals").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalsList,
    )
    .await
}

pub async fn relay_get_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalsRetrieve(eval_id),
    )
    .await
}

pub async fn relay_update_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalsUpdate(eval_id, request),
    )
    .await
}

pub async fn relay_delete_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalsDelete(eval_id),
    )
    .await
}

pub async fn relay_list_eval_runs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRunsList(eval_id),
    )
    .await
}

pub async fn relay_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &CreateEvalRunRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRuns(eval_id, request),
    )
    .await
}

pub async fn relay_get_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRunsRetrieve(eval_id, run_id),
    )
    .await
}

pub async fn relay_delete_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRunsDelete(eval_id, run_id),
    )
    .await
}

pub async fn relay_cancel_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRunsCancel(eval_id, run_id),
    )
    .await
}

pub async fn relay_list_eval_run_output_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRunOutputItemsList(eval_id, run_id),
    )
    .await
}

pub async fn relay_get_eval_run_output_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::EvalRunOutputItemsRetrieve(eval_id, run_id, output_item_id),
    )
    .await
}

pub async fn relay_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateBatchRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "batches",
        &request.endpoint,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Batches(request),
    )
    .await
}

pub async fn relay_list_batches_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", "batches").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::BatchesList,
    )
    .await
}

pub async fn relay_get_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", batch_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::BatchesRetrieve(batch_id),
    )
    .await
}

pub async fn relay_cancel_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", batch_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::BatchesCancel(batch_id),
    )
    .await
}

pub async fn relay_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        &request.name,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStores(request),
    )
    .await
}

pub async fn relay_list_vector_stores_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        "vector_stores",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresList,
    )
    .await
}

pub async fn relay_get_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresRetrieve(vector_store_id),
    )
    .await
}

pub async fn relay_update_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &UpdateVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresUpdate(vector_store_id, request),
    )
    .await
}

pub async fn relay_delete_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresDelete(vector_store_id),
    )
    .await
}

pub async fn relay_search_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &SearchVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_search",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresSearch(vector_store_id, request),
    )
    .await
}

pub async fn relay_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &CreateVectorStoreFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFiles(vector_store_id, request),
    )
    .await
}

pub async fn relay_list_vector_store_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFilesList(vector_store_id),
    )
    .await
}

pub async fn relay_get_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFilesRetrieve(vector_store_id, file_id),
    )
    .await
}

pub async fn relay_delete_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFilesDelete(vector_store_id, file_id),
    )
    .await
}

pub async fn relay_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &CreateVectorStoreFileBatchRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatches(vector_store_id, request),
    )
    .await
}

pub async fn relay_get_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatchesRetrieve(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_cancel_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatchesCancel(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_list_vector_store_file_batch_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatchesListFiles(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateMusicRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Music(request),
    )
    .await
}

pub async fn relay_list_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", "music").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::MusicList,
    )
    .await
}

pub async fn relay_get_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", music_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::MusicRetrieve(music_id),
    )
    .await
}

pub async fn relay_delete_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", music_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::MusicDelete(music_id),
    )
    .await
}

pub async fn relay_music_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", music_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::MusicContent(music_id),
    )
    .await
}

pub async fn relay_music_lyrics_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateMusicLyricsRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", "lyrics").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::MusicLyrics(request),
    )
    .await
}

pub async fn relay_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "videos",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Videos(request),
    )
    .await
}

pub async fn relay_list_videos_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", "videos").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosList,
    )
    .await
}

pub async fn relay_get_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosRetrieve(video_id),
    )
    .await
}

pub async fn relay_delete_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosDelete(video_id),
    )
    .await
}

pub async fn relay_video_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosContent(video_id),
    )
    .await
}

pub async fn relay_remix_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    request: &RemixVideoRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosRemix(video_id, request),
    )
    .await
}

pub async fn relay_create_video_character_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateVideoCharacterRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "videos",
        &request.video_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideoCharactersCreate(request),
    )
    .await
}

pub async fn relay_list_video_characters_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideoCharactersList(video_id),
    )
    .await
}

pub async fn relay_get_video_character_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    character_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideoCharactersRetrieve(video_id, character_id),
    )
    .await
}

pub async fn relay_update_video_character_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    character_id: &str,
    request: &UpdateVideoCharacterRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideoCharactersUpdate(video_id, character_id, request),
    )
    .await
}

pub async fn relay_get_video_character_canonical_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    character_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "videos", character_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideoCharactersCanonicalRetrieve(character_id),
    )
    .await
}

pub async fn relay_edit_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &EditVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "videos",
        &request.video_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosEdits(request),
    )
    .await
}

pub async fn relay_extensions_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &ExtendVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "videos",
        request.video_id.as_deref().unwrap_or("videos"),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosExtensions(request),
    )
    .await
}

pub async fn relay_extend_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    request: &ExtendVideoRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosExtend(video_id, request),
    )
    .await
}

pub fn create_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ChatCompletionResponse> {
    Ok(ChatCompletionResponse::empty("chatcmpl_1", model))
}

pub fn create_conversation(_tenant_id: &str, _project_id: &str) -> Result<ConversationObject> {
    Ok(ConversationObject::new("conv_1"))
}

pub fn list_conversations(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListConversationsResponse> {
    Ok(ListConversationsResponse::new(vec![
        ConversationObject::new("conv_1"),
    ]))
}

fn ensure_local_conversation_exists(conversation_id: &str) -> Result<()> {
    if conversation_id != "conv_1" {
        bail!("conversation not found");
    }

    Ok(())
}

pub fn get_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ConversationObject> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ConversationObject::new(conversation_id))
}

pub fn update_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> Result<ConversationObject> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ConversationObject::with_metadata(conversation_id, metadata))
}

pub fn delete_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<DeleteConversationResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(DeleteConversationResponse::deleted(conversation_id))
}

pub fn create_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ListConversationItemsResponse::new(vec![
        ConversationItemObject::message("item_1", "assistant", "hello"),
    ]))
}

pub fn list_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ListConversationItemsResponse::new(vec![
        ConversationItemObject::message("item_1", "assistant", "hello"),
    ]))
}

fn ensure_local_conversation_item_exists(conversation_id: &str, item_id: &str) -> Result<()> {
    ensure_local_conversation_exists(conversation_id)?;
    if item_id != "item_1" {
        bail!("conversation item not found");
    }

    Ok(())
}

pub fn get_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<ConversationItemObject> {
    ensure_local_conversation_item_exists(conversation_id, item_id)?;
    Ok(ConversationItemObject::message(
        item_id,
        "assistant",
        "hello",
    ))
}

pub fn delete_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<DeleteConversationItemResponse> {
    ensure_local_conversation_item_exists(conversation_id, item_id)?;
    Ok(DeleteConversationItemResponse::deleted(item_id))
}

pub fn list_chat_completions(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListChatCompletionsResponse> {
    Ok(ListChatCompletionsResponse::new(vec![
        ChatCompletionResponse::empty("chatcmpl_1", "gpt-4.1"),
    ]))
}

pub fn get_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<ChatCompletionResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    Ok(ChatCompletionResponse::empty(completion_id, "gpt-4.1"))
}

pub fn update_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> Result<ChatCompletionResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    Ok(ChatCompletionResponse::with_metadata(
        completion_id,
        "gpt-4.1",
        metadata,
    ))
}

pub fn delete_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<DeleteChatCompletionResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    Ok(DeleteChatCompletionResponse::deleted(completion_id))
}

fn ensure_local_chat_completion_exists(completion_id: &str) -> Result<()> {
    if completion_id != "chatcmpl_1" {
        bail!("chat completion not found");
    }

    Ok(())
}

pub fn list_chat_completion_messages(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<ListChatCompletionMessagesResponse> {
    ensure_local_chat_completion_exists(completion_id)?;
    Ok(ListChatCompletionMessagesResponse::new(vec![
        ChatCompletionMessageObject::assistant("msg_1", "hello"),
    ]))
}

fn ensure_local_response_model_present(model: &str) -> Result<()> {
    if model.trim().is_empty() {
        bail!("Response model is required.");
    }

    Ok(())
}

pub fn create_response(_tenant_id: &str, _project_id: &str, model: &str) -> Result<ResponseObject> {
    ensure_local_response_model_present(model)?;
    Ok(ResponseObject::empty("resp_1", model))
}

pub fn count_response_input_tokens(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ResponseInputTokensObject> {
    ensure_local_response_model_present(model)?;
    Ok(ResponseInputTokensObject::new(42))
}

fn ensure_local_response_exists(response_id: &str) -> Result<()> {
    if response_id != "resp_1" {
        bail!("response not found");
    }

    Ok(())
}

pub fn get_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ResponseObject> {
    ensure_local_response_exists(response_id)?;
    Ok(ResponseObject::empty(response_id, "gpt-4.1"))
}

pub fn list_response_input_items(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ListResponseInputItemsResponse> {
    ensure_local_response_exists(response_id)?;
    Ok(ListResponseInputItemsResponse::new(vec![
        ResponseInputItemObject::message("item_1"),
    ]))
}

pub fn delete_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<DeleteResponseResponse> {
    ensure_local_response_exists(response_id)?;
    Ok(DeleteResponseResponse::deleted(response_id))
}

pub fn cancel_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ResponseObject> {
    ensure_local_response_exists(response_id)?;
    Ok(ResponseObject::cancelled(response_id, "gpt-4.1"))
}

pub fn compact_response(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ResponseCompactionObject> {
    if model.trim().is_empty() {
        bail!("Response compaction model is required.");
    }
    Ok(ResponseCompactionObject::new("resp_cmp_1", model))
}

pub fn create_completion(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<CompletionObject> {
    if model.trim().is_empty() {
        bail!("Completion model is required.");
    }

    Ok(CompletionObject::new("cmpl_1", "SDKWork completion"))
}

pub fn create_embedding(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<CreateEmbeddingResponse> {
    if model.trim().is_empty() {
        bail!("Embedding model is required.");
    }

    Ok(CreateEmbeddingResponse::empty(model))
}

pub fn create_moderation(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ModerationResponse> {
    if model.trim().is_empty() {
        bail!("Moderation model is required.");
    }

    Ok(ModerationResponse {
        id: "modr_1".to_owned(),
        model: model.to_owned(),
        results: vec![ModerationResult {
            flagged: false,
            category_scores: ModerationCategoryScores { violence: 0.0 },
        }],
    })
}

pub fn create_image_generation(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ImagesResponse> {
    if model.trim().is_empty() {
        bail!("Image generation model is required.");
    }

    Ok(ImagesResponse::new(vec![ImageObject::base64(
        "sdkwork-image",
    )]))
}

pub fn create_image_edit(
    _tenant_id: &str,
    _project_id: &str,
    _request: &CreateImageEditRequest,
) -> Result<ImagesResponse> {
    Ok(ImagesResponse::new(vec![ImageObject::base64(
        "sdkwork-image",
    )]))
}

pub fn create_image_variation(
    _tenant_id: &str,
    _project_id: &str,
    _request: &CreateImageVariationRequest,
) -> Result<ImagesResponse> {
    Ok(ImagesResponse::new(vec![ImageObject::base64(
        "sdkwork-image",
    )]))
}

pub fn create_transcription(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<TranscriptionObject> {
    Ok(TranscriptionObject::new("sdkwork transcription"))
}

pub fn create_translation(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<TranslationObject> {
    Ok(TranslationObject::new("sdkwork translation"))
}

pub fn list_audio_voices(_tenant_id: &str, _project_id: &str) -> Result<ListVoicesResponse> {
    Ok(ListVoicesResponse::new(vec![VoiceObject::new(
        "voice_1", "Alloy",
    )]))
}

pub fn create_audio_voice_consent(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateVoiceConsentRequest,
) -> Result<VoiceConsentObject> {
    Ok(VoiceConsentObject::approved(
        "voice_consent_1",
        &request.voice,
        &request.name,
    ))
}

pub fn create_file(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<FileObject> {
    if request.purpose.trim().is_empty() {
        bail!("File purpose is required.");
    }
    if request.filename.trim().is_empty() {
        bail!("File filename is required.");
    }
    Ok(FileObject::with_bytes(
        "file_1",
        &request.filename,
        &request.purpose,
        request.bytes.len() as u64,
    ))
}

pub fn list_files(_tenant_id: &str, _project_id: &str) -> Result<ListFilesResponse> {
    Ok(ListFilesResponse::new(vec![FileObject::with_bytes(
        "file_1",
        "train.jsonl",
        "fine-tune",
        2,
    )]))
}

fn ensure_local_file_exists(file_id: &str) -> Result<()> {
    if file_id != "file_1" {
        bail!("file not found");
    }

    Ok(())
}

pub fn get_file(_tenant_id: &str, _project_id: &str, file_id: &str) -> Result<FileObject> {
    ensure_local_file_exists(file_id)?;
    Ok(FileObject::with_bytes(
        file_id,
        "train.jsonl",
        "fine-tune",
        2,
    ))
}

pub fn delete_file(
    _tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<DeleteFileResponse> {
    ensure_local_file_exists(file_id)?;
    Ok(DeleteFileResponse::deleted(file_id))
}

pub fn file_content(_tenant_id: &str, _project_id: &str, _file_id: &str) -> Result<Vec<u8>> {
    ensure_local_file_exists(_file_id)?;
    Ok(b"{}".to_vec())
}

pub fn create_container(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateContainerRequest,
) -> Result<ContainerObject> {
    Ok(ContainerObject::new("container_1", &request.name))
}

pub fn list_containers(_tenant_id: &str, _project_id: &str) -> Result<ListContainersResponse> {
    Ok(ListContainersResponse::new(vec![ContainerObject::new(
        "container_1",
        "ci-container",
    )]))
}

fn ensure_local_container_exists(container_id: &str) -> Result<()> {
    if container_id != "container_1" {
        bail!("container not found");
    }

    Ok(())
}

fn ensure_local_container_file_exists(container_id: &str, file_id: &str) -> Result<()> {
    ensure_local_container_exists(container_id)?;
    if file_id != "file_1" {
        bail!("container file not found");
    }

    Ok(())
}

pub fn get_container(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
) -> Result<ContainerObject> {
    ensure_local_container_exists(container_id)?;
    Ok(ContainerObject::new(container_id, "ci-container"))
}

pub fn delete_container(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
) -> Result<DeleteContainerResponse> {
    ensure_local_container_exists(container_id)?;
    Ok(DeleteContainerResponse::deleted(container_id))
}

pub fn create_container_file(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    request: &CreateContainerFileRequest,
) -> Result<ContainerFileObject> {
    ensure_local_container_exists(container_id)?;
    Ok(ContainerFileObject::new(&request.file_id, container_id))
}

pub fn list_container_files(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
) -> Result<ListContainerFilesResponse> {
    ensure_local_container_exists(container_id)?;
    Ok(ListContainerFilesResponse::new(vec![
        ContainerFileObject::new("file_1", container_id),
    ]))
}

pub fn get_container_file(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<ContainerFileObject> {
    ensure_local_container_file_exists(container_id, file_id)?;
    Ok(ContainerFileObject::new(file_id, container_id))
}

pub fn delete_container_file(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<DeleteContainerFileResponse> {
    ensure_local_container_file_exists(container_id, file_id)?;
    Ok(DeleteContainerFileResponse::deleted(file_id))
}

pub fn container_file_content(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Vec<u8>> {
    ensure_local_container_file_exists(container_id, file_id)?;
    Ok(b"CONTAINER-FILE".to_vec())
}

pub fn create_speech_response(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateSpeechRequest,
) -> Result<SpeechResponse> {
    let format =
        normalize_local_speech_format(request.response_format.as_deref().unwrap_or("wav"))?
            .to_owned();
    let bytes = fallback_speech_bytes(&format);
    Ok(SpeechResponse::new(format, STANDARD.encode(bytes)))
}

pub fn create_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<UploadObject> {
    Ok(UploadObject::with_details(
        "upload_1",
        &request.filename,
        &request.purpose,
        &request.mime_type,
        request.bytes,
        vec![],
    ))
}

pub fn create_music(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateMusicRequest,
) -> Result<MusicTracksResponse> {
    Ok(MusicTracksResponse::new(vec![
        MusicObject::new("music_1")
            .with_status("completed")
            .with_model(&request.model)
            .with_title(
                request
                    .title
                    .clone()
                    .unwrap_or_else(|| "SDKWork Track".to_owned()),
            )
            .with_audio_url("https://example.com/music.mp3")
            .with_lyrics(
                request
                    .lyrics
                    .clone()
                    .unwrap_or_else(|| "We rise with the skyline".to_owned()),
            )
            .with_duration_seconds(request.duration_seconds.unwrap_or(123.0)),
    ]))
}

pub fn list_music(_tenant_id: &str, _project_id: &str) -> Result<MusicTracksResponse> {
    Ok(MusicTracksResponse::new(vec![
        MusicObject::new("music_1")
            .with_status("completed")
            .with_model("suno-v4")
            .with_title("SDKWork Track")
            .with_audio_url("https://example.com/music.mp3")
            .with_duration_seconds(123.0),
    ]))
}

fn ensure_local_music_exists(music_id: &str) -> Result<()> {
    if music_id != "music_1" {
        bail!("music not found");
    }

    Ok(())
}

pub fn get_music(_tenant_id: &str, _project_id: &str, music_id: &str) -> Result<MusicObject> {
    ensure_local_music_exists(music_id)?;
    Ok(MusicObject::new(music_id)
        .with_status("completed")
        .with_model("suno-v4")
        .with_title("SDKWork Track")
        .with_audio_url("https://example.com/music.mp3")
        .with_duration_seconds(123.0))
}

pub fn delete_music(
    _tenant_id: &str,
    _project_id: &str,
    music_id: &str,
) -> Result<DeleteMusicResponse> {
    ensure_local_music_exists(music_id)?;
    Ok(DeleteMusicResponse::deleted(music_id))
}

pub fn music_content(_tenant_id: &str, _project_id: &str, music_id: &str) -> Result<Vec<u8>> {
    ensure_local_music_exists(music_id)?;
    Ok(vec![
        0x49, 0x44, 0x33, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21,
    ])
}

pub fn create_music_lyrics(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateMusicLyricsRequest,
) -> Result<MusicLyricsObject> {
    Ok(
        MusicLyricsObject::new("lyrics_1", "completed", &request.prompt).with_title(
            request
                .title
                .clone()
                .unwrap_or_else(|| "SDKWork Lyrics".to_owned()),
        ),
    )
}

pub fn create_video(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]))
}

pub fn list_videos(_tenant_id: &str, _project_id: &str) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]))
}

pub fn get_video(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<VideoObject> {
    ensure_local_video_exists(video_id)?;
    Ok(VideoObject::new(video_id, "https://example.com/video.mp4"))
}

fn ensure_local_video_exists(video_id: &str) -> Result<()> {
    if video_id != "video_1" {
        bail!("video not found");
    }

    Ok(())
}

pub fn delete_video(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<DeleteVideoResponse> {
    ensure_local_video_exists(video_id)?;
    Ok(DeleteVideoResponse::deleted(video_id))
}

pub fn video_content(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<Vec<u8>> {
    ensure_local_video_exists(video_id)?;
    Ok(b"VIDEO".to_vec())
}

pub fn remix_video(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_remix",
        "https://example.com/video-remix.mp4",
    )]))
}

pub fn create_video_character(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateVideoCharacterRequest,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new("char_1", &request.name))
}

pub fn list_video_characters(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
) -> Result<VideoCharactersResponse> {
    Ok(VideoCharactersResponse::new(vec![
        VideoCharacterObject::new("char_1", "Hero"),
    ]))
}

pub fn get_video_character(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    character_id: &str,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new(character_id, "Hero"))
}

pub fn get_video_character_canonical(
    _tenant_id: &str,
    _project_id: &str,
    character_id: &str,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new(character_id, "Hero"))
}

pub fn update_video_character(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    character_id: &str,
    request: &UpdateVideoCharacterRequest,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new(
        character_id,
        request.name.as_deref().unwrap_or("Hero"),
    ))
}

pub fn extend_video(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_extended",
        "https://example.com/video-extended.mp4",
    )]))
}

pub fn edit_video(
    _tenant_id: &str,
    _project_id: &str,
    _request: &EditVideoRequest,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_edited",
        "https://example.com/video-edited.mp4",
    )]))
}

pub fn extensions_video(
    _tenant_id: &str,
    _project_id: &str,
    _request: &ExtendVideoRequest,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_extended",
        "https://example.com/video-extended.mp4",
    )]))
}

pub fn create_upload_part(
    _tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<UploadPartObject> {
    ensure_local_upload_exists(&request.upload_id)?;
    Ok(UploadPartObject::new("part_1", &request.upload_id))
}

fn ensure_local_upload_exists(upload_id: &str) -> Result<()> {
    if upload_id != "upload_1" {
        bail!("upload not found");
    }

    Ok(())
}

pub fn complete_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CompleteUploadRequest,
) -> Result<UploadObject> {
    ensure_local_upload_exists(&request.upload_id)?;
    Ok(UploadObject::completed(
        &request.upload_id,
        "input.jsonl",
        "batch",
        "application/jsonl",
        0,
        request.part_ids.clone(),
    ))
}

pub fn cancel_upload(_tenant_id: &str, _project_id: &str, upload_id: &str) -> Result<UploadObject> {
    ensure_local_upload_exists(upload_id)?;
    Ok(UploadObject::cancelled(
        upload_id,
        "input.jsonl",
        "batch",
        "application/jsonl",
        0,
        vec![],
    ))
}

pub fn create_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::new("ftjob_1", model))
}

fn ensure_local_fine_tuning_job_exists(job_id: &str) -> Result<()> {
    if job_id != "ftjob_1" {
        bail!("fine tuning job not found");
    }

    Ok(())
}

fn ensure_local_fine_tuning_checkpoint_exists(checkpoint_id: &str) -> Result<()> {
    if checkpoint_id != "ft:gpt-4.1-mini:checkpoint-1" {
        bail!("fine tuning checkpoint not found");
    }

    Ok(())
}

fn ensure_local_fine_tuning_checkpoint_permission_exists(
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<()> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    if permission_id != "perm_1" {
        bail!("fine tuning checkpoint permission not found");
    }

    Ok(())
}

pub fn list_fine_tuning_jobs(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListFineTuningJobsResponse> {
    Ok(ListFineTuningJobsResponse::new(vec![
        FineTuningJobObject::new("ftjob_1", "gpt-4.1-mini"),
    ]))
}

pub fn get_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::new(job_id, "gpt-4.1-mini"))
}

pub fn cancel_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::cancelled(job_id, "gpt-4.1-mini"))
}

pub fn list_fine_tuning_job_events(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<ListFineTuningJobEventsResponse> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(ListFineTuningJobEventsResponse::new(vec![
        FineTuningJobEventObject::new("ftevent_1", "info", "job queued"),
    ]))
}

pub fn list_fine_tuning_job_checkpoints(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<ListFineTuningJobCheckpointsResponse> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(ListFineTuningJobCheckpointsResponse::new(vec![
        FineTuningJobCheckpointObject::new("ftckpt_1", "ft:gpt-4.1-mini:checkpoint-1"),
    ]))
}

pub fn pause_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::paused(job_id, "gpt-4.1-mini"))
}

pub fn resume_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::running(job_id, "gpt-4.1-mini"))
}

pub fn create_fine_tuning_checkpoint_permissions(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Result<ListFineTuningCheckpointPermissionsResponse> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    let project_id = request
        .project_ids
        .first()
        .cloned()
        .unwrap_or_else(|| "project-2".to_owned());
    Ok(ListFineTuningCheckpointPermissionsResponse::new(vec![
        FineTuningCheckpointPermissionObject::new("perm_1", project_id),
    ]))
}

pub fn list_fine_tuning_checkpoint_permissions(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
) -> Result<ListFineTuningCheckpointPermissionsResponse> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    Ok(ListFineTuningCheckpointPermissionsResponse::new(vec![
        FineTuningCheckpointPermissionObject::new("perm_1", "project-2"),
    ]))
}

pub fn delete_fine_tuning_checkpoint_permission(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<DeleteFineTuningCheckpointPermissionResponse> {
    ensure_local_fine_tuning_checkpoint_permission_exists(checkpoint_id, permission_id)?;
    Ok(DeleteFineTuningCheckpointPermissionResponse::deleted(
        permission_id,
    ))
}

pub fn create_assistant(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
    model: &str,
) -> Result<AssistantObject> {
    Ok(AssistantObject::new("asst_1", name, model))
}

pub fn list_assistants(_tenant_id: &str, _project_id: &str) -> Result<ListAssistantsResponse> {
    Ok(ListAssistantsResponse::new(vec![AssistantObject::new(
        "asst_1", "Support", "gpt-4.1",
    )]))
}

fn ensure_local_assistant_exists(assistant_id: &str) -> Result<()> {
    if assistant_id != "asst_1" {
        bail!("assistant not found");
    }

    Ok(())
}

pub fn get_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<AssistantObject> {
    ensure_local_assistant_exists(assistant_id)?;
    Ok(AssistantObject::new(assistant_id, "Support", "gpt-4.1"))
}

pub fn update_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    name: &str,
) -> Result<AssistantObject> {
    ensure_local_assistant_exists(assistant_id)?;
    Ok(AssistantObject::new(assistant_id, name, "gpt-4.1"))
}

pub fn delete_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<DeleteAssistantResponse> {
    ensure_local_assistant_exists(assistant_id)?;
    Ok(DeleteAssistantResponse::deleted(assistant_id))
}

pub fn create_thread(_tenant_id: &str, _project_id: &str) -> Result<ThreadObject> {
    Ok(ThreadObject::new("thread_1"))
}

fn ensure_local_thread_exists(thread_id: &str) -> Result<()> {
    if thread_id != "thread_1" {
        bail!("thread not found");
    }

    Ok(())
}

fn ensure_local_thread_message_exists(thread_id: &str, message_id: &str) -> Result<()> {
    ensure_local_thread_exists(thread_id)?;
    if message_id != "msg_1" {
        bail!("thread message not found");
    }

    Ok(())
}

fn ensure_local_thread_run_exists(thread_id: &str, run_id: &str) -> Result<()> {
    ensure_local_thread_exists(thread_id)?;
    if run_id != "run_1" {
        bail!("run not found");
    }

    Ok(())
}

pub fn get_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ThreadObject::new(thread_id))
}

pub fn update_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ThreadObject::new(thread_id))
}

pub fn delete_thread(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<DeleteThreadResponse> {
    ensure_local_thread_exists(thread_id)?;
    Ok(DeleteThreadResponse::deleted(thread_id))
}

pub fn create_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    role: &str,
    text: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ThreadMessageObject::text("msg_1", thread_id, role, text))
}

pub fn list_thread_messages(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListThreadMessagesResponse> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ListThreadMessagesResponse::new(vec![
        ThreadMessageObject::text("msg_1", thread_id, "assistant", "hello"),
    ]))
}

pub fn get_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    Ok(ThreadMessageObject::text(
        message_id,
        thread_id,
        "assistant",
        "hello",
    ))
}

pub fn update_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    Ok(ThreadMessageObject::text(
        message_id,
        thread_id,
        "assistant",
        "hello",
    ))
}

pub fn delete_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<DeleteThreadMessageResponse> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    Ok(DeleteThreadMessageResponse::deleted(message_id))
}

pub fn create_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> Result<RunObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(RunObject::queued(
        "run_1",
        thread_id,
        assistant_id,
        model.unwrap_or("gpt-4.1"),
    ))
}

pub fn create_thread_and_run(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<RunObject> {
    if assistant_id.trim().is_empty() {
        bail!("Thread and run assistant_id is required.");
    }

    Ok(RunObject::queued(
        "run_1",
        "thread_1",
        assistant_id,
        "gpt-4.1",
    ))
}

pub fn list_thread_runs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListRunsResponse> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ListRunsResponse::new(vec![RunObject::queued(
        "run_1", thread_id, "asst_1", "gpt-4.1",
    )]))
}

pub fn get_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::in_progress(
        run_id, thread_id, "asst_1", "gpt-4.1",
    ))
}

pub fn update_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::with_metadata(
        run_id,
        thread_id,
        "asst_1",
        "gpt-4.1",
        "in_progress",
        serde_json::json!({"priority":"high"}),
    ))
}

pub fn cancel_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::cancelled(run_id, thread_id, "asst_1", "gpt-4.1"))
}

pub fn submit_thread_run_tool_outputs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    _tool_outputs: Vec<(&str, &str)>,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::queued(run_id, thread_id, "asst_1", "gpt-4.1"))
}

pub fn list_thread_run_steps(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<ListRunStepsResponse> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(ListRunStepsResponse::new(vec![
        RunStepObject::message_creation("step_1", thread_id, run_id, "asst_1", "msg_1"),
    ]))
}

pub fn get_thread_run_step(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<RunStepObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    if step_id != "step_1" {
        bail!("run step not found");
    }
    Ok(RunStepObject::message_creation(
        step_id, thread_id, run_id, "asst_1", "msg_1",
    ))
}

pub fn create_webhook(
    _tenant_id: &str,
    _project_id: &str,
    url: &str,
    _events: &[String],
) -> Result<WebhookObject> {
    Ok(WebhookObject::new("wh_1", url))
}

pub fn list_webhooks(_tenant_id: &str, _project_id: &str) -> Result<ListWebhooksResponse> {
    Ok(ListWebhooksResponse::new(vec![WebhookObject::new(
        "wh_1",
        "https://example.com/webhook",
    )]))
}

fn ensure_local_webhook_exists(webhook_id: &str) -> Result<()> {
    if webhook_id != "wh_1" {
        bail!("webhook not found");
    }

    Ok(())
}

pub fn get_webhook(_tenant_id: &str, _project_id: &str, webhook_id: &str) -> Result<WebhookObject> {
    ensure_local_webhook_exists(webhook_id)?;
    Ok(WebhookObject::new(
        webhook_id,
        "https://example.com/webhook",
    ))
}

pub fn update_webhook(
    _tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
    url: &str,
) -> Result<WebhookObject> {
    ensure_local_webhook_exists(webhook_id)?;
    Ok(WebhookObject::new(webhook_id, url))
}

pub fn delete_webhook(
    _tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<DeleteWebhookResponse> {
    ensure_local_webhook_exists(webhook_id)?;
    Ok(DeleteWebhookResponse::deleted(webhook_id))
}

pub fn create_realtime_session(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<RealtimeSessionObject> {
    Ok(RealtimeSessionObject::new("sess_1", model))
}

pub fn create_eval(_tenant_id: &str, _project_id: &str, name: &str) -> Result<EvalObject> {
    Ok(EvalObject::new("eval_1", name))
}

pub fn list_evals(_tenant_id: &str, _project_id: &str) -> Result<ListEvalsResponse> {
    Ok(ListEvalsResponse::new(vec![EvalObject::new(
        "eval_1",
        "qa-benchmark",
    )]))
}

fn ensure_local_eval_exists(eval_id: &str) -> Result<()> {
    if eval_id != "eval_1" {
        bail!("eval not found");
    }

    Ok(())
}

fn ensure_local_eval_run_exists(eval_id: &str, run_id: &str) -> Result<()> {
    ensure_local_eval_exists(eval_id)?;
    if run_id != "run_1" {
        bail!("eval run not found");
    }

    Ok(())
}

fn ensure_local_eval_run_output_item_exists(
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<()> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    if output_item_id != "output_item_1" {
        bail!("eval run output item not found");
    }

    Ok(())
}

pub fn get_eval(_tenant_id: &str, _project_id: &str, eval_id: &str) -> Result<EvalObject> {
    ensure_local_eval_exists(eval_id)?;
    Ok(EvalObject::new(eval_id, "qa-benchmark"))
}

pub fn update_eval(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Result<EvalObject> {
    ensure_local_eval_exists(eval_id)?;
    Ok(EvalObject::new(
        eval_id,
        request.name.as_deref().unwrap_or("qa-benchmark"),
    ))
}

pub fn delete_eval(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<DeleteEvalResponse> {
    ensure_local_eval_exists(eval_id)?;
    Ok(DeleteEvalResponse::deleted(eval_id))
}

pub fn list_eval_runs(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<ListEvalRunsResponse> {
    ensure_local_eval_exists(eval_id)?;
    Ok(ListEvalRunsResponse::new(vec![EvalRunObject::completed(
        "run_1",
    )]))
}

pub fn create_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    _request: &CreateEvalRunRequest,
) -> Result<EvalRunObject> {
    ensure_local_eval_exists(eval_id)?;
    Ok(EvalRunObject::queued("run_1"))
}

pub fn get_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<EvalRunObject> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(EvalRunObject::completed(run_id))
}

pub fn delete_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<DeleteEvalRunResponse> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(DeleteEvalRunResponse::deleted(run_id))
}

pub fn cancel_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<EvalRunObject> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(EvalRunObject {
        id: run_id.to_owned(),
        object: "eval.run",
        status: "cancelled",
    })
}

pub fn list_eval_run_output_items(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<ListEvalRunOutputItemsResponse> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(ListEvalRunOutputItemsResponse::new(vec![
        EvalRunOutputItemObject::passed("output_item_1"),
    ]))
}

pub fn get_eval_run_output_item(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<EvalRunOutputItemObject> {
    ensure_local_eval_run_output_item_exists(eval_id, run_id, output_item_id)?;
    Ok(EvalRunOutputItemObject::passed(output_item_id))
}

pub fn create_batch(
    _tenant_id: &str,
    _project_id: &str,
    endpoint: &str,
    input_file_id: &str,
) -> Result<BatchObject> {
    Ok(BatchObject::new("batch_1", endpoint, input_file_id))
}

pub fn list_batches(_tenant_id: &str, _project_id: &str) -> Result<ListBatchesResponse> {
    Ok(ListBatchesResponse::new(vec![BatchObject::new(
        "batch_1",
        "/v1/responses",
        "file_1",
    )]))
}

fn ensure_local_batch_exists(batch_id: &str) -> Result<()> {
    if batch_id != "batch_1" {
        bail!("batch not found");
    }

    Ok(())
}

pub fn get_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    ensure_local_batch_exists(batch_id)?;
    Ok(BatchObject::new(batch_id, "/v1/responses", "file_1"))
}

pub fn cancel_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    ensure_local_batch_exists(batch_id)?;
    Ok(BatchObject::cancelled(batch_id, "/v1/responses", "file_1"))
}

pub fn create_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    Ok(VectorStoreObject::new("vs_1", name))
}

pub fn list_vector_stores(_tenant_id: &str, _project_id: &str) -> Result<ListVectorStoresResponse> {
    Ok(ListVectorStoresResponse::new(vec![VectorStoreObject::new(
        "vs_1", "kb-main",
    )]))
}

fn ensure_local_vector_store_exists(vector_store_id: &str) -> Result<()> {
    if vector_store_id != "vs_1" {
        bail!("vector store not found");
    }

    Ok(())
}

pub fn get_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<VectorStoreObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(VectorStoreObject::new(vector_store_id, "kb-main"))
}

pub fn update_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(VectorStoreObject::new(vector_store_id, name))
}

pub fn delete_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<DeleteVectorStoreResponse> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(DeleteVectorStoreResponse::deleted(vector_store_id))
}

pub fn search_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    query: &str,
) -> Result<SearchVectorStoreResponse> {
    Ok(SearchVectorStoreResponse::sample(query))
}

fn ensure_local_vector_store_file_exists(vector_store_id: &str, file_id: &str) -> Result<()> {
    if vector_store_id != "vs_1" || file_id != "file_1" {
        bail!("vector store file not found");
    }

    Ok(())
}

fn ensure_local_vector_store_file_batch_exists(
    vector_store_id: &str,
    batch_id: &str,
) -> Result<()> {
    if vector_store_id != "vs_1" || batch_id != "vsfb_1" {
        bail!("vector store file batch not found");
    }

    Ok(())
}

pub fn create_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    Ok(VectorStoreFileObject::new(file_id))
}

pub fn list_vector_store_files(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    Ok(ListVectorStoreFilesResponse::new(vec![
        VectorStoreFileObject::new("file_1"),
    ]))
}

pub fn get_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    ensure_local_vector_store_file_exists(vector_store_id, file_id)?;
    Ok(VectorStoreFileObject::new(file_id))
}

pub fn delete_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<DeleteVectorStoreFileResponse> {
    ensure_local_vector_store_file_exists(vector_store_id, file_id)?;
    Ok(DeleteVectorStoreFileResponse::deleted(file_id))
}

pub fn create_vector_store_file_batch<T: AsRef<str>>(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_ids: &[T],
) -> Result<VectorStoreFileBatchObject> {
    let _ = file_ids.first().map(AsRef::as_ref);
    Ok(VectorStoreFileBatchObject::new("vsfb_1"))
}

pub fn get_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    Ok(VectorStoreFileBatchObject::new(batch_id))
}

pub fn cancel_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    Ok(VectorStoreFileBatchObject::cancelled(batch_id))
}

pub fn list_vector_store_file_batch_files(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    Ok(ListVectorStoreFilesResponse::new(vec![
        VectorStoreFileObject::new("file_1"),
    ]))
}

pub fn builtin_extension_host() -> ExtensionHost {
    let mut host = ExtensionHost::new();
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openai.official",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        )
        .with_supported_modality(ExtensionModality::Image)
        .with_supported_modality(ExtensionModality::Audio)
        .with_supported_modality(ExtensionModality::Video)
        .with_supported_modality(ExtensionModality::File)
        .with_supported_modality(ExtensionModality::Embedding),
        "openai",
        |base_url| Box::new(OpenAiProviderAdapter::new(base_url)),
    ));
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openrouter",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        )
        .with_supported_modality(ExtensionModality::Image)
        .with_supported_modality(ExtensionModality::Audio)
        .with_supported_modality(ExtensionModality::Video)
        .with_supported_modality(ExtensionModality::File)
        .with_supported_modality(ExtensionModality::Embedding),
        "openrouter",
        |base_url| Box::new(OpenRouterProviderAdapter::new(base_url)),
    ));
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.ollama",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        )
        .with_supported_modality(ExtensionModality::Image)
        .with_supported_modality(ExtensionModality::Audio)
        .with_supported_modality(ExtensionModality::Video)
        .with_supported_modality(ExtensionModality::File)
        .with_supported_modality(ExtensionModality::Embedding),
        "ollama",
        |base_url| Box::new(OllamaProviderAdapter::new(base_url)),
    ));
    host
}

fn provider_runtime_key(provider: &ProxyProvider) -> &str {
    &provider.extension_id
}

fn configured_extension_host() -> Result<ExtensionHost> {
    let policy = configured_extension_discovery_policy();
    let cache_key = ConfiguredExtensionHostCacheKey::from(&policy);
    let cache = CONFIGURED_EXTENSION_HOST_CACHE.get_or_init(|| Mutex::new(None));
    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some(cached) = cache_guard.as_ref() {
        if cached.key == cache_key {
            return Ok(cached.host.clone());
        }
    }

    let built = build_configured_extension_host(&policy)?;
    *cache_guard = Some(CachedConfiguredExtensionHost {
        key: cache_key,
        host: built.host.clone(),
    });
    Ok(built.host)
}

pub fn reload_configured_extension_host() -> Result<ConfiguredExtensionHostReloadReport> {
    reload_extension_host_with_scope(&ConfiguredExtensionHostReloadScope::All)
}

pub fn reload_extension_host_with_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<ConfiguredExtensionHostReloadReport> {
    let policy = configured_extension_discovery_policy();
    reload_extension_host_with_policy_and_scope(&policy, scope)
}

pub fn reload_extension_host_with_policy(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<ConfiguredExtensionHostReloadReport> {
    reload_extension_host_with_policy_and_scope(policy, &ConfiguredExtensionHostReloadScope::All)
}

fn reload_extension_host_with_policy_and_scope(
    policy: &ExtensionDiscoveryPolicy,
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<ConfiguredExtensionHostReloadReport> {
    let discovered_packages = discover_extension_packages(policy)?;
    let cache_key = ConfiguredExtensionHostCacheKey::from(policy);

    apply_configured_extension_host_reload_scope(scope)?;
    let built = build_configured_extension_host_from_packages(discovered_packages, policy);
    let cache = CONFIGURED_EXTENSION_HOST_CACHE.get_or_init(|| Mutex::new(None));
    let mut cache_guard = match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *cache_guard = Some(CachedConfiguredExtensionHost {
        key: cache_key,
        host: built.host.clone(),
    });

    Ok(ConfiguredExtensionHostReloadReport {
        discovered_package_count: built.discovered_package_count,
        loadable_package_count: built.loadable_package_count,
    })
}

fn apply_configured_extension_host_reload_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<()> {
    match scope {
        ConfiguredExtensionHostReloadScope::All => {
            shutdown_all_connector_runtimes()?;
            shutdown_all_native_dynamic_runtimes()?;
        }
        ConfiguredExtensionHostReloadScope::Extension { extension_id } => {
            shutdown_connector_runtimes_for_extension(extension_id)?;
            shutdown_native_dynamic_runtimes_for_extension(extension_id)?;
        }
        ConfiguredExtensionHostReloadScope::Instance { instance_id } => {
            shutdown_connector_runtime(instance_id)?;
        }
    }

    Ok(())
}

pub fn start_configured_extension_hot_reload_supervision(
    interval_secs: u64,
) -> Option<JoinHandle<()>> {
    if interval_secs == 0 {
        return None;
    }

    let initial_state = match configured_extension_host_watch_state() {
        Ok(state) => Some(state),
        Err(error) => {
            eprintln!("extension hot reload watch startup state capture failed: {error}");
            None
        }
    };

    Some(tokio::spawn(async move {
        let mut previous_state = initial_state;

        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        interval.tick().await;

        loop {
            interval.tick().await;

            let next_state = match configured_extension_host_watch_state() {
                Ok(state) => state,
                Err(error) => {
                    eprintln!("extension hot reload watch state capture failed: {error}");
                    continue;
                }
            };

            if previous_state.as_ref() == Some(&next_state) {
                continue;
            }

            match reload_configured_extension_host() {
                Ok(report) => {
                    eprintln!(
                        "extension hot reload applied: discovered_package_count={} loadable_package_count={}",
                        report.discovered_package_count, report.loadable_package_count
                    );
                    previous_state = Some(next_state);
                }
                Err(error) => {
                    eprintln!("extension hot reload failed: {error}");
                }
            }
        }
    }))
}

fn build_configured_extension_host(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<BuiltConfiguredExtensionHost> {
    let packages = discover_extension_packages(policy)?;
    Ok(build_configured_extension_host_from_packages(
        packages, policy,
    ))
}

fn build_configured_extension_host_from_packages(
    packages: Vec<DiscoveredExtensionPackage>,
    policy: &ExtensionDiscoveryPolicy,
) -> BuiltConfiguredExtensionHost {
    let mut host = builtin_extension_host();
    let discovered_package_count = packages.len();
    let mut loadable_package_count = 0;

    for package in packages {
        let trust = verify_discovered_extension_package_trust(&package, policy);
        if !trust.load_allowed {
            continue;
        }
        if register_discovered_extension(&mut host, package) {
            loadable_package_count += 1;
        }
    }

    BuiltConfiguredExtensionHost {
        host,
        discovered_package_count,
        loadable_package_count,
    }
}

fn configured_extension_host_watch_state() -> Result<ConfiguredExtensionHostWatchState> {
    let policy = configured_extension_discovery_policy();
    Ok(ConfiguredExtensionHostWatchState {
        key: ConfiguredExtensionHostCacheKey::from(&policy),
        fingerprint: extension_tree_fingerprint(&policy.search_paths)?,
    })
}

fn extension_tree_fingerprint(search_paths: &[PathBuf]) -> Result<Vec<String>> {
    let mut fingerprint = Vec::new();
    for path in search_paths {
        collect_extension_tree_fingerprint(path, &mut fingerprint)?;
    }
    fingerprint.sort();
    Ok(fingerprint)
}

fn collect_extension_tree_fingerprint(path: &Path, fingerprint: &mut Vec<String>) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) => {
            fingerprint.push(fingerprint_entry(path, &metadata));
            if metadata.is_dir() {
                let mut children = fs::read_dir(path)
                    .with_context(|| {
                        format!("failed to read extension directory {}", path.display())
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .with_context(|| {
                        format!("failed to enumerate extension directory {}", path.display())
                    })?;
                children.sort_by_key(|entry| entry.path());
                for child in children {
                    collect_extension_tree_fingerprint(&child.path(), fingerprint)?;
                }
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fingerprint.push(format!("missing|{}", path.display()));
            Ok(())
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to stat extension path {}", path.display()))
        }
    }
}

fn fingerprint_entry(path: &Path, metadata: &fs::Metadata) -> String {
    let kind = if metadata.is_dir() { "dir" } else { "file" };
    format!(
        "{kind}|{}|{}|{}",
        path.display(),
        metadata.len(),
        metadata_modified_ms(metadata),
    )
}

fn metadata_modified_ms(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn configured_extension_discovery_policy() -> ExtensionDiscoveryPolicy {
    let search_paths = std::env::var_os("SDKWORK_EXTENSION_PATHS")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut policy = ExtensionDiscoveryPolicy::new(search_paths)
        .with_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            true,
        ))
        .with_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            true,
        ));
    for (publisher, public_key) in env_trusted_signers("SDKWORK_EXTENSION_TRUSTED_SIGNERS") {
        policy = policy.with_trusted_signer(publisher, public_key);
    }
    policy
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}

fn env_trusted_signers(key: &str) -> Vec<(String, String)> {
    std::env::var(key)
        .ok()
        .map(|value| parse_trusted_signers(&value))
        .unwrap_or_default()
}

fn parse_trusted_signers(value: &str) -> Vec<(String, String)> {
    value
        .split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (publisher, public_key) = entry.split_once('=')?;
            let publisher = publisher.trim();
            let public_key = public_key.trim();
            if publisher.is_empty() || public_key.is_empty() {
                return None;
            }
            Some((publisher.to_owned(), public_key.to_owned()))
        })
        .collect()
}

fn register_discovered_extension(
    host: &mut ExtensionHost,
    package: DiscoveredExtensionPackage,
) -> bool {
    if host.manifest(&package.manifest.id).is_some() {
        return false;
    }

    if package.manifest.runtime == ExtensionRuntime::NativeDynamic {
        return host
            .register_discovered_native_dynamic_provider(package)
            .is_ok();
    }

    match (
        package.manifest.kind.clone(),
        package.manifest.protocol.clone(),
    ) {
        (ExtensionKind::Provider, Some(ExtensionProtocol::OpenAi)) => {
            host.register_discovered_provider(package, "openai", |base_url| {
                Box::new(OpenAiProviderAdapter::new(base_url))
            });
        }
        (ExtensionKind::Provider, Some(ExtensionProtocol::OpenRouter)) => {
            host.register_discovered_provider(package, "openrouter", |base_url| {
                Box::new(OpenRouterProviderAdapter::new(base_url))
            });
        }
        (ExtensionKind::Provider, Some(ExtensionProtocol::Ollama)) => {
            host.register_discovered_provider(package, "ollama", |base_url| {
                Box::new(OllamaProviderAdapter::new(base_url))
            });
        }
        _ => host.register_discovered_manifest(package),
    }

    true
}

async fn execute_json_provider_request_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    let retry_policy = gateway_upstream_retry_policy(&request, None);
    execute_json_provider_request_for_provider_with_options(
        store,
        provider,
        api_key,
        request,
        &options,
        retry_policy,
    )
    .await
}

async fn execute_json_provider_request_for_provider_with_options(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<Value>> {
    let descriptor =
        provider_execution_descriptor_for_provider(store, provider, api_key.to_owned()).await?;
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    let host = build_extension_host_from_store(store).await?;
    let Some(adapter) = host.resolve_provider(&descriptor.runtime_key, descriptor.base_url.clone())
    else {
        return Ok(None);
    };

    let capability = provider_request_metric_capability(&request);
    let mut attempt = 1usize;

    loop {
        record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
        match execute_provider_request_with_execution_context(
            adapter.as_ref(),
            Some(&descriptor.provider_id),
            &descriptor.api_key,
            request,
            options,
        )
        .await
        {
            Ok(response) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store,
                    &descriptor,
                    true,
                    capability,
                    None,
                )
                .await;
                return Ok(response.into_json());
            }
            Err(error) => {
                record_gateway_execution_context_failure_from_error(
                    capability,
                    &descriptor.provider_id,
                    &error,
                );
                let retryable = gateway_upstream_error_is_retryable(&error);
                let can_retry =
                    retry_policy.enabled() && retryable && attempt < retry_policy.max_attempts;
                if can_retry {
                    let retry_reason = gateway_retry_reason_for_error(&error);
                    let retry_delay = gateway_retry_delay_for_error(retry_policy, attempt, &error);
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "scheduled",
                        retry_reason,
                        Some(retry_delay.source),
                        Some(retry_delay.delay.as_millis() as u64),
                    );
                    tokio::time::sleep(retry_delay.delay).await;
                    attempt += 1;
                    continue;
                }

                if retry_policy.enabled() && retryable && attempt > 1 {
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "exhausted",
                        gateway_retry_reason_for_error(&error),
                        None,
                        None,
                    );
                }
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "failure");
                if gateway_error_impacts_provider_health(&error) {
                    persist_gateway_execution_health_snapshot(
                        store,
                        &descriptor,
                        false,
                        capability,
                        Some(&error),
                    )
                    .await;
                }
                return Err(error);
            }
        }
    }
}

async fn execute_stream_provider_request_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    let retry_policy = gateway_upstream_retry_policy(&request, None);
    execute_stream_provider_request_for_provider_with_options(
        store,
        provider,
        api_key,
        request,
        &options,
        retry_policy,
    )
    .await
}

async fn execute_stream_provider_request_for_provider_with_options(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
    retry_policy: GatewayUpstreamRetryPolicy,
) -> Result<Option<ProviderStreamOutput>> {
    let descriptor =
        provider_execution_descriptor_for_provider(store, provider, api_key.to_owned()).await?;
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    let host = build_extension_host_from_store(store).await?;
    let Some(adapter) = host.resolve_provider(&descriptor.runtime_key, descriptor.base_url.clone())
    else {
        return Ok(None);
    };

    let capability = provider_request_metric_capability(&request);
    let mut attempt = 1usize;

    loop {
        record_gateway_upstream_outcome(capability, &descriptor.provider_id, "attempt");
        match execute_provider_request_with_execution_context(
            adapter.as_ref(),
            Some(&descriptor.provider_id),
            &descriptor.api_key,
            request,
            options,
        )
        .await
        {
            Ok(response) => {
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "success");
                persist_gateway_execution_health_snapshot(
                    store,
                    &descriptor,
                    true,
                    capability,
                    None,
                )
                .await;
                return Ok(response.into_stream());
            }
            Err(error) => {
                record_gateway_execution_context_failure_from_error(
                    capability,
                    &descriptor.provider_id,
                    &error,
                );
                let retryable = gateway_upstream_error_is_retryable(&error);
                let can_retry =
                    retry_policy.enabled() && retryable && attempt < retry_policy.max_attempts;
                if can_retry {
                    let retry_reason = gateway_retry_reason_for_error(&error);
                    let retry_delay = gateway_retry_delay_for_error(retry_policy, attempt, &error);
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "scheduled",
                        retry_reason,
                        Some(retry_delay.source),
                        Some(retry_delay.delay.as_millis() as u64),
                    );
                    tokio::time::sleep(retry_delay.delay).await;
                    attempt += 1;
                    continue;
                }

                if retry_policy.enabled() && retryable && attempt > 1 {
                    record_gateway_upstream_retry_with_detail(
                        capability,
                        &descriptor.provider_id,
                        "exhausted",
                        gateway_retry_reason_for_error(&error),
                        None,
                        None,
                    );
                }
                record_gateway_upstream_outcome(capability, &descriptor.provider_id, "failure");
                if gateway_error_impacts_provider_health(&error) {
                    persist_gateway_execution_health_snapshot(
                        store,
                        &descriptor,
                        false,
                        capability,
                        Some(&error),
                    )
                    .await;
                }
                return Err(error);
            }
        }
    }
}

const GATEWAY_UPSTREAM_METRICS_SERVICE: &str = "gateway";

fn record_gateway_upstream_outcome(capability: &str, provider_id: &str, outcome: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE).record_upstream_outcome(
        capability,
        provider_id,
        outcome,
    );
}

fn record_gateway_upstream_retry_with_detail(
    capability: &str,
    provider_id: &str,
    outcome: &str,
    reason: &str,
    delay_source: Option<&str>,
    delay_ms: Option<u64>,
) {
    let registry = HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE);
    registry.record_upstream_retry(capability, provider_id, outcome);
    registry.record_upstream_retry_reason(capability, provider_id, outcome, reason);
    if let (Some(delay_source), Some(delay_ms)) = (delay_source, delay_ms) {
        registry.record_upstream_retry_delay(capability, provider_id, delay_source, delay_ms);
    }
}

fn record_gateway_provider_health(
    provider_id: &str,
    runtime: &str,
    healthy: bool,
    observed_at_ms: u64,
) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE).record_provider_health(
        provider_id,
        runtime,
        healthy,
        observed_at_ms,
    );
}

fn record_gateway_provider_health_persist_failure(provider_id: &str, runtime: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE)
        .record_provider_health_persist_failure(provider_id, runtime);
}

fn record_gateway_provider_health_recovery_probe(provider_id: &str, outcome: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE)
        .record_provider_health_recovery_probe(provider_id, outcome);
}

fn record_gateway_execution_context_failure(capability: &str, provider_id: &str, reason: &str) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE)
        .record_gateway_execution_context_failure(capability, provider_id, reason);
}

fn record_gateway_execution_context_failure_from_error(
    capability: &str,
    provider_id: &str,
    error: &anyhow::Error,
) {
    let Some(error) = gateway_execution_context_error(error) else {
        return;
    };
    record_gateway_execution_context_failure(
        capability,
        provider_id,
        gateway_execution_context_metric_reason(error),
    );
}

fn record_gateway_execution_failover(
    capability: &str,
    from_provider_id: &str,
    to_provider_id: &str,
    outcome: &str,
) {
    HttpMetricsRegistry::new(GATEWAY_UPSTREAM_METRICS_SERVICE).record_gateway_failover(
        capability,
        from_provider_id,
        to_provider_id,
        outcome,
    );
}

fn record_gateway_recovery_probe_from_decision(decision: &RoutingDecision) {
    let Some(probe) = decision.provider_health_recovery_probe.as_ref() else {
        return;
    };
    record_gateway_provider_health_recovery_probe(&probe.provider_id, probe.outcome.as_str());
}

fn provider_request_metric_capability(request: &ProviderRequest<'_>) -> &'static str {
    match request {
        ProviderRequest::ModelsList
        | ProviderRequest::ModelsRetrieve(_)
        | ProviderRequest::ModelsDelete(_) => "models",
        ProviderRequest::ChatCompletions(_)
        | ProviderRequest::ChatCompletionsStream(_)
        | ProviderRequest::ChatCompletionsList
        | ProviderRequest::ChatCompletionsRetrieve(_)
        | ProviderRequest::ChatCompletionsUpdate(_, _)
        | ProviderRequest::ChatCompletionsDelete(_)
        | ProviderRequest::ChatCompletionsMessagesList(_) => "chat_completion",
        ProviderRequest::Completions(_) => "completion",
        ProviderRequest::Containers(_)
        | ProviderRequest::ContainersList
        | ProviderRequest::ContainersRetrieve(_)
        | ProviderRequest::ContainersDelete(_)
        | ProviderRequest::ContainerFiles(_, _)
        | ProviderRequest::ContainerFilesList(_)
        | ProviderRequest::ContainerFilesRetrieve(_, _)
        | ProviderRequest::ContainerFilesDelete(_, _)
        | ProviderRequest::ContainerFilesContent(_, _) => "containers",
        ProviderRequest::Threads(_)
        | ProviderRequest::ThreadsRetrieve(_)
        | ProviderRequest::ThreadsUpdate(_, _)
        | ProviderRequest::ThreadsDelete(_)
        | ProviderRequest::ThreadMessages(_, _)
        | ProviderRequest::ThreadMessagesList(_)
        | ProviderRequest::ThreadMessagesRetrieve(_, _)
        | ProviderRequest::ThreadMessagesUpdate(_, _, _)
        | ProviderRequest::ThreadMessagesDelete(_, _)
        | ProviderRequest::ThreadRuns(_, _)
        | ProviderRequest::ThreadRunsList(_)
        | ProviderRequest::ThreadRunsRetrieve(_, _)
        | ProviderRequest::ThreadRunsUpdate(_, _, _)
        | ProviderRequest::ThreadRunsCancel(_, _)
        | ProviderRequest::ThreadRunsSubmitToolOutputs(_, _, _)
        | ProviderRequest::ThreadRunStepsList(_, _)
        | ProviderRequest::ThreadRunStepsRetrieve(_, _, _)
        | ProviderRequest::ThreadsRuns(_) => "threads",
        ProviderRequest::Conversations(_)
        | ProviderRequest::ConversationsList
        | ProviderRequest::ConversationsRetrieve(_)
        | ProviderRequest::ConversationsUpdate(_, _)
        | ProviderRequest::ConversationsDelete(_)
        | ProviderRequest::ConversationItems(_, _)
        | ProviderRequest::ConversationItemsList(_)
        | ProviderRequest::ConversationItemsRetrieve(_, _)
        | ProviderRequest::ConversationItemsDelete(_, _) => "conversations",
        ProviderRequest::Responses(_)
        | ProviderRequest::ResponsesStream(_)
        | ProviderRequest::ResponsesInputTokens(_)
        | ProviderRequest::ResponsesRetrieve(_)
        | ProviderRequest::ResponsesDelete(_)
        | ProviderRequest::ResponsesInputItemsList(_)
        | ProviderRequest::ResponsesCancel(_)
        | ProviderRequest::ResponsesCompact(_) => "responses",
        ProviderRequest::Embeddings(_) => "embeddings",
        ProviderRequest::Moderations(_) => "moderations",
        ProviderRequest::Music(_)
        | ProviderRequest::MusicList
        | ProviderRequest::MusicRetrieve(_)
        | ProviderRequest::MusicDelete(_)
        | ProviderRequest::MusicContent(_)
        | ProviderRequest::MusicLyrics(_) => "music",
        ProviderRequest::ImagesGenerations(_)
        | ProviderRequest::ImagesEdits(_)
        | ProviderRequest::ImagesVariations(_) => "images",
        ProviderRequest::AudioTranscriptions(_)
        | ProviderRequest::AudioTranslations(_)
        | ProviderRequest::AudioSpeech(_)
        | ProviderRequest::AudioVoicesList
        | ProviderRequest::AudioVoiceConsents(_) => "audio",
        ProviderRequest::Files(_)
        | ProviderRequest::FilesList
        | ProviderRequest::FilesRetrieve(_)
        | ProviderRequest::FilesDelete(_)
        | ProviderRequest::FilesContent(_)
        | ProviderRequest::Uploads(_)
        | ProviderRequest::UploadParts(_)
        | ProviderRequest::UploadComplete(_)
        | ProviderRequest::UploadCancel(_) => "files",
        ProviderRequest::FineTuningJobs(_)
        | ProviderRequest::FineTuningJobsList
        | ProviderRequest::FineTuningJobsRetrieve(_)
        | ProviderRequest::FineTuningJobsCancel(_)
        | ProviderRequest::FineTuningJobsEvents(_)
        | ProviderRequest::FineTuningJobsCheckpoints(_)
        | ProviderRequest::FineTuningJobsPause(_)
        | ProviderRequest::FineTuningJobsResume(_)
        | ProviderRequest::FineTuningCheckpointPermissions(_, _)
        | ProviderRequest::FineTuningCheckpointPermissionsList(_)
        | ProviderRequest::FineTuningCheckpointPermissionsDelete(_, _) => "fine_tuning",
        ProviderRequest::Assistants(_)
        | ProviderRequest::AssistantsList
        | ProviderRequest::AssistantsRetrieve(_)
        | ProviderRequest::AssistantsUpdate(_, _)
        | ProviderRequest::AssistantsDelete(_) => "assistants",
        ProviderRequest::RealtimeSessions(_) => "realtime",
        ProviderRequest::Evals(_)
        | ProviderRequest::EvalsList
        | ProviderRequest::EvalsRetrieve(_)
        | ProviderRequest::EvalsUpdate(_, _)
        | ProviderRequest::EvalsDelete(_)
        | ProviderRequest::EvalRunsList(_)
        | ProviderRequest::EvalRuns(_, _)
        | ProviderRequest::EvalRunsRetrieve(_, _)
        | ProviderRequest::EvalRunsDelete(_, _)
        | ProviderRequest::EvalRunsCancel(_, _)
        | ProviderRequest::EvalRunOutputItemsList(_, _)
        | ProviderRequest::EvalRunOutputItemsRetrieve(_, _, _) => "evals",
        ProviderRequest::Batches(_)
        | ProviderRequest::BatchesList
        | ProviderRequest::BatchesRetrieve(_)
        | ProviderRequest::BatchesCancel(_) => "batches",
        ProviderRequest::VectorStores(_)
        | ProviderRequest::VectorStoresList
        | ProviderRequest::VectorStoresRetrieve(_)
        | ProviderRequest::VectorStoresUpdate(_, _)
        | ProviderRequest::VectorStoresDelete(_)
        | ProviderRequest::VectorStoresSearch(_, _)
        | ProviderRequest::VectorStoreFiles(_, _)
        | ProviderRequest::VectorStoreFilesList(_)
        | ProviderRequest::VectorStoreFilesRetrieve(_, _)
        | ProviderRequest::VectorStoreFilesDelete(_, _)
        | ProviderRequest::VectorStoreFileBatches(_, _)
        | ProviderRequest::VectorStoreFileBatchesRetrieve(_, _)
        | ProviderRequest::VectorStoreFileBatchesCancel(_, _)
        | ProviderRequest::VectorStoreFileBatchesListFiles(_, _) => "vector_stores",
        ProviderRequest::Videos(_)
        | ProviderRequest::VideosList
        | ProviderRequest::VideosRetrieve(_)
        | ProviderRequest::VideosDelete(_)
        | ProviderRequest::VideosContent(_)
        | ProviderRequest::VideosRemix(_, _)
        | ProviderRequest::VideoCharactersCreate(_)
        | ProviderRequest::VideoCharactersList(_)
        | ProviderRequest::VideoCharactersRetrieve(_, _)
        | ProviderRequest::VideoCharactersCanonicalRetrieve(_)
        | ProviderRequest::VideoCharactersUpdate(_, _, _)
        | ProviderRequest::VideosEdits(_)
        | ProviderRequest::VideosExtensions(_)
        | ProviderRequest::VideosExtend(_, _) => "videos",
        ProviderRequest::Webhooks(_)
        | ProviderRequest::WebhooksList
        | ProviderRequest::WebhooksRetrieve(_)
        | ProviderRequest::WebhooksUpdate(_, _)
        | ProviderRequest::WebhooksDelete(_) => "webhooks",
    }
}

async fn execute_json_provider_request(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    execute_json_provider_request_with_options(runtime_key, base_url, api_key, request, &options)
        .await
}

async fn execute_json_provider_request_with_options(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    let host = configured_extension_host()?;
    let Some(adapter) = host.resolve_provider(runtime_key, base_url) else {
        return Ok(None);
    };

    let response = execute_provider_request_with_execution_context(
        adapter.as_ref(),
        None,
        api_key,
        request,
        options,
    )
    .await?;
    Ok(response.into_json())
}

async fn execute_stream_provider_request_with_options(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    let host = configured_extension_host()?;
    let Some(adapter) = host.resolve_provider(runtime_key, base_url) else {
        return Ok(None);
    };

    let response = execute_provider_request_with_execution_context(
        adapter.as_ref(),
        None,
        api_key,
        request,
        options,
    )
    .await?;
    Ok(response.into_stream())
}

pub async fn execute_json_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    execute_json_provider_request_with_runtime_and_options(
        runtime_key,
        base_url,
        api_key,
        request,
        &options,
    )
    .await
}

pub async fn execute_json_provider_request_with_runtime_and_options(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    execute_json_provider_request_with_options(
        runtime_key,
        base_url.into(),
        api_key,
        request,
        options,
    )
    .await
}

pub async fn execute_stream_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    execute_stream_provider_request_with_runtime_and_options(
        runtime_key,
        base_url,
        api_key,
        request,
        &options,
    )
    .await
}

pub async fn execute_stream_provider_request_with_runtime_and_options(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    execute_stream_provider_request_with_options(
        runtime_key,
        base_url.into(),
        api_key,
        request,
        options,
    )
    .await
}

fn fallback_speech_bytes(format: &str) -> Vec<u8> {
    match format {
        "wav" => vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6d,
            0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x40, 0x1f, 0x00, 0x00,
            0x80, 0x3e, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00,
            0x00, 0x00,
        ],
        "mp3" => vec![0x49, 0x44, 0x33, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21],
        "opus" => b"OggS\x00\x02OpusHead\x01\x01\x00\x00\x00\x00\x00\x00\x00".to_vec(),
        "aac" => vec![0xFF, 0xF1, 0x50, 0x80, 0x00, 0x1F, 0xFC],
        "flac" => b"fLaC\x00\x00\x00\x22".to_vec(),
        "pcm" => vec![0x00, 0x00],
        _ => Vec::new(),
    }
}

fn normalize_local_speech_format(format: &str) -> Result<&'static str> {
    match format.to_ascii_lowercase().as_str() {
        "wav" => Ok("wav"),
        "mp3" => Ok("mp3"),
        "opus" => Ok("opus"),
        "aac" => Ok("aac"),
        "flac" => Ok("flac"),
        "pcm" => Ok("pcm"),
        _ => bail!("unsupported local speech response_format: {format}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ROUTING_DECISION_CACHE_NAMESPACE, RoutingDecision, cache_routing_decision,
        configure_route_decision_cache_store, configured_extension_discovery_policy,
        current_request_api_key_group_id, provider_request_metric_capability,
        record_gateway_execution_context_failure, record_gateway_execution_failover,
        record_gateway_provider_health, record_gateway_provider_health_persist_failure,
        record_gateway_provider_health_recovery_probe, record_gateway_upstream_outcome,
        record_gateway_upstream_retry_with_detail, routing_decision_cache_key,
        take_cached_routing_decision, with_request_api_key_group_id, with_request_routing_region,
    };
    use sdkwork_api_cache_core::CacheStore;
    use sdkwork_api_cache_memory::MemoryCacheStore;
    use sdkwork_api_observability::HttpMetricsRegistry;
    use sdkwork_api_provider_core::ProviderRequest;
    use std::path::Path;
    use std::sync::Arc;

    #[test]
    fn configured_extension_discovery_policy_reads_native_dynamic_env_configuration() {
        let temp_root = std::env::temp_dir().join(format!(
            "sdkwork-app-gateway-policy-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("unix time")
                .as_millis()
        ));
        std::fs::create_dir_all(&temp_root).unwrap();
        let _guard = ExtensionEnvGuard::set(
            &[
                (
                    "SDKWORK_EXTENSION_PATHS",
                    std::env::join_paths([temp_root.as_path()])
                        .unwrap()
                        .to_string_lossy()
                        .as_ref(),
                ),
                ("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false"),
                ("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true"),
                (
                    "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
                    "true",
                ),
            ],
            &temp_root,
        );

        let policy = configured_extension_discovery_policy();

        assert_eq!(policy.search_paths, vec![temp_root]);
        assert!(!policy.enable_connector_extensions);
        assert!(policy.enable_native_dynamic_extensions);
        assert!(policy.require_signed_native_dynamic_extensions);
    }

    #[test]
    fn provider_request_metric_capability_groups_requests_by_upstream_surface() {
        assert_eq!(
            provider_request_metric_capability(&ProviderRequest::ChatCompletionsList),
            "chat_completion"
        );
        assert_eq!(
            provider_request_metric_capability(&ProviderRequest::ResponsesRetrieve("resp-1")),
            "responses"
        );
        assert_eq!(
            provider_request_metric_capability(&ProviderRequest::AudioVoicesList),
            "audio"
        );
    }

    #[test]
    fn gateway_upstream_outcomes_are_recorded_to_shared_gateway_metrics() {
        record_gateway_upstream_outcome("chat_completion", "provider-metrics-test", "attempt");
        record_gateway_upstream_outcome("chat_completion", "provider-metrics-test", "success");
        record_gateway_upstream_outcome("chat_completion", "provider-metrics-test", "failure");
        record_gateway_upstream_retry_with_detail(
            "chat_completion",
            "provider-metrics-test",
            "exhausted",
            "status_503",
            None,
            None,
        );
        record_gateway_upstream_retry_with_detail(
            "chat_completion",
            "provider-metrics-test",
            "scheduled",
            "status_429",
            Some("retry_after_seconds"),
            Some(1000),
        );
        record_gateway_execution_failover(
            "chat_completion",
            "provider-primary",
            "provider-backup",
            "success",
        );
        record_gateway_provider_health("provider-health-failed", "builtin", false, 1234);
        record_gateway_provider_health("provider-health-healthy", "builtin", true, 5678);
        record_gateway_provider_health_persist_failure("provider-health-failed", "builtin");
        record_gateway_provider_health_recovery_probe("provider-health-recovery", "selected");
        record_gateway_provider_health_recovery_probe(
            "provider-health-recovery",
            "lease_contended",
        );
        record_gateway_provider_health_recovery_probe("provider-health-recovery", "lease_error");
        record_gateway_execution_context_failure(
            "chat_completion",
            "provider-context-timeout",
            "request_timeout",
        );
        record_gateway_execution_context_failure(
            "chat_completion",
            "provider-context-overload",
            "provider_overloaded",
        );
        record_gateway_execution_context_failure(
            "chat_completion",
            "provider-context-deadline",
            "deadline_exceeded",
        );

        let output = HttpMetricsRegistry::new("gateway").render_prometheus();

        assert!(output.contains(
            "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"attempt\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"success\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"failure\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"scheduled\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"exhausted\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"scheduled\",reason=\"status_429\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",outcome=\"exhausted\",reason=\"status_503\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-metrics-test\",source=\"retry_after_seconds\"} 1000"
        ));
        assert!(output.contains(
            "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"chat_completion\",from_provider=\"provider-primary\",to_provider=\"provider-backup\",outcome=\"success\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-health-failed\",runtime=\"builtin\"} 0"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-health-healthy\",runtime=\"builtin\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_observed_at_ms{service=\"gateway\",provider=\"provider-health-healthy\",runtime=\"builtin\"} 5678"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_persist_failures_total{service=\"gateway\",provider=\"provider-health-failed\",runtime=\"builtin\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_recovery_probes_total{service=\"gateway\",provider=\"provider-health-recovery\",outcome=\"selected\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_recovery_probes_total{service=\"gateway\",provider=\"provider-health-recovery\",outcome=\"lease_contended\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_provider_health_recovery_probes_total{service=\"gateway\",provider=\"provider-health-recovery\",outcome=\"lease_error\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-context-timeout\",reason=\"request_timeout\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-context-overload\",reason=\"provider_overloaded\"} 1"
        ));
        assert!(output.contains(
            "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-context-deadline\",reason=\"deadline_exceeded\"} 1"
        ));
    }

    #[tokio::test]
    async fn request_api_key_group_scope_exposes_group_id_inside_gateway_execution() {
        let value = with_request_api_key_group_id(Some("group-live".to_owned()), async {
            current_request_api_key_group_id()
        })
        .await;

        assert_eq!(value.as_deref(), Some("group-live"));
        assert_eq!(current_request_api_key_group_id(), None);
    }

    #[tokio::test]
    async fn route_decision_cache_uses_configured_cache_store_and_consumes_entries_once() {
        let cache_store: Arc<dyn CacheStore> = Arc::new(MemoryCacheStore::default());
        configure_route_decision_cache_store(cache_store.clone());

        with_request_routing_region(Some("us-east".to_owned()), async {
            let decision = RoutingDecision::new(
                "provider-openrouter",
                vec!["provider-openrouter".to_owned()],
            );
            let cache_key = routing_decision_cache_key(
                "tenant-1",
                Some("project-1"),
                Some("group-1"),
                "chat",
                "gpt-4.1",
                Some("us-east"),
            );

            cache_routing_decision(
                "tenant-1",
                Some("project-1"),
                Some("group-1"),
                "chat",
                "gpt-4.1",
                Some("us-east"),
                &decision,
            )
            .await;

            assert!(
                cache_store
                    .get(ROUTING_DECISION_CACHE_NAMESPACE, &cache_key)
                    .await
                    .unwrap()
                    .is_some()
            );

            let cached = take_cached_routing_decision(
                "tenant-1",
                Some("project-1"),
                Some("group-1"),
                "chat",
                "gpt-4.1",
                Some("us-east"),
            )
            .await
            .expect("cached routing decision");
            let second = take_cached_routing_decision(
                "tenant-1",
                Some("project-1"),
                Some("group-1"),
                "chat",
                "gpt-4.1",
                Some("us-east"),
            )
            .await;

            assert_eq!(cached.selected_provider_id, "provider-openrouter");
            assert!(second.is_none());
        })
        .await;
    }

    struct ExtensionEnvGuard {
        previous: Vec<(&'static str, Option<String>)>,
        cleanup_dir: std::path::PathBuf,
    }

    impl ExtensionEnvGuard {
        fn set(overrides: &[(&'static str, &str)], cleanup_dir: &Path) -> Self {
            let keys = [
                "SDKWORK_EXTENSION_PATHS",
                "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
                "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
                "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            ];
            let previous = keys
                .into_iter()
                .map(|key| (key, std::env::var(key).ok()))
                .collect::<Vec<_>>();

            for key in keys {
                std::env::remove_var(key);
            }
            for (key, value) in overrides {
                std::env::set_var(key, value);
            }

            Self {
                previous,
                cleanup_dir: cleanup_dir.to_path_buf(),
            }
        }
    }

    impl Drop for ExtensionEnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.previous {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            let _ = std::fs::remove_dir_all(&self.cleanup_dir);
        }
    }
}
