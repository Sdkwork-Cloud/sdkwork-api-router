use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{persist_gateway_api_key_with_metadata, PersistGatewayApiKeyInput};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn issue_gateway_api_key_in_byok_group(
    pool: &SqlitePool,
    admin_app: &axum::Router,
    admin_token: &str,
    tenant_id: &str,
    project_id: &str,
) -> String {
    let create_group = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"project_id\":\"{project_id}\",\"environment\":\"live\",\"name\":\"Quota BYOK Keys\",\"slug\":\"quota-byok-keys\",\"default_accounting_mode\":\"byok\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_group.status(), StatusCode::CREATED);
    let group_json = read_json(create_group).await;
    let group_id = group_json["group_id"].as_str().unwrap().to_owned();

    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    persist_gateway_api_key_with_metadata(
        &store,
        PersistGatewayApiKeyInput {
            tenant_id,
            project_id,
            environment: "live",
            label: "Quota BYOK test key",
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: Some(&group_id),
        },
    )
    .await
    .unwrap()
    .plaintext
}

#[tokio::test]
async fn stateful_chat_route_returns_429_when_project_quota_is_exhausted() {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let api_key = issue_gateway_api_key_in_byok_group(
        &pool,
        &admin_app,
        &admin_token,
        "tenant-1",
        "project-1",
    )
    .await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let create_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/quota-policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"quota-project-1\",\"project_id\":\"project-1\",\"max_units\":50,\"enabled\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let response = gateway_app
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

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let json = read_json(response).await;
    assert_eq!(json["error"]["type"], "insufficient_quota");
    assert_eq!(json["error"]["code"], "quota_exceeded");
}
