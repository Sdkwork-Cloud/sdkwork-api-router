use super::*;

#[utoipa::path(
    post,
    path = "/admin/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Admin login session.", body = LoginResponse),
        (status = 401, description = "Invalid admin credentials.", body = ErrorResponse),
        (status = 500, description = "Admin authentication failed.", body = ErrorResponse)
    )
)]
pub(super) async fn auth_login() {}

#[utoipa::path(
    post,
    path = "/admin/auth/change-password",
    tag = "auth",
    request_body = ChangePasswordRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated admin profile after password change.", body = AdminUserProfile),
        (status = 400, description = "Invalid password change request.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Password change failed.", body = ErrorResponse)
    )
)]
pub(super) async fn auth_change_password() {}
