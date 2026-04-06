use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::SigningKey;
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

mod support;

#[serial(extension_env)]
#[tokio::test]
async fn responses_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn responses_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

#[serial(extension_env)]
#[tokio::test]
async fn responses_stream_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\",\"stream\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_retrieve_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_items_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1/input_items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_items_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_missing/input_items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/responses/resp_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_delete_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/responses/resp_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_tokens_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_input_tokens_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_cancel_route_returns_not_found_for_unknown_response() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/resp_missing/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested response was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn response_cancel_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/resp_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_compact_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/compact")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn response_compact_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/compact")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response compaction model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn read_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn assert_no_usage_records(admin_app: Router, admin_token: &str) {
    let usage = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 0);
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    request_count: Arc<AtomicUsize>,
}

fn capture_upstream_request(
    state: &UpstreamCaptureState,
    headers: &axum::http::HeaderMap,
) -> usize {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    state.request_count.fetch_add(1, Ordering::SeqCst) + 1
}

async fn setup_stateful_responses_route_with_single_provider(
    tenant_id: &str,
    project_id: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
    credential_ref: &str,
    secret_value: &str,
    policy_id: &str,
) -> (Router, Router, String, String) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let create_channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"{base_url}\",\"display_name\":\"{display_name}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"{credential_ref}\",\"secret_value\":\"{secret_value}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"external_name\":\"gpt-4.1\",\"provider_id\":\"{provider_id}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": policy_id,
                        "capability": "responses",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [provider_id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    (gateway_app, admin_app, admin_token, api_key)
}

async fn create_openai_channel(admin_app: &Router, admin_token: &str) {
    let create_channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);
}

async fn create_stateful_openai_provider_for_responses(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
    credential_ref: &str,
    secret_value: &str,
) {
    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"{base_url}\",\"display_name\":\"{display_name}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"{credential_ref}\",\"secret_value\":\"{secret_value}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"external_name\":\"gpt-4.1\",\"provider_id\":\"{provider_id}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);
}

async fn create_responses_routing_policy(
    admin_app: &Router,
    admin_token: &str,
    policy_id: &str,
    ordered_provider_ids: Vec<&str>,
) {
    create_responses_routing_policy_with_overrides(
        admin_app,
        admin_token,
        policy_id,
        ordered_provider_ids,
        serde_json::json!({}),
    )
    .await;
}

