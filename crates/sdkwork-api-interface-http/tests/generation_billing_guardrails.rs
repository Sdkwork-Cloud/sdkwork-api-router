use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn stateful_completions_route_keeps_request_model_for_billing_despite_response_id() {
    let tenant_id = "tenant-completions-model-billing";
    let project_id = "project-completions-model-billing";
    let request_model = "gpt-3.5-turbo-instruct";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/completions", post(upstream_completions_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-completions-model-route",
        &format!("http://{address}"),
        "Completions Model Route Provider",
    )
    .await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-completions-created-id",
        "http://127.0.0.1:1",
        "Completions Created Id Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-completions-model-route",
        "cred-completions-model-route",
        "sk-completions-model-route",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-completions-created-id",
        "cred-completions-created-id",
        "sk-completions-created-id",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-completions-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-completions-by-request-model",
        "completions",
        request_model,
        200,
        "provider-completions-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-completions-by-created-id",
        "completions",
        "cmpl_upstream",
        100,
        "provider-completions-created-id",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"model\":\"{request_model}\",\"prompt\":\"keep request model\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "cmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-completions-model-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        request_model,
        "provider-completions-model-route",
        request_model,
    )
    .await;
}

#[tokio::test]
async fn stateful_moderations_route_keeps_request_model_for_billing_despite_response_id() {
    let tenant_id = "tenant-moderations-model-billing";
    let project_id = "project-moderations-model-billing";
    let request_model = "omni-moderation-latest";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/moderations", post(upstream_moderations_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-moderations-model-route",
        &format!("http://{address}"),
        "Moderations Model Route Provider",
    )
    .await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-moderations-created-id",
        "http://127.0.0.1:1",
        "Moderations Created Id Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-moderations-model-route",
        "cred-moderations-model-route",
        "sk-moderations-model-route",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-moderations-created-id",
        "cred-moderations-created-id",
        "sk-moderations-created-id",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-moderations-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-moderations-by-request-model",
        "moderations",
        request_model,
        200,
        "provider-moderations-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-moderations-by-created-id",
        "moderations",
        "modr_upstream",
        100,
        "provider-moderations-created-id",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/moderations")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"model\":\"{request_model}\",\"input\":\"keep request model\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "modr_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-moderations-model-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        request_model,
        "provider-moderations-model-route",
        request_model,
    )
    .await;
}

async fn create_channel(admin_app: &Router, admin_token: &str) {
    let response = admin_app
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
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_provider(
    admin_app: &Router,
    admin_token: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "id": provider_id,
                        "channel_id": "openai",
                        "adapter_kind": "openai",
                        "base_url": base_url,
                        "display_name": display_name,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_credential(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "tenant_id": tenant_id,
                        "provider_id": provider_id,
                        "key_reference": key_reference,
                        "secret_value": secret_value,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_model(
    admin_app: &Router,
    admin_token: &str,
    external_name: &str,
    provider_id: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "external_name": external_name,
                        "provider_id": provider_id,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_routing_policy(
    admin_app: &Router,
    admin_token: &str,
    policy_id: &str,
    capability: &str,
    model_pattern: &str,
    priority: i64,
    provider_id: &str,
) {
    let response = admin_app
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
                        "capability": capability,
                        "model_pattern": model_pattern,
                        "enabled": true,
                        "priority": priority,
                        "ordered_provider_ids": [provider_id],
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

async fn upstream_completions_handler(
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
            "id": "cmpl_upstream",
            "object": "text_completion",
            "choices": [{"index": 0, "text": "relay completion", "finish_reason": "stop"}],
        })),
    )
}

async fn upstream_moderations_handler(
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
            "id": "modr_upstream",
            "model": "omni-moderation-latest",
            "results": [{"flagged": false, "category_scores": {"violence": 0.0}}],
        })),
    )
}
