mod compat_anthropic;
mod compat_gemini;
mod compat_streaming;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::{
    Json, Router,
    body::Body,
    extract::FromRequestParts,
    extract::Json as ExtractJson,
    extract::Multipart,
    extract::Path,
    extract::State,
    http::HeaderMap,
    http::HeaderValue,
    http::Request,
    http::StatusCode,
    http::header,
    http::request::Parts,
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use compat_anthropic::{
    anthropic_bad_gateway_response, anthropic_count_tokens_request,
    anthropic_invalid_request_response, anthropic_request_to_chat_completion,
    anthropic_stream_from_openai, openai_chat_response_to_anthropic,
    openai_count_tokens_to_anthropic,
};
use compat_gemini::{
    gemini_bad_gateway_response, gemini_count_tokens_request, gemini_invalid_request_response,
    gemini_request_to_chat_completion, gemini_stream_from_openai, openai_chat_response_to_gemini,
    openai_count_tokens_to_gemini,
};
use sdkwork_api_app_billing::{
    BillingAccountingMode, CaptureAccountHoldInput, CreateAccountHoldInput,
    CreateBillingEventInput, GatewayCommercialBillingKernel, QuotaCheckResult,
    ReleaseAccountHoldInput, check_quota, create_billing_event, persist_billing_event,
    persist_ledger_entry,
};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::cancel_batch;
use sdkwork_api_app_gateway::cancel_fine_tuning_job;
use sdkwork_api_app_gateway::cancel_response;
use sdkwork_api_app_gateway::cancel_thread_run;
use sdkwork_api_app_gateway::cancel_upload;
use sdkwork_api_app_gateway::cancel_vector_store_file_batch;
use sdkwork_api_app_gateway::compact_response;
use sdkwork_api_app_gateway::complete_upload;
use sdkwork_api_app_gateway::count_response_input_tokens;
use sdkwork_api_app_gateway::create_assistant;
use sdkwork_api_app_gateway::create_audio_voice_consent;
use sdkwork_api_app_gateway::create_batch;
use sdkwork_api_app_gateway::create_chat_completion;
use sdkwork_api_app_gateway::create_completion;
use sdkwork_api_app_gateway::create_conversation;
use sdkwork_api_app_gateway::create_conversation_items;
use sdkwork_api_app_gateway::create_eval;
use sdkwork_api_app_gateway::create_eval_run;
use sdkwork_api_app_gateway::create_file;
use sdkwork_api_app_gateway::create_fine_tuning_job;
use sdkwork_api_app_gateway::create_image_edit;
use sdkwork_api_app_gateway::create_image_generation;
use sdkwork_api_app_gateway::create_image_variation;
use sdkwork_api_app_gateway::create_moderation;
use sdkwork_api_app_gateway::create_music;
use sdkwork_api_app_gateway::create_music_lyrics;
use sdkwork_api_app_gateway::create_realtime_session;
use sdkwork_api_app_gateway::create_speech_response;
use sdkwork_api_app_gateway::create_thread;
use sdkwork_api_app_gateway::create_thread_and_run;
use sdkwork_api_app_gateway::create_thread_message;
use sdkwork_api_app_gateway::create_thread_run;
use sdkwork_api_app_gateway::create_transcription;
use sdkwork_api_app_gateway::create_translation;
use sdkwork_api_app_gateway::create_upload;
use sdkwork_api_app_gateway::create_upload_part;
use sdkwork_api_app_gateway::create_vector_store;
use sdkwork_api_app_gateway::create_vector_store_file;
use sdkwork_api_app_gateway::create_vector_store_file_batch;
use sdkwork_api_app_gateway::create_video;
use sdkwork_api_app_gateway::create_webhook;
use sdkwork_api_app_gateway::delete_assistant;
use sdkwork_api_app_gateway::delete_chat_completion;
use sdkwork_api_app_gateway::delete_conversation;
use sdkwork_api_app_gateway::delete_conversation_item;
use sdkwork_api_app_gateway::delete_eval;
use sdkwork_api_app_gateway::delete_file;
use sdkwork_api_app_gateway::delete_model;
use sdkwork_api_app_gateway::delete_music;
use sdkwork_api_app_gateway::delete_response;
use sdkwork_api_app_gateway::delete_thread;
use sdkwork_api_app_gateway::delete_thread_message;
use sdkwork_api_app_gateway::delete_vector_store;
use sdkwork_api_app_gateway::delete_vector_store_file;
use sdkwork_api_app_gateway::delete_video;
use sdkwork_api_app_gateway::delete_webhook;
use sdkwork_api_app_gateway::extend_video;
use sdkwork_api_app_gateway::file_content;
use sdkwork_api_app_gateway::get_assistant;
use sdkwork_api_app_gateway::get_batch;
use sdkwork_api_app_gateway::get_chat_completion;
use sdkwork_api_app_gateway::get_conversation;
use sdkwork_api_app_gateway::get_conversation_item;
use sdkwork_api_app_gateway::get_eval;
use sdkwork_api_app_gateway::get_eval_run;
use sdkwork_api_app_gateway::get_file;
use sdkwork_api_app_gateway::get_fine_tuning_job;
use sdkwork_api_app_gateway::get_model;
use sdkwork_api_app_gateway::get_model_from_store;
use sdkwork_api_app_gateway::get_music;
use sdkwork_api_app_gateway::get_response;
use sdkwork_api_app_gateway::get_thread;
use sdkwork_api_app_gateway::get_thread_message;
use sdkwork_api_app_gateway::get_thread_run;
use sdkwork_api_app_gateway::get_thread_run_step;
use sdkwork_api_app_gateway::get_vector_store;
use sdkwork_api_app_gateway::get_vector_store_file;
use sdkwork_api_app_gateway::get_vector_store_file_batch;
use sdkwork_api_app_gateway::get_video;
use sdkwork_api_app_gateway::get_video_character;
use sdkwork_api_app_gateway::get_webhook;
use sdkwork_api_app_gateway::list_assistants;
use sdkwork_api_app_gateway::list_audio_voices;
use sdkwork_api_app_gateway::list_batches;
use sdkwork_api_app_gateway::list_chat_completion_messages;
use sdkwork_api_app_gateway::list_chat_completions;
use sdkwork_api_app_gateway::list_conversation_items;
use sdkwork_api_app_gateway::list_conversations;
use sdkwork_api_app_gateway::list_eval_runs;
use sdkwork_api_app_gateway::list_evals;
use sdkwork_api_app_gateway::list_files;
use sdkwork_api_app_gateway::list_fine_tuning_job_checkpoints;
use sdkwork_api_app_gateway::list_fine_tuning_job_events;
use sdkwork_api_app_gateway::list_fine_tuning_jobs;
use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::list_music;
use sdkwork_api_app_gateway::list_response_input_items;
use sdkwork_api_app_gateway::list_thread_messages;
use sdkwork_api_app_gateway::list_thread_run_steps;
use sdkwork_api_app_gateway::list_thread_runs;
use sdkwork_api_app_gateway::list_vector_store_file_batch_files;
use sdkwork_api_app_gateway::list_vector_store_files;
use sdkwork_api_app_gateway::list_vector_stores;
use sdkwork_api_app_gateway::list_video_characters;
use sdkwork_api_app_gateway::list_videos;
use sdkwork_api_app_gateway::list_webhooks;
use sdkwork_api_app_gateway::music_content;
use sdkwork_api_app_gateway::remix_video;
use sdkwork_api_app_gateway::search_vector_store;
use sdkwork_api_app_gateway::submit_thread_run_tool_outputs;
use sdkwork_api_app_gateway::update_assistant;
use sdkwork_api_app_gateway::update_chat_completion;
use sdkwork_api_app_gateway::update_conversation;
use sdkwork_api_app_gateway::update_eval;
use sdkwork_api_app_gateway::update_thread;
use sdkwork_api_app_gateway::update_thread_message;
use sdkwork_api_app_gateway::update_thread_run;
use sdkwork_api_app_gateway::update_vector_store;
use sdkwork_api_app_gateway::update_video_character;
use sdkwork_api_app_gateway::update_webhook;
use sdkwork_api_app_gateway::video_content;
use sdkwork_api_app_gateway::{
    PlannedExecutionUsageContext, create_embedding, create_response, delete_model_from_store,
    execute_json_provider_request_with_runtime_and_options,
    execute_stream_provider_request_with_runtime_and_options, list_models_from_store,
    planned_execution_usage_context_for_route, relay_assistant_from_store,
    relay_audio_voice_consent_from_store, relay_audio_voices_from_store, relay_batch_from_store,
    relay_cancel_batch_from_store, relay_cancel_fine_tuning_job_from_store,
    relay_cancel_response_from_store, relay_cancel_thread_run_from_store,
    relay_cancel_upload_from_store, relay_cancel_vector_store_file_batch_from_store,
    relay_chat_completion_from_store_with_execution_context,
    relay_chat_completion_stream_from_store_with_execution_context,
    relay_compact_response_from_store, relay_complete_upload_from_store,
    relay_completion_from_store, relay_conversation_from_store,
    relay_conversation_items_from_store, relay_count_response_input_tokens_from_store,
    relay_delete_assistant_from_store, relay_delete_chat_completion_from_store,
    relay_delete_conversation_from_store, relay_delete_conversation_item_from_store,
    relay_delete_eval_from_store, relay_delete_file_from_store, relay_delete_music_from_store,
    relay_delete_response_from_store, relay_delete_thread_from_store,
    relay_delete_thread_message_from_store, relay_delete_vector_store_file_from_store,
    relay_delete_vector_store_from_store, relay_delete_video_from_store,
    relay_delete_webhook_from_store, relay_embedding_from_store, relay_eval_from_store,
    relay_eval_run_from_store, relay_extend_video_from_store, relay_file_content_from_store,
    relay_file_from_store, relay_fine_tuning_job_from_store, relay_get_assistant_from_store,
    relay_get_batch_from_store, relay_get_chat_completion_from_store,
    relay_get_conversation_from_store, relay_get_conversation_item_from_store,
    relay_get_eval_from_store, relay_get_eval_run_from_store, relay_get_file_from_store,
    relay_get_fine_tuning_job_from_store, relay_get_music_from_store,
    relay_get_response_from_store, relay_get_thread_from_store,
    relay_get_thread_message_from_store, relay_get_thread_run_from_store,
    relay_get_thread_run_step_from_store, relay_get_vector_store_file_batch_from_store,
    relay_get_vector_store_file_from_store, relay_get_vector_store_from_store,
    relay_get_video_character_from_store, relay_get_video_from_store, relay_get_webhook_from_store,
    relay_image_edit_from_store, relay_image_generation_from_store,
    relay_image_variation_from_store, relay_list_assistants_from_store,
    relay_list_batches_from_store, relay_list_chat_completion_messages_from_store,
    relay_list_chat_completions_from_store, relay_list_conversation_items_from_store,
    relay_list_conversations_from_store, relay_list_eval_runs_from_store,
    relay_list_evals_from_store, relay_list_files_from_store,
    relay_list_fine_tuning_job_checkpoints_from_store,
    relay_list_fine_tuning_job_events_from_store, relay_list_fine_tuning_jobs_from_store,
    relay_list_music_from_store, relay_list_response_input_items_from_store,
    relay_list_thread_messages_from_store, relay_list_thread_run_steps_from_store,
    relay_list_thread_runs_from_store, relay_list_vector_store_file_batch_files_from_store,
    relay_list_vector_store_files_from_store, relay_list_vector_stores_from_store,
    relay_list_video_characters_from_store, relay_list_videos_from_store,
    relay_list_webhooks_from_store, relay_moderation_from_store, relay_music_content_from_store,
    relay_music_from_store, relay_music_lyrics_from_store, relay_realtime_session_from_store,
    relay_remix_video_from_store, relay_response_from_store_with_execution_context,
    relay_response_stream_from_store_with_execution_context, relay_search_vector_store_from_store,
    relay_speech_from_store, relay_submit_thread_run_tool_outputs_from_store,
    relay_thread_and_run_from_store, relay_thread_from_store, relay_thread_messages_from_store,
    relay_thread_run_from_store, relay_transcription_from_store, relay_translation_from_store,
    relay_update_assistant_from_store, relay_update_chat_completion_from_store,
    relay_update_conversation_from_store, relay_update_eval_from_store,
    relay_update_thread_from_store, relay_update_thread_message_from_store,
    relay_update_thread_run_from_store, relay_update_vector_store_from_store,
    relay_update_video_character_from_store, relay_update_webhook_from_store,
    relay_upload_from_store, relay_upload_part_from_store,
    relay_vector_store_file_batch_from_store, relay_vector_store_file_from_store,
    relay_vector_store_from_store, relay_video_content_from_store, relay_video_from_store,
    relay_webhook_from_store, with_request_api_key_group_id, with_request_routing_region,
};
use sdkwork_api_app_identity::{
    GatewayRequestContext as IdentityGatewayRequestContext, resolve_gateway_request_context,
};
use sdkwork_api_app_rate_limit::check_rate_limit;
use sdkwork_api_app_usage::persist_usage_record_with_tokens_and_facts;
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_contract_openai::assistants::{
    AssistantObject, DeleteAssistantResponse, ListAssistantsResponse,
};
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest, ListVoicesResponse, SpeechResponse, TranscriptionObject,
    TranslationObject, VoiceConsentObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::{
    ChatCompletionResponse, CreateChatCompletionRequest, DeleteChatCompletionResponse,
    ListChatCompletionMessagesResponse, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::containers::{
    ContainerFileObject, ContainerObject, CreateContainerFileRequest, CreateContainerRequest,
    DeleteContainerFileResponse, DeleteContainerResponse, ListContainerFilesResponse,
};
use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, CreateConversationItemsRequest,
    CreateConversationRequest, DeleteConversationItemResponse, DeleteConversationResponse,
    ListConversationItemsResponse, ListConversationsResponse, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::{CreateEmbeddingRequest, CreateEmbeddingResponse};
use sdkwork_api_contract_openai::errors::OpenAiErrorResponse;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, DeleteEvalResponse, DeleteEvalRunResponse, EvalObject,
    EvalRunObject, EvalRunOutputItemObject, ListEvalRunOutputItemsResponse, ListEvalRunsResponse,
    UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
    DeleteFineTuningCheckpointPermissionResponse, FineTuningJobObject,
    ListFineTuningCheckpointPermissionsResponse, ListFineTuningJobCheckpointsResponse,
    ListFineTuningJobEventsResponse,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest, ImageUpload,
    ImagesResponse,
};
use sdkwork_api_contract_openai::models::ListModelsResponse;
use sdkwork_api_contract_openai::moderations::{CreateModerationRequest, ModerationResponse};
use sdkwork_api_contract_openai::music::{
    CreateMusicLyricsRequest, CreateMusicRequest, DeleteMusicResponse, MusicObject,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseCompactionObject,
    ResponseInputTokensObject, ResponseObject,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, ListRunStepsResponse, ListRunsResponse, RunObject,
    RunStepObject, SubmitToolOutputsRunRequest, UpdateRunRequest,
};
use sdkwork_api_contract_openai::streaming::SseFrame;
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
    ExtendVideoRequest, RemixVideoRequest, UpdateVideoCharacterRequest, VideoObject,
};
use sdkwork_api_contract_openai::webhooks::{
    CreateWebhookRequest, DeleteWebhookResponse, UpdateWebhookRequest, WebhookObject,
};
use sdkwork_api_domain_rate_limit::RateLimitCheckResult;
use sdkwork_api_observability::{HttpMetricsRegistry, observe_http_metrics, observe_http_tracing};
use sdkwork_api_policy_billing::{
    BillingPolicyExecutionInput, BillingPolicyExecutionResult, GROUP_DEFAULT_BILLING_POLICY_ID,
    builtin_billing_policy_registry,
};
use sdkwork_api_provider_core::{ProviderRequest, ProviderRequestOptions, ProviderStreamOutput};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use utoipa::openapi::Server;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::{Config as SwaggerUiConfig, SwaggerUi, Url as SwaggerUiUrl};

const DEFAULT_STATELESS_TENANT_ID: &str = "sdkwork-stateless";
const DEFAULT_STATELESS_PROJECT_ID: &str = "sdkwork-stateless-default";
#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Gateway API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current gateway router implementation."
    ),
    modifiers(&GatewayApiDocModifier),
    tags(
        (name = "system", description = "Gateway health and system-facing routes."),
        (name = "models", description = "Model listing and model metadata routes."),
        (name = "chat", description = "OpenAI-compatible chat completion routes."),
        (name = "completions", description = "OpenAI-compatible text completion routes."),
        (name = "responses", description = "OpenAI-compatible response generation routes."),
        (name = "conversations", description = "OpenAI-compatible conversation and conversation item routes."),
        (name = "embeddings", description = "Embedding generation routes."),
        (name = "moderations", description = "Moderation and safety evaluation routes."),
        (name = "images", description = "Image generation, edit, and variation routes."),
        (name = "audio", description = "Audio transcription, translation, speech, and voice routes."),
        (name = "files", description = "File upload, listing, and retrieval routes."),
        (name = "uploads", description = "Multi-part upload lifecycle routes."),
        (name = "batches", description = "Batch execution submission and management routes."),
        (name = "vector-stores", description = "Vector store search and file management routes."),
        (name = "assistants", description = "Assistant creation and retrieval routes."),
        (name = "threads", description = "Assistant thread and message management routes."),
        (name = "runs", description = "Assistant run orchestration and run step routes."),
        (name = "realtime", description = "Realtime session bootstrap routes."),
        (name = "compatibility", description = "Anthropic and Gemini compatibility routes.")
    )
)]
struct GatewayApiDoc;

struct GatewayApiDocModifier;

impl Modify for GatewayApiDocModifier {
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
                        .bearer_format("API Key")
                        .build(),
                ),
            );
    }
}

mod openapi_paths {
    use super::*;