async fn create_responses_routing_policy_with_overrides(
    admin_app: &Router,
    admin_token: &str,
    policy_id: &str,
    ordered_provider_ids: Vec<&str>,
    overrides: Value,
) {
    let mut body = serde_json::json!({
        "policy_id": policy_id,
        "capability": "responses",
        "model_pattern": "gpt-4.1",
        "enabled": true,
        "priority": 300,
        "ordered_provider_ids": ordered_provider_ids
    });
    let body_object = body
        .as_object_mut()
        .expect("routing policy body should be an object");
    if let Some(overrides_object) = overrides.as_object() {
        body_object.extend(
            overrides_object
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"relay me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_route_relays_stream_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_stream_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"hi\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("resp_upstream_stream"));
    assert!(body.contains("[DONE]"));
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"relay me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream response"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_responses_stream_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"relay me\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream response stream"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(upstream_response_input_tokens_handler),
        )
        .route(
            "/v1/responses/compact",
            post(upstream_response_compact_handler),
        )
        .route(
            "/v1/responses/resp_1",
            get(upstream_response_retrieve_handler).delete(upstream_response_delete_handler),
        )
        .route(
            "/v1/responses/resp_1/input_items",
            get(upstream_response_input_items_handler),
        )
        .route(
            "/v1/responses/resp_1/cancel",
            post(upstream_response_cancel_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
.header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
.header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(model.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"relay me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "resp_1");

    let input_items_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/responses/resp_1/input_items")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(input_items_response.status(), StatusCode::OK);
    let input_items_json = read_json(input_items_response).await;
    assert_eq!(input_items_json["data"][0]["id"], "item_1");

    let delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/responses/resp_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let input_tokens_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"count me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(input_tokens_response.status(), StatusCode::OK);
    let input_tokens_json = read_json(input_tokens_response).await;
    assert_eq!(input_tokens_json["object"], "response.input_tokens");
    assert_eq!(input_tokens_json["input_tokens"], 21);

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/resp_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let compact_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/compact")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"compact me\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(compact_response.status(), StatusCode::OK);
    let compact_json = read_json(compact_response).await;
    assert_eq!(compact_json["object"], "response.compaction");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_returns_invalid_request_for_missing_model_without_usage() {
    let pool = memory_pool().await;
    let tenant_id = "tenant-responses-invalid-model";
    let project_id = "project-responses-invalid-model";
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_stream_route_returns_invalid_request_for_missing_model_without_usage() {
    let pool = memory_pool().await;
    let tenant_id = "tenant-responses-stream-invalid-model";
    let project_id = "project-responses-stream-invalid-model";
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\",\"stream\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_response_input_tokens_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let tenant_id = "tenant-response-input-tokens-invalid-model";
    let project_id = "project-response-input-tokens-invalid-model";
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses/input_tokens")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"\",\"input\":\"hi\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Response model is required.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_fails_over_to_backup_provider_and_records_actual_provider() {
    let tenant_id = "tenant-responses-failover-json";
    let project_id = "project-responses-failover-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_with_usage))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_channel(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-primary",
        &format!("http://{primary_address}"),
        "Responses Failover Primary",
        "cred-responses-failover-primary",
        "sk-responses-failover-primary",
    )
    .await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-backup",
        &format!("http://{backup_address}"),
        "Responses Failover Backup",
        "cred-responses-failover-backup",
        "sk-responses-failover-backup",
    )
    .await;
    create_responses_routing_policy(
        &admin_app,
        &admin_token,
        "route-responses-failover-json",
        vec![
            "provider-responses-failover-primary",
            "provider-responses-failover-backup",
        ],
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"fail over please for responses\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_upstream");
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-failover-backup")
    );

    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_json[0]["model"], "gpt-4.1");
    assert_eq!(
        usage_json[0]["provider"],
        "provider-responses-failover-backup"
    );
    assert_eq!(usage_json[0]["input_tokens"], 160);
    assert_eq!(usage_json[0]["output_tokens"], 40);
    assert_eq!(usage_json[0]["total_tokens"], 200);

    let logs = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json[0]["route_key"], "gpt-4.1");
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-responses-failover-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-failover-primary\",outcome=\"failure\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-failover-backup\",outcome=\"success\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"responses\",from_provider=\"provider-responses-failover-primary\",to_provider=\"provider-responses-failover-backup\",outcome=\"success\"} 1"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_does_not_fail_over_when_policy_disables_execution_failover() {
    let tenant_id = "tenant-responses-failover-disabled-json";
    let project_id = "project-responses-failover-disabled-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_with_usage))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_channel(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-disabled-primary",
        &format!("http://{primary_address}"),
        "Responses Failover Disabled Primary",
        "cred-responses-failover-disabled-primary",
        "sk-responses-failover-disabled-primary",
    )
    .await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-failover-disabled-backup",
        &format!("http://{backup_address}"),
        "Responses Failover Disabled Backup",
        "cred-responses-failover-disabled-backup",
        "sk-responses-failover-disabled-backup",
    )
    .await;
    create_responses_routing_policy_with_overrides(
        &admin_app,
        &admin_token,
        "route-responses-failover-disabled-json",
        vec![
            "provider-responses-failover-disabled-primary",
            "provider-responses-failover-disabled-backup",
        ],
        serde_json::json!({
            "execution_failover_enabled": false,
            "upstream_retry_max_attempts": 1
        }),
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"responses failover must stay disabled\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 1);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 0);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-failover-disabled-primary\",outcome=\"failure\"} 1"
    ));
    assert!(
        !metrics_text.contains(
            "provider=\"provider-responses-failover-disabled-backup\",outcome=\"success\""
        )
    );
    assert!(!metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"responses\",from_provider=\"provider-responses-failover-disabled-primary\",to_provider=\"provider-responses-failover-disabled-backup\",outcome=\"success\"}"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_keeps_request_model_for_billing_despite_response_id() {
    let tenant_id = "tenant-responses-model-billing";
    let project_id = "project-responses-model-billing";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let create_channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let provider_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-responses-model-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Responses Model Route Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_model.status(), StatusCode::CREATED);

    let provider_created = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-responses-created-id\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Responses Created Id Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider_created.status(), StatusCode::CREATED);

    let model_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-responses-model-route\",\"key_reference\":\"cred-responses-model-route\",\"secret_value\":\"sk-responses-model-route\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model_credential.status(), StatusCode::CREATED);

    let created_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-responses-created-id\",\"key_reference\":\"cred-responses-created-id\",\"secret_value\":\"sk-responses-created-id\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-responses-model-route\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let model_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-responses-by-request-model",
                        "capability": "responses",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-responses-model-route"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model_policy.status(), StatusCode::CREATED);

    let created_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-responses-by-created-id",
                        "capability": "responses",
                        "model_pattern": "resp_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-responses-created-id"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"keep request model\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-model-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gpt-4.1",
        "provider-responses-model-route",
        "gpt-4.1",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_records_upstream_token_usage() {
    let tenant_id = "tenant-responses-token";
    let project_id = "project-responses-token";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_with_usage))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let create_channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let create_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-responses-tokens\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Responses Tokens Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_provider.status(), StatusCode::CREATED);

    let create_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-responses-tokens\",\"key_reference\":\"cred-responses-tokens\",\"secret_value\":\"sk-responses-tokens\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_credential.status(), StatusCode::CREATED);

    let create_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-responses-tokens\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_model.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"count response tokens\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let usage = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json[0]["input_tokens"], 160);
    assert_eq!(usage_json[0]["output_tokens"], 40);
    assert_eq!(usage_json[0]["total_tokens"], 200);
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_relays_stream_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_stream_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(model.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"hi\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("resp_upstream_stream"));
    assert!(body.contains("[DONE]"));
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_stream_route_fails_over_to_backup_provider_and_records_actual_provider()
{
    let tenant_id = "tenant-responses-failover-stream";
    let project_id = "project-responses-failover-stream";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/v1/responses", post(upstream_responses_stream_handler))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_channel(&admin_app, &admin_token).await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-stream-failover-primary",
        &format!("http://{primary_address}"),
        "Responses Stream Failover Primary",
        "cred-responses-stream-failover-primary",
        "sk-responses-stream-failover-primary",
    )
    .await;
    create_stateful_openai_provider_for_responses(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-responses-stream-failover-backup",
        &format!("http://{backup_address}"),
        "Responses Stream Failover Backup",
        "cred-responses-stream-failover-backup",
        "sk-responses-stream-failover-backup",
    )
    .await;
    create_responses_routing_policy(
        &admin_app,
        &admin_token,
        "route-responses-failover-stream",
        vec![
            "provider-responses-stream-failover-primary",
            "provider-responses-stream-failover-backup",
        ],
    )
    .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"stream fail over please for responses\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let stream_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stream_text = String::from_utf8(stream_body.to_vec()).unwrap();
    assert!(stream_text.contains("resp_upstream_stream"));
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-stream-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-responses-stream-failover-backup")
    );

    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 1);
    assert_eq!(
        usage_json[0]["provider"],
        "provider-responses-stream-failover-backup"
    );

    let logs = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-responses-stream-failover-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-responses-retryable-primary";
    let project_id = "project-responses-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-retryable-primary",
            &format!("http://{primary_address}"),
            "Retryable Responses Primary",
            "cred-responses-retryable-primary",
            "sk-responses-retryable-primary",
            "route-responses-retryable-primary",
        )
        .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry transient responses primary failure\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_retry_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retryable-primary\",source=\"backoff\"} 25"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-responses-retryable-primary\",runtime=\"builtin\"} 1"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_does_not_retry_non_retryable_primary_failure() {
    let tenant_id = "tenant-responses-non-retryable-primary";
    let project_id = "project-responses-non-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_non_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-non-retryable-primary",
            &format!("http://{primary_address}"),
            "Non Retryable Responses Primary",
            "cred-responses-non-retryable-primary",
            "sk-responses-non-retryable-primary",
            "route-responses-non-retryable-primary",
        )
        .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"do not retry invalid responses request\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 1);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-non-retryable-primary\",outcome=\"attempt\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-non-retryable-primary\",outcome=\"failure\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-non-retryable-primary\",outcome=\"scheduled\"}"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_stream_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-responses-stream-retryable-primary";
    let project_id = "project-responses-stream-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_stream_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-stream-retryable-primary",
            &format!("http://{primary_address}"),
            "Retryable Responses Stream Primary",
            "cred-responses-stream-retryable-primary",
            "sk-responses-stream-retryable-primary",
            "route-responses-stream-retryable-primary",
        )
        .await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry transient responses stream primary failure\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let stream_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stream_text = String::from_utf8(stream_body.to_vec()).unwrap();
    assert!(stream_text.contains("resp_stream_retry_recovered"));
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-stream-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_honors_retry_after_before_retrying_primary_provider() {
    let tenant_id = "tenant-responses-retry-after-primary";
    let project_id = "project-responses-retry-after-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_retry_after_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-retry-after-primary",
            &format!("http://{primary_address}"),
            "Retry After Responses Primary",
            "cred-responses-retry-after-primary",
            "sk-responses-retry-after-primary",
            "route-responses-retry-after-primary",
        )
        .await;

    let started = Instant::now();
    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry after please for responses\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_retry_after_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert!(
        elapsed >= Duration::from_millis(900),
        "expected retry-after delay to be honored, got {:?}",
        elapsed
    );

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retry-after-primary\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-retry-after-primary\",source=\"retry_after_seconds\"} 1000"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_honors_http_date_retry_after_before_retrying_primary_provider() {
    let _retry_delay_guard =
        EnvVarGuard::set("SDKWORK_GATEWAY_UPSTREAM_RETRY_MAX_DELAY_MS", "1000");
    let tenant_id = "tenant-responses-http-date-retry-after-primary";
    let project_id = "project-responses-http-date-retry-after-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/responses",
            post(upstream_responses_handler_http_date_retry_after_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let (gateway_app, _admin_app, _admin_token, api_key) =
        setup_stateful_responses_route_with_single_provider(
            tenant_id,
            project_id,
            "provider-responses-http-date-retry-after-primary",
            &format!("http://{primary_address}"),
            "HTTP Date Retry After Responses Primary",
            "cred-responses-http-date-retry-after-primary",
            "sk-responses-http-date-retry-after-primary",
            "route-responses-http-date-retry-after-primary",
        )
        .await;

    let started = Instant::now();
    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"retry after http date please for responses\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "resp_http_date_retry_after_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert!(
        elapsed >= Duration::from_millis(900),
        "expected http-date retry-after delay to be honored, got {:?}",
        elapsed
    );

    let metrics = gateway_app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);
    let metrics_body = axum::body::to_bytes(metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let metrics_text = String::from_utf8(metrics_body.to_vec()).unwrap();
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-http-date-retry-after-primary\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"gateway\",capability=\"responses\",provider=\"provider-responses-http-date-retry-after-primary\",source=\"retry_after_http_date\"} 1000"
    ));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_responses_route_relays_stream_to_native_dynamic_provider() {
    let extension_root = temp_extension_root("native-dynamic-responses-stream");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(
        sdkwork_api_extension_core::ExtensionTrustDeclaration::signed(
            "sdkwork",
            sdkwork_api_extension_core::ExtensionSignature::new(
                sdkwork_api_extension_core::ExtensionSignatureAlgorithm::Ed25519,
                public_key.clone(),
                signature,
            ),
        ),
    );
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-native-mock\",\"channel_id\":\"openai\",\"adapter_kind\":\"native-dynamic\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Native Mock\",\"extension_id\":\"{FIXTURE_EXTENSION_ID}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-native-mock\",\"key_reference\":\"cred-native-mock\",\"secret_value\":\"sk-native\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-native-mock\",\"capabilities\":[\"responses\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(model.status(), StatusCode::CREATED);

    let installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "installation_id": "native-mock-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "runtime": "native_dynamic",
                        "enabled": true,
                        "entrypoint": library_path.to_string_lossy(),
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "instance_id": "provider-native-mock",
                        "installation_id": "native-mock-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-native-mock",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"relay me\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("resp_native_dynamic_stream"));
    assert!(body.contains("[DONE]"));

    cleanup_dir(&extension_root);
}

