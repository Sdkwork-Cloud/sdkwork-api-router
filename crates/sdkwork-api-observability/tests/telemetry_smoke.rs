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

#[test]
fn registries_with_same_service_share_metric_state() {
    let writer = HttpMetricsRegistry::new("shared-gateway-service");
    writer.record("GET", "/health", 200, 7);

    let reader = HttpMetricsRegistry::new("shared-gateway-service");
    let output = reader.render_prometheus();

    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"shared-gateway-service\",method=\"GET\",route=\"/health\",status=\"200\"} 1"
    ));
}

#[test]
fn renders_prometheus_metrics_for_recorded_upstream_outcomes() {
    let writer = HttpMetricsRegistry::new("upstream-gateway-service");
    writer.record_upstream_outcome("chat_completion", "provider-openai", "attempt");
    writer.record_upstream_outcome("chat_completion", "provider-openai", "success");
    writer.record_upstream_outcome("chat_completion", "provider-openai", "failure");
    writer.record_upstream_retry("chat_completion", "provider-openai", "scheduled");
    writer.record_upstream_retry("chat_completion", "provider-openai", "exhausted");
    writer.record_upstream_retry_reason(
        "chat_completion",
        "provider-openai",
        "scheduled",
        "status_429",
    );
    writer.record_upstream_retry_reason(
        "chat_completion",
        "provider-openai",
        "exhausted",
        "status_503",
    );
    writer.record_upstream_retry_delay(
        "chat_completion",
        "provider-openai",
        "retry_after_seconds",
        1000,
    );
    writer.record_gateway_failover(
        "chat_completion",
        "provider-openai-primary",
        "provider-openai-backup",
        "success",
    );

    let reader = HttpMetricsRegistry::new("upstream-gateway-service");
    let output = reader.render_prometheus();

    assert!(output.contains(
        "sdkwork_upstream_requests_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"attempt\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_requests_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"success\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_requests_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"failure\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_retries_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"scheduled\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_retries_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"exhausted\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"scheduled\",reason=\"status_429\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_retry_reasons_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",outcome=\"exhausted\",reason=\"status_503\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_upstream_retry_delay_ms_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",provider=\"provider-openai\",source=\"retry_after_seconds\"} 1000"
    ));
    assert!(output.contains(
        "sdkwork_gateway_failovers_total{service=\"upstream-gateway-service\",capability=\"chat_completion\",from_provider=\"provider-openai-primary\",to_provider=\"provider-openai-backup\",outcome=\"success\"} 1"
    ));
}

#[test]
fn renders_prometheus_metrics_for_recorded_commerce_reconciliation_outcomes() {
    let writer = HttpMetricsRegistry::new("commerce-recovery-service");
    writer.record_commerce_reconciliation_success(2, 45, 3, 1_710_000_001_000);
    writer.record_commerce_reconciliation_failure(1, 15, 1_710_000_002_000);

    let reader = HttpMetricsRegistry::new("commerce-recovery-service");
    let output = reader.render_prometheus();

    assert!(output.contains(
        "sdkwork_commerce_reconciliation_attempts_total{service=\"commerce-recovery-service\",outcome=\"success\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_commerce_reconciliation_attempts_total{service=\"commerce-recovery-service\",outcome=\"failure\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_commerce_reconciliation_processed_orders_total{service=\"commerce-recovery-service\"} 3"
    ));
    assert!(output.contains(
        "sdkwork_commerce_reconciliation_backlog_orders{service=\"commerce-recovery-service\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_commerce_reconciliation_checkpoint_lag_ms{service=\"commerce-recovery-service\"} 15"
    ));
    assert!(output.contains(
        "sdkwork_commerce_reconciliation_last_success_at_ms{service=\"commerce-recovery-service\"} 1710000001000"
    ));
    assert!(output.contains(
        "sdkwork_commerce_reconciliation_last_failure_at_ms{service=\"commerce-recovery-service\"} 1710000002000"
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