    #[utoipa::path(
        get,
        path = "/health",
        tag = "system",
        responses((status = 200, description = "Gateway health check response.", body = String))
    )]
    pub(super) async fn health() {}

    #[utoipa::path(
        get,
        path = "/v1/models",
        tag = "models",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible model catalog.", body = ListModelsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load model catalog.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn list_models() {}

    #[utoipa::path(
        get,
        path = "/v1/models/{model_id}",
        tag = "models",
        params(("model_id" = String, Path, description = "Model identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible model metadata.", body = sdkwork_api_contract_openai::models::ModelObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested model was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load model metadata.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn get_model() {}

    #[utoipa::path(
        post,
        path = "/v1/chat/completions",
        tag = "chat",
        request_body = CreateChatCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Chat completion response.", body = ChatCompletionResponse),
            (status = 400, description = "Invalid completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the chat completion.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn chat_completions() {}

    #[utoipa::path(
        post,
        path = "/v1/completions",
        tag = "completions",
        request_body = CreateCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Text completion response.", body = CompletionObject),
            (status = 400, description = "Invalid completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the completion.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn completions() {}

    #[utoipa::path(
        post,
        path = "/v1/responses",
        tag = "responses",
        request_body = CreateResponseRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response generation result.", body = ResponseObject),
            (status = 400, description = "Invalid response payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn responses() {}

    #[utoipa::path(
        post,
        path = "/v1/responses/input_tokens",
        tag = "responses",
        request_body = CountResponseInputTokensRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response input token count result.", body = ResponseInputTokensObject),
            (status = 400, description = "Invalid response token count payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to count response input tokens.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn responses_input_tokens() {}

    #[utoipa::path(
        post,
        path = "/v1/responses/compact",
        tag = "responses",
        request_body = CompactResponseRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response compaction result.", body = ResponseCompactionObject),
            (status = 400, description = "Invalid response compaction payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to compact the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn responses_compact() {}

    #[utoipa::path(
        get,
        path = "/v1/responses/{response_id}",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible response.", body = ResponseObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn response_get() {}

    #[utoipa::path(
        delete,
        path = "/v1/responses/{response_id}",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted response.", body = DeleteResponseResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn response_delete() {}

    #[utoipa::path(
        get,
        path = "/v1/responses/{response_id}/input_items",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible response input items.", body = ListResponseInputItemsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load response input items.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn response_input_items() {}

    #[utoipa::path(
        post,
        path = "/v1/responses/{response_id}/cancel",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled response.", body = ResponseObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn response_cancel() {}

    #[utoipa::path(
        post,
        path = "/v1/embeddings",
        tag = "embeddings",
        request_body = CreateEmbeddingRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Embedding generation result.", body = CreateEmbeddingResponse),
            (status = 400, description = "Invalid embedding payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create embeddings.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn embeddings() {}

    #[utoipa::path(
        post,
        path = "/v1/moderations",
        tag = "moderations",
        request_body = CreateModerationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Moderation result.", body = ModerationResponse),
            (status = 400, description = "Invalid moderation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the moderation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn moderations() {}

    #[utoipa::path(
        post,
        path = "/v1/images/generations",
        tag = "images",
        request_body = CreateImageRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Image generation result.", body = ImagesResponse),
            (status = 400, description = "Invalid image generation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create images.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn image_generations() {}

    #[utoipa::path(
        post,
        path = "/v1/images/edits",
        tag = "images",
        request_body(
            content = CreateImageEditRequest,
            content_type = "multipart/form-data",
            description = "Multipart image edit payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Image edit result.", body = ImagesResponse),
            (status = 400, description = "Invalid image edit payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to edit the image.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn image_edits() {}

    #[utoipa::path(
        post,
        path = "/v1/images/variations",
        tag = "images",
        request_body(
            content = CreateImageVariationRequest,
            content_type = "multipart/form-data",
            description = "Multipart image variation payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Image variation result.", body = ImagesResponse),
            (status = 400, description = "Invalid image variation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the image variation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn image_variations() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/transcriptions",
        tag = "audio",
        request_body = CreateTranscriptionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio transcription result.", body = TranscriptionObject),
            (status = 400, description = "Invalid transcription payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the transcription.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn transcriptions() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/translations",
        tag = "audio",
        request_body = CreateTranslationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio translation result.", body = TranslationObject),
            (status = 400, description = "Invalid translation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the translation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn translations() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/speech",
        tag = "audio",
        request_body = CreateSpeechRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio speech synthesis result.", body = SpeechResponse),
            (status = 400, description = "Invalid speech synthesis payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to synthesize speech.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn audio_speech() {}

    #[utoipa::path(
        get,
        path = "/v1/audio/voices",
        tag = "audio",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Available audio voices.", body = ListVoicesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the voice catalog.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn audio_voices() {}

    #[utoipa::path(
        post,
        path = "/v1/audio/voice_consents",
        tag = "audio",
        request_body = CreateVoiceConsentRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Audio voice consent approval result.", body = VoiceConsentObject),
            (status = 400, description = "Invalid audio voice consent payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the audio voice consent.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn audio_voice_consents() {}

    #[utoipa::path(
        get,
        path = "/v1/assistants",
        tag = "assistants",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible assistants.", body = ListAssistantsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load assistants.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn assistants_list() {}

    #[utoipa::path(
        post,
        path = "/v1/assistants",
        tag = "assistants",
        request_body = CreateAssistantRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created assistant.", body = AssistantObject),
            (status = 400, description = "Invalid assistant payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the assistant.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn assistants_create() {}

    #[utoipa::path(
        get,
        path = "/v1/assistants/{assistant_id}",
        tag = "assistants",
        params(("assistant_id" = String, Path, description = "Assistant identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible assistant metadata.", body = AssistantObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested assistant was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load assistant metadata.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn assistants_get() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations",
        tag = "conversations",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversations.", body = ListConversationsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load conversations.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversations_list() {}

    #[utoipa::path(
        post,
        path = "/v1/conversations",
        tag = "conversations",
        request_body = CreateConversationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created conversation.", body = ConversationObject),
            (status = 400, description = "Invalid conversation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversations_create() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations/{conversation_id}",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversation.", body = ConversationObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_get() {}

    #[utoipa::path(
        post,
        path = "/v1/conversations/{conversation_id}",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        request_body = UpdateConversationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated conversation.", body = ConversationObject),
            (status = 400, description = "Invalid conversation update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/conversations/{conversation_id}",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted conversation.", body = DeleteConversationResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_delete() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations/{conversation_id}/items",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversation items.", body = ListConversationItemsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load conversation items.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_items_list() {}

    #[utoipa::path(
        post,
        path = "/v1/conversations/{conversation_id}/items",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        request_body = CreateConversationItemsRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created conversation items.", body = ListConversationItemsResponse),
            (status = 400, description = "Invalid conversation item payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create conversation items.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_items_create() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations/{conversation_id}/items/{item_id}",
        tag = "conversations",
        params(
            ("conversation_id" = String, Path, description = "Conversation identifier."),
            ("item_id" = String, Path, description = "Conversation item identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversation item.", body = ConversationItemObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation item was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the conversation item.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_item_get() {}

    #[utoipa::path(
        delete,
        path = "/v1/conversations/{conversation_id}/items/{item_id}",
        tag = "conversations",
        params(
            ("conversation_id" = String, Path, description = "Conversation identifier."),
            ("item_id" = String, Path, description = "Conversation item identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted conversation item.", body = DeleteConversationItemResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation item was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the conversation item.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_item_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/threads",
        tag = "threads",
        request_body = CreateThreadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread.", body = ThreadObject),
            (status = 400, description = "Invalid thread payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn threads_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread metadata.", body = ThreadObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_get() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        request_body = UpdateThreadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated thread.", body = ThreadObject),
            (status = 400, description = "Invalid thread update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/threads/{thread_id}",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted thread.", body = DeleteThreadResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_delete() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/messages",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread messages.", body = ListThreadMessagesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load thread messages.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_messages_list() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/messages",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        request_body = CreateThreadMessageRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread message.", body = ThreadMessageObject),
            (status = 400, description = "Invalid thread message payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_messages_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/messages/{message_id}",
        tag = "threads",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("message_id" = String, Path, description = "Thread message identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread message metadata.", body = ThreadMessageObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread message was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_message_get() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/messages/{message_id}",
        tag = "threads",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("message_id" = String, Path, description = "Thread message identifier.")
        ),
        request_body = UpdateThreadMessageRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated thread message.", body = ThreadMessageObject),
            (status = 400, description = "Invalid thread message update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread message was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_message_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/threads/{thread_id}/messages/{message_id}",
        tag = "threads",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("message_id" = String, Path, description = "Thread message identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted thread message.", body = DeleteThreadMessageResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread message was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_message_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/runs",
        tag = "runs",
        request_body = CreateThreadAndRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread and run.", body = RunObject),
            (status = 400, description = "Invalid thread-and-run payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread and run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_and_run_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs",
        tag = "runs",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread runs.", body = ListRunsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load thread runs.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_runs_list() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs",
        tag = "runs",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        request_body = CreateRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread run.", body = RunObject),
            (status = 400, description = "Invalid thread run payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_runs_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs/{run_id}",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible run metadata.", body = RunObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_get() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs/{run_id}",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        request_body = UpdateRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated run.", body = RunObject),
            (status = 400, description = "Invalid run update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_update() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs/{run_id}/cancel",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled run.", body = RunObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_cancel() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        request_body = SubmitToolOutputsRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Run after tool outputs submission.", body = RunObject),
            (status = 400, description = "Invalid tool outputs payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to submit tool outputs to the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_submit_tool_outputs() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs/{run_id}/steps",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible run steps.", body = ListRunStepsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load run steps.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_steps_list() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier."),
            ("step_id" = String, Path, description = "Run step identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible run step metadata.", body = RunStepObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run step was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the run step.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_step_get() {}

    #[utoipa::path(
        post,
        path = "/v1/realtime/sessions",
        tag = "realtime",
        request_body = CreateRealtimeSessionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Realtime session bootstrap result.", body = RealtimeSessionObject),
            (status = 400, description = "Invalid realtime session payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the realtime session.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn realtime_sessions() {}

    #[utoipa::path(
        get,
        path = "/v1/files",
        tag = "files",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible files.", body = ListFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load files.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn files_list() {}

    #[utoipa::path(
        post,
        path = "/v1/files",
        tag = "files",
        request_body(
            content = CreateFileRequest,
            content_type = "multipart/form-data",
            description = "Multipart file upload payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created file.", body = FileObject),
            (status = 400, description = "Invalid file upload payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the file.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn files_create() {}

    #[utoipa::path(
        get,
        path = "/v1/files/{file_id}",
        tag = "files",
        params(("file_id" = String, Path, description = "File identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible file metadata.", body = FileObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the file.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn file_get() {}

    #[utoipa::path(
        delete,
        path = "/v1/files/{file_id}",
        tag = "files",
        params(("file_id" = String, Path, description = "File identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted file.", body = DeleteFileResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the file.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn file_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/uploads",
        tag = "uploads",
        request_body = CreateUploadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created upload session.", body = UploadObject),
            (status = 400, description = "Invalid upload payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the upload session.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn uploads_create() {}

    #[utoipa::path(
        post,
        path = "/v1/uploads/{upload_id}/parts",
        tag = "uploads",
        params(("upload_id" = String, Path, description = "Upload session identifier.")),
        request_body(
            content = AddUploadPartRequest,
            content_type = "multipart/form-data",
            description = "Multipart upload part payload."
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created upload part.", body = UploadPartObject),
            (status = 400, description = "Invalid upload part payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to add the upload part.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn upload_parts_create() {}

    #[utoipa::path(
        post,
        path = "/v1/uploads/{upload_id}/complete",
        tag = "uploads",
        params(("upload_id" = String, Path, description = "Upload session identifier.")),
        request_body = CompleteUploadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Completed upload session.", body = UploadObject),
            (status = 400, description = "Invalid upload completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested upload session was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to complete the upload session.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn upload_complete() {}

    #[utoipa::path(
        post,
        path = "/v1/uploads/{upload_id}/cancel",
        tag = "uploads",
        params(("upload_id" = String, Path, description = "Upload session identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled upload session.", body = UploadObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested upload session was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the upload session.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn upload_cancel() {}

    #[utoipa::path(
        get,
        path = "/v1/batches",
        tag = "batches",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible batches.", body = ListBatchesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load batches.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn batches_list() {}

    #[utoipa::path(
        post,
        path = "/v1/batches",
        tag = "batches",
        request_body = CreateBatchRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created batch.", body = BatchObject),
            (status = 400, description = "Invalid batch payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the batch.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn batches_create() {}

    #[utoipa::path(
        get,
        path = "/v1/batches/{batch_id}",
        tag = "batches",
        params(("batch_id" = String, Path, description = "Batch identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible batch metadata.", body = BatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the batch.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn batch_get() {}

    #[utoipa::path(
        post,
        path = "/v1/batches/{batch_id}/cancel",
        tag = "batches",
        params(("batch_id" = String, Path, description = "Batch identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled batch.", body = BatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the batch.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn batch_cancel() {}

    #[utoipa::path(
        get,
        path = "/v1/vector_stores",
        tag = "vector-stores",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector stores.", body = ListVectorStoresResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load vector stores.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_stores_list() {}

    #[utoipa::path(
        post,
        path = "/v1/vector_stores",
        tag = "vector-stores",
        request_body = CreateVectorStoreRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created vector store.", body = VectorStoreObject),
            (status = 400, description = "Invalid vector store payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the vector store.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_stores_create() {}

    #[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store metadata.", body = VectorStoreObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the vector store.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_get() {}

    #[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = UpdateVectorStoreRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated vector store.", body = VectorStoreObject),
            (status = 400, description = "Invalid vector store update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the vector store.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/vector_stores/{vector_store_id}",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted vector store.", body = DeleteVectorStoreResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the vector store.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/search",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = SearchVectorStoreRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Vector store search result.", body = SearchVectorStoreResponse),
            (status = 400, description = "Invalid vector store search payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to search the vector store.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_search() {}

    #[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/files",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store files.", body = ListVectorStoreFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load vector store files.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_files_list() {}

    #[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/files",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = CreateVectorStoreFileRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created vector store file link.", body = VectorStoreFileObject),
            (status = 400, description = "Invalid vector store file payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the vector store file link.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_files_create() {}

    #[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/files/{file_id}",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("file_id" = String, Path, description = "File identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store file metadata.", body = VectorStoreFileObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the vector store file.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_file_get() {}

    #[utoipa::path(
        delete,
        path = "/v1/vector_stores/{vector_store_id}/files/{file_id}",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("file_id" = String, Path, description = "File identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted vector store file link.", body = DeleteVectorStoreFileResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the vector store file link.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_file_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/file_batches",
        tag = "vector-stores",
        params(("vector_store_id" = String, Path, description = "Vector store identifier.")),
        request_body = CreateVectorStoreFileBatchRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created vector store file batch.", body = VectorStoreFileBatchObject),
            (status = 400, description = "Invalid vector store file batch payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the vector store file batch.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_file_batches_create() {}

    #[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("batch_id" = String, Path, description = "Vector store file batch identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store file batch metadata.", body = VectorStoreFileBatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the vector store file batch.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_file_batch_get() {}

    #[utoipa::path(
        post,
        path = "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("batch_id" = String, Path, description = "Vector store file batch identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled vector store file batch.", body = VectorStoreFileBatchObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the vector store file batch.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_file_batch_cancel() {}

    #[utoipa::path(
        get,
        path = "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
        tag = "vector-stores",
        params(
            ("vector_store_id" = String, Path, description = "Vector store identifier."),
            ("batch_id" = String, Path, description = "Vector store file batch identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible vector store file batch files.", body = ListVectorStoreFilesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested vector store file batch was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load vector store file batch files.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn vector_store_file_batch_files_list() {}

    #[utoipa::path(
        post,
        path = "/v1/messages",
        tag = "compatibility",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Anthropic-compatible message result.", body = Value),
            (status = 400, description = "Invalid Anthropic compatibility payload.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = Value),
            (status = 500, description = "Gateway failed to serve the Anthropic compatibility route.", body = Value)
        )
    )]
    pub(super) async fn anthropic_messages() {}

    #[utoipa::path(
        post,
        path = "/v1/messages/count_tokens",
        tag = "compatibility",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Anthropic-compatible token count result.", body = Value),
            (status = 400, description = "Invalid Anthropic token count payload.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = Value),
            (status = 500, description = "Gateway failed to serve the Anthropic token count route.", body = Value)
        )
    )]
    pub(super) async fn anthropic_count_tokens() {}

    #[utoipa::path(
        post,
        path = "/v1beta/models/{tail}",
        tag = "compatibility",
        params(("tail" = String, Path, description = "Gemini compatibility route suffix.")),
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Gemini-compatible route result.", body = Value),
            (status = 400, description = "Invalid Gemini compatibility payload.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = Value),
            (status = 500, description = "Gateway failed to serve the Gemini compatibility route.", body = Value)
        )
    )]
    pub(super) async fn gemini_models_compat() {}
}

fn gateway_openapi() -> utoipa::openapi::OpenApi {
    OpenApiRouter::<()>::with_openapi(GatewayApiDoc::openapi())
        .routes(routes!(openapi_paths::health))
        .routes(routes!(openapi_paths::list_models))
        .routes(routes!(openapi_paths::get_model))
        .routes(routes!(openapi_paths::chat_completions))
        .routes(routes!(openapi_paths::completions))
        .routes(routes!(openapi_paths::responses))
        .routes(routes!(openapi_paths::responses_input_tokens))
        .routes(routes!(openapi_paths::responses_compact))
        .routes(routes!(openapi_paths::response_get))
        .routes(routes!(openapi_paths::response_delete))
        .routes(routes!(openapi_paths::response_input_items))
        .routes(routes!(openapi_paths::response_cancel))
        .routes(routes!(openapi_paths::embeddings))
        .routes(routes!(openapi_paths::moderations))
        .routes(routes!(openapi_paths::image_generations))
        .routes(routes!(openapi_paths::image_edits))
        .routes(routes!(openapi_paths::image_variations))
        .routes(routes!(openapi_paths::transcriptions))
        .routes(routes!(openapi_paths::translations))
        .routes(routes!(openapi_paths::audio_speech))
        .routes(routes!(openapi_paths::audio_voices))
        .routes(routes!(openapi_paths::audio_voice_consents))
        .routes(routes!(openapi_paths::assistants_list))
        .routes(routes!(openapi_paths::assistants_create))
        .routes(routes!(openapi_paths::assistants_get))
        .routes(routes!(openapi_paths::conversations_list))
        .routes(routes!(openapi_paths::conversations_create))
        .routes(routes!(openapi_paths::conversation_get))
        .routes(routes!(openapi_paths::conversation_update))
        .routes(routes!(openapi_paths::conversation_delete))
        .routes(routes!(openapi_paths::conversation_items_list))
        .routes(routes!(openapi_paths::conversation_items_create))
        .routes(routes!(openapi_paths::conversation_item_get))
        .routes(routes!(openapi_paths::conversation_item_delete))
        .routes(routes!(openapi_paths::threads_create))
        .routes(routes!(openapi_paths::thread_get))
        .routes(routes!(openapi_paths::thread_update))
        .routes(routes!(openapi_paths::thread_delete))
        .routes(routes!(openapi_paths::thread_messages_list))
        .routes(routes!(openapi_paths::thread_messages_create))
        .routes(routes!(openapi_paths::thread_message_get))
        .routes(routes!(openapi_paths::thread_message_update))
        .routes(routes!(openapi_paths::thread_message_delete))
        .routes(routes!(openapi_paths::thread_and_run_create))
        .routes(routes!(openapi_paths::thread_runs_list))
        .routes(routes!(openapi_paths::thread_runs_create))
        .routes(routes!(openapi_paths::thread_run_get))
        .routes(routes!(openapi_paths::thread_run_update))
        .routes(routes!(openapi_paths::thread_run_cancel))
        .routes(routes!(openapi_paths::thread_run_submit_tool_outputs))
        .routes(routes!(openapi_paths::thread_run_steps_list))
        .routes(routes!(openapi_paths::thread_run_step_get))
        .routes(routes!(openapi_paths::realtime_sessions))
        .routes(routes!(openapi_paths::files_list))
        .routes(routes!(openapi_paths::files_create))
        .routes(routes!(openapi_paths::file_get))
        .routes(routes!(openapi_paths::file_delete))
        .routes(routes!(openapi_paths::uploads_create))
        .routes(routes!(openapi_paths::upload_parts_create))
        .routes(routes!(openapi_paths::upload_complete))
        .routes(routes!(openapi_paths::upload_cancel))
        .routes(routes!(openapi_paths::batches_list))
        .routes(routes!(openapi_paths::batches_create))
        .routes(routes!(openapi_paths::batch_get))
        .routes(routes!(openapi_paths::batch_cancel))
        .routes(routes!(openapi_paths::vector_stores_list))
        .routes(routes!(openapi_paths::vector_stores_create))
        .routes(routes!(openapi_paths::vector_store_get))
        .routes(routes!(openapi_paths::vector_store_update))
        .routes(routes!(openapi_paths::vector_store_delete))
        .routes(routes!(openapi_paths::vector_store_search))
        .routes(routes!(openapi_paths::vector_store_files_list))
        .routes(routes!(openapi_paths::vector_store_files_create))
        .routes(routes!(openapi_paths::vector_store_file_get))
        .routes(routes!(openapi_paths::vector_store_file_delete))
        .routes(routes!(openapi_paths::vector_store_file_batches_create))
        .routes(routes!(openapi_paths::vector_store_file_batch_get))
        .routes(routes!(openapi_paths::vector_store_file_batch_cancel))
        .routes(routes!(openapi_paths::vector_store_file_batch_files_list))
        .routes(routes!(openapi_paths::anthropic_messages))
        .routes(routes!(openapi_paths::anthropic_count_tokens))
        .routes(routes!(openapi_paths::gemini_models_compat))
        .into_openapi()
}

async fn gateway_openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(gateway_openapi())
}

async fn gateway_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Gateway API</title>
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
        <h1>SDKWORK Gateway API</h1>
        <p>Interactive documentation is backed by the live schema endpoint <code>/openapi.json</code>.</p>
      </section>
      <iframe src="/docs/ui/" title="SDKWORK Gateway API"></iframe>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

fn gateway_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/openapi.json", get(gateway_openapi_handler))
        .route("/docs", get(gateway_docs_index_handler))
        .merge(SwaggerUi::new("/docs/ui/").config(SwaggerUiConfig::new([
            SwaggerUiUrl::with_primary("SDKWORK Gateway API", "/openapi.json", true),
        ])))
}

fn http_exposure_config() -> anyhow::Result<HttpExposureConfig> {
    HttpExposureConfig::from_env()
}

fn browser_cors_layer(http_exposure: &HttpExposureConfig) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    if http_exposure.browser_allowed_origins.is_empty() {
        return layer;
    }

    let origins = http_exposure
        .browser_allowed_origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!(
                    "ignoring invalid browser allowed origin while building gateway cors layer: {origin} ({error})"
                );
                None
            }
        })
        .collect::<Vec<_>>();
    if origins.is_empty() {
        return layer;
    }
    layer.allow_origin(origins)
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

pub struct GatewayApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    live_commercial_billing: Option<Reloadable<Arc<dyn GatewayCommercialBillingKernel>>>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    commercial_billing: Option<Arc<dyn GatewayCommercialBillingKernel>>,
}

impl Clone for GatewayApiState {
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
        }
    }
}

impl GatewayApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_master_key(pool, "local-dev-master-key")
    }

    pub fn with_master_key(pool: SqlitePool, credential_master_key: impl Into<String>) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_secret_manager_and_commercial_billing(
            store.clone(),
            CredentialSecretManager::database_encrypted(credential_master_key),
            Some(store),
        )
    }

    pub fn with_secret_manager(pool: SqlitePool, secret_manager: CredentialSecretManager) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_secret_manager_and_commercial_billing(
            store.clone(),
            secret_manager,
            Some(store),
        )
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            None,
        )
    }

    pub fn with_live_store_and_secret_manager(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            live_store,
            Reloadable::new(secret_manager),
            None,
        )
    }

    pub fn with_live_store_and_secret_manager_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            live_store,
            live_secret_manager,
            None,
        )
    }

    fn with_store_secret_manager_and_commercial_billing(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        commercial_billing: Option<Arc<dyn GatewayCommercialBillingKernel>>,
    ) -> Self {
        Self::with_live_store_secret_manager_and_commercial_billing_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            commercial_billing.map(Reloadable::new),
        )
    }

    fn with_live_store_secret_manager_and_commercial_billing_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn GatewayCommercialBillingKernel>>>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_store,
            live_secret_manager,
            live_commercial_billing,
        }
    }
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_CONTEXT: IdentityGatewayRequestContext;
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_STARTED_AT: Instant;
}

const GATEWAY_COMMERCIAL_HOLD_TTL_MS: u64 = 5 * 60 * 1000;
const GATEWAY_COMMERCIAL_ID_SEQUENCE_BITS: u32 = 15;
const GATEWAY_COMMERCIAL_ID_SEQUENCE_MASK: u64 = (1_u64 << GATEWAY_COMMERCIAL_ID_SEQUENCE_BITS) - 1;

static GATEWAY_COMMERCIAL_ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct GatewayCommercialAdmission {
    request_id: u64,
    billing_settlement: BillingPolicyExecutionResult,
}

#[derive(Debug, Clone, Copy)]
struct GatewayCommercialAdmissionSpec {
    quoted_amount: f64,
}

enum GatewayCommercialAdmissionDecision {
    Canonical(GatewayCommercialAdmission),
    LegacyQuota,
}

#[derive(Clone, Debug)]
struct AuthenticatedGatewayRequest(IdentityGatewayRequestContext);

impl AuthenticatedGatewayRequest {
    fn tenant_id(&self) -> &str {
        self.0.tenant_id()
    }

    fn project_id(&self) -> &str {
        self.0.project_id()
    }

    fn context(&self) -> &IdentityGatewayRequestContext {
        &self.0
    }
}

impl FromRequestParts<GatewayApiState> for AuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Ok(header_value) = header_value.to_str() else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Some(token) = header_value
                .strip_prefix("Bearer ")
                .or_else(|| header_value.strip_prefix("bearer "))
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(context) = resolve_gateway_request_context(state.store.as_ref(), token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            context
        };

        Ok(Self(context))
    }
}

#[derive(Clone, Debug)]
struct CompatAuthenticatedGatewayRequest(IdentityGatewayRequestContext);

impl CompatAuthenticatedGatewayRequest {
    fn tenant_id(&self) -> &str {
        self.0.tenant_id()
    }

    fn project_id(&self) -> &str {
        self.0.project_id()
    }

    fn context(&self) -> &IdentityGatewayRequestContext {
        &self.0
    }
}

impl FromRequestParts<GatewayApiState> for CompatAuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(token) = extract_compat_gateway_token(parts) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(context) = resolve_gateway_request_context(state.store.as_ref(), &token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            context
        };

        Ok(Self(context))
    }
}

fn extract_compat_gateway_token(parts: &Parts) -> Option<String> {
    extract_bearer_token(&parts.headers)
        .or_else(|| header_value(parts.headers.get("x-api-key")))
        .or_else(|| header_value(parts.headers.get("x-goog-api-key")))
        .or_else(|| query_parameter(parts.uri.query(), "key"))
}

async fn evaluate_gateway_request_rate_limit(
    store: &dyn AdminStore,
    context: &IdentityGatewayRequestContext,
    route_key: &str,
) -> anyhow::Result<RateLimitCheckResult> {
    check_rate_limit(
        store,
        context.project_id(),
        Some(context.api_key_hash()),
        route_key,
        None,
        1,
    )
    .await
}

fn rate_limit_exceeded_response(
    project_id: &str,
    route_key: &str,
    evaluation: &RateLimitCheckResult,
) -> Response {
    let mut error = OpenAiErrorResponse::new(
        rate_limit_exceeded_message(project_id, route_key, evaluation),
        "rate_limit_exceeded",
    );
    error.error.code = Some("rate_limit_exceeded".to_owned());
    (axum::http::StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

fn apply_rate_limit_headers(
    headers: &mut HeaderMap,
    evaluation: &RateLimitCheckResult,
    include_retry_after: bool,
) {
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-policy",
        evaluation.policy_id.as_deref(),
    );
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-limit",
        evaluation.limit_requests.map(|value| value.to_string()),
    );
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-remaining",
        evaluation.remaining_requests.map(|value| value.to_string()),
    );

    let reset_after_secs = rate_limit_reset_after_secs(evaluation);
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-reset",
        reset_after_secs.map(|value| value.to_string()),
    );

    if include_retry_after {
        if let Some(retry_after) = reset_after_secs {
            if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
                headers.insert(header::RETRY_AFTER, value);
            }
        }
    }
}

fn insert_optional_rate_limit_header<T>(
    headers: &mut HeaderMap,
    name: &'static str,
    value: Option<T>,
) where
    T: Into<String>,
{
    let Some(value) = value.map(Into::into) else {
        return;
    };
    if let Ok(value) = HeaderValue::from_str(&value) {
        headers.insert(name, value);
    }
}

fn rate_limit_reset_after_secs(evaluation: &RateLimitCheckResult) -> Option<u64> {
    let window_end_ms = evaluation.window_end_ms?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    let remaining_ms = window_end_ms.saturating_sub(now_ms);
    Some(remaining_ms.saturating_add(999).saturating_div(1000).max(1))
}

fn rate_limit_exceeded_message(
    project_id: &str,
    route_key: &str,
    evaluation: &RateLimitCheckResult,
) -> String {
    match (evaluation.policy_id.as_deref(), evaluation.limit_requests) {
        (Some(policy_id), Some(limit_requests)) => format!(
            "Rate limit exceeded for project {project_id} on route {route_key} under policy {policy_id}: requested {} requests with {} already used against a limit of {limit_requests}.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
        (_, Some(limit_requests)) => format!(
            "Rate limit exceeded for project {project_id} on route {route_key}: requested {} requests with {} already used against a limit of {limit_requests}.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
        _ => format!(
            "Rate limit exceeded for project {project_id} on route {route_key}: requested {} requests with {} already used.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
    }
}

fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let header_value = header_value(headers.get(header::AUTHORIZATION))?;
    header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .map(ToOwned::to_owned)
}

fn header_value(value: Option<&axum::http::HeaderValue>) -> Option<String> {
    value
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn request_options_from_header_names(
    headers: &HeaderMap,
    header_names: &[&str],
) -> ProviderRequestOptions {
    header_names.iter().fold(
        ProviderRequestOptions::default(),
        |options, name| match header_value(headers.get(*name)) {
            Some(value) => options.with_header(*name, value),
            None => options,
        },
    )
}

fn anthropic_request_options(headers: &HeaderMap) -> ProviderRequestOptions {
    request_options_from_header_names(headers, &["anthropic-version", "anthropic-beta"])
}

fn query_parameter(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == key {
            Some(value.to_owned())
        } else {
            None
        }
    })
}

fn current_gateway_request_context() -> Option<IdentityGatewayRequestContext> {
    CURRENT_GATEWAY_REQUEST_CONTEXT.try_with(Clone::clone).ok()
}

fn current_gateway_request_latency_ms() -> Option<u64> {
    CURRENT_GATEWAY_REQUEST_STARTED_AT
        .try_with(|started_at| started_at.elapsed().as_millis() as u64)
        .ok()
}

async fn apply_gateway_request_context(
    State(state): State<GatewayApiState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = extract_bearer_token(request.headers())
        .or_else(|| header_value(request.headers().get("x-api-key")))
        .or_else(|| header_value(request.headers().get("x-goog-api-key")))
        .or_else(|| query_parameter(request.uri().query(), "key"));

    let Some(token) = token else {
        return next.run(request).await;
    };

    let Ok(Some(context)) = resolve_gateway_request_context(state.store.as_ref(), &token).await
    else {
        return next.run(request).await;
    };

    request.extensions_mut().insert(context.clone());
    CURRENT_GATEWAY_REQUEST_CONTEXT
        .scope(
            context,
            with_request_api_key_group_id(
                request
                    .extensions()
                    .get::<IdentityGatewayRequestContext>()
                    .and_then(|context| context.api_key_group_id.clone()),
                CURRENT_GATEWAY_REQUEST_STARTED_AT.scope(Instant::now(), next.run(request)),
            ),
        )
        .await
}

async fn apply_gateway_rate_limit(
    State(state): State<GatewayApiState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !request.uri().path().starts_with("/v1") {
        return next.run(request).await;
    }

    let Some(context) = request
        .extensions()
        .get::<IdentityGatewayRequestContext>()
        .cloned()
    else {
        return next.run(request).await;
    };

    let route_key = request.uri().path().to_owned();
    let evaluation =
        match evaluate_gateway_request_rate_limit(state.store.as_ref(), &context, &route_key).await
        {
            Ok(result) => result,
            Err(_) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to evaluate rate limit",
                )
                    .into_response();
            }
        };

    if !evaluation.allowed {
        let mut response =
            rate_limit_exceeded_response(context.project_id(), &route_key, &evaluation);
        apply_rate_limit_headers(response.headers_mut(), &evaluation, true);
        return response;
    }

    let mut response = next.run(request).await;
    apply_rate_limit_headers(response.headers_mut(), &evaluation, false);
    response
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayUpstream {
    runtime_key: String,
    base_url: String,
    api_key: String,
}

impl StatelessGatewayUpstream {
    pub fn new(
        runtime_key: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            runtime_key: runtime_key.into(),
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    pub fn from_adapter_kind(
        adapter_kind: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new(adapter_kind, base_url, api_key)
    }

    pub fn runtime_key(&self) -> &str {
        &self.runtime_key
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayConfig {
    tenant_id: String,
    project_id: String,
    upstream: Option<StatelessGatewayUpstream>,
}

impl Default for StatelessGatewayConfig {
    fn default() -> Self {
        Self {
            tenant_id: DEFAULT_STATELESS_TENANT_ID.to_owned(),
            project_id: DEFAULT_STATELESS_PROJECT_ID.to_owned(),
            upstream: None,
        }
    }
}

impl StatelessGatewayConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_identity(
        mut self,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Self {
        self.tenant_id = tenant_id.into();
        self.project_id = project_id.into();
        self
    }

    pub fn with_upstream(mut self, upstream: StatelessGatewayUpstream) -> Self {
        self.upstream = Some(upstream);
        self
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.upstream.as_ref()
    }

    fn into_context(self) -> StatelessGatewayContext {
        StatelessGatewayContext {
            tenant_id: Arc::from(self.tenant_id),
            project_id: Arc::from(self.project_id),
            upstream: self.upstream.map(Arc::new),
        }
    }
}

#[derive(Clone, Debug)]
struct StatelessGatewayContext {
    tenant_id: Arc<str>,
    project_id: Arc<str>,
    upstream: Option<Arc<StatelessGatewayUpstream>>,
}

#[derive(Clone, Debug)]
struct StatelessGatewayRequest(StatelessGatewayContext);

impl StatelessGatewayRequest {
    fn tenant_id(&self) -> &str {
        &self.0.tenant_id
    }

    fn project_id(&self) -> &str {
        &self.0.project_id
    }

    fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.0.upstream.as_deref()
    }
}

impl FromRequestParts<StatelessGatewayContext> for StatelessGatewayRequest {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &StatelessGatewayContext,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.clone()))
    }
}

async fn apply_request_routing_region(request: Request<Body>, next: Next) -> Response {
    let requested_region = request
        .headers()
        .get("x-sdkwork-region")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    with_request_routing_region(requested_region, next.run(request)).await
}

pub fn try_gateway_router() -> anyhow::Result<Router> {
    try_gateway_router_with_stateless_config(StatelessGatewayConfig::default())
}

pub fn gateway_router() -> Router {
    try_gateway_router().expect("http exposure config should load from process env")
}

pub fn try_gateway_router_with_stateless_config(
    config: StatelessGatewayConfig,
) -> anyhow::Result<Router> {
    Ok(gateway_router_with_stateless_config_and_http_exposure(
        config,
        http_exposure_config()?,
    ))
}

pub fn gateway_router_with_stateless_config(config: StatelessGatewayConfig) -> Router {
    try_gateway_router_with_stateless_config(config)
        .expect("http exposure config should load from process env")
}

pub fn gateway_router_with_stateless_config_and_http_exposure(
    config: StatelessGatewayConfig,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("gateway");
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    Router::new()
        .merge(gateway_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/health", get(|| async { "ok" }))
        .route("/v1/messages", post(anthropic_messages_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_handler),
        )
        .route("/v1beta/models/{*tail}", post(gemini_models_compat_handler))
        .route("/v1/models", get(list_models_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_handler).delete(model_delete_handler),
        )
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_handler).post(chat_completions_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_handler)
                .post(chat_completion_update_handler)
                .delete(chat_completion_delete_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_handler),
        )
        .route("/v1/completions", post(completions_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_handler).post(conversations_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_handler)
                .post(conversation_update_handler)
                .delete(conversation_delete_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_handler).post(conversation_items_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_handler).delete(conversation_item_delete_handler),
        )
        .route("/v1/threads", post(threads_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_handler)
                .post(thread_update_handler)
                .delete(thread_delete_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_handler).post(thread_messages_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_handler)
                .post(thread_message_update_handler)
                .delete(thread_message_delete_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_handler).post(thread_runs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_handler).post(thread_run_update_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_handler),
        )
        .route("/v1/responses", post(responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_handler),
        )
        .route("/v1/responses/compact", post(response_compact_handler))
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_handler).delete(response_delete_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_handler),
        )
        .route("/v1/embeddings", post(embeddings_handler))
        .route("/v1/moderations", post(moderations_handler))
        .route("/v1/images/generations", post(image_generations_handler))
        .route("/v1/images/edits", post(image_edits_handler))
        .route("/v1/images/variations", post(image_variations_handler))
        .route("/v1/audio/transcriptions", post(transcriptions_handler))
        .route("/v1/audio/translations", post(translations_handler))
        .route("/v1/audio/speech", post(audio_speech_handler))
        .route("/v1/audio/voices", get(audio_voices_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_handler).post(containers_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_handler).delete(container_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_handler).post(container_files_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_handler).delete(container_file_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_handler),
        )
        .route("/v1/files", get(files_list_handler).post(files_handler))
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_handler).delete(file_delete_handler),
        )
        .route("/v1/files/{file_id}/content", get(file_content_handler))
        .route("/v1/videos", get(videos_list_handler).post(videos_handler))
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_handler).delete(video_delete_handler),
        )
        .route("/v1/videos/{video_id}/content", get(video_content_handler))
        .route("/v1/videos/{video_id}/remix", post(video_remix_handler))
        .route(
            "/v1/videos/characters",
            post(video_character_create_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_handler),
        )
        .route("/v1/videos/edits", post(video_edits_handler))
        .route("/v1/videos/extensions", post(video_extensions_handler))
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_handler).post(video_character_update_handler),
        )
        .route("/v1/videos/{video_id}/extend", post(video_extend_handler))
        .route("/v1/music", get(music_list_handler).post(music_handler))
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_handler).delete(music_delete_handler),
        )
        .route("/v1/music/{music_id}/content", get(music_content_handler))
        .route("/v1/music/lyrics", post(music_lyrics_handler))
        .route("/v1/uploads", post(uploads_handler))
        .route("/v1/uploads/{upload_id}/parts", post(upload_parts_handler))
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_handler).post(fine_tuning_jobs_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_handler)
                .post(fine_tuning_checkpoint_permissions_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_handler).post(assistants_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_handler)
                .post(assistant_update_handler)
                .delete(assistant_delete_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_handler).post(webhooks_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_handler)
                .post(webhook_update_handler)
                .delete(webhook_delete_handler),
        )
        .route("/v1/realtime/sessions", post(realtime_sessions_handler))
        .route("/v1/evals", get(evals_list_handler).post(evals_handler))
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_handler)
                .post(eval_update_handler)
                .delete(eval_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_handler).post(eval_runs_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_handler).delete(eval_run_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_handler),
        )
        .route(
            "/v1/batches",
            get(batches_list_handler).post(batches_handler),
        )
        .route("/v1/batches/{batch_id}", get(batch_retrieve_handler))
        .route("/v1/batches/{batch_id}/cancel", post(batch_cancel_handler))
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_handler).post(vector_stores_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_handler)
                .post(vector_store_update_handler)
                .delete(vector_store_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_handler).post(vector_store_files_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_handler).delete(vector_store_file_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_handler),
        )
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(config.into_context())
}

pub fn gateway_router_with_pool(pool: SqlitePool) -> Router {
    gateway_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn gateway_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    gateway_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn gateway_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    gateway_router_with_state(GatewayApiState::with_master_key(
        pool,
        credential_master_key,
    ))
}

pub fn gateway_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    gateway_router_with_state(GatewayApiState::with_secret_manager(pool, secret_manager))
}

pub fn gateway_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> Router {
    gateway_router_with_state(GatewayApiState::with_store_and_secret_manager(
        store,
        secret_manager,
    ))
}

pub fn try_gateway_router_with_state(state: GatewayApiState) -> anyhow::Result<Router> {
    Ok(gateway_router_with_state_and_http_exposure(
        state,
        http_exposure_config()?,
    ))
}

pub fn gateway_router_with_state(state: GatewayApiState) -> Router {
    try_gateway_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn gateway_router_with_state_and_http_exposure(
    state: GatewayApiState,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("gateway");
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    Router::new()
        .merge(gateway_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/health", get(|| async { "ok" }))
        .route("/v1/messages", post(anthropic_messages_with_state_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_with_state_handler),
        )
        .route(
            "/v1beta/models/{*tail}",
            post(gemini_models_compat_with_state_handler),
        )
        .route("/v1/models", get(list_models_from_store_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_from_store_handler).delete(model_delete_from_store_handler),
        )
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_with_state_handler).post(chat_completions_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_with_state_handler)
                .post(chat_completion_update_with_state_handler)
                .delete(chat_completion_delete_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_with_state_handler),
        )
        .route("/v1/completions", post(completions_with_state_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_with_state_handler).post(conversations_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_with_state_handler)
                .post(conversation_update_with_state_handler)
                .delete(conversation_delete_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_with_state_handler)
                .post(conversation_items_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_with_state_handler)
                .delete(conversation_item_delete_with_state_handler),
        )
        .route("/v1/threads", post(threads_with_state_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_with_state_handler)
                .post(thread_update_with_state_handler)
                .delete(thread_delete_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_with_state_handler).post(thread_messages_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_with_state_handler)
                .post(thread_message_update_with_state_handler)
                .delete(thread_message_delete_with_state_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_with_state_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_with_state_handler).post(thread_runs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_with_state_handler).post(thread_run_update_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_with_state_handler),
        )
        .route("/v1/responses", post(responses_with_state_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_with_state_handler),
        )
        .route(
            "/v1/responses/compact",
            post(response_compact_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_with_state_handler).delete(response_delete_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_with_state_handler),
        )
        .route("/v1/embeddings", post(embeddings_with_state_handler))
        .route("/v1/moderations", post(moderations_with_state_handler))
        .route(
            "/v1/images/generations",
            post(image_generations_with_state_handler),
        )
        .route("/v1/images/edits", post(image_edits_with_state_handler))
        .route(
            "/v1/images/variations",
            post(image_variations_with_state_handler),
        )
        .route(
            "/v1/audio/transcriptions",
            post(transcriptions_with_state_handler),
        )
        .route(
            "/v1/audio/translations",
            post(translations_with_state_handler),
        )
        .route("/v1/audio/speech", post(audio_speech_with_state_handler))
        .route("/v1/audio/voices", get(audio_voices_with_state_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_with_state_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_with_state_handler).post(containers_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_with_state_handler).delete(container_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_with_state_handler).post(container_files_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_with_state_handler)
                .delete(container_file_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_with_state_handler),
        )
        .route(
            "/v1/files",
            get(files_list_with_state_handler).post(files_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_with_state_handler).delete(file_delete_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}/content",
            get(file_content_with_state_handler),
        )
        .route(
            "/v1/videos",
            get(videos_list_with_state_handler).post(videos_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_with_state_handler).delete(video_delete_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/content",
            get(video_content_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/remix",
            post(video_remix_with_state_handler),
        )
        .route(
            "/v1/videos/characters",
            post(video_character_create_with_state_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_with_state_handler),
        )
        .route("/v1/videos/edits", post(video_edits_with_state_handler))
        .route(
            "/v1/videos/extensions",
            post(video_extensions_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_with_state_handler)
                .post(video_character_update_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/extend",
            post(video_extend_with_state_handler),
        )
        .route(
            "/v1/music",
            get(music_list_with_state_handler).post(music_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_with_state_handler).delete(music_delete_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}/content",
            get(music_content_with_state_handler),
        )
        .route("/v1/music/lyrics", post(music_lyrics_with_state_handler))
        .route("/v1/uploads", post(uploads_with_state_handler))
        .route(
            "/v1/uploads/{upload_id}/parts",
            post(upload_parts_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_with_state_handler).post(fine_tuning_jobs_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_with_state_handler)
                .post(fine_tuning_checkpoint_permissions_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_with_state_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_with_state_handler).post(assistants_with_state_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_with_state_handler)
                .post(assistant_update_with_state_handler)
                .delete(assistant_delete_with_state_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_with_state_handler).post(webhooks_with_state_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_with_state_handler)
                .post(webhook_update_with_state_handler)
                .delete(webhook_delete_with_state_handler),
        )
        .route(
            "/v1/realtime/sessions",
            post(realtime_sessions_with_state_handler),
        )
        .route(
            "/v1/evals",
            get(evals_list_with_state_handler).post(evals_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_with_state_handler)
                .post(eval_update_with_state_handler)
                .delete(eval_delete_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_with_state_handler).post(eval_runs_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_with_state_handler).delete(eval_run_delete_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_with_state_handler),
        )
        .route(
            "/v1/batches",
            get(batches_list_with_state_handler).post(batches_with_state_handler),
        )
        .route(
            "/v1/batches/{batch_id}",
            get(batch_retrieve_with_state_handler),
        )
        .route(
            "/v1/batches/{batch_id}/cancel",
            post(batch_cancel_with_state_handler),
        )
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_with_state_handler).post(vector_stores_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_with_state_handler)
                .post(vector_store_update_with_state_handler)
                .delete(vector_store_delete_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_with_state_handler)
                .post(vector_store_files_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_with_state_handler)
                .delete(vector_store_file_delete_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_with_state_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_rate_limit,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_request_context,
        ))
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

async fn list_models_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model list");
        }
    }

    Json(
        list_models(request_context.tenant_id(), request_context.project_id())
            .expect("models response"),
    )
    .into_response()
}

fn local_model_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested model was not found.")
}

fn local_model_retrieve_response(tenant_id: &str, project_id: &str, model_id: &str) -> Response {
    match get_model(tenant_id, project_id, model_id).map_err(local_model_not_found_response) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_model_delete_response(tenant_id: &str, project_id: &str, model_id: &str) -> Response {
    match delete_model(tenant_id, project_id, model_id).map_err(local_model_not_found_response) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn model_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsRetrieve(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model");
        }
    }

    local_model_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
}

async fn model_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsDelete(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model delete");
        }
    }

    local_model_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
}

async fn list_models_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Result<Json<sdkwork_api_contract_openai::models::ListModelsResponse>, Response> {
    list_models_from_store(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    .map(Json)
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load models",
        )
            .into_response()
    })
}

async fn model_retrieve_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<sdkwork_api_contract_openai::models::ModelObject>, Response> {
    get_model_from_store(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load model",
        )
            .into_response()
    })?
    .map(Json)
    .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "model not found").into_response())
}

