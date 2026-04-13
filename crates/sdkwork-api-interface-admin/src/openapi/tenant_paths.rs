use super::*;

#[utoipa::path(
    get,
    path = "/admin/tenants",
    tag = "tenants",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible tenant catalog.", body = [Tenant]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load tenants.")
    )
)]
pub(super) async fn tenants_list() {}

#[utoipa::path(
    post,
    path = "/admin/tenants",
    tag = "tenants",
    request_body = CreateTenantRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created tenant.", body = Tenant),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create tenant.")
    )
)]
pub(super) async fn tenants_create() {}

#[utoipa::path(
    get,
    path = "/admin/projects",
    tag = "tenants",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible project catalog.", body = [Project]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load projects.")
    )
)]
pub(super) async fn projects_list() {}

#[utoipa::path(
    post,
    path = "/admin/projects",
    tag = "tenants",
    request_body = CreateProjectRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created project.", body = Project),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create project.")
    )
)]
pub(super) async fn projects_create() {}
