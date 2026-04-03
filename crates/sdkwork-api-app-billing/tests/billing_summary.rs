use sdkwork_api_app_billing::{create_billing_event, summarize_billing_events, summarize_billing_snapshot, CreateBillingEventInput};
use sdkwork_api_domain_billing::{BillingAccountingMode, LedgerEntry, QuotaPolicy};

#[test]
fn summarizes_billing_posture_and_quota_exhaustion_by_project() {
    let entries = vec![
        LedgerEntry::new("project-1", 40, 0.40),
        LedgerEntry::new("project-1", 70, 0.70),
        LedgerEntry::new("project-2", 10, 0.10),
    ];
    let policies = vec![
        QuotaPolicy::new("quota-project-1", "project-1", 100),
        QuotaPolicy::new("quota-project-2", "project-2", 500).with_enabled(false),
        QuotaPolicy::new("quota-project-3", "project-3", 200),
    ];

    let summary = summarize_billing_snapshot(&entries, &policies);

    assert_eq!(summary.total_entries, 3);
    assert_eq!(summary.project_count, 3);
    assert_eq!(summary.total_units, 120);
    assert!((summary.total_amount - 1.20).abs() < 1e-9);
    assert_eq!(summary.active_quota_policy_count, 2);
    assert_eq!(summary.exhausted_project_count, 1);

    assert_eq!(summary.projects.len(), 3);

    assert_eq!(summary.projects[0].project_id, "project-1");
    assert_eq!(summary.projects[0].entry_count, 2);
    assert_eq!(summary.projects[0].used_units, 110);
    assert!((summary.projects[0].booked_amount - 1.10).abs() < 1e-9);
    assert_eq!(
        summary.projects[0].quota_policy_id.as_deref(),
        Some("quota-project-1")
    );
    assert_eq!(summary.projects[0].quota_limit_units, Some(100));
    assert_eq!(summary.projects[0].remaining_units, Some(0));
    assert!(summary.projects[0].exhausted);

    assert_eq!(summary.projects[1].project_id, "project-3");
    assert_eq!(summary.projects[1].entry_count, 0);
    assert_eq!(summary.projects[1].used_units, 0);
    assert!((summary.projects[1].booked_amount - 0.0).abs() < 1e-9);
    assert_eq!(
        summary.projects[1].quota_policy_id.as_deref(),
        Some("quota-project-3")
    );
    assert_eq!(summary.projects[1].quota_limit_units, Some(200));
    assert_eq!(summary.projects[1].remaining_units, Some(200));
    assert!(!summary.projects[1].exhausted);

    assert_eq!(summary.projects[2].project_id, "project-2");
    assert_eq!(summary.projects[2].entry_count, 1);
    assert_eq!(summary.projects[2].used_units, 10);
    assert!((summary.projects[2].booked_amount - 0.10).abs() < 1e-9);
    assert_eq!(summary.projects[2].quota_policy_id, None);
    assert_eq!(summary.projects[2].quota_limit_units, None);
    assert_eq!(summary.projects[2].remaining_units, None);
    assert!(!summary.projects[2].exhausted);
}

