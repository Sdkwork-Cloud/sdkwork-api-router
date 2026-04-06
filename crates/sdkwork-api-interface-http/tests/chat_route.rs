use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn chat_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Chat completion model is required."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}

#[tokio::test]
async fn chat_list_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_retrieve_route_returns_not_found_for_unknown_completion() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[tokio::test]
async fn chat_update_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions/chatcmpl_1")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"tier\":\"gold\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_update_route_returns_not_found_for_unknown_completion() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions/chatcmpl_missing")
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"tier\":\"gold\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[tokio::test]
async fn chat_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/chat/completions/chatcmpl_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_delete_route_returns_not_found_for_unknown_completion() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/chat/completions/chatcmpl_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[tokio::test]
async fn chat_messages_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_1/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn chat_messages_route_returns_not_found_for_unknown_completion() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_missing/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
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

#[tokio::test]
async fn stateful_chat_route_records_usage_and_billing() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

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
    assert_eq!(usage_json[0]["model"], "gpt-4.1");

    let ledger = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/ledger")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ledger.status(), StatusCode::OK);
    let ledger_json = read_json(ledger).await;
    assert_eq!(ledger_json[0]["project_id"], "project-1");

    let logs = admin_app
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
    assert_eq!(logs_json[0]["decision_source"], "gateway");
    assert_eq!(logs_json[0]["tenant_id"], "tenant-1");
    assert_eq!(logs_json[0]["project_id"], "project-1");
    assert_eq!(logs_json[0]["capability"], "chat_completion");
    assert_eq!(logs_json[0]["route_key"], "gpt-4.1");
}

#[tokio::test]
async fn stateful_chat_route_returns_invalid_request_for_missing_model_without_usage() {
    let pool = memory_pool().await;
    let api_key =
        support::issue_gateway_api_key(&pool, "tenant-chat-invalid", "project-chat-invalid").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Chat completion model is required."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_chat_retrieve_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-chat-retrieve-missing",
        "project-chat-retrieve-missing",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_chat_update_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-chat-update-missing",
        "project-chat-update-missing",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions/chatcmpl_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"tier\":\"gold\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_chat_delete_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-chat-delete-missing",
        "project-chat-delete-missing",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/chat/completions/chatcmpl_missing")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_chat_messages_route_returns_not_found_without_usage() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-chat-messages-missing",
        "project-chat-messages-missing",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_missing/messages")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested chat completion was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_chat_route_keeps_request_model_for_billing_despite_response_id() {
    let tenant_id = "tenant-chat-model-billing";
    let project_id = "project-chat-model-billing";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
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
                    "{{\"id\":\"provider-chat-model-route\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Chat Model Route Provider\"}}"
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
                    "{\"id\":\"provider-chat-created-id\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"Chat Created Id Provider\"}",
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-model-route\",\"key_reference\":\"cred-chat-model-route\",\"secret_value\":\"sk-chat-model-route\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-created-id\",\"key_reference\":\"cred-chat-created-id\",\"secret_value\":\"sk-chat-created-id\"}}"
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
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-model-route\"}",
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
                        "policy_id": "route-chat-by-request-model",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": ["provider-chat-model-route"]
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
                        "policy_id": "route-chat-by-created-id",
                        "capability": "chat_completion",
                        "model_pattern": "chatcmpl_upstream",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-chat-created-id"]
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
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"route by request model\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-model-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gpt-4.1",
        "provider-chat-model-route",
        "gpt-4.1",
    )
    .await;
}

#[tokio::test]
async fn stateful_chat_route_records_upstream_token_usage() {
    let tenant_id = "tenant-chat-token";
    let project_id = "project-chat-token";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_with_usage),
        )
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
                    "{{\"id\":\"provider-chat-tokens\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"Chat Tokens Provider\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-tokens\",\"key_reference\":\"cred-chat-tokens\",\"secret_value\":\"sk-chat-tokens\"}}"
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
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-tokens\"}",
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
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"count my tokens\"}]}",
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
    assert_eq!(usage_json[0]["input_tokens"], 120);
    assert_eq!(usage_json[0]["output_tokens"], 80);
    assert_eq!(usage_json[0]["total_tokens"], 200);
}

