use async_trait::async_trait;
use sdkwork_api_app_billing::{
    book_usage_cost, check_quota, create_billing_event, create_quota_policy,
    list_billing_events, list_ledger_entries, persist_billing_event, persist_ledger_entry,
    persist_quota_policy, BillingQuotaStore, CreateBillingEventInput,
};
use sdkwork_api_domain_billing::{
    BillingAccountingMode, BillingEventRecord, LedgerEntry, QuotaPolicy,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use std::sync::Mutex;

#[test]
fn booking_usage_creates_ledger_entry() {
    let ledger = book_usage_cost("project-1", 100, 0.25).unwrap();
    assert_eq!(ledger.project_id, "project-1");
}

#[test]
fn creating_billing_event_captures_group_and_multimodal_dimensions() {
    let event = create_billing_event(CreateBillingEventInput {
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
        modality: "multimodal",
        api_key_hash: Some("key-live"),
        channel_id: Some("openai"),
        reference_id: Some("resp_123"),
        latency_ms: Some(850),
        units: 240,
        request_count: 1,
        input_tokens: 120,
        output_tokens: 80,
        total_tokens: 200,
        cache_read_tokens: 30,
        cache_write_tokens: 10,
        image_count: 2,
        audio_seconds: 3.5,
        video_seconds: 0.0,
        music_seconds: 0.0,
        upstream_cost: 0.42,
        customer_charge: 0.89,
        applied_routing_profile_id: Some("route-profile-1"),
        compiled_routing_snapshot_id: Some("snapshot-1"),
        fallback_reason: Some("latency_guardrail"),
        created_at_ms: 1_717_171_717,
    })
    .unwrap();

    assert_eq!(
        event,
        BillingEventRecord::new(
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
        .with_request_count(1)
        .with_token_usage(120, 80, 200)
        .with_cache_token_usage(30, 10)
        .with_media_usage(2, 3.5, 0.0, 0.0)
        .with_financials(0.42, 0.89)
        .with_routing_evidence(
            Some("route-profile-1"),
            Some("snapshot-1"),
            Some("latency_guardrail"),
        )
    );
}

#[tokio::test]
async fn persisted_billing_events_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let event = create_billing_event(CreateBillingEventInput {
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
        reference_id: Some("resp_123"),
        latency_ms: Some(850),
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
        created_at_ms: 1_717_171_717,
    })
    .unwrap();

    persist_billing_event(&store, &event).await.unwrap();

    let events = list_billing_events(&store).await.unwrap();
    assert_eq!(events, vec![event]);
}

#[tokio::test]
async fn persisted_ledger_can_be_listed() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_ledger_entry(&store, "project-1", 100, 0.25)
        .await
        .unwrap();

    let ledger = list_ledger_entries(&store).await.unwrap();
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger[0].amount, 0.25);
}

#[tokio::test]
async fn quota_evaluation_rejects_requests_past_configured_limit() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = create_quota_policy("quota-project-1", "project-1", 100, true).unwrap();
    persist_quota_policy(&store, &policy).await.unwrap();
    persist_ledger_entry(&store, "project-1", 70, 0.25)
        .await
        .unwrap();

    let evaluation = check_quota(&store, "project-1", 40).await.unwrap();
    assert!(!evaluation.allowed);
    assert_eq!(evaluation.policy_id.as_deref(), Some("quota-project-1"));
    assert_eq!(evaluation.used_units, 70);
    assert_eq!(evaluation.limit_units, Some(100));
}

#[tokio::test]
async fn quota_evaluation_uses_project_scoped_reads_only() {
    let store = RecordingQuotaStore::new(
        vec![
            LedgerEntry::new("project-1", 70, 0.25),
            LedgerEntry::new("project-2", 999, 9.99),
        ],
        vec![
            QuotaPolicy::new("quota-project-1", "project-1", 100).with_enabled(true),
            QuotaPolicy::new("quota-project-2", "project-2", 5).with_enabled(true),
        ],
    );

    let evaluation = check_quota(&store, "project-1", 40).await.unwrap();

    assert!(!evaluation.allowed);
    assert_eq!(evaluation.policy_id.as_deref(), Some("quota-project-1"));
    assert_eq!(evaluation.used_units, 70);
    assert_eq!(evaluation.limit_units, Some(100));
    assert_eq!(
        store.last_ledger_project.lock().unwrap().as_deref(),
        Some("project-1")
    );
    assert_eq!(
        store.last_policy_project.lock().unwrap().as_deref(),
        Some("project-1")
    );
}

struct RecordingQuotaStore {
    ledger_entries: Vec<LedgerEntry>,
    quota_policies: Vec<QuotaPolicy>,
    last_ledger_project: Mutex<Option<String>>,
    last_policy_project: Mutex<Option<String>>,
}

impl RecordingQuotaStore {
    fn new(ledger_entries: Vec<LedgerEntry>, quota_policies: Vec<QuotaPolicy>) -> Self {
        Self {
            ledger_entries,
            quota_policies,
            last_ledger_project: Mutex::new(None),
            last_policy_project: Mutex::new(None),
        }
    }
}

#[async_trait]
impl BillingQuotaStore for RecordingQuotaStore {
    async fn list_ledger_entries_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<LedgerEntry>> {
        *self.last_ledger_project.lock().unwrap() = Some(project_id.to_owned());
        Ok(self
            .ledger_entries
            .iter()
            .filter(|entry| entry.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn list_quota_policies_for_project(
        &self,
        project_id: &str,
    ) -> anyhow::Result<Vec<QuotaPolicy>> {
        *self.last_policy_project.lock().unwrap() = Some(project_id.to_owned());
        Ok(self
            .quota_policies
            .iter()
            .filter(|policy| policy.project_id == project_id)
            .cloned()
            .collect())
    }
}
