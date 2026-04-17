#[utoipa::path(
        post,
        path = "/v1beta/models/{tail}",
        tag = "code.gemini",
        params(("tail" = String, Path, description = "Gemini mirror route suffix.")),
        request_body = serde_json::Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Gemini-compatible route result.", body = serde_json::Value),
            (status = 400, description = "Invalid Gemini payload.", body = serde_json::Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = serde_json::Value),
            (status = 500, description = "Gateway failed to serve the Gemini mirror route.", body = serde_json::Value)
        )
    )]
pub(crate) async fn gemini_models_compat() {}