async fn model_delete_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<Value>, Response> {
    delete_model_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to delete model",
        )
            .into_response()
    })?
    .map(Json)
    .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "model not found").into_response())
}

async fn chat_completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response(
                    "failed to relay upstream chat completion stream",
                );
            }
        }

        return local_chat_completion_stream_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        );
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_chat_completion_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion"),
    }
}

async fn chat_completions_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletionsList).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            list_chat_completions(request_context.tenant_id(), request_context.project_id())
                .expect("chat completions"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion list"),
    }
}

async fn chat_completion_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsRetrieve(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    local_chat_completion_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
}

async fn chat_completion_update_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsUpdate(&completion_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    local_chat_completion_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        request.metadata.unwrap_or(serde_json::json!({})),
    )
}

async fn chat_completion_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsDelete(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    local_chat_completion_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
}

async fn chat_completion_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsMessagesList(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion messages",
            );
        }
    }

    local_chat_completion_messages_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
}

async fn conversations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Conversations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    Json(
        create_conversation(request_context.tenant_id(), request_context.project_id())
            .expect("conversation"),
    )
    .into_response()
}

async fn conversations_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ConversationsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    Json(
        list_conversations(request_context.tenant_id(), request_context.project_id())
            .expect("conversation list"),
    )
    .into_response()
}

