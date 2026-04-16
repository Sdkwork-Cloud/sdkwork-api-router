#[utoipa::path(
        get,
        path = "/health",
        tag = "system.sdkwork",
        responses((status = 200, description = "Gateway health check."))
    )]
pub(crate) async fn health() {}
