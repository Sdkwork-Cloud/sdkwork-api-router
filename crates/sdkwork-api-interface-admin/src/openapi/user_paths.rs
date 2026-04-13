use super::*;

#[utoipa::path(
    get,
    path = "/admin/users/operators",
    tag = "users",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible operator users.", body = [AdminUserProfile]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load operator users.", body = ErrorResponse)
    )
)]
pub(super) async fn operator_users_list() {}

#[utoipa::path(
    post,
    path = "/admin/users/operators",
    tag = "users",
    request_body = UpsertOperatorUserRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created or updated operator user.", body = AdminUserProfile),
        (status = 400, description = "Invalid operator user payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to persist operator user.", body = ErrorResponse)
    )
)]
pub(super) async fn operator_users_upsert() {}

#[utoipa::path(
    post,
    path = "/admin/users/operators/{user_id}/status",
    tag = "users",
    params(("user_id" = String, Path, description = "Operator user id.")),
    request_body = UpdateUserStatusRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated operator user status.", body = AdminUserProfile),
        (status = 400, description = "Invalid operator user status payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to update operator user status.", body = ErrorResponse)
    )
)]
pub(super) async fn operator_user_status_update() {}

#[utoipa::path(
    get,
    path = "/admin/users/portal",
    tag = "users",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible portal users.", body = [PortalUserProfile]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load portal users.", body = ErrorResponse)
    )
)]
pub(super) async fn portal_users_list() {}

#[utoipa::path(
    post,
    path = "/admin/users/portal",
    tag = "users",
    request_body = UpsertPortalUserRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created or updated portal user.", body = PortalUserProfile),
        (status = 400, description = "Invalid portal user payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to persist portal user.", body = ErrorResponse)
    )
)]
pub(super) async fn portal_users_upsert() {}

#[utoipa::path(
    post,
    path = "/admin/users/portal/{user_id}/status",
    tag = "users",
    params(("user_id" = String, Path, description = "Portal user id.")),
    request_body = UpdateUserStatusRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated portal user status.", body = PortalUserProfile),
        (status = 400, description = "Invalid portal user status payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to update portal user status.", body = ErrorResponse)
    )
)]
pub(super) async fn portal_user_status_update() {}