async fn conversation_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsRetrieve(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    local_conversation_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_update_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsUpdate(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    local_conversation_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        request.metadata.unwrap_or(serde_json::json!({})),
    )
}

async fn conversation_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsDelete(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    local_conversation_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_items_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItems(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    local_conversation_items_create_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsList(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    local_conversation_items_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsRetrieve(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    local_conversation_item_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
}

async fn conversation_item_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsDelete(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item delete",
            );
        }
    }

    local_conversation_item_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
}

async fn threads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Threads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread");
        }
    }

    Json(create_thread(request_context.tenant_id(), request_context.project_id()).expect("thread"))
        .into_response()
}

async fn thread_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsRetrieve(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread retrieve");
        }
    }

    local_thread_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
}

async fn thread_update_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsUpdate(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread update");
        }
    }

    local_thread_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
}

async fn thread_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsDelete(&thread_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread delete");
        }
    }

    local_thread_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
}

async fn thread_messages_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessages(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message");
        }
    }

    let text = request.content.as_str().unwrap_or("hello");
    match local_thread_messages_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.role,
        text,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn thread_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread messages list");
        }
    }

    local_thread_messages_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
}

async fn thread_message_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesRetrieve(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message retrieve");
        }
    }

    local_thread_message_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
}

async fn thread_message_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesUpdate(&thread_id, &message_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message update");
        }
    }

    local_thread_message_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
}

async fn thread_message_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesDelete(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message delete");
        }
    }

    local_thread_message_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
}

async fn thread_and_run_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsRuns(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    local_thread_and_run_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.assistant_id,
    )
}

fn local_thread_and_run_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_assistant_id");
    }

    bad_gateway_openai_response(message)
}

fn local_thread_and_run_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> std::result::Result<RunObject, Response> {
    create_thread_and_run(tenant_id, project_id, assistant_id)
        .map_err(local_thread_and_run_error_response)
}

fn local_thread_and_run_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> Response {
    match local_thread_and_run_result(tenant_id, project_id, assistant_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn thread_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRuns(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    match local_thread_runs_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.assistant_id,
        request.model.as_deref(),
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn thread_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    local_thread_runs_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
}

async fn thread_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsRetrieve(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    local_thread_run_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsUpdate(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    local_thread_run_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsCancel(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    local_thread_run_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_submit_tool_outputs_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsSubmitToolOutputs(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run submit tool outputs",
            );
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    match local_thread_run_submit_tool_outputs_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        tool_outputs,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn thread_run_steps_list_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsList(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run steps list");
        }
    }

    local_thread_run_steps_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_step_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id, step_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsRetrieve(&thread_id, &run_id, &step_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run step retrieve",
            );
        }
    }

    local_thread_run_step_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &step_id,
    )
}

async fn responses_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ResponsesStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        return local_response_stream_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        );
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::Responses(&request)).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_response_create_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream response"),
    }
}

async fn response_input_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    local_response_input_tokens_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
}

async fn response_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesRetrieve(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    match get_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_input_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputItemsList(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    match list_response_input_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesDelete(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    match delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCancel(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    match cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_compact_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCompact(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    local_response_compact_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
}

fn local_response_compact_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match compact_response(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_invalid_model_response(error),
    }
}

fn local_response_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested response was not found.")
}

fn local_response_create_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ResponseObject, Response> {
    create_response(tenant_id, project_id, model).map_err(local_response_invalid_model_response)
}

fn local_response_create_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_response_create_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_response_input_tokens_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ResponseInputTokensObject, Response> {
    count_response_input_tokens(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_response_input_tokens_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    match local_response_input_tokens_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_response_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_model");
    }

    bad_gateway_openai_response(message)
}

fn local_gateway_error_is_invalid_request(message: &str) -> bool {
    message.to_ascii_lowercase().contains("required")
}

fn local_chat_completion_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ChatCompletionResponse> {
    if model.trim().is_empty() {
        return Err(anyhow::anyhow!("Chat completion model is required."));
    }

    create_chat_completion(tenant_id, project_id, model)
}

fn local_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ChatCompletionResponse, Response> {
    local_chat_completion_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_chat_completion_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_chat_completion_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_stream_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    match local_chat_completion_result(tenant_id, project_id, model) {
        Ok(_) => local_chat_completion_stream_body_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested chat completion was not found.")
}

fn local_completion_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<CompletionObject> {
    create_completion(tenant_id, project_id, model)
}

fn local_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<CompletionObject, Response> {
    local_completion_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_completion_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_completion_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_embedding_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<CreateEmbeddingResponse> {
    create_embedding(tenant_id, project_id, model)
}

fn local_embedding_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<CreateEmbeddingResponse, Response> {
    local_embedding_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_embedding_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_embedding_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_moderation_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ModerationResponse> {
    create_moderation(tenant_id, project_id, model)
}

fn local_moderation_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ModerationResponse, Response> {
    local_moderation_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_moderation_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_moderation_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_image_generation_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ImagesResponse> {
    create_image_generation(tenant_id, project_id, model)
}

fn local_image_generation_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ImagesResponse, Response> {
    local_image_generation_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_image_generation_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_image_generation_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> std::result::Result<ChatCompletionResponse, Response> {
    get_chat_completion(tenant_id, project_id, completion_id)
        .map_err(local_chat_completion_not_found_response)
}

fn local_chat_completion_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> Response {
    match local_chat_completion_retrieve_result(tenant_id, project_id, completion_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_update_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> std::result::Result<ChatCompletionResponse, Response> {
    update_chat_completion(tenant_id, project_id, completion_id, metadata)
        .map_err(local_chat_completion_not_found_response)
}

fn local_chat_completion_update_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> Response {
    match local_chat_completion_update_result(tenant_id, project_id, completion_id, metadata) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_delete_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> std::result::Result<DeleteChatCompletionResponse, Response> {
    delete_chat_completion(tenant_id, project_id, completion_id)
        .map_err(local_chat_completion_not_found_response)
}

fn local_chat_completion_delete_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> Response {
    match local_chat_completion_delete_result(tenant_id, project_id, completion_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_stream_body_response() -> Response {
    let body = format!(
        "{}{}",
        SseFrame::data("{\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\"}"),
        SseFrame::data("[DONE]")
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn local_chat_completion_messages_result(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> std::result::Result<ListChatCompletionMessagesResponse, Response> {
    list_chat_completion_messages(tenant_id, project_id, completion_id).map_err(|error| {
        local_gateway_error_response(error, "Requested chat completion was not found.")
    })
}

fn local_chat_completion_messages_response(
    tenant_id: &str,
    project_id: &str,
    completion_id: &str,
) -> Response {
    match local_chat_completion_messages_result(tenant_id, project_id, completion_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested conversation was not found.")
}

fn local_conversation_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<ConversationObject, Response> {
    get_conversation(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_retrieve_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_update_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> std::result::Result<ConversationObject, Response> {
    update_conversation(tenant_id, project_id, conversation_id, metadata)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_update_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> Response {
    match local_conversation_update_result(tenant_id, project_id, conversation_id, metadata) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_delete_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<DeleteConversationResponse, Response> {
    delete_conversation(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_delete_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_delete_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_items_create_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<ListConversationItemsResponse, Response> {
    create_conversation_items(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_items_create_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_items_create_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_items_list_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> std::result::Result<ListConversationItemsResponse, Response> {
    list_conversation_items(tenant_id, project_id, conversation_id)
        .map_err(local_conversation_not_found_response)
}

fn local_conversation_items_list_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
) -> Response {
    match local_conversation_items_list_result(tenant_id, project_id, conversation_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_item_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested conversation item was not found.")
}

fn local_conversation_item_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> std::result::Result<ConversationItemObject, Response> {
    get_conversation_item(tenant_id, project_id, conversation_id, item_id)
        .map_err(local_conversation_item_not_found_response)
}

fn local_conversation_item_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Response {
    match local_conversation_item_retrieve_result(tenant_id, project_id, conversation_id, item_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_conversation_item_delete_result(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> std::result::Result<DeleteConversationItemResponse, Response> {
    delete_conversation_item(tenant_id, project_id, conversation_id, item_id)
        .map_err(local_conversation_item_not_found_response)
}

fn local_conversation_item_delete_response(
    tenant_id: &str,
    project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Response {
    match local_conversation_item_delete_result(tenant_id, project_id, conversation_id, item_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested thread was not found.")
}

fn local_thread_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ThreadObject, Response> {
    get_thread(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_retrieve_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_retrieve_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_update_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ThreadObject, Response> {
    update_thread(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_update_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_update_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_delete_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<DeleteThreadResponse, Response> {
    delete_thread(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_delete_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_delete_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("thread message not found")
    {
        return not_found_openai_response("Requested thread message was not found.");
    }

    local_thread_not_found_response(error)
}

fn local_thread_messages_create_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    role: &str,
    text: &str,
) -> std::result::Result<ThreadMessageObject, Response> {
    create_thread_message(tenant_id, project_id, thread_id, role, text)
        .map_err(local_thread_not_found_response)
}

fn local_thread_messages_list_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ListThreadMessagesResponse, Response> {
    list_thread_messages(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_messages_list_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> Response {
    match local_thread_messages_list_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> std::result::Result<ThreadMessageObject, Response> {
    get_thread_message(tenant_id, project_id, thread_id, message_id)
        .map_err(local_thread_message_not_found_response)
}

fn local_thread_message_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Response {
    match local_thread_message_retrieve_result(tenant_id, project_id, thread_id, message_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_update_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> std::result::Result<ThreadMessageObject, Response> {
    update_thread_message(tenant_id, project_id, thread_id, message_id)
        .map_err(local_thread_message_not_found_response)
}

fn local_thread_message_update_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Response {
    match local_thread_message_update_result(tenant_id, project_id, thread_id, message_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_message_delete_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> std::result::Result<DeleteThreadMessageResponse, Response> {
    delete_thread_message(tenant_id, project_id, thread_id, message_id)
        .map_err(local_thread_message_not_found_response)
}

fn local_thread_message_delete_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Response {
    match local_thread_message_delete_result(tenant_id, project_id, thread_id, message_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("run not found")
    {
        return not_found_openai_response("Requested run was not found.");
    }

    local_thread_not_found_response(error)
}

fn local_thread_run_step_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("run step not found")
    {
        return not_found_openai_response("Requested run step was not found.");
    }

    local_thread_run_not_found_response(error)
}

fn local_thread_runs_create_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> std::result::Result<RunObject, Response> {
    create_thread_run(tenant_id, project_id, thread_id, assistant_id, model)
        .map_err(local_thread_not_found_response)
}

fn local_thread_runs_list_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
) -> std::result::Result<ListRunsResponse, Response> {
    list_thread_runs(tenant_id, project_id, thread_id).map_err(local_thread_not_found_response)
}

fn local_thread_runs_list_response(tenant_id: &str, project_id: &str, thread_id: &str) -> Response {
    match local_thread_runs_list_result(tenant_id, project_id, thread_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<RunObject, Response> {
    get_thread_run(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_retrieve_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_update_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<RunObject, Response> {
    update_thread_run(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_update_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_update_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_cancel_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<RunObject, Response> {
    cancel_thread_run(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_cancel_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_cancel_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_submit_tool_outputs_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
    tool_outputs: Vec<(&str, &str)>,
) -> std::result::Result<RunObject, Response> {
    submit_thread_run_tool_outputs(tenant_id, project_id, thread_id, run_id, tool_outputs)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_steps_list_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> std::result::Result<ListRunStepsResponse, Response> {
    list_thread_run_steps(tenant_id, project_id, thread_id, run_id)
        .map_err(local_thread_run_not_found_response)
}

fn local_thread_run_steps_list_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Response {
    match local_thread_run_steps_list_result(tenant_id, project_id, thread_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_thread_run_step_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> std::result::Result<RunStepObject, Response> {
    get_thread_run_step(tenant_id, project_id, thread_id, run_id, step_id)
        .map_err(local_thread_run_step_not_found_response)
}

fn local_thread_run_step_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Response {
    match local_thread_run_step_retrieve_result(tenant_id, project_id, thread_id, run_id, step_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_anthropic_count_tokens_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    match count_response_input_tokens(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Json(openai_count_tokens_to_anthropic(&value)).into_response(),
            Err(error) => anthropic_bad_gateway_response(error.to_string()),
        },
        Err(error) => {
            let message = error.to_string();
            if message.to_ascii_lowercase().contains("required") {
                return anthropic_invalid_request_response(message);
            }

            anthropic_bad_gateway_response(message)
        }
    }
}

fn local_anthropic_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return anthropic_invalid_request_response(message);
    }

    anthropic_bad_gateway_response(message)
}

fn local_anthropic_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_chat_response_to_anthropic(&value)),
            Err(error) => Err(anthropic_bad_gateway_response(error.to_string())),
        },
        Err(error) => Err(local_anthropic_invalid_model_response(error)),
    }
}

fn local_anthropic_stream_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Response, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(_) => Ok(local_anthropic_stream_body_response(model)),
        Err(error) => Err(local_anthropic_invalid_model_response(error)),
    }
}

fn local_gemini_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return gemini_invalid_request_response(message);
    }

    gemini_bad_gateway_response(message)
}

fn local_gemini_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_chat_response_to_gemini(&value)),
            Err(error) => Err(gemini_bad_gateway_response(error.to_string())),
        },
        Err(error) => Err(local_gemini_invalid_model_response(error)),
    }
}

fn local_gemini_stream_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Response, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(_) => Ok(local_gemini_stream_body_response()),
        Err(error) => Err(local_gemini_invalid_model_response(error)),
    }
}

fn local_gemini_count_tokens_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match count_response_input_tokens(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_count_tokens_to_gemini(&value)),
            Err(error) => Err(gemini_bad_gateway_response(error.to_string())),
        },
        Err(error) => {
            let message = error.to_string();
            if local_gateway_error_is_invalid_request(&message) {
                return Err(gemini_invalid_request_response(message));
            }

            Err(gemini_bad_gateway_response(message))
        }
    }
}

async fn completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Completions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_completion_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => bad_gateway_openai_response("failed to relay upstream completion"),
    }
}

async fn embeddings_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Embeddings(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_embedding_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => bad_gateway_openai_response("failed to relay upstream embedding"),
    }
}

async fn moderations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Moderations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {
            return local_moderation_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            );
        }
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }
}

async fn image_generations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ImagesGenerations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {
            return local_image_generation_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            );
        }
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }
}

async fn image_edits_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_edit_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesEdits(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image edit");
                }
            }

            Json(
                create_image_edit(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image edit"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn image_variations_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_variation_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesVariations(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image variation");
                }
            }

            Json(
                create_image_variation(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image variation"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn transcriptions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioTranscriptions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    Json(
        create_transcription(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("transcription"),
    )
    .into_response()
}

async fn translations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioTranslations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    Json(
        create_translation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("translation"),
    )
    .into_response()
}

async fn audio_speech_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::AudioSpeech(&request))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio speech");
        }
    }

    local_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
}

async fn audio_voices_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AudioVoicesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voices list");
        }
    }

    Json(
        list_audio_voices(request_context.tenant_id(), request_context.project_id())
            .expect("audio voices list"),
    )
    .into_response()
}

async fn audio_voice_consents_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVoiceConsentRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioVoiceConsents(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voice consent");
        }
    }

    Json(
        create_audio_voice_consent(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("audio voice consent"),
    )
    .into_response()
}

async fn files_handler(request_context: StatelessGatewayRequest, multipart: Multipart) -> Response {
    match parse_file_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(&request_context, ProviderRequest::Files(&request))
                .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream file");
                }
            }

            local_file_create_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
        }
        Err(response) => response,
    }
}

async fn files_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    Json(list_files(request_context.tenant_id(), request_context.project_id()).expect("files list"))
        .into_response()
}

async fn file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesRetrieve(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    local_file_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesDelete(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    local_file_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn file_content_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::FilesContent(&file_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

fn local_container_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested container was not found.")
}

fn local_container_file_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("container file not found")
    {
        return not_found_openai_response("Requested container file was not found.");
    }

    local_container_not_found_response(error)
}

fn local_container_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> std::result::Result<ContainerObject, Response> {
    sdkwork_api_app_gateway::get_container(tenant_id, project_id, container_id)
        .map_err(local_container_not_found_response)
}

fn local_container_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Response {
    match local_container_retrieve_result(tenant_id, project_id, container_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_delete_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> std::result::Result<DeleteContainerResponse, Response> {
    sdkwork_api_app_gateway::delete_container(tenant_id, project_id, container_id)
        .map_err(local_container_not_found_response)
}

fn local_container_delete_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Response {
    match local_container_delete_result(tenant_id, project_id, container_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_create_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    request: &CreateContainerFileRequest,
) -> std::result::Result<ContainerFileObject, Response> {
    sdkwork_api_app_gateway::create_container_file(tenant_id, project_id, container_id, request)
        .map_err(local_container_not_found_response)
}

fn local_container_files_list_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> std::result::Result<ListContainerFilesResponse, Response> {
    sdkwork_api_app_gateway::list_container_files(tenant_id, project_id, container_id)
        .map_err(local_container_not_found_response)
}

fn local_container_files_list_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Response {
    match local_container_files_list_result(tenant_id, project_id, container_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> std::result::Result<ContainerFileObject, Response> {
    sdkwork_api_app_gateway::get_container_file(tenant_id, project_id, container_id, file_id)
        .map_err(local_container_file_not_found_response)
}

fn local_container_file_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    match local_container_file_retrieve_result(tenant_id, project_id, container_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_delete_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteContainerFileResponse, Response> {
    sdkwork_api_app_gateway::delete_container_file(tenant_id, project_id, container_id, file_id)
        .map_err(local_container_file_not_found_response)
}

fn local_container_file_delete_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    match local_container_file_delete_result(tenant_id, project_id, container_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn containers_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateContainerRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Containers(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container");
        }
    }

    match sdkwork_api_app_gateway::create_container(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => bad_gateway_openai_response(error.to_string()),
    }
}

async fn containers_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ContainersList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream containers list");
        }
    }

    match sdkwork_api_app_gateway::list_containers(
        request_context.tenant_id(),
        request_context.project_id(),
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => bad_gateway_openai_response(error.to_string()),
    }
}

async fn container_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersRetrieve(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container retrieve");
        }
    }

    local_container_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
}

async fn container_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersDelete(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container delete");
        }
    }

    local_container_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
}

async fn container_files_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateContainerFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFiles(&container_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file");
        }
    }

    match local_container_file_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn container_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesList(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container files list");
        }
    }

    local_container_files_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
}

async fn container_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesRetrieve(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file retrieve");
        }
    }

    local_container_file_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn container_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesDelete(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file delete");
        }
    }

    local_container_file_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn container_file_content_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::ContainerFilesContent(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file content");
        }
    }

    local_container_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn music_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Music(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    Json(
        create_music(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("music create"),
    )
    .into_response()
}

async fn music_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    Json(list_music(request_context.tenant_id(), request_context.project_id()).expect("music list"))
        .into_response()
}

