use super::*;

pub(super) fn gateway_service_name() -> Arc<str> {
    Arc::from("gateway")
}

pub(super) fn gateway_http_metrics() -> Arc<HttpMetricsRegistry> {
    Arc::new(HttpMetricsRegistry::new("gateway"))
}

pub(super) fn gateway_base_router(metrics: Arc<HttpMetricsRegistry>) -> Router {
    Router::new()
        .route("/openapi.json", get(gateway_openapi_handler))
        .route("/docs", get(gateway_docs_handler))
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                move || {
                    let metrics = metrics.clone();
                    async move {
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            metrics.render_prometheus(),
                        )
                    }
                }
            }),
        )
        .route("/health", get(|| async { "ok" }))
}

pub(super) fn finalize_stateful_gateway_router(
    router: Router,
    state: GatewayApiState,
    service_name: Arc<str>,
    metrics: Arc<HttpMetricsRegistry>,
) -> Router {
    router
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_request_context,
        ))
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

pub(super) fn finalize_stateless_gateway_router(
    router: Router,
    config: StatelessGatewayConfig,
    service_name: Arc<str>,
    metrics: Arc<HttpMetricsRegistry>,
) -> Router {
    router
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer())
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(config.into_context())
}
