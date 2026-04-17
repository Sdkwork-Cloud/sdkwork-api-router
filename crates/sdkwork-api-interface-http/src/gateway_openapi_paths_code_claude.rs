#[utoipa::path(
        post,
        path = "/v1/messages",
        tag = "code.claude",
        request_body = serde_json::Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Claude-compatible message result.", body = serde_json::Value),
            (status = 400, description = "Invalid Claude message payload.", body = serde_json::Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = serde_json::Value),
            (status = 500, description = "Gateway failed to serve the Claude mirror route.", body = serde_json::Value)
        )
    )]
pub(crate) async fn anthropic_messages() {}

#[utoipa::path(
        post,
        path = "/v1/messages/count_tokens",
        tag = "code.claude",
        request_body = serde_json::Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Claude-compatible token count result.", body = serde_json::Value),
            (status = 400, description = "Invalid Claude token count payload.", body = serde_json::Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = serde_json::Value),
            (status = 500, description = "Gateway failed to serve the Claude token count route.", body = serde_json::Value)
        )
    )]
pub(crate) async fn anthropic_count_tokens() {}