async fn music_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicRetrieve(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    local_music_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicDelete(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    local_music_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_content_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::MusicContent(&music_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_lyrics_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicLyrics(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    Json(
        create_music_lyrics(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("music lyrics"),
    )
    .into_response()
}

async fn videos_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Videos(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video");
        }
    }

    Json(
        create_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
            &request.prompt,
        )
        .expect("video"),
    )
    .into_response()
}

async fn videos_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    Json(
        list_videos(request_context.tenant_id(), request_context.project_id())
            .expect("videos list"),
    )
    .into_response()
}

async fn video_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosRetrieve(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    local_video_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosDelete(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    local_video_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_content_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::VideosContent(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    local_video_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_remix_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosRemix(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    Json(
        remix_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &request.prompt,
        )
        .expect("video remix"),
    )
    .into_response()
}

async fn video_characters_list_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersList(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video characters list");
        }
    }

    Json(
        list_video_characters(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video characters list"),
    )
    .into_response()
}

async fn video_character_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((video_id, character_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersRetrieve(&video_id, &character_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    Json(
        get_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
        )
        .expect("video character retrieve"),
    )
    .into_response()
}

async fn video_character_update_handler(
    request_context: StatelessGatewayRequest,
    Path((video_id, character_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateVideoCharacterRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersUpdate(&video_id, &character_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character update");
        }
    }

    Json(
        update_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
            &request,
        )
        .expect("video character update"),
    )
    .into_response()
}

async fn video_extend_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosExtend(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extend");
        }
    }

    Json(
        extend_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &request.prompt,
        )
        .expect("video extend"),
    )
    .into_response()
}

async fn video_character_create_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoCharacterRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCreate(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character create");
        }
    }

    Json(
        sdkwork_api_app_gateway::create_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("video character create"),
    )
    .into_response()
}

async fn video_character_retrieve_canonical_handler(
    request_context: StatelessGatewayRequest,
    Path(character_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCanonicalRetrieve(&character_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::get_video_character_canonical(
            request_context.tenant_id(),
            request_context.project_id(),
            &character_id,
        )
        .expect("video character canonical retrieve"),
    )
    .into_response()
}

async fn video_edits_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<EditVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosEdits(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video edits");
        }
    }

    Json(
        sdkwork_api_app_gateway::edit_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("video edits"),
    )
    .into_response()
}

async fn video_extensions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosExtensions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extensions");
        }
    }

    Json(
        sdkwork_api_app_gateway::extensions_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("video extensions"),
    )
    .into_response()
}

fn local_upload_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested upload session was not found.")
}

fn local_upload_part_result(
    tenant_id: &str,
    project_id: &str,
    request: &AddUploadPartRequest,
) -> std::result::Result<UploadPartObject, Response> {
    create_upload_part(tenant_id, project_id, request).map_err(local_upload_not_found_response)
}

fn local_upload_part_response(
    tenant_id: &str,
    project_id: &str,
    request: &AddUploadPartRequest,
) -> Response {
    match local_upload_part_result(tenant_id, project_id, request) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_upload_complete_result(
    tenant_id: &str,
    project_id: &str,
    request: &CompleteUploadRequest,
) -> std::result::Result<UploadObject, Response> {
    complete_upload(tenant_id, project_id, request).map_err(local_upload_not_found_response)
}

fn local_upload_complete_response(
    tenant_id: &str,
    project_id: &str,
    request: &CompleteUploadRequest,
) -> Response {
    match local_upload_complete_result(tenant_id, project_id, request) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_upload_cancel_result(
    tenant_id: &str,
    project_id: &str,
    upload_id: &str,
) -> std::result::Result<UploadObject, Response> {
    cancel_upload(tenant_id, project_id, upload_id).map_err(local_upload_not_found_response)
}

fn local_upload_cancel_response(tenant_id: &str, project_id: &str, upload_id: &str) -> Response {
    match local_upload_cancel_result(tenant_id, project_id, upload_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn uploads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Uploads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload");
        }
    }

    Json(
        create_upload(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("upload"),
    )
    .into_response()
}

async fn upload_parts_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
    multipart: Multipart,
) -> Response {
    match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::UploadParts(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream upload part");
                }
            }

            local_upload_part_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
        }
        Err(response) => response,
    }
}

async fn upload_complete_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;
    match relay_stateless_json_request(&request_context, ProviderRequest::UploadComplete(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload complete");
        }
    }

    local_upload_complete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
}

async fn upload_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::UploadCancel(&upload_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload cancel");
        }
    }

    local_upload_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    )
}

fn local_fine_tuning_job_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested fine-tuning job was not found.")
}

fn local_fine_tuning_checkpoint_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested fine-tuning checkpoint was not found.")
}

fn local_fine_tuning_checkpoint_permission_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("fine tuning checkpoint permission not found")
    {
        return not_found_openai_response(
            "Requested fine-tuning checkpoint permission was not found.",
        );
    }

    local_fine_tuning_checkpoint_not_found_response(error)
}

fn local_fine_tuning_job_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    get_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_retrieve_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_cancel_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    cancel_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_cancel_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_cancel_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_events_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<ListFineTuningJobEventsResponse, Response> {
    list_fine_tuning_job_events(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_events_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_events_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_checkpoints_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<ListFineTuningJobCheckpointsResponse, Response> {
    list_fine_tuning_job_checkpoints(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_checkpoints_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_checkpoints_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_pause_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    sdkwork_api_app_gateway::pause_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_pause_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_pause_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_resume_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    sdkwork_api_app_gateway::resume_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_resume_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_resume_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_checkpoint_permissions_create_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> std::result::Result<ListFineTuningCheckpointPermissionsResponse, Response> {
    sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        request,
    )
    .map_err(local_fine_tuning_checkpoint_not_found_response)
}

fn local_fine_tuning_checkpoint_permissions_create_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Response {
    match local_fine_tuning_checkpoint_permissions_create_result(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_checkpoint_permissions_list_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
) -> std::result::Result<ListFineTuningCheckpointPermissionsResponse, Response> {
    sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
    )
    .map_err(local_fine_tuning_checkpoint_not_found_response)
}

fn local_fine_tuning_checkpoint_permissions_list_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
) -> Response {
    match local_fine_tuning_checkpoint_permissions_list_result(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_checkpoint_permission_delete_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    permission_id: &str,
) -> std::result::Result<DeleteFineTuningCheckpointPermissionResponse, Response> {
    sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        permission_id,
    )
    .map_err(local_fine_tuning_checkpoint_permission_not_found_response)
}

fn local_fine_tuning_checkpoint_permission_delete_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    permission_id: &str,
) -> Response {
    match local_fine_tuning_checkpoint_permission_delete_result(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        permission_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn fine_tuning_jobs_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobs(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    Json(
        create_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("fine tuning"),
    )
    .into_response()
}

async fn fine_tuning_jobs_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobsList).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    Json(
        list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id())
            .expect("fine tuning list"),
    )
    .into_response()
}

async fn fine_tuning_job_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsRetrieve(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    local_fine_tuning_job_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCancel(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    local_fine_tuning_job_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_events_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsEvents(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    local_fine_tuning_job_events_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_checkpoints_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCheckpoints(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    local_fine_tuning_job_checkpoints_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_pause_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsPause(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job pause");
        }
    }

    local_fine_tuning_job_pause_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_resume_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsResume(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job resume");
        }
    }

    local_fine_tuning_job_resume_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_checkpoint_permissions_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissions(&fine_tuned_model_checkpoint, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    local_fine_tuning_checkpoint_permissions_create_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    )
}

async fn fine_tuning_checkpoint_permissions_list_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsList(&fine_tuned_model_checkpoint),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    local_fine_tuning_checkpoint_permissions_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    )
}

async fn fine_tuning_checkpoint_permission_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsDelete(
            &fine_tuned_model_checkpoint,
            &permission_id,
        ),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    local_fine_tuning_checkpoint_permission_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    )
}

async fn assistants_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Assistants(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    Json(
        create_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
            &request.model,
        )
        .expect("assistant"),
    )
    .into_response()
}

async fn assistants_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AssistantsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    Json(
        list_assistants(request_context.tenant_id(), request_context.project_id())
            .expect("assistants list"),
    )
    .into_response()
}

fn local_assistant_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested assistant was not found.")
}

fn local_assistant_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> std::result::Result<AssistantObject, Response> {
    get_assistant(tenant_id, project_id, assistant_id).map_err(local_assistant_not_found_response)
}

fn local_assistant_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> Response {
    match local_assistant_retrieve_result(tenant_id, project_id, assistant_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_assistant_update_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
    name: &str,
) -> std::result::Result<AssistantObject, Response> {
    update_assistant(tenant_id, project_id, assistant_id, name)
        .map_err(local_assistant_not_found_response)
}

fn local_assistant_update_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
    name: &str,
) -> Response {
    match local_assistant_update_result(tenant_id, project_id, assistant_id, name) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_assistant_delete_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> std::result::Result<DeleteAssistantResponse, Response> {
    delete_assistant(tenant_id, project_id, assistant_id)
        .map_err(local_assistant_not_found_response)
}

fn local_assistant_delete_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> Response {
    match local_assistant_delete_result(tenant_id, project_id, assistant_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn assistant_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsRetrieve(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    local_assistant_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
}

async fn assistant_update_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsUpdate(&assistant_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    local_assistant_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        request.name.as_deref().unwrap_or("assistant"),
    )
}

async fn assistant_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsDelete(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    local_assistant_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
}

async fn webhooks_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Webhooks(&request)).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    Json(
        create_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.url,
            &request.events,
        )
        .expect("webhook"),
    )
    .into_response()
}

async fn webhooks_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::WebhooksList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    Json(
        list_webhooks(request_context.tenant_id(), request_context.project_id())
            .expect("webhooks list"),
    )
    .into_response()
}

fn local_webhook_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested webhook was not found.")
}

fn local_webhook_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
) -> std::result::Result<WebhookObject, Response> {
    get_webhook(tenant_id, project_id, webhook_id).map_err(local_webhook_not_found_response)
}

fn local_webhook_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
) -> Response {
    match local_webhook_retrieve_result(tenant_id, project_id, webhook_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_webhook_update_result(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
    url: &str,
) -> std::result::Result<WebhookObject, Response> {
    update_webhook(tenant_id, project_id, webhook_id, url).map_err(local_webhook_not_found_response)
}

fn local_webhook_update_response(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
    url: &str,
) -> Response {
    match local_webhook_update_result(tenant_id, project_id, webhook_id, url) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_webhook_delete_result(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
) -> std::result::Result<DeleteWebhookResponse, Response> {
    delete_webhook(tenant_id, project_id, webhook_id).map_err(local_webhook_not_found_response)
}

fn local_webhook_delete_response(tenant_id: &str, project_id: &str, webhook_id: &str) -> Response {
    match local_webhook_delete_result(tenant_id, project_id, webhook_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn webhook_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksRetrieve(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    local_webhook_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
}

async fn webhook_update_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksUpdate(&webhook_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    local_webhook_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        request
            .url
            .as_deref()
            .unwrap_or("https://example.com/webhook"),
    )
}

async fn webhook_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksDelete(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    local_webhook_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
}

async fn realtime_sessions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::RealtimeSessions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream realtime session");
        }
    }

    Json(
        create_realtime_session(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("realtime session"),
    )
    .into_response()
}

async fn evals_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Evals(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    match create_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => bad_gateway_openai_response(error.to_string()),
    }
}

async fn evals_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    match list_evals(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => Json(response).into_response(),
        Err(error) => bad_gateway_openai_response(error.to_string()),
    }
}

fn local_eval_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested eval was not found.")
}

fn local_eval_run_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("eval run not found")
    {
        return not_found_openai_response("Requested eval run was not found.");
    }

    local_eval_not_found_response(error)
}

fn local_eval_run_output_item_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("eval run output item not found")
    {
        return not_found_openai_response("Requested eval run output item was not found.");
    }

    local_eval_run_not_found_response(error)
}

fn local_eval_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
) -> std::result::Result<EvalObject, Response> {
    get_eval(tenant_id, project_id, eval_id).map_err(local_eval_not_found_response)
}

fn local_eval_retrieve_response(tenant_id: &str, project_id: &str, eval_id: &str) -> Response {
    match local_eval_retrieve_result(tenant_id, project_id, eval_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_update_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> std::result::Result<EvalObject, Response> {
    update_eval(tenant_id, project_id, eval_id, request).map_err(local_eval_not_found_response)
}

fn local_eval_update_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Response {
    match local_eval_update_result(tenant_id, project_id, eval_id, request) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_delete_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
) -> std::result::Result<DeleteEvalResponse, Response> {
    delete_eval(tenant_id, project_id, eval_id).map_err(local_eval_not_found_response)
}

fn local_eval_delete_response(tenant_id: &str, project_id: &str, eval_id: &str) -> Response {
    match local_eval_delete_result(tenant_id, project_id, eval_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_runs_list_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
) -> std::result::Result<ListEvalRunsResponse, Response> {
    list_eval_runs(tenant_id, project_id, eval_id).map_err(local_eval_not_found_response)
}

fn local_eval_runs_list_response(tenant_id: &str, project_id: &str, eval_id: &str) -> Response {
    match local_eval_runs_list_result(tenant_id, project_id, eval_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_create_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    request: &CreateEvalRunRequest,
) -> std::result::Result<EvalRunObject, Response> {
    create_eval_run(tenant_id, project_id, eval_id, request).map_err(local_eval_not_found_response)
}

fn local_eval_run_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<EvalRunObject, Response> {
    get_eval_run(tenant_id, project_id, eval_id, run_id).map_err(local_eval_run_not_found_response)
}

fn local_eval_run_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_retrieve_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_delete_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<DeleteEvalRunResponse, Response> {
    sdkwork_api_app_gateway::delete_eval_run(tenant_id, project_id, eval_id, run_id)
        .map_err(local_eval_run_not_found_response)
}

fn local_eval_run_delete_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_delete_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_cancel_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<EvalRunObject, Response> {
    sdkwork_api_app_gateway::cancel_eval_run(tenant_id, project_id, eval_id, run_id)
        .map_err(local_eval_run_not_found_response)
}

fn local_eval_run_cancel_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_cancel_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_output_items_list_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<ListEvalRunOutputItemsResponse, Response> {
    sdkwork_api_app_gateway::list_eval_run_output_items(tenant_id, project_id, eval_id, run_id)
        .map_err(local_eval_run_not_found_response)
}

fn local_eval_run_output_items_list_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_output_items_list_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_output_item_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> std::result::Result<EvalRunOutputItemObject, Response> {
    sdkwork_api_app_gateway::get_eval_run_output_item(
        tenant_id,
        project_id,
        eval_id,
        run_id,
        output_item_id,
    )
    .map_err(local_eval_run_output_item_not_found_response)
}

fn local_eval_run_output_item_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Response {
    match local_eval_run_output_item_retrieve_result(
        tenant_id,
        project_id,
        eval_id,
        run_id,
        output_item_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn eval_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsRetrieve(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    local_eval_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
}

async fn eval_update_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalsUpdate(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    local_eval_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
}

async fn eval_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsDelete(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    local_eval_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
}

async fn eval_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalRunsList(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    local_eval_runs_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
}

async fn eval_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRuns(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    match local_eval_run_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn eval_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsRetrieve(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    local_eval_run_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsDelete(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    local_eval_run_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsCancel(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    local_eval_run_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_output_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsList(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    local_eval_run_output_items_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_output_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsRetrieve(&eval_id, &run_id, &output_item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    local_eval_run_output_item_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    )
}

async fn batches_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Batches(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    Json(
        create_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.endpoint,
            &request.input_file_id,
        )
        .expect("batch"),
    )
    .into_response()
}

async fn batches_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    Json(
        list_batches(request_context.tenant_id(), request_context.project_id())
            .expect("batches list"),
    )
    .into_response()
}

fn local_batch_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested batch was not found.")
}

fn local_batch_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    batch_id: &str,
) -> std::result::Result<BatchObject, Response> {
    get_batch(tenant_id, project_id, batch_id).map_err(local_batch_not_found_response)
}

fn local_batch_retrieve_response(tenant_id: &str, project_id: &str, batch_id: &str) -> Response {
    match local_batch_retrieve_result(tenant_id, project_id, batch_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_batch_cancel_result(
    tenant_id: &str,
    project_id: &str,
    batch_id: &str,
) -> std::result::Result<BatchObject, Response> {
    cancel_batch(tenant_id, project_id, batch_id).map_err(local_batch_not_found_response)
}

fn local_batch_cancel_response(tenant_id: &str, project_id: &str, batch_id: &str) -> Response {
    match local_batch_cancel_result(tenant_id, project_id, batch_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::BatchesRetrieve(&batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    local_batch_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
}

async fn batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesCancel(&batch_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    local_batch_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
}

async fn vector_stores_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStoresList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }

    Json(
        list_vector_stores(request_context.tenant_id(), request_context.project_id())
            .expect("vector stores list"),
    )
    .into_response()
}

async fn vector_stores_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStores(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }

    Json(
        create_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
        )
        .expect("vector store"),
    )
    .into_response()
}

fn local_vector_store_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested vector store was not found.")
}

fn local_vector_store_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> std::result::Result<VectorStoreObject, Response> {
    get_vector_store(tenant_id, project_id, vector_store_id)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> Response {
    match local_vector_store_retrieve_result(tenant_id, project_id, vector_store_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_update_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> std::result::Result<VectorStoreObject, Response> {
    update_vector_store(tenant_id, project_id, vector_store_id, name)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_update_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> Response {
    match local_vector_store_update_result(tenant_id, project_id, vector_store_id, name) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_delete_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> std::result::Result<DeleteVectorStoreResponse, Response> {
    delete_vector_store(tenant_id, project_id, vector_store_id)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_delete_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> Response {
    match local_vector_store_delete_result(tenant_id, project_id, vector_store_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn vector_store_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresRetrieve(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }

    local_vector_store_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
}

async fn vector_store_update_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresUpdate(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }

    local_vector_store_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        request.name.as_deref().unwrap_or("vector-store"),
    )
}

async fn vector_store_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresDelete(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }

    local_vector_store_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
}

async fn vector_store_search_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresSearch(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    Json(
        search_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.query,
        )
        .expect("vector store search"),
    )
    .into_response()
}

async fn vector_store_files_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFiles(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }

    Json(
        create_vector_store_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.file_id,
        )
        .expect("vector store file"),
    )
    .into_response()
}

async fn vector_store_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesList(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }

    Json(
        list_vector_store_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store files list"),
    )
    .into_response()
}

async fn vector_store_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesRetrieve(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }

    local_vector_store_file_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
}

async fn vector_store_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesDelete(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }

    local_vector_store_file_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
}

async fn vector_store_file_batches_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatches(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }

    Json(
        create_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.file_ids,
        )
        .expect("vector store file batch"),
    )
    .into_response()
}

async fn vector_store_file_batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesRetrieve(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }

    local_vector_store_file_batch_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
}

