use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use sdkwork_api_provider_core::{
    OpenRouterDataCollectionPolicy, OpenRouterProviderPreferences, ProviderExecutionAdapter,
    ProviderRequest, ProviderRequestOptions,
};
use serde_json::{Value, json};
use tokio::net::TcpListener;

#[derive(Clone, Default)]
struct CaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
    idempotency_key: Arc<Mutex<Option<String>>>,
    request_id: Arc<Mutex<Option<String>>>,
}

#[tokio::test]
async fn adapter_posts_authorized_json_to_openrouter_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/chat/completions", post(capture_chat_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter = sdkwork_api_provider_openrouter::OpenRouterProviderAdapter::new(format!(
        "http://{address}"
    ));
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "openai/gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: Value::String("hello".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: Some(false),
        extra: serde_json::Map::new(),
    };

    let response = adapter
        .chat_completions("sk-or-v1-upstream", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "chat.completion");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-or-v1-upstream")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["model"],
        "openai/gpt-4.1"
    );
}

#[tokio::test]
async fn adapter_applies_openrouter_execution_context_to_upstream_request() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/chat/completions", post(capture_chat_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter = sdkwork_api_provider_openrouter::OpenRouterProviderAdapter::new(format!(
        "http://{address}"
    ));
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "openai/gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: Value::String("hello".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: Some(false),
        extra: serde_json::Map::new(),
    };
    let options = ProviderRequestOptions::new()
        .with_idempotency_key("idem-openrouter")
        .with_request_trace_id("trace-openrouter")
        .with_openrouter_provider_preferences(
            OpenRouterProviderPreferences::new()
                .with_order(vec!["anthropic".to_owned(), "openai".to_owned()])
                .with_allow_fallbacks(false)
                .with_require_parameters(true)
                .with_data_collection(OpenRouterDataCollectionPolicy::Deny)
                .with_zero_data_retention(true),
        );

    let response = adapter
        .execute_with_options(
            "sk-or-v1-upstream",
            ProviderRequest::ChatCompletions(&request),
            &options,
        )
        .await
        .unwrap()
        .into_json()
        .expect("json output");

    assert_eq!(response["object"], "chat.completion");
    assert_eq!(
        state.idempotency_key.lock().unwrap().as_deref(),
        Some("idem-openrouter")
    );
    assert_eq!(
        state.request_id.lock().unwrap().as_deref(),
        Some("trace-openrouter")
    );
    let body = state.body.lock().unwrap();
    let provider = &body.as_ref().unwrap()["provider"];
    assert_eq!(provider["order"], json!(["anthropic", "openai"]));
    assert_eq!(provider["allow_fallbacks"], false);
    assert_eq!(provider["require_parameters"], true);
    assert_eq!(provider["data_collection"], "deny");
    assert_eq!(provider["zdr"], true);
}

async fn capture_chat_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.idempotency_key.lock().unwrap() = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.request_id.lock().unwrap() = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"openai/gpt-4.1",
            "choices":[]
        })),
    )
}