#[tokio::test]
async fn stateful_chat_route_fails_over_to_backup_provider_and_records_actual_provider() {
    let tenant_id = "tenant-chat-failover-json";
    let project_id = "project-chat-failover-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_backup_with_usage),
        )
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
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

    let primary_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Chat Failover Primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_provider.status(), StatusCode::CREATED);

    let backup_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-backup\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{backup_address}\",\"display_name\":\"Chat Failover Backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_provider.status(), StatusCode::CREATED);

    let primary_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-primary\",\"key_reference\":\"cred-chat-failover-primary\",\"secret_value\":\"sk-chat-failover-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_credential.status(), StatusCode::CREATED);

    let backup_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-backup\",\"key_reference\":\"cred-chat-failover-backup\",\"secret_value\":\"sk-chat-failover-backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_credential.status(), StatusCode::CREATED);

    let primary_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_model.status(), StatusCode::CREATED);

    let backup_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-backup\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_model.status(), StatusCode::CREATED);

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
                        "policy_id": "route-chat-failover-json",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-failover-primary",
                            "provider-chat-failover-backup"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"fail over please\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_backup");
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-failover-backup")
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
    assert_eq!(usage_json[0]["provider"], "provider-chat-failover-backup");
    assert_eq!(usage_json[0]["input_tokens"], 42);
    assert_eq!(usage_json[0]["output_tokens"], 18);
    assert_eq!(usage_json[0]["total_tokens"], 60);

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
        "provider-chat-failover-backup"
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-failover-primary\",outcome=\"failure\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-failover-backup\",outcome=\"success\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"chat_completion\",from_provider=\"provider-chat-failover-primary\",to_provider=\"provider-chat-failover-backup\",outcome=\"success\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-chat-failover-primary\",runtime=\"builtin\"} 0"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-chat-failover-backup\",runtime=\"builtin\"} 1"
    ));
}

#[tokio::test]
async fn stateful_chat_route_does_not_fail_over_when_policy_disables_execution_failover() {
    let tenant_id = "tenant-chat-failover-disabled-json";
    let project_id = "project-chat-failover-disabled-json";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_backup_with_usage),
        )
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
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

    let primary_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-disabled-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Chat Failover Disabled Primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_provider.status(), StatusCode::CREATED);

    let backup_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-failover-disabled-backup\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{backup_address}\",\"display_name\":\"Chat Failover Disabled Backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_provider.status(), StatusCode::CREATED);

    let primary_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-disabled-primary\",\"key_reference\":\"cred-chat-failover-disabled-primary\",\"secret_value\":\"sk-chat-failover-disabled-primary\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_credential.status(), StatusCode::CREATED);

    let backup_credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-failover-disabled-backup\",\"key_reference\":\"cred-chat-failover-disabled-backup\",\"secret_value\":\"sk-chat-failover-disabled-backup\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_credential.status(), StatusCode::CREATED);

    let primary_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-disabled-primary\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(primary_model.status(), StatusCode::CREATED);

    let backup_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-failover-disabled-backup\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(backup_model.status(), StatusCode::CREATED);

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
                        "policy_id": "route-chat-failover-disabled-json",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "execution_failover_enabled": false,
                        "upstream_retry_max_attempts": 1,
                        "ordered_provider_ids": [
                            "provider-chat-failover-disabled-primary",
                            "provider-chat-failover-disabled-backup"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"primary must fail without failover\"}]}",
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-failover-disabled-primary\",outcome=\"failure\"} 1"
    ));
    assert!(
        !metrics_text
            .contains("provider=\"provider-chat-failover-disabled-backup\",outcome=\"success\"")
    );
    assert!(!metrics_text.contains(
        "sdkwork_gateway_failovers_total{service=\"gateway\",capability=\"chat_completion\",from_provider=\"provider-chat-failover-disabled-primary\",to_provider=\"provider-chat-failover-disabled-backup\",outcome=\"success\"}"
    ));
}

