use super::*;

pub fn gemini_invalid_request_response(message: impl Into<String>) -> Response {
    gemini_error_response(StatusCode::BAD_REQUEST, "INVALID_ARGUMENT", message.into())
}

pub fn gemini_bad_gateway_response(message: impl Into<String>) -> Response {
    gemini_error_response(StatusCode::BAD_GATEWAY, "BAD_GATEWAY", message.into())
}

pub fn gemini_error_response(status: StatusCode, google_status: &str, message: String) -> Response {
    (
        status,
        Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
                "status": google_status
            }
        })),
    )
        .into_response()
}