async fn vector_store_file_batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesCancel(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }

    local_vector_store_file_batch_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
}

async fn vector_store_file_batch_files_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesListFiles(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }

    local_vector_store_file_batch_files_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
}

async fn chat_completions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    let options = ProviderRequestOptions::default();
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.10,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 100)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    if request.stream.unwrap_or(false) {
        match relay_chat_completion_stream_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    let usage_result = record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        100,
                        0.10,
                        usage_context.as_ref(),
                    )
                    .await;
                    if usage_result.is_err() {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return upstream_passthrough_response(response);
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "failed to relay upstream chat completion stream",
                );
            }
        }
    } else {
        match relay_chat_completion_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    let token_usage = extract_token_usage_metrics(&response);
                    let usage_result =
                        record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        100,
                        0.10,
                        token_usage,
                        response_usage_id_or_single_data_item_id(&response),
                        usage_context.as_ref(),
                    )
                    .await;
                    if usage_result.is_err() {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return Json(response).into_response();
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("failed to relay upstream chat completion");
            }
        }
    }

    let local_chat_completion = match local_chat_completion_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    let usage_result = record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        100,
        0.10,
    )
    .await;
    if usage_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if request.stream.unwrap_or(false) {
        local_chat_completion_stream_body_response()
    } else {
        Json(local_chat_completion).into_response()
    }
}

async fn anthropic_messages_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(response));
            }
            Ok(None) => {
                return match local_anthropic_stream_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) | Err(response) => response,
                };
            }
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }
    }

    match relay_stateless_json_request_with_options(
        &request_context,
        ProviderRequest::ChatCompletions(&request),
        &options,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_chat_response_to_anthropic(&response)).into_response(),
        Ok(None) => match local_anthropic_chat_completion_result(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ) {
            Ok(response) => Json(response).into_response(),
            Err(response) => response,
        },
        Err(_) => anthropic_bad_gateway_response("failed to relay upstream anthropic message"),
    }
}

async fn anthropic_count_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => local_anthropic_count_tokens_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}

async fn anthropic_messages_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.10,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 100)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    if request.stream.unwrap_or(false) {
        match relay_chat_completion_stream_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    if record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        100,
                        0.10,
                        usage_context.as_ref(),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return upstream_passthrough_response(anthropic_stream_from_openai(response));
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        let local_response = match local_anthropic_stream_result(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ) {
            Ok(response) => response,
            Err(response) => return response,
        };

        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }

        if record_gateway_usage_for_project(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "chat_completion",
            &request.model,
            100,
            0.10,
        )
        .await
        .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to record usage",
            )
                .into_response();
        }

        return local_response;
    }

    match relay_chat_completion_from_store_with_execution_context(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
        &options,
    )
    .await
    {
        Ok(execution) => {
            let usage_context = execution.usage_context;
            if let Some(response) = execution.response {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        capture_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                let token_usage = extract_token_usage_metrics(&response);
                if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "chat_completion",
                    &request.model,
                    &request.model,
                    100,
                    0.10,
                    token_usage,
                    response_usage_id_or_single_data_item_id(&response),
                    usage_context.as_ref(),
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return Json(openai_chat_response_to_anthropic(&response)).into_response();
            }
        }
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return anthropic_bad_gateway_response("failed to relay upstream anthropic message");
        }
    }

    let local_response = match local_anthropic_chat_completion_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        100,
        0.10,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn anthropic_count_tokens_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_count_response_input_tokens_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => local_anthropic_count_tokens_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}

async fn gemini_models_compat_handler(
    request_context: StatelessGatewayRequest,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ChatCompletions(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_chat_response_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_chat_completion_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini generateContent request",
                ),
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

            match relay_stateless_stream_request(
                &request_context,
                ProviderRequest::ChatCompletionsStream(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    upstream_passthrough_response(gemini_stream_from_openai(response))
                }
                Ok(None) => match local_gemini_stream_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) | Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini streamGenerateContent request",
                ),
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ResponsesInputTokens(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_count_tokens_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

async fn gemini_models_compat_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

            let commercial_admission = match begin_gateway_commercial_admission(
                &state,
                request_context.context(),
                GatewayCommercialAdmissionSpec {
                    quoted_amount: 0.10,
                },
            )
            .await
            {
                Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
                Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        100,
                    )
                    .await
                    {
                        Ok(Some(response)) => return response,
                        Ok(None) => {}
                        Err(_) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to evaluate quota",
                            )
                                .into_response();
                        }
                    }
                    None
                }
                Err(response) => return response,
            };

            let options = ProviderRequestOptions::default();
            match relay_chat_completion_from_store_with_execution_context(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
                &options,
            )
            .await
            {
                Ok(execution) => {
                    let usage_context = execution.usage_context;
                    if let Some(response) = execution.response {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                capture_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        let token_usage = extract_token_usage_metrics(&response);
                        if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            &request.model,
                            100,
                            0.10,
                            token_usage,
                            response_usage_id_or_single_data_item_id(&response),
                            usage_context.as_ref(),
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        Json(openai_chat_response_to_gemini(&response)).into_response()
                    } else {
                        let local_response = match local_gemini_chat_completion_result(
                            request_context.tenant_id(),
                            request_context.project_id(),
                            &request.model,
                        ) {
                            Ok(response) => response,
                            Err(response) => return response,
                        };

                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                capture_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        if record_gateway_usage_for_project(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            100,
                            0.10,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        Json(local_response).into_response()
                    }
                }
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    )
                }
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

            let commercial_admission = match begin_gateway_commercial_admission(
                &state,
                request_context.context(),
                GatewayCommercialAdmissionSpec {
                    quoted_amount: 0.10,
                },
            )
            .await
            {
                Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
                Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        100,
                    )
                    .await
                    {
                        Ok(Some(response)) => return response,
                        Ok(None) => {}
                        Err(_) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to evaluate quota",
                            )
                                .into_response();
                        }
                    }
                    None
                }
                Err(response) => return response,
            };

            let options = ProviderRequestOptions::default();
            match relay_chat_completion_stream_from_store_with_execution_context(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
                &options,
            )
            .await
            {
                Ok(execution) => {
                    let usage_context = execution.usage_context;
                    if let Some(response) = execution.response {
                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                capture_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        if record_gateway_usage_for_project_with_context(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            100,
                            0.10,
                            usage_context.as_ref(),
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        upstream_passthrough_response(gemini_stream_from_openai(response))
                    } else {
                        let local_response = match local_gemini_stream_result(
                            request_context.tenant_id(),
                            request_context.project_id(),
                            &request.model,
                        ) {
                            Ok(response) => response,
                            Err(response) => return response,
                        };

                        if let Some(admission) = commercial_admission.as_ref() {
                            if let Err(response) =
                                capture_gateway_commercial_admission(&state, admission).await
                            {
                                return response;
                            }
                        }
                        if record_gateway_usage_for_project(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            100,
                            0.10,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        local_response
                    }
                }
                Err(_) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    )
                }
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            match relay_count_response_input_tokens_from_store(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_count_tokens_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum GeminiCompatAction {
    GenerateContent,
    StreamGenerateContent,
    CountTokens,
}

fn parse_gemini_compat_tail(tail: &str) -> Option<(String, GeminiCompatAction)> {
    let tail = tail.trim_start_matches('/');
    let (model, action) = tail.split_once(':')?;
    let action = match action {
        "generateContent" => GeminiCompatAction::GenerateContent,
        "streamGenerateContent" => GeminiCompatAction::StreamGenerateContent,
        "countTokens" => GeminiCompatAction::CountTokens,
        _ => return None,
    };
    Some((model.to_owned(), action))
}

fn local_anthropic_stream_body_response(model: &str) -> Response {
    let body = format!(
        "event: message_start\ndata: {}\n\n\
event: message_delta\ndata: {}\n\n\
event: message_stop\ndata: {}\n\n",
        serde_json::json!({
            "type": "message_start",
            "message": {
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "model": model,
                "content": [],
                "stop_reason": Value::Null,
                "stop_sequence": Value::Null,
                "usage": {
                    "input_tokens": 0,
                    "output_tokens": 0
                }
            }
        }),
        serde_json::json!({
            "type": "message_delta",
            "delta": {
                "stop_reason": "end_turn",
                "stop_sequence": Value::Null
            },
            "usage": {
                "output_tokens": 0
            }
        }),
        serde_json::json!({
            "type": "message_stop"
        })
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn local_gemini_stream_body_response() -> Response {
    let body = format!(
        "data: {}\n\n",
        serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [
                        { "text": "" }
                    ]
                },
                "finishReason": "STOP"
            }]
        })
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

async fn chat_completions_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_chat_completions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                "chat_completions",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        "chat_completions",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_chat_completions(request_context.tenant_id(), request_context.project_id())
            .expect("chat completions"),
    )
    .into_response()
}

async fn chat_completion_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_get_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    let local_response = match local_chat_completion_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn chat_completion_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_update_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    let local_response = match local_chat_completion_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        request.metadata.unwrap_or(serde_json::json!({})),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn chat_completion_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_delete_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    let local_response = match local_chat_completion_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn chat_completion_messages_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_list_chat_completion_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion messages",
            );
        }
    }

    let local_response = match local_chat_completion_messages_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn conversations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let conversation_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("conversations");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                "conversations",
                conversation_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    let response = create_conversation(request_context.tenant_id(), request_context.project_id())
        .expect("conversation");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        "conversations",
        response.id.as_str(),
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn conversations_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_conversations_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                "conversations",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        "conversations",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_conversations(request_context.tenant_id(), request_context.project_id())
            .expect("conversation list"),
    )
    .into_response()
}

async fn conversation_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_get_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    let local_response = match local_conversation_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn conversation_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_update_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    let local_response = match local_conversation_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        request.metadata.unwrap_or(serde_json::json!({})),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn conversation_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_delete_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    let local_response = match local_conversation_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn conversation_items_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_conversation_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    let response = match local_conversation_items_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => conversation_id.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        usage_model,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn conversation_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_list_conversation_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    let local_response = match local_conversation_items_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn conversation_item_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_get_conversation_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                &item_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    let local_response = match local_conversation_item_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        &item_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn conversation_item_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_conversation_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                &item_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item delete",
            );
        }
    }

    let local_response = match local_conversation_item_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        &item_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn threads_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateThreadRequest>,
) -> Response {
    match relay_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let thread_usage_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("threads");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "threads",
                thread_usage_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread");
        }
    }

    let response =
        create_thread(request_context.tenant_id(), request_context.project_id()).expect("thread");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "threads",
        response.id.as_str(),
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_get_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread retrieve");
        }
    }

    let local_response = match local_thread_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn thread_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateThreadRequest>,
) -> Response {
    match relay_update_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread update");
        }
    }

    let local_response = match local_thread_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn thread_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_delete_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread delete");
        }
    }

    let local_response = match local_thread_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn thread_messages_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateThreadMessageRequest>,
) -> Response {
    match relay_thread_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let message_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(thread_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                message_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message");
        }
    }

    let text = request.content.as_str().unwrap_or("hello");
    let response = match local_thread_messages_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.role,
        text,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        response.id.as_str(),
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_messages_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_list_thread_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message list");
        }
    }

    let response = match local_thread_messages_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_message_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_get_thread_message_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &message_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message retrieve");
        }
    }

    let response = match local_thread_message_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &message_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_message_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, message_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateThreadMessageRequest>,
) -> Response {
    match relay_update_thread_message_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &message_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message update");
        }
    }

    let response = match local_thread_message_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &message_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_message_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_thread_message_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &message_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message delete");
        }
    }

    let response = match local_thread_message_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &message_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_and_run_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_thread_and_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or("threads/runs");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "threads/runs",
                usage_model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    let response = match local_thread_and_run_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.assistant_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "threads/runs",
        response.id.as_str(),
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let run_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(thread_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                run_id,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    let response = create_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.assistant_id,
        request.model.as_deref(),
    );

    let response = match response {
        Ok(response) => response,
        Err(error) => return local_thread_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        response.id.as_str(),
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_runs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_list_thread_runs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    let response = match local_thread_runs_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_run_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_get_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    let response = match local_thread_run_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_run_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_update_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    let response = match local_thread_run_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_run_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    let response = match local_thread_run_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_run_submit_tool_outputs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_submit_thread_run_tool_outputs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run tool outputs");
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    let response = match local_thread_run_submit_tool_outputs_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        tool_outputs,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_run_steps_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_list_thread_run_steps_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run steps");
        }
    }

    let response = match local_thread_run_steps_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn thread_run_step_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id, step_id)): Path<(String, String, String)>,
) -> Response {
    match relay_get_thread_run_step_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &step_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &step_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run step retrieve",
            );
        }
    }

    let response = match local_thread_run_step_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &step_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &step_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

fn upstream_passthrough_response(response: ProviderStreamOutput) -> Response {
    let content_type = response.content_type().to_owned();
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from_stream(response.into_body_stream()))
        .expect("valid upstream stream response")
}

fn local_file_content_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    match file_content(tenant_id, project_id, file_id) {
        Ok(bytes) => Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/jsonl")
            .body(Body::from(bytes))
            .expect("valid local file content response"),
        Err(error) => local_gateway_error_response(error, "Requested file was not found."),
    }
}

fn local_file_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested file was not found.")
}

fn local_file_create_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_file");
    }

    bad_gateway_openai_response(message)
}

fn local_file_create_result(
    tenant_id: &str,
    project_id: &str,
    request: &CreateFileRequest,
) -> std::result::Result<FileObject, Response> {
    create_file(tenant_id, project_id, request).map_err(local_file_create_error_response)
}

fn local_file_create_response(
    tenant_id: &str,
    project_id: &str,
    request: &CreateFileRequest,
) -> Response {
    match local_file_create_result(tenant_id, project_id, request) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_file_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    file_id: &str,
) -> std::result::Result<FileObject, Response> {
    get_file(tenant_id, project_id, file_id).map_err(local_file_not_found_response)
}

fn local_file_retrieve_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    match local_file_retrieve_result(tenant_id, project_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_file_delete_result(
    tenant_id: &str,
    project_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteFileResponse, Response> {
    delete_file(tenant_id, project_id, file_id).map_err(local_file_not_found_response)
}

fn local_file_delete_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    match local_file_delete_result(tenant_id, project_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested vector store file was not found.")
}

fn local_vector_store_file_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> std::result::Result<VectorStoreFileObject, Response> {
    get_vector_store_file(tenant_id, project_id, vector_store_id, file_id)
        .map_err(local_vector_store_file_not_found_response)
}

fn local_vector_store_file_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Response {
    match local_vector_store_file_retrieve_result(tenant_id, project_id, vector_store_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_delete_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteVectorStoreFileResponse, Response> {
    delete_vector_store_file(tenant_id, project_id, vector_store_id, file_id)
        .map_err(local_vector_store_file_not_found_response)
}

fn local_vector_store_file_delete_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Response {
    match local_vector_store_file_delete_result(tenant_id, project_id, vector_store_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_batch_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested vector store file batch was not found.")
}

fn local_vector_store_file_batch_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    get_vector_store_file_batch(tenant_id, project_id, vector_store_id, batch_id)
        .map_err(local_vector_store_file_batch_not_found_response)
}

fn local_vector_store_file_batch_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Response {
    match local_vector_store_file_batch_retrieve_result(
        tenant_id,
        project_id,
        vector_store_id,
        batch_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_batch_cancel_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    cancel_vector_store_file_batch(tenant_id, project_id, vector_store_id, batch_id)
        .map_err(local_vector_store_file_batch_not_found_response)
}

fn local_vector_store_file_batch_cancel_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Response {
    match local_vector_store_file_batch_cancel_result(
        tenant_id,
        project_id,
        vector_store_id,
        batch_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_batch_files_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<ListVectorStoreFilesResponse, Response> {
    list_vector_store_file_batch_files(tenant_id, project_id, vector_store_id, batch_id)
        .map_err(local_vector_store_file_batch_not_found_response)
}

fn local_vector_store_file_batch_files_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Response {
    match local_vector_store_file_batch_files_result(
        tenant_id,
        project_id,
        vector_store_id,
        batch_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_content_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> std::result::Result<Vec<u8>, Response> {
    sdkwork_api_app_gateway::container_file_content(tenant_id, project_id, container_id, file_id)
        .map_err(local_container_file_not_found_response)
}

fn local_container_file_content_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    let bytes =
        match local_container_file_content_result(tenant_id, project_id, container_id, file_id) {
            Ok(bytes) => bytes,
            Err(response) => return response,
        };
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(bytes))
        .expect("valid local container file content response")
}

fn local_video_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video was not found.")
}

fn local_video_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> std::result::Result<VideoObject, Response> {
    get_video(tenant_id, project_id, video_id).map_err(local_video_not_found_response)
}

fn local_video_retrieve_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    match local_video_retrieve_result(tenant_id, project_id, video_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_video_delete_result(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> std::result::Result<DeleteVideoResponse, Response> {
    delete_video(tenant_id, project_id, video_id).map_err(local_video_not_found_response)
}

fn local_video_delete_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    match local_video_delete_result(tenant_id, project_id, video_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_video_content_result(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> std::result::Result<Vec<u8>, Response> {
    video_content(tenant_id, project_id, video_id).map_err(|error| {
        local_gateway_error_response(error, "Requested video asset was not found.")
    })
}

fn local_video_content_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    let bytes = match local_video_content_result(tenant_id, project_id, video_id) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .expect("valid local video content response")
}

fn local_music_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested music was not found.")
}

fn local_music_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> std::result::Result<MusicObject, Response> {
    get_music(tenant_id, project_id, music_id).map_err(local_music_not_found_response)
}

fn local_music_retrieve_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    match local_music_retrieve_result(tenant_id, project_id, music_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_music_delete_result(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> std::result::Result<DeleteMusicResponse, Response> {
    delete_music(tenant_id, project_id, music_id).map_err(local_music_not_found_response)
}

fn local_music_delete_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    match local_music_delete_result(tenant_id, project_id, music_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_music_content_result(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> std::result::Result<Vec<u8>, Response> {
    music_content(tenant_id, project_id, music_id).map_err(|error| {
        local_gateway_error_response(error, "Requested music asset was not found.")
    })
}

fn local_music_content_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    let bytes = match local_music_content_result(tenant_id, project_id, music_id) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(Body::from(bytes))
        .expect("valid local music content response")
}

async fn responses_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.12,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 120)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    if request.stream.unwrap_or(false) {
        match relay_response_stream_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    if record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "responses",
                        &request.model,
                        120,
                        0.12,
                        usage_context.as_ref(),
                    )
                    .await
                    .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return upstream_passthrough_response(response);
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        let _local_response = match local_response_create_result(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ) {
            Ok(response) => response,
            Err(response) => return response,
        };

        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }

        if record_gateway_usage_for_project(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "responses",
            &request.model,
            120,
            0.12,
        )
        .await
        .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to record usage",
            )
                .into_response();
        }

        return local_response_stream_body_response("resp_1", &request.model);
    }

    match relay_response_from_store_with_execution_context(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(execution) => {
            let usage_context = execution.usage_context;
            if let Some(response) = execution.response {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        capture_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                let token_usage = extract_token_usage_metrics(&response);
                if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "responses",
                    &request.model,
                    &request.model,
                    120,
                    0.12,
                    token_usage,
                    response_usage_id_or_single_data_item_id(&response),
                    usage_context.as_ref(),
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return Json(response).into_response();
            }
        }
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream response");
        }
    }

    let local_response = match local_response_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
        120,
        0.12,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn response_input_tokens_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_count_response_input_tokens_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &request.model,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    let local_response = match local_response_input_tokens_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_response).into_response()
}

async fn response_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_get_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    let response = match get_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn response_input_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_list_response_input_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    let response = match list_response_input_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn response_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_delete_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    let response = match delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn response_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_cancel_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    let response = match cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn response_compact_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_compact_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &request.model,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    let compacted_response = match compact_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => {
            let message = error.to_string();
            if message.to_ascii_lowercase().contains("required") {
                return invalid_request_openai_response(message, "invalid_model");
            }

            return bad_gateway_openai_response(message);
        }
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(compacted_response).into_response()
}

async fn completions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.08,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 80)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    match relay_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "completions",
                &request.model,
                &request.model,
                80,
                0.08,
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream completion");
        }
    }

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    let local_completion = match local_completion_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "completions",
        &request.model,
        80,
        0.08,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_completion).into_response()
}