#[tokio::test]
async fn stateful_chat_stream_route_fails_over_to_backup_provider_and_records_actual_provider() {
    let tenant_id = "tenant-chat-failover-stream";
    let project_id = "project-chat-failover-stream";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_stream_handler_success),
        )
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
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

    for (provider_id, address, secret_value) in [
        (
            "provider-chat-stream-failover-primary",
            primary_address,
            "sk-chat-stream-failover-primary",
        ),
        (
            "provider-chat-stream-failover-backup",
            backup_address,
            "sk-chat-stream-failover-backup",
        ),
    ] {
        let provider = admin_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/providers")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"{provider_id}\"}}"
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
                        "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"cred-{provider_id}\",\"secret_value\":\"{secret_value}\"}}"
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
                        "policy_id": "route-chat-failover-stream",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-stream-failover-primary",
                            "provider-chat-stream-failover-backup"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"stream fail over please\"}],\"stream\":true}",
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
    assert!(stream_text.contains("chatcmpl_stream_backup"));
    assert_eq!(
        primary_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-stream-failover-primary")
    );
    assert_eq!(
        backup_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-chat-stream-failover-backup")
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
        "provider-chat-stream-failover-backup"
    );

    let logs = admin_app
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
        "provider-chat-stream-failover-backup"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

#[tokio::test]
async fn stateful_chat_route_skips_recently_failed_primary_provider_on_following_request() {
    let tenant_id = "tenant-chat-circuit-breaker";
    let project_id = "project-chat-circuit-breaker";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler_failure))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_backup_with_usage),
        )
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
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

    for (provider_id, address, secret_value) in [
        (
            "provider-chat-circuit-breaker-primary",
            primary_address,
            "sk-chat-circuit-breaker-primary",
        ),
        (
            "provider-chat-circuit-breaker-backup",
            backup_address,
            "sk-chat-circuit-breaker-backup",
        ),
    ] {
        let provider = admin_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/providers")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"{provider_id}\"}}"
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
                        "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"cred-{provider_id}\",\"secret_value\":\"{secret_value}\"}}"
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
                        "policy_id": "route-chat-circuit-breaker",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-circuit-breaker-primary",
                            "provider-chat-circuit-breaker-backup"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let first_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"first request should fail over\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = read_json(first_response).await;
    assert_eq!(first_json["id"], "chatcmpl_backup");

    let second_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"second request should bypass failed primary\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_json = read_json(second_response).await;
    assert_eq!(second_json["id"], "chatcmpl_backup");

    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert_eq!(backup_state.request_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn stateful_chat_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-chat-retryable-primary";
    let project_id = "project-chat-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-retryable-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Retryable Primary\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-retryable-primary\",\"key_reference\":\"cred-chat-retryable-primary\",\"secret_value\":\"sk-chat-retryable-primary\"}}"
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
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-retryable-primary\"}",
                ))
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
                        "policy_id": "route-chat-retryable-primary",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-retryable-primary"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"retry transient primary failure\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_retry_recovered");
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_provider_health_status{service=\"gateway\",provider=\"provider-chat-retryable-primary\",runtime=\"builtin\"} 1"
    ));
}

#[tokio::test]
async fn stateful_chat_route_does_not_retry_when_policy_limits_retry_attempts_to_one() {
    let tenant_id = "tenant-chat-retry-max-attempts-one";
    let project_id = "project-chat-retry-max-attempts-one";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-retry-max-attempts-one\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Retry Max Attempts One\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-retry-max-attempts-one\",\"key_reference\":\"cred-chat-retry-max-attempts-one\",\"secret_value\":\"sk-chat-retry-max-attempts-one\"}}"
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
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-retry-max-attempts-one\"}",
                ))
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
                        "policy_id": "route-chat-retry-max-attempts-one",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "upstream_retry_max_attempts": 1,
                        "ordered_provider_ids": [
                            "provider-chat-retry-max-attempts-one"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"retry once must stay disabled by policy\"}]}",
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retry-max-attempts-one\",outcome=\"attempt\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retry-max-attempts-one\",outcome=\"failure\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-retry-max-attempts-one\",outcome=\"scheduled\"}"
    ));
}

#[tokio::test]
async fn stateful_chat_route_does_not_retry_non_retryable_primary_failure() {
    let tenant_id = "tenant-chat-non-retryable-primary";
    let project_id = "project-chat-non-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_non_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-non-retryable-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Non Retryable Primary\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-non-retryable-primary\",\"key_reference\":\"cred-chat-non-retryable-primary\",\"secret_value\":\"sk-chat-non-retryable-primary\"}}"
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
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-non-retryable-primary\"}",
                ))
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
                        "policy_id": "route-chat-non-retryable-primary",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-non-retryable-primary"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"do not retry invalid request\"}]}",
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-non-retryable-primary\",outcome=\"attempt\"} 1"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-non-retryable-primary\",outcome=\"failure\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-non-retryable-primary\",outcome=\"scheduled\"}"
    ));
}

#[tokio::test]
async fn stateful_chat_stream_route_retries_retryable_primary_failure_before_failing_over() {
    let tenant_id = "tenant-chat-stream-retryable-primary";
    let project_id = "project-chat-stream-retryable-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_stream_handler_retryable_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-stream-retryable-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Retryable Stream Primary\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-stream-retryable-primary\",\"key_reference\":\"cred-chat-stream-retryable-primary\",\"secret_value\":\"sk-chat-stream-retryable-primary\"}}"
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
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-stream-retryable-primary\"}",
                ))
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
                        "policy_id": "route-chat-stream-retryable-primary",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-stream-retryable-primary"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"retry transient primary failure for stream\"}],\"stream\":true}",
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
    assert!(stream_text.contains("chatcmpl_stream_retry_recovered"));
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
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-stream-retryable-primary\",outcome=\"attempt\"} 2"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-stream-retryable-primary\",outcome=\"success\"} 1"
    ));
    assert!(!metrics_text.contains(
        "sdkwork_upstream_requests_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-stream-retryable-primary\",outcome=\"failure\"}"
    ));
    assert!(metrics_text.contains(
        "sdkwork_upstream_retries_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-chat-stream-retryable-primary\",outcome=\"scheduled\"} 1"
    ));
}