async fn upstream_responses_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_upstream",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

async fn upstream_responses_handler_with_usage(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_upstream",
            "object":"response",
            "model":"gpt-4.1",
            "output":[],
            "usage":{
                "input_tokens":160,
                "output_tokens":40,
                "total_tokens":200
            }
        })),
    )
}

async fn upstream_responses_handler_failure(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error":{
                "message":"primary responses upstream failed",
                "type":"server_error",
                "code":"upstream_failed"
            }
        })),
    )
}

async fn upstream_responses_stream_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    capture_upstream_request(&state, &headers);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        "data: {\"id\":\"resp_upstream_stream\",\"type\":\"response.output_text.delta\"}\n\ndata: [DONE]\n\n",
    )
        .into_response()
}

async fn upstream_responses_handler_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error":{
                    "message":"responses upstream rate limited",
                    "type":"rate_limit_error",
                    "code":"retry_later"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_retry_recovered",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

async fn upstream_responses_handler_retry_after_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("content-type", "application/json")
            .header("retry-after", "1")
            .body(Body::from(
                serde_json::json!({
                    "error":{
                        "message":"responses upstream rate limited with retry-after",
                        "type":"rate_limit_error",
                        "code":"retry_later"
                    }
                })
                .to_string(),
            ))
            .unwrap();
    }

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "id":"resp_retry_after_recovered",
                "object":"response",
                "model":"gpt-4.1",
                "output":[]
            })
            .to_string(),
        ))
        .unwrap()
}

