use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use futures_util::stream::Stream;
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::{
    CreateChatCompletionRequest, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::containers::{CreateContainerFileRequest, CreateContainerRequest};
use sdkwork_api_contract_openai::conversations::{
    CreateConversationItemsRequest, CreateConversationRequest, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningCheckpointPermissionsRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest,
};
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::music::{CreateMusicLyricsRequest, CreateMusicRequest};
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, SubmitToolOutputsRunRequest, UpdateRunRequest,
};
use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, UpdateThreadMessageRequest,
    UpdateThreadRequest,
};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    SearchVectorStoreRequest, UpdateVectorStoreRequest,
};
use sdkwork_api_contract_openai::videos::{
    CreateVideoCharacterRequest, CreateVideoRequest, EditVideoRequest, ExtendVideoRequest,
    RemixVideoRequest, UpdateVideoCharacterRequest,
};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, UpdateWebhookRequest};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRetryAfterSource {
    Seconds,
    HttpDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHttpError {
    status: Option<reqwest::StatusCode>,
    retry_after_secs: Option<u64>,
    retry_after_source: Option<ProviderRetryAfterSource>,
    body_excerpt: Option<String>,
}

impl ProviderHttpError {
    pub fn new(
        status: Option<reqwest::StatusCode>,
        retry_after_secs: Option<u64>,
        retry_after_source: Option<ProviderRetryAfterSource>,
        body_excerpt: Option<String>,
    ) -> Self {
        Self {
            status,
            retry_after_secs,
            retry_after_source,
            body_excerpt,
        }
    }

    pub fn status(&self) -> Option<reqwest::StatusCode> {
        self.status
    }

    pub fn retry_after_secs(&self) -> Option<u64> {
        self.retry_after_secs
    }

    pub fn retry_after_source(&self) -> Option<ProviderRetryAfterSource> {
        self.retry_after_source
    }

    pub fn body_excerpt(&self) -> Option<&str> {
        self.body_excerpt.as_deref()
    }
}

impl fmt::Display for ProviderHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (
            self.status,
            self.retry_after_secs,
            self.body_excerpt.as_deref(),
        ) {
            (Some(status), Some(retry_after_secs), Some(body_excerpt)) => write!(
                f,
                "provider upstream returned HTTP {} with retry-after {}s: {}",
                status.as_u16(),
                retry_after_secs,
                body_excerpt
            ),
            (Some(status), Some(retry_after_secs), None) => write!(
                f,
                "provider upstream returned HTTP {} with retry-after {}s",
                status.as_u16(),
                retry_after_secs
            ),
            (Some(status), None, Some(body_excerpt)) => write!(
                f,
                "provider upstream returned HTTP {}: {}",
                status.as_u16(),
                body_excerpt
            ),
            (Some(status), None, None) => {
                write!(f, "provider upstream returned HTTP {}", status.as_u16())
            }
            (None, _, Some(body_excerpt)) => {
                write!(f, "provider upstream request failed: {body_excerpt}")
            }
            (None, _, None) => write!(f, "provider upstream request failed"),
        }
    }
}

impl Error for ProviderHttpError {}

pub trait ProviderAdapter {
    fn id(&self) -> &'static str;
}

