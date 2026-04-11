use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use sdkwork_api_observability::{
    init_tracing, observe_http_metrics, observe_http_tracing, CommercialEventDimensions,
    CommercialEventKind, HttpMetricDimensions, HttpMetricsRegistry, PaymentMetricDimensions,
    ProviderExecutionMetricDimensions, TelemetryCardinalityLimits,
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
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"GET\",route=\"/health\",status=\"200\",tenant=\"none\",model=\"none\",provider=\"none\",billing_mode=\"none\",retry_outcome=\"none\",failover_outcome=\"none\",payment_outcome=\"none\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"200\",tenant=\"none\",model=\"none\",provider=\"none\",billing_mode=\"none\",retry_outcome=\"none\",failover_outcome=\"none\",payment_outcome=\"none\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"429\",tenant=\"none\",model=\"none\",provider=\"none\",billing_mode=\"none\",retry_outcome=\"none\",failover_outcome=\"none\",payment_outcome=\"none\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_request_duration_ms_sum{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"200\",tenant=\"none\",model=\"none\",provider=\"none\",billing_mode=\"none\",retry_outcome=\"none\",failover_outcome=\"none\",payment_outcome=\"none\"} 48"
    ));
    assert!(output.contains(
        "sdkwork_http_request_duration_ms_count{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"429\",tenant=\"none\",model=\"none\",provider=\"none\",billing_mode=\"none\",retry_outcome=\"none\",failover_outcome=\"none\",payment_outcome=\"none\"} 1"
    ));
}

#[test]
fn renders_commercial_metrics_with_dimensions_histograms_and_cardinality_controls() {
    let registry = HttpMetricsRegistry::with_cardinality_limits(
        "gateway-service",
        TelemetryCardinalityLimits::default()
            .with_model_limit(1)
            .with_tenant_limit(1)
            .with_provider_limit(1),
    );

    registry.record_with_dimensions(
        "POST",
        "/v1/chat/completions",
        200,
        48,
        HttpMetricDimensions::default()
            .with_tenant("tenant-alpha")
            .with_model("gpt-4.1")
            .with_provider("provider-secondary")
            .with_billing_mode("canonical_account")
            .with_retry_outcome("retried")
            .with_failover_outcome("activated")
            .with_payment_outcome("none"),
    );
    registry.record_with_dimensions(
        "POST",
        "/v1/chat/completions",
        502,
        75,
        HttpMetricDimensions::default()
            .with_tenant("tenant-beta")
            .with_model("gpt-4.1-mini")
            .with_provider("provider-tertiary")
            .with_billing_mode("canonical_account")
            .with_retry_outcome("exhausted")
            .with_failover_outcome("not_available")
            .with_payment_outcome("none"),
    );
    registry.record_provider_execution(
        37,
        ProviderExecutionMetricDimensions::default()
            .with_route("/v1/chat/completions")
            .with_tenant("tenant-alpha")
            .with_model("gpt-4.1")
            .with_provider("provider-secondary")
            .with_billing_mode("canonical_account")
            .with_retry_outcome("retried")
            .with_failover_outcome("activated")
            .with_result("succeeded"),
    );

    let output = registry.render_prometheus();

    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"200\",tenant=\"tenant-alpha\",model=\"gpt-4.1\",provider=\"provider-secondary\",billing_mode=\"canonical_account\",retry_outcome=\"retried\",failover_outcome=\"activated\",payment_outcome=\"none\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_requests_total{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"502\",tenant=\"other\",model=\"other\",provider=\"other\",billing_mode=\"canonical_account\",retry_outcome=\"exhausted\",failover_outcome=\"not_available\",payment_outcome=\"none\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_http_request_duration_ms_bucket{service=\"gateway-service\",method=\"POST\",route=\"/v1/chat/completions\",status=\"200\",tenant=\"tenant-alpha\",model=\"gpt-4.1\",provider=\"provider-secondary\",billing_mode=\"canonical_account\",retry_outcome=\"retried\",failover_outcome=\"activated\",payment_outcome=\"none\",le=\"50\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_provider_execution_total{service=\"gateway-service\",route=\"/v1/chat/completions\",tenant=\"tenant-alpha\",model=\"gpt-4.1\",provider=\"provider-secondary\",billing_mode=\"canonical_account\",retry_outcome=\"retried\",failover_outcome=\"activated\",result=\"succeeded\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_provider_execution_duration_ms_bucket{service=\"gateway-service\",route=\"/v1/chat/completions\",tenant=\"tenant-alpha\",model=\"gpt-4.1\",provider=\"provider-secondary\",billing_mode=\"canonical_account\",retry_outcome=\"retried\",failover_outcome=\"activated\",result=\"succeeded\",le=\"50\"} 1"
    ));
}

#[test]
fn renders_payment_outcome_and_structured_event_counters() {
    let registry = HttpMetricsRegistry::new("portal");
    registry.record_payment_callback(
        PaymentMetricDimensions::default()
            .with_provider("stripe")
            .with_tenant("project-alpha")
            .with_payment_outcome("settled"),
    );
    registry.record_payment_callback(
        PaymentMetricDimensions::default()
            .with_provider("stripe")
            .with_tenant("project-alpha")
            .with_payment_outcome("duplicate"),
    );
    registry.record_commercial_event(
        CommercialEventKind::CallbackReplay,
        CommercialEventDimensions::default()
            .with_route("/portal/internal/payments/stripe/webhook")
            .with_tenant("project-alpha")
            .with_provider("stripe")
            .with_payment_outcome("duplicate")
            .with_result("ignored"),
    );
    registry.record_commercial_event(
        CommercialEventKind::FailoverActivation,
        CommercialEventDimensions::default()
            .with_route("/v1/chat/completions")
            .with_tenant("tenant-alpha")
            .with_provider("provider-primary")
            .with_model("gpt-4.1")
            .with_result("activated"),
    );

    let output = registry.render_prometheus();

    assert!(output.contains(
        "sdkwork_payment_callbacks_total{service=\"portal\",provider=\"stripe\",tenant=\"project-alpha\",payment_outcome=\"settled\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_payment_callbacks_total{service=\"portal\",provider=\"stripe\",tenant=\"project-alpha\",payment_outcome=\"duplicate\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_commercial_events_total{service=\"portal\",event_kind=\"callback_replay\",route=\"/portal/internal/payments/stripe/webhook\",tenant=\"project-alpha\",provider=\"stripe\",model=\"none\",payment_outcome=\"duplicate\",result=\"ignored\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_commercial_events_total{service=\"portal\",event_kind=\"failover_activation\",route=\"/v1/chat/completions\",tenant=\"tenant-alpha\",provider=\"provider-primary\",model=\"gpt-4.1\",payment_outcome=\"none\",result=\"activated\"} 1"
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
