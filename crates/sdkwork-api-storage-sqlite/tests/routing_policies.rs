use sdkwork_api_domain_identity::ApiKeyGroupRecord;
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProviderHealthSnapshot, RoutingCandidateAssessment,
    RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy, RoutingProfileRecord,
    RoutingStrategy,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_routing_policies_with_provider_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official");

    store.insert_routing_policy(&policy).await.unwrap();

    let policies = store.list_routing_policies().await.unwrap();
    assert_eq!(policies, vec![policy]);
    assert_eq!(policies[0].strategy, RoutingStrategy::WeightedRandom);
}

#[tokio::test]
async fn sqlite_store_round_trips_slo_policy_fields() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-slo", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_cost(0.30)
        .with_max_latency_ms(250)
        .with_require_healthy(true)
        .with_ordered_provider_ids(vec!["provider-a".to_owned()])
        .with_default_provider_id("provider-a");

    store.insert_routing_policy(&policy).await.unwrap();

    let policies = store.list_routing_policies().await.unwrap();
    assert_eq!(policies, vec![policy]);
    assert_eq!(policies[0].strategy, RoutingStrategy::SloAware);
    assert_eq!(policies[0].max_cost, Some(0.30));
    assert_eq!(policies[0].max_latency_ms, Some(250));
    assert!(policies[0].require_healthy);
}

#[tokio::test]
async fn sqlite_store_persists_routing_profiles_with_provider_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let profile = RoutingProfileRecord::new(
        "profile-priority",
        "tenant-1",
        "project-1",
        "Priority Live",
        "priority-live",
    )
    .with_description("Prefer OpenRouter first")
    .with_strategy(RoutingStrategy::GeoAffinity)
    .with_ordered_provider_ids(vec![
        "provider-openrouter".to_owned(),
        "provider-openai-official".to_owned(),
    ])
    .with_default_provider_id("provider-openai-official")
    .with_max_cost(0.25)
    .with_max_latency_ms(200)
    .with_require_healthy(true)
    .with_preferred_region("us-east")
    .with_created_at_ms(123)
    .with_updated_at_ms(456);

    store.insert_routing_profile(&profile).await.unwrap();

    let found = store
        .find_routing_profile("profile-priority")
        .await
        .unwrap()
        .expect("routing profile");
    assert_eq!(found, profile);

    let profiles = store.list_routing_profiles().await.unwrap();
    assert_eq!(profiles, vec![profile]);
}

#[tokio::test]
async fn sqlite_store_persists_compiled_routing_snapshots() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let snapshot = CompiledRoutingSnapshotRecord::new(
        "snapshot-tenant-1-project-1-group-live-chat_completion-gpt-4-1",
        "chat_completion",
        "gpt-4.1",
    )
    .with_tenant_id("tenant-1")
    .with_project_id("project-1")
    .with_api_key_group_id("group-live")
    .with_matched_policy_id("policy-gpt-4-1")
    .with_project_routing_preferences_project_id("project-1")
    .with_applied_routing_profile_id("profile-priority")
    .with_strategy("geo_affinity")
    .with_ordered_provider_ids(vec![
        "provider-openrouter".to_owned(),
        "provider-openai-official".to_owned(),
    ])
    .with_default_provider_id("provider-openrouter")
    .with_max_cost(0.25)
    .with_max_latency_ms(200)
    .with_require_healthy(true)
    .with_preferred_region("us-east")
    .with_created_at_ms(123)
    .with_updated_at_ms(456);

    store
        .insert_compiled_routing_snapshot(&snapshot)
        .await
        .unwrap();

    let snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    assert_eq!(snapshots, vec![snapshot]);
}

