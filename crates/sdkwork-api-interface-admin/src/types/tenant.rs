use super::*;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateTenantRequest {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProjectRequest {
    pub(crate) tenant_id: String,
    pub(crate) id: String,
    pub(crate) name: String,
}
