use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use sdkwork_api_observability::{
    init_tracing, observe_http_metrics, observe_http_tracing, HttpMetricsRegistry,
};
use tower::ServiceExt;

#[test]
fn renders_prometheus_metrics_for_recorded_requests() {
    let registry = HttpMetricsRegistry::new("gateway-service");
    registry.record("GET", "/health", 200, 12);
    registry.record("POST", "/v1/chat/completions", 200, 48);
    registry.record("POST", "/v1/chat/completions", 429, 5);

    let output = registry.render_prometheus();

    assert!(output.contains("sdkwork_service_info{service=\"gateway-service\"} 1"));
    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"GET\",route=\"/health\",status=\"200\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"200\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"429\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_request_duration_ms_sum{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"200\"} 48"
    ));
    assert!(output.contains(
        "sdkwork_http_request_duration_ms_count{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"429\"} 1"
    ));
}

#[tokio::test]
async fn tracing_middleware_generates_and_preserves_request_ids() {
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    let service_name: Arc<str> = Arc::from("gateway");
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ));

    let generated = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generated.status(), StatusCode::OK);
    let generated_request_id = generated
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap()
        .to_owned();
    assert!(generated_request_id.starts_with("sdkw-"));

    let preserved = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("x-request-id", "caller-request-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(preserved.status(), StatusCode::OK);
    assert_eq!(
        preserved
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap(),
        "caller-request-id"
    );
}

#[test]
fn tracing_initialization_is_idempotent() {
    init_tracing("test-service");
    init_tracing("test-service");
}
