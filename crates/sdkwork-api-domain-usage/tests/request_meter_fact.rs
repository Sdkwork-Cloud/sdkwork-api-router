use sdkwork_api_domain_usage::{
    RequestMeterFactRecord, RequestMeterMetricRecord, RequestStatus, UsageCaptureStatus,
};

#[test]
fn request_meter_fact_tracks_canonical_subject_and_pricing_refs() {
    let fact = RequestMeterFactRecord::new(
        6001,
        1001,
        2002,
        9001,
        7001,
        "api_key",
        "responses",
        "openai",
        "gpt-4.1",
        "provider-openai-official",
    )
    .with_api_key_id(Some(778899))
    .with_api_key_hash(Some("key_hash_live".to_owned()))
    .with_jwt_subject(None)
    .with_platform(Some("web".to_owned()))
    .with_owner(Some("tenant-owner".to_owned()))
    .with_request_trace_id(Some("trace-1".to_owned()))
    .with_gateway_request_ref(Some("req_1".to_owned()))
    .with_upstream_request_ref(Some("resp_1".to_owned()))
    .with_protocol_family("openai")
    .with_request_status(RequestStatus::Succeeded)
    .with_usage_capture_status(UsageCaptureStatus::Captured)
    .with_cost_pricing_plan_id(Some(9101))
    .with_retail_pricing_plan_id(Some(9102))
    .with_estimated_credit_hold(24.0)
    .with_actual_credit_charge(Some(22.0))
    .with_actual_provider_cost(Some(8.0))
    .with_started_at_ms(1_717_171_700)
    .with_finished_at_ms(Some(1_717_171_888))
    .with_created_at_ms(1_717_171_700)
    .with_updated_at_ms(1_717_171_888);

    assert_eq!(fact.request_id, 6001);
    assert_eq!(fact.tenant_id, 1001);
    assert_eq!(fact.organization_id, 2002);
    assert_eq!(fact.user_id, 9001);
    assert_eq!(fact.account_id, 7001);
    assert_eq!(fact.api_key_id, Some(778899));
    assert_eq!(fact.api_key_hash.as_deref(), Some("key_hash_live"));
    assert_eq!(fact.auth_type, "api_key");
    assert_eq!(fact.capability_code, "responses");
    assert_eq!(fact.channel_code, "openai");
    assert_eq!(fact.model_code, "gpt-4.1");
    assert_eq!(fact.provider_code, "provider-openai-official");
    assert_eq!(fact.request_status, RequestStatus::Succeeded);
    assert_eq!(fact.usage_capture_status, UsageCaptureStatus::Captured);
    assert_eq!(fact.cost_pricing_plan_id, Some(9101));
    assert_eq!(fact.retail_pricing_plan_id, Some(9102));
}

#[test]
fn request_meter_metric_normalizes_metric_rows() {
    let metric = RequestMeterMetricRecord::new(7001001, 1001, 2002, 6001, "token.input", 128.0)
        .with_provider_field(Some("prompt_tokens".to_owned()))
        .with_source_kind("provider")
        .with_capture_stage("final")
        .with_is_billable(true)
        .with_captured_at_ms(1_717_171_889);

    assert_eq!(metric.request_metric_id, 7001001);
    assert_eq!(metric.request_id, 6001);
    assert_eq!(metric.metric_code, "token.input");
    assert_eq!(metric.quantity, 128.0);
    assert_eq!(metric.provider_field.as_deref(), Some("prompt_tokens"));
    assert_eq!(metric.source_kind, "provider");
    assert_eq!(metric.capture_stage, "final");
    assert!(metric.is_billable);
}
