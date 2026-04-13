use super::presentation::{
    gateway_operation_summary, gateway_route_requires_bearer_auth, gateway_tag_for_path,
};
use super::*;

const GATEWAY_OPENAPI_SPEC: OpenApiServiceSpec<'static> = OpenApiServiceSpec {
    title: "SDKWORK Gateway API",
    version: env!("CARGO_PKG_VERSION"),
    description: "OpenAPI 3.1 inventory generated from the current gateway router implementation.",
    openapi_path: "/openapi.json",
    docs_path: "/docs",
};

fn gateway_route_inventory() -> &'static [RouteEntry] {
    static ROUTES: OnceLock<Vec<RouteEntry>> = OnceLock::new();
    ROUTES
        .get_or_init(|| {
            extract_routes_from_function(
                include_str!("../gateway_stateful_router.rs"),
                "build_stateful_gateway_router",
            )
            .expect("gateway route inventory")
        })
        .as_slice()
}

fn gateway_openapi_document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        build_openapi_document(
            &GATEWAY_OPENAPI_SPEC,
            gateway_route_inventory(),
            gateway_tag_for_path,
            gateway_route_requires_bearer_auth,
            gateway_operation_summary,
        )
    })
}

fn gateway_docs_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| render_docs_html(&GATEWAY_OPENAPI_SPEC))
        .as_str()
}

pub(super) async fn gateway_openapi_handler() -> Json<Value> {
    Json(gateway_openapi_document().clone())
}

pub(super) async fn gateway_docs_handler() -> Html<String> {
    Html(gateway_docs_html().to_owned())
}
