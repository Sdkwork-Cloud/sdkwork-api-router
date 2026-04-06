use axum::body::Body;
use axum::http::Request;
use sdkwork_api_config::HttpExposureConfig;
use serial_test::serial;
use tower::ServiceExt;

#[serial(browser_cors_env)]
#[tokio::test]
async fn gateway_chat_completions_preflight_includes_cors_headers() {
    let app = sdkwork_api_interface_http::gateway_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/chat/completions")
                .header("origin", "http://localhost:5174")
                .header("access-control-request-method", "POST")
                .header(
                    "access-control-request-headers",
                    "content-type,authorization",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("http://localhost:5174")
    );
    assert!(response
        .headers()
        .get("access-control-allow-methods")
        .is_some());
    assert!(response
        .headers()
        .get("access-control-allow-headers")
        .is_some());
}

#[serial(browser_cors_env)]
#[tokio::test]
async fn gateway_chat_completions_preflight_uses_configured_origin_allowlist() {
    let previous = std::env::var("SDKWORK_BROWSER_ALLOWED_ORIGINS").ok();
    std::env::set_var(
        "SDKWORK_BROWSER_ALLOWED_ORIGINS",
        "https://console.example.com;https://portal.example.com",
    );

    let app = sdkwork_api_interface_http::gateway_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/chat/completions")
                .header("origin", "https://console.example.com")
                .header("access-control-request-method", "POST")
                .header(
                    "access-control-request-headers",
                    "content-type,authorization",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    match previous {
        Some(value) => std::env::set_var("SDKWORK_BROWSER_ALLOWED_ORIGINS", value),
        None => std::env::remove_var("SDKWORK_BROWSER_ALLOWED_ORIGINS"),
    }

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("https://console.example.com")
    );
}

#[tokio::test]
async fn gateway_chat_completions_preflight_ignores_invalid_origin_entries() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config_and_http_exposure(
        Default::default(),
        HttpExposureConfig {
            metrics_bearer_token: "test-metrics-token".to_owned(),
            browser_allowed_origins: vec![
                "https://console.example.com".to_owned(),
                "https://bad\norigin.example.com".to_owned(),
            ],
        },
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/chat/completions")
                .header("origin", "https://console.example.com")
                .header("access-control-request-method", "POST")
                .header(
                    "access-control-request-headers",
                    "content-type,authorization",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("https://console.example.com")
    );
}