#[tokio::test]
async fn stateful_chat_route_honors_retry_after_before_retrying_primary_provider() {
    let tenant_id = "tenant-chat-retry-after-primary";
    let project_id = "project-chat-retry-after-primary";
    let primary_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route(
            "/v1/chat/completions",
            post(upstream_chat_handler_retry_after_once_then_success),
        )
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-chat-retry-after-primary\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{primary_address}\",\"display_name\":\"Retry After Primary\"}}"
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
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-chat-retry-after-primary\",\"key_reference\":\"cred-chat-retry-after-primary\",\"secret_value\":\"sk-chat-retry-after-primary\"}}"
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
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-chat-retry-after-primary\"}",
                ))
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
                        "policy_id": "route-chat-retry-after-primary",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
                        "ordered_provider_ids": [
                            "provider-chat-retry-after-primary"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let started = Instant::now();
    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"retry after please\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_retry_after_recovered");
    assert_eq!(primary_state.request_count.load(Ordering::SeqCst), 2);
    assert!(
        elapsed >= Duration::from_millis(900),
        "expected retry-after delay to be honored, got {:?}",
        elapsed
    );
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

#[tokio::test]
async fn stateless_chat_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "custom-openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[tokio::test]
async fn stateless_chat_route_returns_openai_error_envelope_on_upstream_failure() {
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
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream chat completion"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

#[tokio::test]
async fn stateless_chat_stream_route_returns_openai_error_envelope_on_upstream_failure() {
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
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}],\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream chat completion stream"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

#[tokio::test]
async fn stateful_chat_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/chat/completions",
            get(upstream_chat_list_handler).post(upstream_chat_handler),
        )
        .route(
            "/v1/chat/completions/chatcmpl_1",
            get(upstream_chat_retrieve_handler)
                .post(upstream_chat_update_handler)
                .delete(upstream_chat_delete_handler),
        )
        .route(
            "/v1/chat/completions/chatcmpl_1/messages",
            get(upstream_chat_messages_handler),
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
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"custom-openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
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
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "chatcmpl_1");

    let update_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions/chatcmpl_1")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"metadata\":{\"tier\":\"gold\"}}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let update_json = read_json(update_response).await;
    assert_eq!(update_json["metadata"]["tier"], "gold");

    let delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/chat/completions/chatcmpl_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let messages_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/chat/completions/chatcmpl_1/messages")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_json = read_json(messages_response).await;
    assert_eq!(messages_json["data"][0]["id"], "msg_1");
}

#[tokio::test]
async fn stateful_chat_route_relays_to_openrouter_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
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
                .body(Body::from(
                    "{\"id\":\"openrouter\",\"name\":\"OpenRouter\"}",
                ))
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
                    "{{\"id\":\"provider-openrouter-main\",\"channel_id\":\"openrouter\",\"adapter_kind\":\"openrouter\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenRouter Main\"}}"
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openrouter-main\",\"key_reference\":\"cred-openrouter\",\"secret_value\":\"sk-or-v1-upstream\"}",
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
                    "{\"external_name\":\"openai/gpt-4.1\",\"provider_id\":\"provider-openrouter-main\"}",
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
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"openai/gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay openrouter\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-or-v1-upstream")
    );
}

#[tokio::test]
async fn stateful_chat_route_relays_to_ollama_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
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
                .body(Body::from("{\"id\":\"ollama\",\"name\":\"Ollama\"}"))
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
                    "{{\"id\":\"provider-ollama-local\",\"channel_id\":\"ollama\",\"adapter_kind\":\"ollama\",\"base_url\":\"http://{address}\",\"display_name\":\"Ollama Local\"}}"
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-ollama-local\",\"key_reference\":\"cred-ollama\",\"secret_value\":\"ollama-local-token\"}",
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
                    "{\"external_name\":\"llama3.2\",\"provider_id\":\"provider-ollama-local\"}",
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
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"llama3.2\",\"messages\":[{\"role\":\"user\",\"content\":\"relay ollama\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer ollama-local-token")
    );
}