async fn upstream_responses_handler_http_date_retry_after_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("content-type", "application/json")
            .header("retry-after", "Thu, 01 Jan 2099 00:00:00 GMT")
            .body(Body::from(
                serde_json::json!({
                    "error":{
                        "message":"responses upstream rate limited with http-date retry-after",
                        "type":"rate_limit_error",
                        "code":"retry_later"
                    }
                })
                .to_string(),
            ))
            .unwrap();
    }

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "id":"resp_http_date_retry_after_recovered",
                "object":"response",
                "model":"gpt-4.1",
                "output":[]
            })
            .to_string(),
        ))
        .unwrap()
}

async fn upstream_responses_handler_non_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error":{
                    "message":"invalid responses upstream payload",
                    "type":"invalid_request_error",
                    "code":"invalid_request"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_non_retryable_unexpected_retry",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

async fn upstream_responses_stream_handler_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "error":{
                        "message":"responses stream temporarily unavailable",
                        "type":"server_error",
                        "code":"retry_later"
                    }
                })
                .to_string(),
            ))
            .unwrap();
    }

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(Body::from(
            "data: {\"id\":\"resp_stream_retry_recovered\",\"type\":\"response.output_text.delta\"}\n\ndata: [DONE]\n\n",
        ))
        .unwrap()
}

