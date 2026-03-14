use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
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