#[tokio::test]
async fn sqlite_store_persists_routing_decision_logs_newest_first() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let older = RoutingDecisionLog::new(
        "decision-older",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-a",
        "slo_aware",
        100,
    )
    .with_selection_reason("older decision")
    .with_slo_state(true, true)
    .with_assessments(vec![
        RoutingCandidateAssessment::new("provider-a").with_slo_eligible(false)
    ]);
    let newer = RoutingDecisionLog::new(
        "decision-newer",
        RoutingDecisionSource::AdminSimulation,
        "responses",
        "gpt-4.1",
        "provider-b",
        "weighted_random",
        200,
    )
    .with_selection_seed(7)
    .with_selection_reason("newer decision")
    .with_slo_state(false, false)
    .with_assessments(vec![RoutingCandidateAssessment::new("provider-b")]);

    store.insert_routing_decision_log(&older).await.unwrap();
    store.insert_routing_decision_log(&newer).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].decision_id, "decision-newer");
    assert_eq!(
        logs[0].decision_source,
        RoutingDecisionSource::AdminSimulation
    );
    assert_eq!(logs[0].selection_seed, Some(7));
    assert_eq!(logs[1].decision_id, "decision-older");
}

#[tokio::test]
async fn sqlite_store_round_trips_requested_region_in_routing_decision_logs() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-region",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-us-east",
        "geo_affinity",
        300,
    )
    .with_requested_region("us-east")
    .with_assessments(vec![RoutingCandidateAssessment::new("provider-us-east")
        .with_region("us-east")
        .with_region_match(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].requested_region.as_deref(), Some("us-east"));
    assert_eq!(logs[0].assessments[0].region.as_deref(), Some("us-east"));
    assert_eq!(logs[0].assessments[0].region_match, Some(true));
}

#[tokio::test]
async fn sqlite_store_round_trips_api_key_group_id_in_routing_decision_logs() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-group",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-openai-official",
        "deterministic_priority",
        400,
    )
    .with_tenant_id("tenant-1")
    .with_project_id("project-1")
    .with_api_key_group_id("group-live")
    .with_applied_routing_profile_id("profile-priority")
    .with_compiled_routing_snapshot_id("snapshot-live")
    .with_fallback_reason("no fallback applied");

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].api_key_group_id.as_deref(), Some("group-live"));
    assert_eq!(
        logs[0].applied_routing_profile_id.as_deref(),
        Some("profile-priority")
    );
    assert_eq!(
        logs[0].compiled_routing_snapshot_id.as_deref(),
        Some("snapshot-live")
    );
    assert_eq!(
        logs[0].fallback_reason.as_deref(),
        Some("no fallback applied")
    );
}

#[tokio::test]
async fn sqlite_store_round_trips_default_routing_profile_id_in_api_key_groups() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = ApiKeyGroupRecord::new(
        "group-live",
        "tenant-1",
        "project-1",
        "live",
        "Production Keys",
        "production-keys",
    )
    .with_default_routing_profile_id("profile-priority")
    .with_created_at_ms(123)
    .with_updated_at_ms(456);

    store.insert_api_key_group(&group).await.unwrap();

    let found = store
        .find_api_key_group("group-live")
        .await
        .unwrap()
        .expect("api key group");
    assert_eq!(
        found.default_routing_profile_id.as_deref(),
        Some("profile-priority")
    );
}

#[tokio::test]
async fn sqlite_store_round_trips_default_accounting_mode_in_api_key_groups() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let group = ApiKeyGroupRecord::new(
        "group-byok",
        "tenant-1",
        "project-1",
        "live",
        "BYOK Keys",
        "byok-keys",
    )
    .with_default_accounting_mode("byok")
    .with_created_at_ms(123)
    .with_updated_at_ms(456);

    store.insert_api_key_group(&group).await.unwrap();

    let found = store
        .find_api_key_group("group-byok")
        .await
        .unwrap()
        .expect("api key group");
    assert_eq!(found.default_accounting_mode.as_deref(), Some("byok"));
}

#[tokio::test]
async fn sqlite_store_persists_provider_health_snapshots_newest_first() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let older = ProviderHealthSnapshot::new("provider-a", "sdkwork.provider.a", "builtin", 100)
        .with_healthy(false)
        .with_running(true);
    let newer = ProviderHealthSnapshot::new("provider-a", "sdkwork.provider.a", "builtin", 200)
        .with_healthy(true)
        .with_running(true)
        .with_message("recovered");

    store.insert_provider_health_snapshot(&older).await.unwrap();
    store.insert_provider_health_snapshot(&newer).await.unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].observed_at_ms, 200);
    assert_eq!(snapshots[0].message.as_deref(), Some("recovered"));
}
