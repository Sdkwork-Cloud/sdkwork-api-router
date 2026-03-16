use sdkwork_api_domain_billing::QuotaPolicy;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_secret_core::encrypt;
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};

#[tokio::test]
async fn postgres_store_persists_catalog_and_credentials_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let channel = store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    assert_eq!(channel.id, "openai");

    let provider = store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    assert_eq!(provider.adapter_kind, "openai");

    let model = store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();
    assert_eq!(model.external_name, "gpt-4.1");

    let credential = UpstreamCredential::new("tenant-1", "provider-openai-official", "cred-openai");
    let envelope = encrypt("local-dev-master-key", "sk-upstream-openai").unwrap();
    store
        .insert_encrypted_credential(&credential, &envelope)
        .await
        .unwrap();

    let stored = store
        .find_credential_envelope("tenant-1", "provider-openai-official", "cred-openai")
        .await
        .unwrap()
        .expect("credential envelope");
    assert_eq!(stored, envelope);

    let models = store.list_models().await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].provider_id, "provider-openai-official");
}

#[tokio::test]
async fn postgres_store_persists_routing_policies_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official");

    store.insert_routing_policy(&policy).await.unwrap();

    let stored = store
        .list_routing_policies()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.policy_id == "policy-gpt-4-1")
        .expect("routing policy");
    assert_eq!(
        stored.ordered_provider_ids,
        vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ]
    );
    assert_eq!(
        stored.default_provider_id.as_deref(),
        Some("provider-openai-official")
    );
    assert_eq!(stored.strategy, RoutingStrategy::WeightedRandom);
}

#[tokio::test]
async fn postgres_store_round_trips_slo_policy_fields_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-slo", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_cost(0.35)
        .with_max_latency_ms(300)
        .with_require_healthy(true)
        .with_ordered_provider_ids(vec!["provider-openrouter".to_owned()]);

    store.insert_routing_policy(&policy).await.unwrap();

    let stored = store
        .list_routing_policies()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.policy_id == "policy-slo")
        .expect("routing policy");
    assert_eq!(stored.strategy, RoutingStrategy::SloAware);
    assert_eq!(stored.max_cost, Some(0.35));
    assert_eq!(stored.max_latency_ms, Some(300));
    assert!(stored.require_healthy);
}

#[tokio::test]
async fn postgres_store_persists_routing_decision_logs_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-postgres",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-openai-official",
        "slo_aware",
        1234,
    )
    .with_tenant_id("tenant-1")
    .with_project_id("project-1")
    .with_selection_reason(
        "selected provider-openai-official as the top-ranked SLO-compliant candidate",
    )
    .with_slo_state(true, false)
    .with_assessments(vec![RoutingCandidateAssessment::new(
        "provider-openai-official",
    )
    .with_slo_eligible(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert!(logs.iter().any(|entry| entry == &log));
}

#[tokio::test]
async fn postgres_store_round_trips_requested_region_in_routing_decision_logs_when_url_is_provided()
{
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-postgres-region",
        RoutingDecisionSource::AdminSimulation,
        "chat_completion",
        "gpt-4.1",
        "provider-us-east",
        "geo_affinity",
        4321,
    )
    .with_requested_region("us-east")
    .with_assessments(vec![RoutingCandidateAssessment::new("provider-us-east")
        .with_region("us-east")
        .with_region_match(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert!(logs.iter().any(|entry| entry == &log));
}

#[tokio::test]
async fn postgres_store_persists_provider_health_snapshots_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let snapshot = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1234,
    )
    .with_running(true)
    .with_healthy(true)
    .with_message("healthy");

    store
        .insert_provider_health_snapshot(&snapshot)
        .await
        .unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert!(snapshots.iter().any(|entry| entry == &snapshot));
}

#[tokio::test]
async fn postgres_store_persists_quota_policies_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = QuotaPolicy::new("quota-project-1", "project-1", 1_000).with_enabled(true);

    store.insert_quota_policy(&policy).await.unwrap();

    let policies = store.list_quota_policies().await.unwrap();
    assert!(policies.iter().any(|entry| entry == &policy));
}
