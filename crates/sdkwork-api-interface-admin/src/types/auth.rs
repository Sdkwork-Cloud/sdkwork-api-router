use super::*;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LoginResponse {
    pub(crate) token: String,
    pub(crate) claims: Claims,
    pub(crate) user: AdminUserProfile,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ChangePasswordRequest {
    pub(crate) current_password: String,
    pub(crate) new_password: String,
}
