use super::*;

pub(super) fn local_anthropic_stream_response(model: &str) -> Response {
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

pub(super) fn local_gemini_stream_response() -> Response {
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

pub(super) fn upstream_passthrough_response(response: ProviderStreamOutput) -> Response {
    let content_type = response.content_type().to_owned();
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from_stream(response.into_body_stream()))
        .expect("valid upstream stream response")
}

pub(super) fn local_response_stream_response(response_id: &str, model: &str) -> Response {
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
