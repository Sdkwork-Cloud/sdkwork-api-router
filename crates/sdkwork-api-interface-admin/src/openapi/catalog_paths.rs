use super::*;

#[utoipa::path(
    get,
    path = "/admin/tenants/{tenant_id}/providers/readiness",
    tag = "catalog",
    security(("bearerAuth" = [])),
    params(
        ("tenant_id" = String, Path, description = "Tenant identifier whose provider credential-readiness overlay should be listed.")
    ),
    responses(
        (status = 200, description = "Tenant-scoped provider readiness inventory. This endpoint focuses on tenant overlay state and keeps global execution truth on `/admin/providers`.", body = [TenantProviderReadinessResponse]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load tenant-scoped provider readiness.", body = ErrorResponse)
    )
)]
pub(super) async fn tenant_provider_readiness_list() {}

#[utoipa::path(
    get,
    path = "/admin/providers",
    tag = "catalog",
    security(("bearerAuth" = [])),
    params(
        ("tenant_id" = Option<String>, Query, description = "Optional tenant scope. When present, each provider entry adds tenant-specific `credential_readiness` without changing the global catalog semantics of `integration` and `execution`.")
    ),
    responses(
        (status = 200, description = "Visible provider catalog. `credential_readiness` is returned only when `tenant_id` is requested.", body = [ProviderCatalogResponse]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load providers.", body = ErrorResponse)
    )
)]
pub(super) async fn providers_list() {}

#[utoipa::path(
    post,
    path = "/admin/providers",
    tag = "catalog",
    request_body = CreateProviderRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created provider with normalized integration metadata.", body = ProviderCreateResponse),
        (status = 400, description = "Invalid provider payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create provider.", body = ErrorResponse)
    )
)]
pub(super) async fn providers_create() {}