#[derive(Clone, Copy)]
pub enum ProviderRequest<'a> {
    ModelsList,
    ModelsRetrieve(&'a str),
    ChatCompletions(&'a CreateChatCompletionRequest),
    ChatCompletionsStream(&'a CreateChatCompletionRequest),
    ChatCompletionsList,
    ChatCompletionsRetrieve(&'a str),
    ChatCompletionsUpdate(&'a str, &'a UpdateChatCompletionRequest),
    ChatCompletionsDelete(&'a str),
    ChatCompletionsMessagesList(&'a str),
    Completions(&'a CreateCompletionRequest),
    Containers(&'a CreateContainerRequest),
    ContainersList,
    ContainersRetrieve(&'a str),
    ContainersDelete(&'a str),
    ContainerFiles(&'a str, &'a CreateContainerFileRequest),
    ContainerFilesList(&'a str),
    ContainerFilesRetrieve(&'a str, &'a str),
    ContainerFilesDelete(&'a str, &'a str),
    ContainerFilesContent(&'a str, &'a str),
    ModelsDelete(&'a str),
    Threads(&'a CreateThreadRequest),
    ThreadsRetrieve(&'a str),
    ThreadsUpdate(&'a str, &'a UpdateThreadRequest),
    ThreadsDelete(&'a str),
    ThreadMessages(&'a str, &'a CreateThreadMessageRequest),
    ThreadMessagesList(&'a str),
    ThreadMessagesRetrieve(&'a str, &'a str),
    ThreadMessagesUpdate(&'a str, &'a str, &'a UpdateThreadMessageRequest),
    ThreadMessagesDelete(&'a str, &'a str),
    ThreadRuns(&'a str, &'a CreateRunRequest),
    ThreadRunsList(&'a str),
    ThreadRunsRetrieve(&'a str, &'a str),
    ThreadRunsUpdate(&'a str, &'a str, &'a UpdateRunRequest),
    ThreadRunsCancel(&'a str, &'a str),
    ThreadRunsSubmitToolOutputs(&'a str, &'a str, &'a SubmitToolOutputsRunRequest),
    ThreadRunStepsList(&'a str, &'a str),
    ThreadRunStepsRetrieve(&'a str, &'a str, &'a str),
    ThreadsRuns(&'a CreateThreadAndRunRequest),
    Conversations(&'a CreateConversationRequest),
    ConversationsList,
    ConversationsRetrieve(&'a str),
    ConversationsUpdate(&'a str, &'a UpdateConversationRequest),
    ConversationsDelete(&'a str),
    ConversationItems(&'a str, &'a CreateConversationItemsRequest),
    ConversationItemsList(&'a str),
    ConversationItemsRetrieve(&'a str, &'a str),
    ConversationItemsDelete(&'a str, &'a str),
    Responses(&'a CreateResponseRequest),
    ResponsesStream(&'a CreateResponseRequest),
    ResponsesInputTokens(&'a CountResponseInputTokensRequest),
    ResponsesRetrieve(&'a str),
    ResponsesDelete(&'a str),
    ResponsesInputItemsList(&'a str),
    ResponsesCancel(&'a str),
    ResponsesCompact(&'a CompactResponseRequest),
    Embeddings(&'a CreateEmbeddingRequest),
    Moderations(&'a CreateModerationRequest),
    Music(&'a CreateMusicRequest),
    MusicList,
    MusicRetrieve(&'a str),
    MusicDelete(&'a str),
    MusicContent(&'a str),
    MusicLyrics(&'a CreateMusicLyricsRequest),
    ImagesGenerations(&'a CreateImageRequest),
    ImagesEdits(&'a CreateImageEditRequest),
    ImagesVariations(&'a CreateImageVariationRequest),
    AudioTranscriptions(&'a CreateTranscriptionRequest),
    AudioTranslations(&'a CreateTranslationRequest),
    AudioSpeech(&'a CreateSpeechRequest),
    AudioVoicesList,
    AudioVoiceConsents(&'a CreateVoiceConsentRequest),
    Files(&'a CreateFileRequest),
    FilesList,
    FilesRetrieve(&'a str),
    FilesDelete(&'a str),
    FilesContent(&'a str),
    Uploads(&'a CreateUploadRequest),
    UploadParts(&'a AddUploadPartRequest),
    UploadComplete(&'a CompleteUploadRequest),
    UploadCancel(&'a str),
    FineTuningJobs(&'a CreateFineTuningJobRequest),
    FineTuningJobsList,
    FineTuningJobsRetrieve(&'a str),
    FineTuningJobsCancel(&'a str),
    FineTuningJobsEvents(&'a str),
    FineTuningJobsCheckpoints(&'a str),
    FineTuningJobsPause(&'a str),
    FineTuningJobsResume(&'a str),
    FineTuningCheckpointPermissions(&'a str, &'a CreateFineTuningCheckpointPermissionsRequest),
    FineTuningCheckpointPermissionsList(&'a str),
    FineTuningCheckpointPermissionsDelete(&'a str, &'a str),
    Assistants(&'a CreateAssistantRequest),
    AssistantsList,
    AssistantsRetrieve(&'a str),
    AssistantsUpdate(&'a str, &'a UpdateAssistantRequest),
    AssistantsDelete(&'a str),
    RealtimeSessions(&'a CreateRealtimeSessionRequest),
    Evals(&'a CreateEvalRequest),
    EvalsList,
    EvalsRetrieve(&'a str),
    EvalsUpdate(&'a str, &'a UpdateEvalRequest),
    EvalsDelete(&'a str),
    EvalRunsList(&'a str),
    EvalRuns(&'a str, &'a CreateEvalRunRequest),
    EvalRunsRetrieve(&'a str, &'a str),
    EvalRunsDelete(&'a str, &'a str),
    EvalRunsCancel(&'a str, &'a str),
    EvalRunOutputItemsList(&'a str, &'a str),
    EvalRunOutputItemsRetrieve(&'a str, &'a str, &'a str),
    Batches(&'a CreateBatchRequest),
    BatchesList,
    BatchesRetrieve(&'a str),
    BatchesCancel(&'a str),
    VectorStores(&'a CreateVectorStoreRequest),
    VectorStoresList,
    VectorStoresRetrieve(&'a str),
    VectorStoresUpdate(&'a str, &'a UpdateVectorStoreRequest),
    VectorStoresDelete(&'a str),
    VectorStoresSearch(&'a str, &'a SearchVectorStoreRequest),
    VectorStoreFiles(&'a str, &'a CreateVectorStoreFileRequest),
    VectorStoreFilesList(&'a str),
    VectorStoreFilesRetrieve(&'a str, &'a str),
    VectorStoreFilesDelete(&'a str, &'a str),
    VectorStoreFileBatches(&'a str, &'a CreateVectorStoreFileBatchRequest),
    VectorStoreFileBatchesRetrieve(&'a str, &'a str),
    VectorStoreFileBatchesCancel(&'a str, &'a str),
    VectorStoreFileBatchesListFiles(&'a str, &'a str),
    Videos(&'a CreateVideoRequest),
    VideosList,
    VideosRetrieve(&'a str),
    VideosDelete(&'a str),
    VideosContent(&'a str),
    VideosRemix(&'a str, &'a RemixVideoRequest),
    VideoCharactersCreate(&'a CreateVideoCharacterRequest),
    VideoCharactersList(&'a str),
    VideoCharactersRetrieve(&'a str, &'a str),
    VideoCharactersCanonicalRetrieve(&'a str),
    VideoCharactersUpdate(&'a str, &'a str, &'a UpdateVideoCharacterRequest),
    VideosEdits(&'a EditVideoRequest),
    VideosExtensions(&'a ExtendVideoRequest),
    VideosExtend(&'a str, &'a ExtendVideoRequest),
    Webhooks(&'a CreateWebhookRequest),
    WebhooksList,
    WebhooksRetrieve(&'a str),
    WebhooksUpdate(&'a str, &'a UpdateWebhookRequest),
    WebhooksDelete(&'a str),
}

pub enum ProviderOutput {
    Json(Value),
    Stream(ProviderStreamOutput),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OpenRouterDataCollectionPolicy {
    #[default]
    Allow,
    Deny,
}

impl OpenRouterDataCollectionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenRouterProviderPreferences {
    order: Vec<String>,
    allow_fallbacks: Option<bool>,
    require_parameters: Option<bool>,
    data_collection: Option<OpenRouterDataCollectionPolicy>,
    zero_data_retention: Option<bool>,
}

impl OpenRouterProviderPreferences {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_order(mut self, order: Vec<String>) -> Self {
        self.order = order;
        self
    }

    pub fn with_allow_fallbacks(mut self, allow_fallbacks: bool) -> Self {
        self.allow_fallbacks = Some(allow_fallbacks);
        self
    }

    pub fn with_require_parameters(mut self, require_parameters: bool) -> Self {
        self.require_parameters = Some(require_parameters);
        self
    }

    pub fn with_data_collection(mut self, data_collection: OpenRouterDataCollectionPolicy) -> Self {
        self.data_collection = Some(data_collection);
        self
    }

    pub fn with_zero_data_retention(mut self, zero_data_retention: bool) -> Self {
        self.zero_data_retention = Some(zero_data_retention);
        self
    }

    pub fn order(&self) -> &[String] {
        &self.order
    }

    pub fn allow_fallbacks(&self) -> Option<bool> {
        self.allow_fallbacks
    }

    pub fn require_parameters(&self) -> Option<bool> {
        self.require_parameters
    }

    pub fn data_collection(&self) -> Option<OpenRouterDataCollectionPolicy> {
        self.data_collection
    }

    pub fn zero_data_retention(&self) -> Option<bool> {
        self.zero_data_retention
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
            && self.allow_fallbacks.is_none()
            && self.require_parameters.is_none()
            && self.data_collection.is_none()
            && self.zero_data_retention.is_none()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderRequestOptions {
    headers: HashMap<String, String>,
    request_timeout_ms: Option<u64>,
    deadline_at_ms: Option<u64>,
    idempotency_key: Option<String>,
    request_trace_id: Option<String>,
    openrouter_provider_preferences: Option<OpenRouterProviderPreferences>,
}

impl ProviderRequestOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn with_request_timeout_ms(mut self, request_timeout_ms: u64) -> Self {
        self.request_timeout_ms = Some(request_timeout_ms);
        self
    }

    pub fn request_timeout_ms(&self) -> Option<u64> {
        self.request_timeout_ms
    }

    pub fn with_deadline_at_ms(mut self, deadline_at_ms: u64) -> Self {
        self.deadline_at_ms = Some(deadline_at_ms);
        self
    }

    pub fn deadline_at_ms(&self) -> Option<u64> {
        self.deadline_at_ms
    }

    pub fn with_idempotency_key(mut self, idempotency_key: impl Into<String>) -> Self {
        self.idempotency_key = Some(idempotency_key.into());
        self
    }

    pub fn idempotency_key(&self) -> Option<&str> {
        self.idempotency_key.as_deref()
    }

    pub fn with_request_trace_id(mut self, request_trace_id: impl Into<String>) -> Self {
        self.request_trace_id = Some(request_trace_id.into());
        self
    }

    pub fn request_trace_id(&self) -> Option<&str> {
        self.request_trace_id.as_deref()
    }

    pub fn with_openrouter_provider_preferences(
        mut self,
        preferences: OpenRouterProviderPreferences,
    ) -> Self {
        self.openrouter_provider_preferences = Some(preferences);
        self
    }

    pub fn openrouter_provider_preferences(&self) -> Option<&OpenRouterProviderPreferences> {
        self.openrouter_provider_preferences.as_ref()
    }

    pub fn resolved_headers(&self) -> HashMap<String, String> {
        let mut headers = self.headers.clone();
        if let Some(idempotency_key) = self.idempotency_key() {
            headers.insert("idempotency-key".to_owned(), idempotency_key.to_owned());
        }
        if let Some(request_trace_id) = self.request_trace_id() {
            headers.insert("x-request-id".to_owned(), request_trace_id.to_owned());
        }
        headers
    }

    pub fn effective_timeout_ms(&self, now_ms: u64) -> Option<u64> {
        match (self.request_timeout_ms, self.deadline_at_ms) {
            (Some(request_timeout_ms), Some(deadline_at_ms)) => {
                Some(request_timeout_ms.min(deadline_at_ms.saturating_sub(now_ms)))
            }
            (Some(request_timeout_ms), None) => Some(request_timeout_ms),
            (None, Some(deadline_at_ms)) => Some(deadline_at_ms.saturating_sub(now_ms)),
            (None, None) => None,
        }
    }

    pub fn deadline_expired(&self, now_ms: u64) -> bool {
        self.deadline_at_ms
            .is_some_and(|deadline_at_ms| deadline_at_ms <= now_ms)
    }

    pub fn is_empty(&self) -> bool {
        let openrouter_preferences_empty = match &self.openrouter_provider_preferences {
            Some(preferences) => preferences.is_empty(),
            None => true,
        };
        self.headers.is_empty()
            && self.request_timeout_ms.is_none()
            && self.deadline_at_ms.is_none()
            && self.idempotency_key.is_none()
            && self.request_trace_id.is_none()
            && openrouter_preferences_empty
    }
}

pub type ProviderByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>;

pub struct ProviderStreamOutput {
    content_type: String,
    body: ProviderByteStream,
}

impl ProviderStreamOutput {
    pub fn new<S>(content_type: impl Into<String>, body: S) -> Self
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send + 'static,
    {
        Self {
            content_type: content_type.into(),
            body: Box::pin(body),
        }
    }

    pub fn from_reqwest_response(response: reqwest::Response) -> Self {
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let body = response
            .bytes_stream()
            .map(|item| item.map_err(io::Error::other));
        Self::new(content_type, body)
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn into_body_stream(self) -> ProviderByteStream {
        self.body
    }
}

impl ProviderOutput {
    pub fn into_json(self) -> Option<Value> {
        match self {
            Self::Json(value) => Some(value),
            Self::Stream(_) => None,
        }
    }

    pub fn into_stream(self) -> Option<ProviderStreamOutput> {
        match self {
            Self::Json(_) => None,
            Self::Stream(stream) => Some(stream),
        }
    }
}

#[async_trait]
pub trait ProviderExecutionAdapter: ProviderAdapter + Send + Sync {
    async fn execute(&self, api_key: &str, request: ProviderRequest<'_>) -> Result<ProviderOutput>;

    async fn execute_with_options(
        &self,
        api_key: &str,
        request: ProviderRequest<'_>,
        options: &ProviderRequestOptions,
    ) -> Result<ProviderOutput> {
        let _ = options;
        self.execute(api_key, request).await
    }
}

type AdapterFactory =
    Arc<dyn Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static>;

#[derive(Default, Clone)]
pub struct ProviderRegistry {
    factories: HashMap<String, AdapterFactory>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_factory<F>(&mut self, adapter_kind: impl Into<String>, factory: F)
    where
        F: Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static,
    {
        self.factories
            .insert(adapter_kind.into(), Arc::new(factory));
    }

    pub fn resolve(
        &self,
        adapter_kind: &str,
        base_url: impl Into<String>,
    ) -> Option<Box<dyn ProviderExecutionAdapter>> {
        self.factories
            .get(adapter_kind)
            .map(|factory| factory(base_url.into()))
    }
}