async fn upstream_response_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_1",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

async fn upstream_response_input_items_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"item_1",
                "object":"response.input_item",
                "type":"message"
            }]
        })),
    )
}

async fn upstream_response_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_1",
            "object":"response.deleted",
            "deleted":true
        })),
    )
}

async fn upstream_response_input_tokens_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"response.input_tokens",
            "input_tokens":21
        })),
    )
}

async fn upstream_response_cancel_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_1",
            "object":"response",
            "model":"gpt-4.1",
            "status":"cancelled",
            "output":[]
        })),
    )
}

async fn upstream_response_compact_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_cmp_1",
            "object":"response.compaction",
            "model":"gpt-4.1"
        })),
    )
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-interface-http-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn native_dynamic_env_guard(path: &Path, public_key: &str) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_connector_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS").ok();
    let previous_native_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_trusted_signers = std::env::var("SDKWORK_EXTENSION_TRUSTED_SIGNERS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false");
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "true",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
        format!("sdkwork={public_key}"),
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature,
        previous_trusted_signers,
    }
}

struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_connector_signature: Option<String>,
    previous_native_signature: Option<String>,
    previous_trusted_signers: Option<String>,
}

impl Drop for ExtensionEnvGuard {
    fn drop(&mut self) {
        restore_env_var("SDKWORK_EXTENSION_PATHS", self.previous_paths.as_deref());
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            self.previous_connector.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            self.previous_connector_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            self.previous_trusted_signers.as_deref(),
        );
    }
}

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}

fn native_dynamic_manifest(library_path: &Path) -> sdkwork_api_extension_core::ExtensionManifest {
    sdkwork_api_extension_core::ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        sdkwork_api_extension_core::ExtensionKind::Provider,
        "0.1.0",
        sdkwork_api_extension_core::ExtensionRuntime::NativeDynamic,
    )
    .with_display_name("Native Mock")
    .with_protocol(sdkwork_api_extension_core::ExtensionProtocol::OpenAi)
    .with_entrypoint(library_path.to_string_lossy())
    .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::Audio)
    .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::Video)
    .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::File)
    .with_channel_binding("sdkwork.channel.openai")
    .with_permission(sdkwork_api_extension_core::ExtensionPermission::NetworkOutbound)
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.create",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.stream",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.create",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.stream",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "audio.speech.create",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "files.content",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "videos.content",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
}

fn sign_native_dynamic_package(
    package_dir: &Path,
    manifest: &sdkwork_api_extension_core::ExtensionManifest,
    signing_key: &SigningKey,
) -> String {
    use ed25519_dalek::Signer;

    #[derive(serde::Serialize)]
    struct PackageSignaturePayload<'a> {
        manifest: &'a sdkwork_api_extension_core::ExtensionManifest,
        files: Vec<PackageFileDigest>,
    }

    #[derive(serde::Serialize)]
    struct PackageFileDigest {
        path: String,
        sha256: String,
    }

    let files = std::fs::read_dir(package_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().and_then(|value| value.to_str()) != Some("sdkwork-extension.toml")
        })
        .map(|path| PackageFileDigest {
            path: path
                .strip_prefix(package_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            sha256: support::sha256_hex_path(&path),
        })
        .collect::<Vec<_>>();

    let payload = serde_json::to_vec(&PackageSignaturePayload { manifest, files }).unwrap();
    let signature = signing_key.sign(&payload);
    STANDARD.encode(signature.to_bytes())
}
