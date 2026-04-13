use super::*;

pub fn anthropic_invalid_request_response(message: impl Into<String>) -> Response {
    anthropic_error_response(
        StatusCode::BAD_REQUEST,
        "invalid_request_error",
        message.into(),
    )
}

pub fn anthropic_bad_gateway_response(message: impl Into<String>) -> Response {
    anthropic_error_response(StatusCode::BAD_GATEWAY, "api_error", message.into())
}

pub fn anthropic_error_response(status: StatusCode, error_type: &str, message: String) -> Response {
    (
        status,
        Json(json!({
            "type": "error",
            "error": {
                "type": error_type,
                "message": message
            }
        })),
    )
        .into_response()
}
