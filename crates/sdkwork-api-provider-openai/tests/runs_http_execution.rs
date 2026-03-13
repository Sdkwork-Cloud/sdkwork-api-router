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
    beta: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
}

#[tokio::test]
async fn adapter_manages_thread_runs_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/threads/thread_1/runs",
            get(capture_thread_runs_list_request).post(capture_thread_run_create_request),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1",
            get(capture_thread_run_retrieve_request).post(capture_thread_run_update_request),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/cancel",
            post(capture_thread_run_cancel_request),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/submit_tool_outputs",
            post(capture_thread_run_submit_tool_outputs_request),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps",
            get(capture_thread_run_steps_list_request),
        )
        .route(
            "/v1/threads/thread_1/runs/run_1/steps/step_1",
            get(capture_thread_run_step_retrieve_request),
        )
        .route("/v1/threads/runs", post(capture_thread_and_run_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let create_request = sdkwork_api_contract_openai::runs::CreateRunRequest::new("asst_1");

    let created = adapter
        .create_thread_run("sk-upstream-openai", "thread_1", &create_request)
        .await
        .unwrap();
    assert_eq!(created["id"], "run_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(state.beta.lock().unwrap().as_deref(), Some("assistants=v2"));
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["assistant_id"],
        "asst_1"
    );

    let list = adapter
        .list_thread_runs("sk-upstream-openai", "thread_1")
        .await
        .unwrap();
    assert_eq!(list["object"], "list");
    assert_eq!(list["data"][0]["id"], "run_1");

    let retrieve = adapter
        .retrieve_thread_run("sk-upstream-openai", "thread_1", "run_1")
        .await
        .unwrap();
    assert_eq!(retrieve["id"], "run_1");

    let update_request =
        sdkwork_api_contract_openai::runs::UpdateRunRequest::with_metadata(json!({
            "priority":"high"
        }));
    let update = adapter
        .update_thread_run("sk-upstream-openai", "thread_1", "run_1", &update_request)
        .await
        .unwrap();
    assert_eq!(update["metadata"]["priority"], "high");

    let cancel = adapter
        .cancel_thread_run("sk-upstream-openai", "thread_1", "run_1")
        .await
        .unwrap();
    assert_eq!(cancel["status"], "cancelled");

    let submit_request = sdkwork_api_contract_openai::runs::SubmitToolOutputsRunRequest::new(vec![
        sdkwork_api_contract_openai::runs::RunToolOutput::new("call_1", "{\"ok\":true}"),
    ]);
    let submit = adapter
        .submit_thread_run_tool_outputs("sk-upstream-openai", "thread_1", "run_1", &submit_request)
        .await
        .unwrap();
    assert_eq!(submit["id"], "run_1");

    let steps = adapter
        .list_thread_run_steps("sk-upstream-openai", "thread_1", "run_1")
        .await
        .unwrap();
    assert_eq!(steps["object"], "list");
    assert_eq!(steps["data"][0]["id"], "step_1");

    let step = adapter
        .retrieve_thread_run_step("sk-upstream-openai", "thread_1", "run_1", "step_1")
        .await
        .unwrap();
    assert_eq!(step["id"], "step_1");
}

#[tokio::test]
async fn adapter_posts_thread_and_run_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/threads/runs", post(capture_thread_and_run_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::runs::CreateThreadAndRunRequest::new(
        "asst_1",
        json!({"metadata":{"workspace":"default"}}),
    );

    let response = adapter
        .create_thread_and_run("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "run_1");
    assert_eq!(state.beta.lock().unwrap().as_deref(), Some("assistants=v2"));
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["thread"]["metadata"]["workspace"],
        "default"
    );
}

async fn capture_thread_run_create_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

async fn capture_thread_runs_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[thread_run_json("run_1", "queued")],
            "first_id":"run_1",
            "last_id":"run_1",
            "has_more":false
        })),
    )
}

async fn capture_thread_run_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (
        StatusCode::OK,
        Json(thread_run_json("run_1", "in_progress")),
    )
}

async fn capture_thread_run_update_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);
    (
        StatusCode::OK,
        Json(json!({
            "id":"run_1",
            "object":"thread.run",
            "thread_id":"thread_1",
            "assistant_id":"asst_1",
            "status":"in_progress",
            "model":"gpt-4.1",
            "metadata":{"priority":"high"}
        })),
    )
}

async fn capture_thread_run_cancel_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_json("run_1", "cancelled")))
}

async fn capture_thread_run_submit_tool_outputs_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

async fn capture_thread_run_steps_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[thread_run_step_json("step_1")],
            "first_id":"step_1",
            "last_id":"step_1",
            "has_more":false
        })),
    )
}

async fn capture_thread_run_step_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    (StatusCode::OK, Json(thread_run_step_json("step_1")))
}

async fn capture_thread_and_run_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);
    (StatusCode::OK, Json(thread_run_json("run_1", "queued")))
}

fn capture_headers(state: &CaptureState, headers: &HeaderMap) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.beta.lock().unwrap() = headers
        .get("openai-beta")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
}

fn thread_run_json(id: &str, status: &str) -> Value {
    json!({
        "id":id,
        "object":"thread.run",
        "thread_id":"thread_1",
        "assistant_id":"asst_1",
        "status":status,
        "model":"gpt-4.1",
        "metadata":{"priority":"high"}
    })
}

fn thread_run_step_json(id: &str) -> Value {
    json!({
        "id":id,
        "object":"thread.run.step",
        "thread_id":"thread_1",
        "assistant_id":"asst_1",
        "run_id":"run_1",
        "type":"message_creation",
        "status":"completed",
        "step_details":{
            "message_creation":{
                "message_id":"msg_1"
            }
        }
    })
}
