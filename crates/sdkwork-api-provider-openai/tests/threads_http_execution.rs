use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Clone, Default)]
struct CaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
}

#[tokio::test]
async fn adapter_posts_threads_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/threads", post(capture_thread_create_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::threads::CreateThreadRequest::with_metadata(json!({
        "workspace":"default"
    }));

    let response = adapter
        .threads("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "thread_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["metadata"]["workspace"],
        "default"
    );
}

#[tokio::test]
async fn adapter_retrieves_updates_and_deletes_thread_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/threads/thread_1",
            get(capture_thread_retrieve_request)
                .post(capture_thread_update_request)
                .delete(capture_thread_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));

    let retrieve = adapter
        .retrieve_thread("sk-upstream-openai", "thread_1")
        .await
        .unwrap();
    assert_eq!(retrieve["id"], "thread_1");

    let update_request =
        sdkwork_api_contract_openai::threads::UpdateThreadRequest::with_metadata(json!({
            "workspace":"next"
        }));
    let update = adapter
        .update_thread("sk-upstream-openai", "thread_1", &update_request)
        .await
        .unwrap();
    assert_eq!(update["metadata"]["workspace"], "next");

    let delete = adapter
        .delete_thread("sk-upstream-openai", "thread_1")
        .await
        .unwrap();
    assert_eq!(delete["deleted"], true);
}

#[tokio::test]
async fn adapter_manages_thread_messages_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/threads/thread_1/messages",
            get(capture_thread_messages_list_request).post(capture_thread_message_create_request),
        )
        .route(
            "/v1/threads/thread_1/messages/msg_1",
            get(capture_thread_message_retrieve_request)
                .post(capture_thread_message_update_request)
                .delete(capture_thread_message_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let create_request =
        sdkwork_api_contract_openai::threads::CreateThreadMessageRequest::text("user", "hello");

    let created = adapter
        .create_thread_message("sk-upstream-openai", "thread_1", &create_request)
        .await
        .unwrap();
    assert_eq!(created["id"], "msg_1");
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["content"],
        "hello"
    );

    let list = adapter
        .list_thread_messages("sk-upstream-openai", "thread_1")
        .await
        .unwrap();
    assert_eq!(list["object"], "list");
    assert_eq!(list["data"][0]["id"], "msg_1");

    let retrieve = adapter
        .retrieve_thread_message("sk-upstream-openai", "thread_1", "msg_1")
        .await
        .unwrap();
    assert_eq!(retrieve["id"], "msg_1");

    let update_request =
        sdkwork_api_contract_openai::threads::UpdateThreadMessageRequest::with_metadata(json!({
            "pinned":"true"
        }));
    let update = adapter
        .update_thread_message("sk-upstream-openai", "thread_1", "msg_1", &update_request)
        .await
        .unwrap();
    assert_eq!(update["metadata"]["pinned"], "true");

    let delete = adapter
        .delete_thread_message("sk-upstream-openai", "thread_1", "msg_1")
        .await
        .unwrap();
    assert_eq!(delete["deleted"], true);
}

async fn capture_thread_create_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"thread_1",
            "object":"thread",
            "metadata":{"workspace":"default"}
        })),
    )
}

async fn capture_thread_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"thread_1",
            "object":"thread",
            "metadata":{"workspace":"default"}
        })),
    )
}

async fn capture_thread_update_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"thread_1",
            "object":"thread",
            "metadata":{"workspace":"next"}
        })),
    )
}

async fn capture_thread_delete_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"thread_1",
            "object":"thread.deleted",
            "deleted":true
        })),
    )
}

async fn capture_thread_message_create_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(thread_message_json("msg_1", json!({"pinned":"true"}))),
    )
}

async fn capture_thread_messages_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[thread_message_json("msg_1", json!({"pinned":"true"}))],
            "first_id":"msg_1",
            "last_id":"msg_1",
            "has_more":false
        })),
    )
}

async fn capture_thread_message_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(thread_message_json("msg_1", json!({"pinned":"true"}))),
    )
}

async fn capture_thread_message_update_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(thread_message_json("msg_1", json!({"pinned":"true"}))),
    )
}

async fn capture_thread_message_delete_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"msg_1",
            "object":"thread.message.deleted",
            "deleted":true
        })),
    )
}

fn thread_message_json(id: &str, metadata: Value) -> Value {
    json!({
        "id":id,
        "object":"thread.message",
        "thread_id":"thread_1",
        "assistant_id":null,
        "run_id":null,
        "role":"assistant",
        "status":"completed",
        "metadata":metadata,
        "content":[{
            "type":"text",
            "text":{
                "value":"hello",
                "annotations":[]
            }
        }]
    })
}