async fn embeddings_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.01,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 10)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    match relay_embedding_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            let token_usage = extract_token_usage_metrics(&response);
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "embeddings",
                &request.model,
                &request.model,
                10,
                0.01,
                token_usage,
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream embedding");
        }
    }

    let local_embedding = match local_embedding_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(release_response) =
                    release_gateway_commercial_admission(&state, admission).await
                {
                    return release_response;
                }
            }
            return response;
        }
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "embeddings",
        &request.model,
        10,
        0.01,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_embedding).into_response()
}

async fn moderations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_moderation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "moderations",
                &request.model,
                &request.model,
                1,
                0.001,
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }

    let local_moderation = match local_moderation_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "moderations",
        &request.model,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(local_moderation).into_response()
}

async fn image_generations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_image_generation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &request.model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    let response = match local_image_generation_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &request.model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn image_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_edit_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();

    match relay_image_edit_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image edit");
        }
    }

    let response = create_image_edit(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("image edit");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn image_variations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_variation_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();

    match relay_image_variation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image variation");
        }
    }

    let response = create_image_variation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("image variation");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn transcriptions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    match relay_transcription_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_transcriptions",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_transcriptions",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        create_transcription(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("transcription"),
    )
    .into_response()
}

async fn translations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    match relay_translation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_translations",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_translations",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        create_translation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("translation"),
    )
    .into_response()
}

async fn audio_speech_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    match relay_speech_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_speech",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream speech");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_speech",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    local_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
}

async fn audio_voices_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_audio_voices_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio",
                "voices",
                5,
                0.005,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voices list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio",
        "voices",
        5,
        0.005,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_audio_voices(request_context.tenant_id(), request_context.project_id())
            .expect("audio voices list"),
    )
    .into_response()
}

async fn audio_voice_consents_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVoiceConsentRequest>,
) -> Response {
    match relay_audio_voice_consent_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let consent_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.voice.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio",
                &request.voice,
                consent_id,
                5,
                0.005,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voice consent");
        }
    }

    let response = create_audio_voice_consent(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("audio voice consent");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio",
        &request.voice,
        response.id.as_str(),
        5,
        0.005,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_file_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };

    match relay_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let file_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.purpose.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &request.purpose,
                file_id,
                5,
                0.005,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file");
        }
    }

    let response = match local_file_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &request.purpose,
        response.id.as_str(),
        5,
        0.005,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                "list",
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        "list",
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_files(request_context.tenant_id(), request_context.project_id()).expect("files list"))
        .into_response()
}

async fn file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_get_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    let response = match local_file_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_delete_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    let response = match local_file_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn file_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn containers_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateContainerRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let container_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.name.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &request.name,
                container_id,
                10,
                0.01,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container create");
        }
    }

    let response = match sdkwork_api_app_gateway::create_container(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return bad_gateway_openai_response(error.to_string()),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &request.name,
        response.id.as_str(),
        10,
        0.01,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn containers_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_containers_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                "containers",
                5,
                0.005,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream containers list");
        }
    }

    let response = match sdkwork_api_app_gateway::list_containers(
        request_context.tenant_id(),
        request_context.project_id(),
    ) {
        Ok(response) => response,
        Err(error) => return bad_gateway_openai_response(error.to_string()),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        "containers",
        5,
        0.005,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_container_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container retrieve");
        }
    }

    let response = match local_container_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_container_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container delete");
        }
    }

    let response = match local_container_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateContainerFileRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.file_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                usage_model,
                8,
                0.008,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file create");
        }
    }

    let response = match local_container_file_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        response.id.as_str(),
        8,
        0.008,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_container_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container files list");
        }
    }

    let response = match local_container_files_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file retrieve");
        }
    }

    let response = match local_container_file_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file delete");
        }
    }

    let response = match local_container_file_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn container_file_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file content");
        }
    }

    let bytes = match local_container_file_content_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(bytes))
        .expect("valid local container file content response")
}

async fn music_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    match relay_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.model.as_str());
            let music_seconds = request
                .duration_seconds
                .unwrap_or_else(|| music_seconds_from_response(&response));
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &request.model,
                music_billing_units(music_seconds),
                music_billing_amount(music_seconds),
                BillingMediaMetrics {
                    music_seconds,
                    ..BillingMediaMetrics::default()
                },
                Some(usage_model),
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    let response = create_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("music create");
    let usage_model = match response.data.as_slice() {
        [track] => track.id.as_str(),
        _ => request.model.as_str(),
    };
    let music_seconds = match response.data.as_slice() {
        [track] => track
            .duration_seconds
            .unwrap_or(request.duration_seconds.unwrap_or(0.0)),
        _ => request.duration_seconds.unwrap_or(0.0),
    };

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &request.model,
        music_billing_units(music_seconds),
        music_billing_amount(music_seconds),
        BillingMediaMetrics {
            music_seconds,
            ..BillingMediaMetrics::default()
        },
        Some(usage_model),
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn music_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                "music",
                10,
                0.01,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "music",
        10,
        0.01,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_music(request_context.tenant_id(), request_context.project_id()).expect("music list"))
        .into_response()
}

async fn music_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_get_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    let response = match local_music_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn music_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_delete_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    let response = match local_music_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn music_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_music_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    let bytes = match local_music_content_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(Body::from(bytes))
        .expect("valid local music content response")
}

async fn music_lyrics_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_music_lyrics_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("lyrics");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                "lyrics",
                usage_model,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    let response = create_music_lyrics(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("music lyrics");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "lyrics",
        &response.id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn videos_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.model,
                usage_model,
                90,
                0.09,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video create");
        }
    }

    let response = create_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
        &request.prompt,
    )
    .expect("video");
    let usage_model = match response.data.as_slice() {
        [video] => video.id.as_str(),
        _ => request.model.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &request.model,
        usage_model,
        90,
        0.09,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn videos_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_videos_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                "videos",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        "videos",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_videos(request_context.tenant_id(), request_context.project_id())
            .expect("videos list"),
    )
    .into_response()
}

async fn video_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_get_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    let response = match local_video_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_delete_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    let response = match local_video_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_video_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    let bytes = match local_video_content_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .expect("valid local video content response")
}

async fn video_remix_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    match relay_remix_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or(video_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                usage_model,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    let response = remix_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request.prompt,
    )
    .expect("video remix");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => video_id.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        usage_model,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_characters_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_list_video_characters_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video characters list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_video_characters(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video characters list"),
    )
    .into_response()
}

async fn video_character_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((video_id, character_id)): Path<(String, String)>,
) -> Response {
    match relay_get_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                &character_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        &character_id,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        get_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
        )
        .expect("video character retrieve"),
    )
    .into_response()
}

async fn video_character_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((video_id, character_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateVideoCharacterRequest>,
) -> Response {
    match relay_update_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                &character_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character update");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        &character_id,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        update_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
            &request,
        )
        .expect("video character update"),
    )
    .into_response()
}

async fn video_extend_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_extend_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or(video_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                usage_model,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extend");
        }
    }

    let response = extend_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request.prompt,
    )
    .expect("video extend");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => video_id.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        usage_model,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_character_create_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVideoCharacterRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_create_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let character_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.video_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.video_id,
                character_id,
                40,
                0.04,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character create");
        }
    }

    let response = sdkwork_api_app_gateway::create_video_character(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("video character create");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &request.video_id,
        response.id.as_str(),
        40,
        0.04,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_character_retrieve_canonical_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(character_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_video_character_canonical_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &character_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &character_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &character_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        sdkwork_api_app_gateway::get_video_character_canonical(
            request_context.tenant_id(),
            request_context.project_id(),
            &character_id,
        )
        .expect("video character canonical retrieve"),
    )
    .into_response()
}

async fn video_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<EditVideoRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_edit_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.video_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.video_id,
                usage_model,
                80,
                0.08,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video edits");
        }
    }

    let response = sdkwork_api_app_gateway::edit_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("video edits");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => request.video_id.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &request.video_id,
        usage_model,
        80,
        0.08,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_extensions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_extensions_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let route_key = request.video_id.as_deref().unwrap_or("videos");
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or(route_key);
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                route_key,
                usage_model,
                80,
                0.08,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extensions");
        }
    }

    let response = sdkwork_api_app_gateway::extensions_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("video extensions");
    let route_key = request.video_id.as_deref().unwrap_or("videos");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => route_key,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        route_key,
        usage_model,
        80,
        0.08,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn uploads_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let upload_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.purpose.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.purpose,
                upload_id,
                8,
                0.008,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload");
        }
    }

    let response = create_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("upload");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.purpose,
        response.id.as_str(),
        8,
        0.008,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn upload_parts_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    multipart: Multipart,
) -> Response {
    let request = match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };

    match relay_upload_part_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let part_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.upload_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.upload_id,
                part_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload part");
        }
    }

    let response = match local_upload_part_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.upload_id,
        response.id.as_str(),
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn upload_complete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;

    match relay_complete_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.upload_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload completion");
        }
    }

    let response = match local_upload_complete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.upload_id,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn upload_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_cancel_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &upload_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload cancellation");
        }
    }

    let response = match local_upload_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &upload_id,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_jobs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let fine_tuning_job_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &request.model,
                fine_tuning_job_id,
                200,
                0.2,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    let response = create_fine_tuning_job(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
    .expect("fine tuning");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &request.model,
        response.id.as_str(),
        200,
        0.2,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_jobs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_fine_tuning_jobs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                "jobs",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        "jobs",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id())
            .expect("fine tuning list"),
    )
    .into_response()
}

async fn fine_tuning_job_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_get_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    let response = match local_fine_tuning_job_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_job_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_cancel_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    let response = match local_fine_tuning_job_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_job_events_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_events_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    let response = match local_fine_tuning_job_events_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_job_checkpoints_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_checkpoints_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    let response = match local_fine_tuning_job_checkpoints_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_job_pause_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_pause_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                8,
                0.008,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job pause");
        }
    }

    let response = match local_fine_tuning_job_pause_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        8,
        0.008,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_job_resume_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_resume_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                8,
                0.008,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job resume");
        }
    }

    let response = match local_fine_tuning_job_resume_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        8,
        0.008,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_checkpoint_permissions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_fine_tuning_checkpoint_permissions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(fine_tuned_model_checkpoint.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                usage_model,
                5,
                0.005,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    let response = match local_fine_tuning_checkpoint_permissions_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };
    let usage_model = match response.data.as_slice() {
        [permission] => permission.id.as_str(),
        _ => fine_tuned_model_checkpoint.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        usage_model,
        5,
        0.005,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_checkpoint_permissions_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_fine_tuning_checkpoint_permissions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    let response = match local_fine_tuning_checkpoint_permissions_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn fine_tuning_checkpoint_permission_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_fine_tuning_checkpoint_permission_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                &permission_id,
                4,
                0.004,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    let response = match local_fine_tuning_checkpoint_permission_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        &permission_id,
        4,
        0.004,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn assistants_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let assistant_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &request.model,
                assistant_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    let response = create_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
        &request.model,
    )
    .expect("assistant");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &request.model,
        response.id.as_str(),
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn assistants_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_assistants_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "assistants",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "assistants",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_assistants(request_context.tenant_id(), request_context.project_id())
            .expect("assistants list"),
    )
    .into_response()
}

async fn assistant_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_get_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &assistant_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    let response = match local_assistant_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &assistant_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn assistant_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_update_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                usage_target,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
    let response = match local_assistant_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        request.name.as_deref().unwrap_or("assistant"),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        usage_target,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn assistant_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_delete_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &assistant_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    let response = match local_assistant_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &assistant_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn webhooks_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let webhook_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.url.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &request.url,
                webhook_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    let response = create_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.url,
        &request.events,
    )
    .expect("webhook");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &request.url,
        response.id.as_str(),
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn webhooks_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_webhooks_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                "webhooks",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        "webhooks",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_webhooks(request_context.tenant_id(), request_context.project_id())
            .expect("webhooks list"),
    )
    .into_response()
}

async fn webhook_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_get_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &webhook_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    let response = match local_webhook_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &webhook_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn webhook_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_update_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                usage_target,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
    let response = match local_webhook_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        request
            .url
            .as_deref()
            .unwrap_or("https://example.com/webhook"),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        usage_target,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn webhook_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_delete_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &webhook_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    let response = match local_webhook_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &webhook_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn realtime_sessions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_realtime_session_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let realtime_session_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "realtime_sessions",
                &request.model,
                realtime_session_id,
                30,
                0.03,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream realtime session");
        }
    }

    let response = create_realtime_session(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
    .expect("realtime");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "realtime_sessions",
        &request.model,
        response.id.as_str(),
        30,
        0.03,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn evals_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let eval_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.name.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &request.name,
                eval_id,
                40,
                0.04,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    let response = match create_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    ) {
        Ok(response) => response,
        Err(error) => return bad_gateway_openai_response(error.to_string()),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &request.name,
        response.id.as_str(),
        40,
        0.04,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn evals_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_evals_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                "evals",
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    let response = match list_evals(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return bad_gateway_openai_response(error.to_string()),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        "evals",
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_get_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    let response = match local_eval_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_update_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    let response = match local_eval_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_delete_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    let response = match local_eval_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_runs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_list_eval_runs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    let response = match local_eval_runs_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let run_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(eval_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                run_id,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    let response = match local_eval_run_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        response.id.as_str(),
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_run_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_get_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    let response = match local_eval_run_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_run_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    let response = match local_eval_run_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_run_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_cancel_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    let response = match local_eval_run_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_run_output_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_eval_run_output_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    let response = match local_eval_run_output_items_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn eval_run_output_item_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_eval_run_output_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &output_item_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    let response = match local_eval_run_output_item_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &output_item_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn batches_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let batch_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.endpoint.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &request.endpoint,
                batch_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    let response = create_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.endpoint,
        &request.input_file_id,
    )
    .expect("batch");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &request.endpoint,
        response.id.as_str(),
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn batches_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_batches_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                "batches",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        "batches",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_batches(request_context.tenant_id(), request_context.project_id())
            .expect("batches list"),
    )
    .into_response()
}

async fn batch_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_get_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &batch_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    let response = match local_batch_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &batch_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn batch_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_cancel_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &batch_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    let response = match local_batch_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &batch_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_stores_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let vector_store_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.name.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &request.name,
                vector_store_id,
                35,
                0.035,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }

    let response = create_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    )
    .expect("vector store");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &request.name,
        response.id.as_str(),
        35,
        0.035,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_stores_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_vector_stores_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                "vector_stores",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        "vector_stores",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_vector_stores(request_context.tenant_id(), request_context.project_id())
            .expect("vector stores list"),
    )
    .into_response()
}

async fn vector_store_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_get_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &vector_store_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }

    let response = match local_vector_store_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &vector_store_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_update_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &vector_store_id,
                35,
                0.035,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }

    let response = match local_vector_store_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        request.name.as_deref().unwrap_or("vector-store"),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &vector_store_id,
        35,
        0.035,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_delete_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &vector_store_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }

    let response = match local_vector_store_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &vector_store_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_search_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_search_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_search",
                &vector_store_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_search",
        &vector_store_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        search_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.query,
        )
        .expect("vector store search"),
    )
    .into_response()
}

async fn vector_store_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.file_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                usage_model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }

    let response = create_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_id,
    )
    .expect("vector store file");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        response.id.as_str(),
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_list_vector_store_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_vector_store_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store files list"),
    )
    .into_response()
}

async fn vector_store_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }

    let response = match local_vector_store_file_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }

    let response = match local_vector_store_file_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_file_batches_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let batch_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(vector_store_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                batch_id,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }

    let response = create_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_ids,
    )
    .expect("vector store file batch");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        response.id.as_str(),
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_file_batch_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }

    let response = match local_vector_store_file_batch_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_file_batch_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }

    let response = match local_vector_store_file_batch_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn vector_store_file_batch_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_list_vector_store_file_batch_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
                15,
                0.015,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }

    let response = match local_vector_store_file_batch_files_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
        15,
        0.015,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn enforce_project_quota<S>(
    store: &S,
    project_id: &str,
    requested_units: u64,
) -> anyhow::Result<Option<Response>>
where
    S: sdkwork_api_app_billing::BillingQuotaStore + ?Sized,
{
    let evaluation = check_quota(store, project_id, requested_units).await?;
    if evaluation.allowed {
        Ok(None)
    } else {
        Ok(Some(quota_exceeded_response(project_id, &evaluation)))
    }
}

fn quota_exceeded_response(project_id: &str, evaluation: &QuotaCheckResult) -> Response {
    let mut error = OpenAiErrorResponse::new(
        quota_exceeded_message(project_id, evaluation),
        "insufficient_quota",
    );
    error.error.code = Some("quota_exceeded".to_owned());
    (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

fn bad_gateway_openai_response(message: impl Into<String>) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "server_error");
    error.error.code = Some("bad_gateway".to_owned());
    (StatusCode::BAD_GATEWAY, Json(error)).into_response()
}

fn not_found_openai_response(message: impl Into<String>) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "invalid_request_error");
    error.error.code = Some("not_found".to_owned());
    (StatusCode::NOT_FOUND, Json(error)).into_response()
}

fn invalid_request_openai_response(
    message: impl Into<String>,
    code: impl Into<String>,
) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "invalid_request_error");
    error.error.code = Some(code.into());
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

fn local_gateway_error_response(error: anyhow::Error, not_found_message: &'static str) -> Response {
    if error.to_string().to_ascii_lowercase().contains("not found") {
        return not_found_openai_response(not_found_message);
    }

    bad_gateway_openai_response(error.to_string())
}

