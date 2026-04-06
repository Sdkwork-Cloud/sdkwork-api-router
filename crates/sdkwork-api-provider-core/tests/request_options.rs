use sdkwork_api_provider_core::{
    OpenRouterDataCollectionPolicy, OpenRouterProviderPreferences, ProviderRequestOptions,
};

#[test]
fn request_options_merge_standard_headers_and_openrouter_preferences() {
    let options = ProviderRequestOptions::new()
        .with_header("x-custom-header", "enabled")
        .with_request_timeout_ms(1_500)
        .with_deadline_at_ms(10_500)
        .with_idempotency_key("idem-123")
        .with_request_trace_id("trace-123")
        .with_openrouter_provider_preferences(
            OpenRouterProviderPreferences::new()
                .with_order(vec!["anthropic".to_owned(), "openai".to_owned()])
                .with_allow_fallbacks(false)
                .with_require_parameters(true)
                .with_data_collection(OpenRouterDataCollectionPolicy::Deny)
                .with_zero_data_retention(true),
        );

    let headers = options.resolved_headers();
    assert_eq!(
        headers.get("x-custom-header").map(String::as_str),
        Some("enabled")
    );
    assert_eq!(
        headers.get("idempotency-key").map(String::as_str),
        Some("idem-123")
    );
    assert_eq!(
        headers.get("x-request-id").map(String::as_str),
        Some("trace-123")
    );
    assert_eq!(options.request_timeout_ms(), Some(1_500));
    assert_eq!(options.deadline_at_ms(), Some(10_500));
    assert_eq!(options.effective_timeout_ms(10_000), Some(500));

    let preferences = options
        .openrouter_provider_preferences()
        .expect("openrouter preferences");
    assert_eq!(
        preferences.order(),
        &["anthropic".to_owned(), "openai".to_owned()]
    );
    assert_eq!(preferences.allow_fallbacks(), Some(false));
    assert_eq!(preferences.require_parameters(), Some(true));
    assert_eq!(
        preferences.data_collection(),
        Some(OpenRouterDataCollectionPolicy::Deny)
    );
    assert_eq!(preferences.zero_data_retention(), Some(true));
}

#[test]
fn request_options_preserve_shorter_timeout_and_detect_expired_deadline() {
    let options = ProviderRequestOptions::new()
        .with_request_timeout_ms(250)
        .with_deadline_at_ms(10_500);

    assert_eq!(options.effective_timeout_ms(10_000), Some(250));
    assert!(options.deadline_expired(10_600));
}