#[tokio::test]
async fn stateful_chat_route_uses_requested_region_for_geo_affinity() {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
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
                .body(Body::from(
                    "{\"id\":\"geo-openai\",\"name\":\"Geo OpenAI\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let create_eu_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-eu-west\",\"channel_id\":\"geo-openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"EU West Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_eu_provider.status(), StatusCode::CREATED);

    let create_us_provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-us-east\",\"channel_id\":\"geo-openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"openai\",\"base_url\":\"http://127.0.0.1:1\",\"display_name\":\"US East Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_us_provider.status(), StatusCode::CREATED);

    let eu_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-eu-west\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(eu_model.status(), StatusCode::CREATED);

    let us_model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(us_model.status(), StatusCode::CREATED);

    let openrouter_installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"geo-eu-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openrouter_installation.status(), StatusCode::CREATED);

    let openai_installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"geo-us-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openai_installation.status(), StatusCode::CREATED);

    let openrouter_instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-eu-west\",\"installation_id\":\"geo-eu-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"enabled\":true,\"base_url\":\"https://eu-west.example/v1\",\"credential_ref\":null,\"config\":{\"region\":\"eu-west\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openrouter_instance.status(), StatusCode::CREATED);

    let openai_instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-us-east\",\"installation_id\":\"geo-us-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"enabled\":true,\"base_url\":\"https://us-east.example/v1\",\"credential_ref\":null,\"config\":{\"routing\":{\"region\":\"us-east\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openai_instance.status(), StatusCode::CREATED);

    let create_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-geo-affinity\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"geo_affinity\",\"ordered_provider_ids\":[\"provider-eu-west\",\"provider-us-east\"]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("x-sdkwork-region", "us-east")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"route me close\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["object"], "chat.completion");

    let logs = admin_app
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
    assert_eq!(logs_json[0]["selected_provider_id"], "provider-us-east");
    assert_eq!(logs_json[0]["requested_region"], "us-east");
}

async fn upstream_chat_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

async fn upstream_chat_handler_with_usage(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":120,
                "completion_tokens":80,
                "total_tokens":200
            }
        })),
    )
}

async fn upstream_chat_handler_failure(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error":{
                "message":"primary upstream failed",
                "type":"server_error",
                "code":"upstream_failed"
            }
        })),
    )
}

async fn upstream_chat_handler_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error":{
                    "message":"upstream rate limited",
                    "type":"rate_limit_error",
                    "code":"retry_later"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_retry_recovered",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":24,
                "completion_tokens":12,
                "total_tokens":36
            }
        })),
    )
}

async fn upstream_chat_handler_retry_after_once_then_success(
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
                        "message":"upstream rate limited with retry-after",
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
                "id":"chatcmpl_retry_after_recovered",
                "object":"chat.completion",
                "model":"gpt-4.1",
                "choices":[],
                "usage":{
                    "prompt_tokens":21,
                    "completion_tokens":9,
                    "total_tokens":30
                }
            })
            .to_string(),
        ))
        .unwrap()
}

async fn upstream_chat_handler_non_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error":{
                    "message":"invalid upstream payload",
                    "type":"invalid_request_error",
                    "code":"invalid_request"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_non_retryable_unexpected_retry",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

async fn upstream_chat_handler_backup_with_usage(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_backup",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":42,
                "completion_tokens":18,
                "total_tokens":60
            }
        })),
    )
}

async fn upstream_chat_stream_handler_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    capture_upstream_request(&state, &headers);

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(Body::from(
            "data: {\"id\":\"chatcmpl_stream_backup\",\"object\":\"chat.completion.chunk\"}\n\ndata: [DONE]\n\n",
        ))
        .unwrap()
}

async fn upstream_chat_stream_handler_retryable_once_then_success(
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
                        "message":"stream temporarily unavailable",
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
            "data: {\"id\":\"chatcmpl_stream_retry_recovered\",\"object\":\"chat.completion.chunk\"}\n\ndata: [DONE]\n\n",
        ))
        .unwrap()
}

async fn upstream_chat_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"chatcmpl_1",
                "object":"chat.completion",
                "model":"gpt-4.1",
                "choices":[]
            }]
        })),
    )
}

async fn upstream_chat_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

async fn upstream_chat_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "metadata":{"tier":"gold"},
            "choices":[]
        })),
    )
}

async fn upstream_chat_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_1",
            "object":"chat.completion.deleted",
            "deleted":true
        })),
    )
}

async fn upstream_chat_messages_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"msg_1",
                "object":"chat.completion.message",
                "role":"assistant",
                "content":"hello"
            }]
        })),
    )
}