fn quota_exceeded_message(project_id: &str, evaluation: &QuotaCheckResult) -> String {
    match (evaluation.policy_id.as_deref(), evaluation.limit_units) {
        (Some(policy_id), Some(limit_units)) => format!(
            "Quota exceeded for project {project_id} under policy {policy_id}: requested {} units with {} already used against a limit of {limit_units}.",
            evaluation.requested_units, evaluation.used_units,
        ),
        (_, Some(limit_units)) => format!(
            "Quota exceeded for project {project_id}: requested {} units with {} already used against a limit of {limit_units}.",
            evaluation.requested_units, evaluation.used_units,
        ),
        _ => format!(
            "Quota exceeded for project {project_id}: requested {} units with {} already used.",
            evaluation.requested_units, evaluation.used_units,
        ),
    }
}

fn next_gateway_commercial_record_id(now_ms: u64) -> u64 {
    let sequence = GATEWAY_COMMERCIAL_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        & GATEWAY_COMMERCIAL_ID_SEQUENCE_MASK;
    compose_gateway_commercial_record_id(now_ms, sequence)
}

fn compose_gateway_commercial_record_id(now_ms: u64, sequence: u64) -> u64 {
    (now_ms << GATEWAY_COMMERCIAL_ID_SEQUENCE_BITS)
        | (sequence & GATEWAY_COMMERCIAL_ID_SEQUENCE_MASK)
}

async fn begin_gateway_commercial_admission(
    state: &GatewayApiState,
    request_context: &IdentityGatewayRequestContext,
    spec: GatewayCommercialAdmissionSpec,
) -> Result<GatewayCommercialAdmissionDecision, Response> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(GatewayCommercialAdmissionDecision::LegacyQuota);
    };

    let billing_settlement = resolve_gateway_billing_settlement(
        state.store.as_ref(),
        request_context.api_key_group_id(),
        None,
        spec.quoted_amount,
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to evaluate commercial billing admission",
        )
            .into_response()
    })?;

    if billing_settlement.accounting_mode != BillingAccountingMode::PlatformCredit
        || billing_settlement.customer_charge <= f64::EPSILON
    {
        return Ok(GatewayCommercialAdmissionDecision::LegacyQuota);
    }

    let Some(account) = commercial_billing
        .resolve_payable_account_for_gateway_request_context(request_context)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to resolve payable account",
            )
                .into_response()
        })?
    else {
        return Ok(GatewayCommercialAdmissionDecision::LegacyQuota);
    };

    let now_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to allocate commercial billing hold",
        )
            .into_response()
    })?;
    let hold_plan = commercial_billing
        .plan_account_hold(
            account.account_id,
            billing_settlement.customer_charge,
            now_ms,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to plan commercial billing hold",
            )
                .into_response()
        })?;
    if !hold_plan.sufficient_balance {
        return Err(commercial_balance_exceeded_response(
            request_context.project_id(),
            account.account_id,
            hold_plan.requested_quantity,
            hold_plan.covered_quantity,
            hold_plan.shortfall_quantity,
        ));
    }

    let request_id = next_gateway_commercial_record_id(now_ms);
    let hold_id = next_gateway_commercial_record_id(now_ms);
    let hold_allocation_start_id = next_gateway_commercial_record_id(now_ms);
    commercial_billing
        .create_account_hold(CreateAccountHoldInput {
            hold_id,
            hold_allocation_start_id,
            request_id,
            account_id: account.account_id,
            requested_quantity: billing_settlement.customer_charge,
            expires_at_ms: now_ms + GATEWAY_COMMERCIAL_HOLD_TTL_MS,
            now_ms,
        })
        .await
        .map_err(|error| {
            if looks_like_insufficient_account_balance(&error) {
                commercial_balance_exceeded_response(
                    request_context.project_id(),
                    account.account_id,
                    billing_settlement.customer_charge,
                    0.0,
                    billing_settlement.customer_charge,
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to create commercial billing hold",
                )
                    .into_response()
            }
        })?;

    Ok(GatewayCommercialAdmissionDecision::Canonical(
        GatewayCommercialAdmission {
            request_id,
            billing_settlement,
        },
    ))
}

async fn capture_gateway_commercial_admission(
    state: &GatewayApiState,
    admission: &GatewayCommercialAdmission,
) -> Result<(), Response> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(());
    };

    let settled_at_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to finalize commercial billing settlement",
        )
            .into_response()
    })?;
    commercial_billing
        .capture_account_hold(CaptureAccountHoldInput {
            request_settlement_id: next_gateway_commercial_record_id(settled_at_ms),
            request_id: admission.request_id,
            captured_quantity: admission.billing_settlement.customer_charge,
            provider_cost_amount: admission.billing_settlement.upstream_cost,
            retail_charge_amount: admission.billing_settlement.customer_charge,
            settled_at_ms,
        })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to finalize commercial billing settlement",
            )
                .into_response()
        })?;
    Ok(())
}

async fn release_gateway_commercial_admission(
    state: &GatewayApiState,
    admission: &GatewayCommercialAdmission,
) -> Result<(), Response> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(());
    };

    let released_at_ms = current_billing_timestamp_ms().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to release commercial billing hold",
        )
            .into_response()
    })?;
    commercial_billing
        .release_account_hold(ReleaseAccountHoldInput {
            request_id: admission.request_id,
            released_at_ms,
        })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to release commercial billing hold",
            )
                .into_response()
        })?;
    Ok(())
}

fn commercial_balance_exceeded_response(
    project_id: &str,
    account_id: u64,
    requested_quantity: f64,
    covered_quantity: f64,
    shortfall_quantity: f64,
) -> Response {
    let mut error = OpenAiErrorResponse::new(
        format!(
            "Insufficient balance for project {project_id} on primary account {account_id}: requested {requested_quantity:.4} credits, available {covered_quantity:.4}, shortfall {shortfall_quantity:.4}."
        ),
        "payment_required",
    );
    error.error.code = Some("insufficient_balance".to_owned());
    (StatusCode::PAYMENT_REQUIRED, Json(error)).into_response()
}

#[cfg(test)]
mod gateway_commercial_id_tests {
    use super::compose_gateway_commercial_record_id;

    #[test]
    fn gateway_commercial_record_ids_leave_headroom_for_ledger_suffixes() {
        let future_now_ms = 4_102_444_800_000_u64;
        let max_sequence = 0x0000_7fff_u64;
        let record_id = compose_gateway_commercial_record_id(future_now_ms, max_sequence);
        let derived_ledger_id = record_id.saturating_mul(10).saturating_add(4);

        assert!(
            i64::try_from(derived_ledger_id).is_ok(),
            "commercial record id {record_id} must stay representable after ledger suffix expansion"
        );
    }
}

fn looks_like_insufficient_account_balance(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .to_ascii_lowercase()
        .contains("insufficient available balance")
}

async fn record_gateway_usage_for_project(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_context(
        store, tenant_id, project_id, capability, model, units, amount, None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_reference_id_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        model,
        model,
        units,
        amount,
        None,
        usage_context_override,
    )
    .await
}

#[derive(Debug, Clone, Copy, Default)]
struct TokenUsageMetrics {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct BillingMediaMetrics {
    image_count: u64,
    audio_seconds: f64,
    video_seconds: f64,
    music_seconds: f64,
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| value.as_u64())
}

fn extract_token_usage_metrics(response: &Value) -> Option<TokenUsageMetrics> {
    if let Some(usage) = response.get("usage") {
        let input_tokens = json_u64(usage.get("prompt_tokens"))
            .or_else(|| json_u64(usage.get("input_tokens")))
            .unwrap_or(0);
        let output_tokens = json_u64(usage.get("completion_tokens"))
            .or_else(|| json_u64(usage.get("output_tokens")))
            .unwrap_or(0);
        let total_tokens = json_u64(usage.get("total_tokens"))
            .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

        if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
            return Some(TokenUsageMetrics {
                input_tokens,
                output_tokens,
                total_tokens,
            });
        }
    }

    let input_tokens = json_u64(response.get("input_tokens")).unwrap_or(0);
    let output_tokens = json_u64(response.get("output_tokens")).unwrap_or(0);
    let total_tokens = json_u64(response.get("total_tokens"))
        .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

    if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
        return Some(TokenUsageMetrics {
            input_tokens,
            output_tokens,
            total_tokens,
        });
    }

    None
}

fn response_usage_id_or_single_data_item_id(response: &Value) -> Option<&str> {
    response.get("id").and_then(Value::as_str).or_else(|| {
        match response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
        {
            Some([item]) => item.get("id").and_then(Value::as_str),
            _ => None,
        }
    })
}

fn image_count_from_response(response: &Value) -> u64 {
    response
        .get("data")
        .and_then(Value::as_array)
        .and_then(|data| u64::try_from(data.len()).ok())
        .unwrap_or(0)
}

fn music_seconds_from_response(response: &Value) -> f64 {
    response
        .get("duration_seconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            response
                .get("data")
                .and_then(Value::as_array)
                .and_then(|data| match data.as_slice() {
                    [item] => item.get("duration_seconds").and_then(Value::as_f64),
                    _ => None,
                })
        })
        .unwrap_or(0.0)
}

fn music_billing_units(music_seconds: f64) -> u64 {
    music_seconds.max(1.0).ceil() as u64
}

fn music_billing_amount(music_seconds: f64) -> f64 {
    music_seconds.max(1.0) * 0.001
}

fn current_billing_timestamp_ms() -> anyhow::Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64)
}

fn build_gateway_billing_event_id(
    project_id: &str,
    capability: &str,
    route_key: &str,
    provider_id: &str,
    reference_id: Option<&str>,
    created_at_ms: u64,
) -> String {
    format!(
        "bill_evt:{project_id}:{capability}:{route_key}:{provider_id}:{}:{created_at_ms}",
        reference_id.unwrap_or("none")
    )
}

fn billing_modality_for_capability(capability: &str) -> &'static str {
    match capability {
        "responses" => "multimodal",
        "images" | "image_edits" | "image_variations" => "image",
        "audio" | "speech" | "transcriptions" | "translations" => "audio",
        "videos" => "video",
        "music" => "music",
        _ => "text",
    }
}

async fn record_gateway_usage_for_project_with_route_key_and_reference_id(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_reference_id_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        reference_id,
        None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_reference_id_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    reference_id: Option<&str>,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
        reference_id,
        usage_context_override,
    )
    .await
}

async fn record_gateway_usage_for_project_with_media_and_reference_id(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
    media_metrics: BillingMediaMetrics,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        model,
        model,
        units,
        amount,
        None,
        reference_id,
        media_metrics,
        None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
        None,
        None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        token_usage,
        reference_id,
        None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        token_usage,
        reference_id,
        BillingMediaMetrics::default(),
        usage_context_override,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
    media_metrics: BillingMediaMetrics,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    let usage_context = match usage_context_override {
        Some(context) => (*context).clone(),
        None => {
            planned_execution_usage_context_for_route(
                store, tenant_id, project_id, capability, route_key,
            )
            .await?
        }
    };
    let token_usage = token_usage.unwrap_or_default();
    let request_context = current_gateway_request_context();
    let api_key_hash = request_context
        .as_ref()
        .map(|context| context.api_key_hash().to_owned());
    let billing_settlement = resolve_gateway_billing_settlement(
        store,
        request_context
            .as_ref()
            .and_then(|context| context.api_key_group_id()),
        usage_context.reference_amount,
        amount,
    )
    .await?;
    let latency_ms = current_gateway_request_latency_ms().or(usage_context.latency_ms);
    persist_usage_record_with_tokens_and_facts(
        store,
        project_id,
        usage_model,
        &usage_context.provider_id,
        units,
        amount,
        token_usage.input_tokens,
        token_usage.output_tokens,
        token_usage.total_tokens,
        api_key_hash.as_deref(),
        usage_context.channel_id.as_deref(),
        latency_ms,
        usage_context.reference_amount,
    )
    .await?;
    let created_at_ms = current_billing_timestamp_ms()?;
    let billing_event = create_billing_event(CreateBillingEventInput {
        event_id: &build_gateway_billing_event_id(
            project_id,
            capability,
            route_key,
            &usage_context.provider_id,
            reference_id,
            created_at_ms,
        ),
        tenant_id,
        project_id,
        api_key_group_id: usage_context.api_key_group_id.as_deref(),
        capability,
        route_key,
        usage_model,
        provider_id: &usage_context.provider_id,
        accounting_mode: billing_settlement.accounting_mode,
        operation_kind: capability,
        modality: billing_modality_for_capability(capability),
        api_key_hash: api_key_hash.as_deref(),
        channel_id: usage_context.channel_id.as_deref(),
        reference_id,
        latency_ms,
        units,
        request_count: 1,
        input_tokens: token_usage.input_tokens,
        output_tokens: token_usage.output_tokens,
        total_tokens: token_usage.total_tokens,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: media_metrics.image_count,
        audio_seconds: media_metrics.audio_seconds,
        video_seconds: media_metrics.video_seconds,
        music_seconds: media_metrics.music_seconds,
        upstream_cost: billing_settlement.upstream_cost,
        customer_charge: billing_settlement.customer_charge,
        applied_routing_profile_id: usage_context.applied_routing_profile_id.as_deref(),
        compiled_routing_snapshot_id: usage_context.compiled_routing_snapshot_id.as_deref(),
        fallback_reason: usage_context.fallback_reason.as_deref(),
        created_at_ms,
    })?;
    persist_billing_event(store, &billing_event).await?;
    persist_ledger_entry(store, project_id, units, amount).await?;
    Ok(())
}

async fn resolve_gateway_billing_settlement(
    store: &dyn AdminStore,
    api_key_group_id: Option<&str>,
    upstream_cost: Option<f64>,
    customer_charge: f64,
) -> anyhow::Result<BillingPolicyExecutionResult> {
    let group_default_accounting_mode =
        load_api_key_group_default_accounting_mode(store, api_key_group_id).await?;
    let registry = builtin_billing_policy_registry();
    let plugin = registry
        .resolve(GROUP_DEFAULT_BILLING_POLICY_ID)
        .expect("builtin group-default billing policy plugin must exist");

    plugin.execute(BillingPolicyExecutionInput {
        api_key_group_default_accounting_mode: group_default_accounting_mode.as_deref(),
        default_accounting_mode: BillingAccountingMode::PlatformCredit,
        upstream_cost,
        customer_charge,
    })
}

async fn load_api_key_group_default_accounting_mode(
    store: &dyn AdminStore,
    api_key_group_id: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(api_key_group_id) = api_key_group_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    Ok(store
        .find_api_key_group(api_key_group_id)
        .await?
        .and_then(|group| group.default_accounting_mode))
}

async fn relay_stateless_json_request(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
) -> anyhow::Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    relay_stateless_json_request_with_options(request_context, request, &options).await
}

async fn relay_stateless_json_request_with_options(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> anyhow::Result<Option<Value>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };

    execute_json_provider_request_with_runtime_and_options(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        request,
        options,
    )
    .await
}

async fn relay_stateless_stream_request(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    relay_stateless_stream_request_with_options(request_context, request, &options).await
}

async fn relay_stateless_stream_request_with_options(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };

    execute_stream_provider_request_with_runtime_and_options(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        request,
        options,
    )
    .await
}

fn local_speech_response(
    tenant_id: &str,
    project_id: &str,
    request: &CreateSpeechRequest,
) -> Response {
    let speech = match create_speech_response(tenant_id, project_id, request) {
        Ok(speech) => speech,
        Err(error) => {
            return invalid_request_openai_response(error.to_string(), "invalid_response_format");
        }
    };
    if request.stream_format.as_deref() == Some("sse") {
        let delta = serde_json::json!({
            "type":"response.output_audio.delta",
            "delta": speech.audio_base64,
            "format": speech.format,
        })
        .to_string();
        let done = serde_json::json!({
            "type":"response.completed"
        })
        .to_string();
        let body = format!("{}{}", SseFrame::data(&delta), SseFrame::data(&done));
        return ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response();
    }

    let bytes = STANDARD
        .decode(speech.audio_base64.as_bytes())
        .unwrap_or_default();

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, speech_content_type(&speech.format))
        .body(Body::from(bytes))
        .expect("valid speech response")
}

fn local_response_stream_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_response_create_result(tenant_id, project_id, model) {
        Ok(_) => local_response_stream_body_response("resp_1", model),
        Err(response) => response,
    }
}

fn local_response_stream_body_response(response_id: &str, model: &str) -> Response {
    let created = serde_json::json!({
        "type":"response.created",
        "response": {
            "id": response_id,
            "object": "response",
            "model": model
        }
    })
    .to_string();
    let delta = serde_json::json!({
        "type":"response.output_text.delta",
        "delta":"hello"
    })
    .to_string();
    let completed = serde_json::json!({
        "type":"response.completed",
        "response": {
            "id": response_id
        }
    })
    .to_string();
    let body = format!(
        "{}{}{}",
        SseFrame::data(&created),
        SseFrame::data(&delta),
        SseFrame::data(&completed)
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn speech_content_type(format: &str) -> &'static str {
    match format {
        "mp3" => "audio/mpeg",
        "opus" => "audio/opus",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "pcm" => "audio/pcm",
        _ => "audio/wav",
    }
}

async fn parse_file_request(mut multipart: Multipart) -> Result<CreateFileRequest, Response> {
    let mut purpose = None;
    let mut filename = None;
    let mut bytes = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("purpose") => {
                purpose = Some(field.text().await.map_err(bad_multipart)?);
            }
            Some("file") => {
                filename = field.file_name().map(ToOwned::to_owned);
                content_type = field.content_type().map(ToOwned::to_owned);
                bytes = Some(field.bytes().await.map_err(bad_multipart)?.to_vec());
            }
            _ => {}
        }
    }

    let mut request = CreateFileRequest::new(
        purpose.ok_or_else(missing_multipart_field)?,
        filename.ok_or_else(missing_multipart_field)?,
        bytes.ok_or_else(missing_multipart_field)?,
    );
    if let Some(content_type) = content_type {
        request = request.with_content_type(content_type);
    }
    Ok(request)
}

async fn parse_image_edit_request(
    mut multipart: Multipart,
) -> Result<CreateImageEditRequest, Response> {
    let mut model = None;
    let mut prompt = None;
    let mut image = None;
    let mut mask = None;
    let mut n = None;
    let mut quality = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("prompt") => prompt = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("mask") => mask = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("quality") => quality = Some(field.text().await.map_err(bad_multipart)?),
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageEditRequest::new(
        prompt.ok_or_else(missing_multipart_field)?,
        image.ok_or_else(missing_multipart_field)?,
    );
    if let Some(model) = model {
        request = request.with_model(model);
    }
    if let Some(mask) = mask {
        request = request.with_mask(mask);
    }
    request.n = n;
    request.quality = quality;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_variation_request(
    mut multipart: Multipart,
) -> Result<CreateImageVariationRequest, Response> {
    let mut model = None;
    let mut image = None;
    let mut n = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageVariationRequest::new(image.ok_or_else(missing_multipart_field)?);
    if let Some(model) = model {
        request = request.with_model(model);
    }
    request.n = n;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_upload_field(
    field: axum::extract::multipart::Field<'_>,
) -> Result<ImageUpload, Response> {
    let filename = field
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(missing_multipart_field)?;
    let content_type = field.content_type().map(ToOwned::to_owned);
    let bytes = field.bytes().await.map_err(bad_multipart)?.to_vec();
    let mut upload = ImageUpload::new(filename, bytes);
    if let Some(content_type) = content_type {
        upload = upload.with_content_type(content_type);
    }
    Ok(upload)
}

fn parse_u32_field(value: String) -> Result<u32, &'static str> {
    value
        .parse::<u32>()
        .map_err(|_| "invalid numeric multipart field")
}

async fn parse_upload_part_request(
    upload_id: String,
    mut multipart: Multipart,
) -> Result<AddUploadPartRequest, Response> {
    let mut data = None;
    let mut filename = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        if field.name() == Some("data") {
            filename = field.file_name().map(ToOwned::to_owned);
            content_type = field.content_type().map(ToOwned::to_owned);
            data = Some(field.bytes().await.map_err(bad_multipart)?.to_vec());
        }
    }

    let mut request =
        AddUploadPartRequest::new(upload_id, data.ok_or_else(missing_multipart_field)?);
    if let Some(filename) = filename {
        request = request.with_filename(filename);
    }
    if let Some(content_type) = content_type {
        request = request.with_content_type(content_type);
    }
    Ok(request)
}

fn bad_multipart(error: axum::extract::multipart::MultipartError) -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        format!("invalid multipart payload: {error}"),
    )
        .into_response()
}

fn missing_multipart_field() -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        "missing multipart field",
    )
        .into_response()
}
