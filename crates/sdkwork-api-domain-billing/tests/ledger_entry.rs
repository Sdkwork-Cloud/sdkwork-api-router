use sdkwork_api_domain_billing::LedgerEntry;
use sdkwork_api_domain_billing::{BillingAccountingMode, BillingEventRecord};

#[test]
fn ledger_entry_tracks_units() {
    let entry = LedgerEntry::new("project-1", 100, 0.25);
    assert_eq!(entry.units, 100);
}

#[test]
fn billing_event_record_tracks_multimodal_dimensions_and_routing_evidence() {
    let event = BillingEventRecord::new(
        "evt_1",
        "tenant-1",
        "project-1",
        "responses",
        "gpt-4.1",
        "gpt-4.1",
        "provider-openrouter",
        BillingAccountingMode::PlatformCredit,
        1_717_171_717,
    )
    .with_api_key_group_id("group-blue")
    .with_operation("responses.create", "multimodal")
    .with_request_facts(
        Some("key-live"),
        Some("openai"),
        Some("resp_123"),
        Some(850),
    )
    .with_units(240)
    .with_token_usage(120, 80, 200)
    .with_cache_token_usage(30, 10)
    .with_media_usage(2, 3.5, 0.0, 12.0)
    .with_financials(0.42, 0.89)
    .with_routing_evidence(
        Some("route-profile-1"),
        Some("snapshot-1"),
        Some("latency_guardrail"),
    );

    assert_eq!(event.event_id, "evt_1");
    assert_eq!(event.tenant_id, "tenant-1");
    assert_eq!(event.project_id, "project-1");
    assert_eq!(event.api_key_group_id.as_deref(), Some("group-blue"));
    assert_eq!(event.capability, "responses");
    assert_eq!(event.route_key, "gpt-4.1");
    assert_eq!(event.usage_model, "gpt-4.1");
    assert_eq!(event.provider_id, "provider-openrouter");
    assert_eq!(event.accounting_mode, BillingAccountingMode::PlatformCredit);
    assert_eq!(event.operation_kind, "responses.create");
    assert_eq!(event.modality, "multimodal");
    assert_eq!(event.api_key_hash.as_deref(), Some("key-live"));
    assert_eq!(event.channel_id.as_deref(), Some("openai"));
    assert_eq!(event.reference_id.as_deref(), Some("resp_123"));
    assert_eq!(event.latency_ms, Some(850));
    assert_eq!(event.units, 240);
    assert_eq!(event.request_count, 1);
    assert_eq!(event.input_tokens, 120);
    assert_eq!(event.output_tokens, 80);
    assert_eq!(event.total_tokens, 200);
    assert_eq!(event.cache_read_tokens, 30);
    assert_eq!(event.cache_write_tokens, 10);
    assert_eq!(event.image_count, 2);
    assert!((event.audio_seconds - 3.5).abs() < 1e-9);
    assert!((event.video_seconds - 0.0).abs() < 1e-9);
    assert!((event.music_seconds - 12.0).abs() < 1e-9);
    assert!((event.upstream_cost - 0.42).abs() < 1e-9);
    assert!((event.customer_charge - 0.89).abs() < 1e-9);
    assert_eq!(
        event.applied_routing_profile_id.as_deref(),
        Some("route-profile-1")
    );
    assert_eq!(
        event.compiled_routing_snapshot_id.as_deref(),
        Some("snapshot-1")
    );
    assert_eq!(
        event.fallback_reason.as_deref(),
        Some("latency_guardrail")
    );
    assert_eq!(event.created_at_ms, 1_717_171_717);
}
