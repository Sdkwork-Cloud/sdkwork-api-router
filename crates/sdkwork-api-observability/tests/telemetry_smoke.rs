use sdkwork_api_observability::HttpMetricsRegistry;

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