#[test]
fn summarizes_billing_events_by_project_group_capability_and_accounting_mode() {
    let events = vec![
        create_billing_event(CreateBillingEventInput {
            event_id: "evt_1",
            tenant_id: "tenant-1",
            project_id: "project-1",
            api_key_group_id: Some("group-blue"),
            capability: "responses",
            route_key: "gpt-4.1",
            usage_model: "gpt-4.1",
            provider_id: "provider-openrouter",
            accounting_mode: BillingAccountingMode::PlatformCredit,
            operation_kind: "responses.create",
            modality: "text",
            api_key_hash: Some("key-live"),
            channel_id: Some("openai"),
            reference_id: Some("resp_1"),
            latency_ms: Some(650),
            units: 240,
            request_count: 1,
            input_tokens: 120,
            output_tokens: 80,
            total_tokens: 200,
            cache_read_tokens: 20,
            cache_write_tokens: 10,
            image_count: 0,
            audio_seconds: 0.0,
            video_seconds: 0.0,
            music_seconds: 0.0,
            upstream_cost: 0.42,
            customer_charge: 0.89,
            applied_routing_profile_id: Some("route-profile-1"),
            compiled_routing_snapshot_id: Some("snapshot-1"),
            fallback_reason: None,
            created_at_ms: 100,
        })
        .unwrap(),
        create_billing_event(CreateBillingEventInput {
            event_id: "evt_2",
            tenant_id: "tenant-1",
            project_id: "project-1",
            api_key_group_id: Some("group-blue"),
            capability: "images",
            route_key: "gpt-image-1",
            usage_model: "gpt-image-1",
            provider_id: "provider-openai",
            accounting_mode: BillingAccountingMode::PlatformCredit,
            operation_kind: "images.generate",
            modality: "image",
            api_key_hash: Some("key-live"),
            channel_id: Some("openai"),
            reference_id: Some("img_1"),
            latency_ms: Some(900),
            units: 40,
            request_count: 1,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            image_count: 2,
            audio_seconds: 0.0,
            video_seconds: 0.0,
            music_seconds: 0.0,
            upstream_cost: 0.80,
            customer_charge: 1.50,
            applied_routing_profile_id: Some("route-profile-1"),
            compiled_routing_snapshot_id: Some("snapshot-2"),
            fallback_reason: Some("provider_capacity"),
            created_at_ms: 200,
        })
        .unwrap(),
        create_billing_event(CreateBillingEventInput {
            event_id: "evt_3",
            tenant_id: "tenant-1",
            project_id: "project-2",
            api_key_group_id: None,
            capability: "audio",
            route_key: "gpt-4o-mini-transcribe",
            usage_model: "gpt-4o-mini-transcribe",
            provider_id: "provider-byok",
            accounting_mode: BillingAccountingMode::Byok,
            operation_kind: "audio.transcriptions.create",
            modality: "audio",
            api_key_hash: Some("key-byok"),
            channel_id: Some("openai"),
            reference_id: Some("aud_1"),
            latency_ms: Some(1200),
            units: 60,
            request_count: 2,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            image_count: 0,
            audio_seconds: 35.0,
            video_seconds: 0.0,
            music_seconds: 0.0,
            upstream_cost: 0.0,
            customer_charge: 0.0,
            applied_routing_profile_id: None,
            compiled_routing_snapshot_id: None,
            fallback_reason: None,
            created_at_ms: 300,
        })
        .unwrap(),
    ];

    let summary = summarize_billing_events(&events);

    assert_eq!(summary.total_events, 3);
    assert_eq!(summary.project_count, 2);
    assert_eq!(summary.group_count, 2);
    assert_eq!(summary.capability_count, 3);
    assert_eq!(summary.total_request_count, 4);
    assert_eq!(summary.total_units, 340);
    assert_eq!(summary.total_input_tokens, 120);
    assert_eq!(summary.total_output_tokens, 80);
    assert_eq!(summary.total_tokens, 200);
    assert_eq!(summary.total_image_count, 2);
    assert!((summary.total_audio_seconds - 35.0).abs() < 1e-9);
    assert!((summary.total_upstream_cost - 1.22).abs() < 1e-9);
    assert!((summary.total_customer_charge - 2.39).abs() < 1e-9);

    assert_eq!(summary.projects.len(), 2);
    assert_eq!(summary.projects[0].project_id, "project-1");
    assert_eq!(summary.projects[0].event_count, 2);
    assert_eq!(summary.projects[0].request_count, 2);
    assert!((summary.projects[0].total_customer_charge - 2.39).abs() < 1e-9);
    assert_eq!(summary.projects[1].project_id, "project-2");
    assert_eq!(summary.projects[1].event_count, 1);
    assert_eq!(summary.projects[1].request_count, 2);

    assert_eq!(summary.groups.len(), 2);
    assert_eq!(summary.groups[0].api_key_group_id.as_deref(), Some("group-blue"));
    assert_eq!(summary.groups[0].event_count, 2);
    assert_eq!(summary.groups[0].project_count, 1);
    assert!((summary.groups[0].total_customer_charge - 2.39).abs() < 1e-9);
    assert_eq!(summary.groups[1].api_key_group_id, None);
    assert_eq!(summary.groups[1].event_count, 1);

    assert_eq!(summary.capabilities.len(), 3);
    assert_eq!(summary.capabilities[0].capability, "audio");
    assert_eq!(summary.capabilities[0].request_count, 2);
    assert_eq!(summary.capabilities[1].capability, "images");
    assert_eq!(summary.capabilities[1].image_count, 2);
    assert_eq!(summary.capabilities[2].capability, "responses");
    assert_eq!(summary.capabilities[2].total_tokens, 200);

    assert_eq!(summary.accounting_modes.len(), 2);
    assert_eq!(
        summary.accounting_modes[0].accounting_mode,
        BillingAccountingMode::PlatformCredit
    );
    assert_eq!(summary.accounting_modes[0].event_count, 2);
    assert_eq!(
        summary.accounting_modes[1].accounting_mode,
        BillingAccountingMode::Byok
    );
    assert_eq!(summary.accounting_modes[1].event_count, 1);
}
