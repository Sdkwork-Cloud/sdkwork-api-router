use super::*;

#[utoipa::path(
    get,
    path = "/admin/api-keys",
    tag = "gateway",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible gateway API keys.", body = [GatewayApiKeyRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load gateway API keys.")
    )
)]
pub(super) async fn api_keys_list() {}

#[utoipa::path(
    post,
    path = "/admin/api-keys",
    tag = "gateway",
    request_body = CreateApiKeyRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created gateway API key.", body = CreatedGatewayApiKey),
        (status = 400, description = "Invalid gateway API key payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create gateway API key.", body = ErrorResponse)
    )
)]
pub(super) async fn api_keys_create() {}

#[utoipa::path(
    put,
    path = "/admin/api-keys/{hashed_key}",
    tag = "gateway",
    params(("hashed_key" = String, Path, description = "Hashed gateway API key identifier.")),
    request_body = UpdateApiKeyRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated gateway API key metadata.", body = GatewayApiKeyRecord),
        (status = 400, description = "Invalid gateway API key update payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Gateway API key not found.", body = ErrorResponse),
        (status = 500, description = "Failed to update gateway API key.", body = ErrorResponse)
    )
)]
pub(super) async fn api_key_update() {}

#[utoipa::path(
    get,
    path = "/admin/api-key-groups",
    tag = "gateway",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible gateway API key groups.", body = [ApiKeyGroupRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load gateway API key groups.")
    )
)]
pub(super) async fn api_key_groups_list() {}

#[utoipa::path(
    post,
    path = "/admin/api-key-groups",
    tag = "gateway",
    request_body = CreateApiKeyGroupRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created gateway API key group.", body = ApiKeyGroupRecord),
        (status = 400, description = "Invalid gateway API key group payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create gateway API key group.", body = ErrorResponse)
    )
)]
pub(super) async fn api_key_groups_create() {}

#[utoipa::path(
    patch,
    path = "/admin/api-key-groups/{group_id}",
    tag = "gateway",
    params(("group_id" = String, Path, description = "Gateway API key group identifier.")),
    request_body = UpdateApiKeyGroupRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated gateway API key group.", body = ApiKeyGroupRecord),
        (status = 400, description = "Invalid gateway API key group update payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Gateway API key group not found.", body = ErrorResponse),
        (status = 500, description = "Failed to update gateway API key group.", body = ErrorResponse)
    )
)]
pub(super) async fn api_key_group_update() {}
