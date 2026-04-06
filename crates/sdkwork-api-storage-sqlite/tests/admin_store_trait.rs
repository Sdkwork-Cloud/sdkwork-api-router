use sdkwork_api_domain_billing::{BillingAccountingMode, BillingEventRecord};
use sdkwork_api_domain_catalog::{Channel, ProxyProvider};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_routing::{RoutingDecisionLog, RoutingDecisionSource};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_storage_core::{
    AdminStore, BillingStore, CatalogStore, CredentialStore,
    ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord, ExtensionStore,
    IdentityStore, MarketingStore, RoutingStore, ServiceRuntimeNodeRecord,
    StandaloneConfigRolloutParticipantRecord, StandaloneConfigRolloutRecord, StorageDialect,
    TenantStore, UsageStore,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_implements_admin_store_trait() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let trait_store: &dyn AdminStore = &store;
    let identity_store: &dyn IdentityStore = &store;
    let tenant_store: &dyn TenantStore = &store;
    let catalog_store: &dyn CatalogStore = &store;
    let credential_store: &dyn CredentialStore = &store;
    let routing_store: &dyn RoutingStore = &store;
    let usage_store: &dyn UsageStore = &store;
    let billing_store: &dyn BillingStore = &store;
    let extension_store: &dyn ExtensionStore = &store;
    let marketing_store: &dyn MarketingStore = &store;

    assert_eq!(trait_store.dialect(), StorageDialect::Sqlite);
    let channels = trait_store.list_channels().await.unwrap();
    assert!(channels.len() >= 5);
    assert!(channels.iter().any(|channel| channel.id == "openai"));
    assert!(identity_store.list_portal_users().await.unwrap().is_empty());
    assert!(tenant_store.list_tenants().await.unwrap().is_empty());
    assert!(catalog_store.list_providers().await.unwrap().is_empty());
    assert!(credential_store
        .list_credentials()
        .await
        .unwrap()
        .is_empty());
    assert!(routing_store
        .list_routing_policies()
        .await
        .unwrap()
        .is_empty());
    assert!(usage_store.list_usage_records().await.unwrap().is_empty());
    assert!(billing_store
        .list_ledger_entries()
        .await
        .unwrap()
        .is_empty());
    assert!(billing_store
        .list_billing_events()
        .await
        .unwrap()
        .is_empty());
    assert!(extension_store
        .list_extension_installations()
        .await
        .unwrap()
        .is_empty());
    assert!(
        MarketingStore::list_coupon_template_records(marketing_store)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn sqlite_store_round_trips_runtime_rollout_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let now_ms = 1_234_567_u64;

    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms,
        ))
        .await
        .unwrap();
    store
        .insert_extension_runtime_rollout(&ExtensionRuntimeRolloutRecord::new(
            "rollout-1",
            "extension",
            Some("sdkwork.provider.native.mock".to_owned()),
            None,
            Some("sdkwork.provider.native.mock".to_owned()),
            "admin-user",
            now_ms,
            now_ms + 30_000,
        ))
        .await
        .unwrap();
    store
        .insert_extension_runtime_rollout_participant(
            &ExtensionRuntimeRolloutParticipantRecord::new(
                "rollout-1",
                "gateway-node-a",
                "gateway",
                "pending",
                now_ms,
            ),
        )
        .await
        .unwrap();

    let nodes = store.list_service_runtime_nodes().await.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].node_id, "gateway-node-a");

    let rollouts = store.list_extension_runtime_rollouts().await.unwrap();
    assert_eq!(rollouts.len(), 1);
    assert_eq!(rollouts[0].rollout_id, "rollout-1");

    let participants = store
        .list_extension_runtime_rollout_participants("rollout-1")
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].node_id, "gateway-node-a");
}

#[tokio::test]
async fn sqlite_store_round_trips_standalone_config_rollout_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let now_ms = 7_654_321_u64;

    store
        .insert_standalone_config_rollout(&StandaloneConfigRolloutRecord::new(
            "config-rollout-1",
            Some("portal".to_owned()),
            "admin-user",
            now_ms,
            now_ms + 30_000,
        ))
        .await
        .unwrap();
    store
        .insert_standalone_config_rollout_participant(
            &StandaloneConfigRolloutParticipantRecord::new(
                "config-rollout-1",
                "portal-node-a",
                "portal",
                "pending",
                now_ms,
            ),
        )
        .await
        .unwrap();

    let rollouts = store.list_standalone_config_rollouts().await.unwrap();
    assert_eq!(rollouts.len(), 1);
    assert_eq!(rollouts[0].rollout_id, "config-rollout-1");
    assert_eq!(
        rollouts[0].requested_service_kind.as_deref(),
        Some("portal")
    );

    let participants = store
        .list_standalone_config_rollout_participants("config-rollout-1")
        .await
        .unwrap();
    assert_eq!(participants.len(), 1);
    assert_eq!(participants[0].node_id, "portal-node-a");
}

#[tokio::test]
async fn sqlite_store_finds_latest_project_routing_log_and_usage_record() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_routing_decision_log(
            &RoutingDecisionLog::new(
                "decision-old",
                RoutingDecisionSource::Gateway,
                "chat_completion",
                "gpt-4.1",
                "provider-a",
                "deterministic_priority",
                100,
            )
            .with_project_id("project-1"),
        )
        .await
        .unwrap();
    store
        .insert_routing_decision_log(
            &RoutingDecisionLog::new(
                "decision-new",
                RoutingDecisionSource::Gateway,
                "chat_completion",
                "gpt-4.1",
                "provider-b",
                "deterministic_priority",
                200,
            )
            .with_project_id("project-1"),
        )
        .await
        .unwrap();
    store
        .insert_usage_record(&UsageRecord {
            project_id: "project-1".to_owned(),
            model: "gpt-4.1".to_owned(),
            provider: "provider-a".to_owned(),
            units: 1,
            amount: 0.01,
            input_tokens: 1,
            output_tokens: 2,
            total_tokens: 3,
            created_at_ms: 100,
            api_key_hash: None,
            channel_id: None,
            latency_ms: None,
            reference_amount: None,
        })
        .await
        .unwrap();
    store
        .insert_usage_record(&UsageRecord {
            project_id: "project-1".to_owned(),
            model: "gpt-4.1-mini".to_owned(),
            provider: "provider-b".to_owned(),
            units: 2,
            amount: 0.02,
            input_tokens: 4,
            output_tokens: 5,
            total_tokens: 9,
            created_at_ms: 200,
            api_key_hash: None,
            channel_id: None,
            latency_ms: None,
            reference_amount: None,
        })
        .await
        .unwrap();

    let latest_log = store
        .find_latest_routing_decision_log_for_project("project-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest_log.decision_id, "decision-new");

    let latest_usage = store
        .find_latest_usage_record_for_project("project-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest_usage.model, "gpt-4.1-mini");
}

#[tokio::test]
async fn sqlite_store_finds_any_model_without_full_scan() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "z-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "a-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let model = store.find_any_model().await.unwrap().unwrap();
    assert_eq!(model.external_name, "a-model");
}

#[tokio::test]
async fn sqlite_store_round_trips_billing_events() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

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

    store.insert_billing_event(&event).await.unwrap();

    let events = store.list_billing_events().await.unwrap();
    assert_eq!(events, vec![event]);
}

#[tokio::test]
async fn sqlite_store_lists_providers_for_model_without_full_scan() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://api.openai.com",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "a-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let providers = store.list_providers_for_model("a-model").await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].id, "provider-openai-official");
}

#[tokio::test]
async fn sqlite_store_lists_commerce_orders_for_project_after_checkpoint() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let older_order = CommerceOrderRecord::new(
        "order-old",
        "project-1",
        "user-1",
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        4_000,
        "$40.00",
        "$40.00",
        100_000,
        0,
        "fulfilled",
        "workspace_seed",
        100,
    )
    .with_updated_at_ms(120);
    let same_timestamp_order = CommerceOrderRecord::new(
        "order-same-ts",
        "project-1",
        "user-1",
        "coupon_redemption",
        "TEAMREADY",
        "TEAMREADY",
        0,
        0,
        "$0.00",
        "$0.00",
        0,
        25_000,
        "fulfilled",
        "workspace_seed",
        120,
    )
    .with_updated_at_ms(120);
    let newer_order = CommerceOrderRecord::new(
        "order-new",
        "project-1",
        "user-1",
        "custom_recharge",
        "custom-50",
        "Custom Recharge",
        5_000,
        5_000,
        "$50.00",
        "$50.00",
        140_000,
        0,
        "pending_payment",
        "workspace_seed",
        130,
    )
    .with_updated_at_ms(150);

    store.insert_commerce_order(&older_order).await.unwrap();
    store
        .insert_commerce_order(&same_timestamp_order)
        .await
        .unwrap();
    store.insert_commerce_order(&newer_order).await.unwrap();

    let orders = store
        .list_commerce_orders_for_project_after("project-1", 120, 100, "order-old")
        .await
        .unwrap();

    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].order_id, "order-new");
    assert_eq!(orders[1].order_id, "order-same-ts");
}

#[tokio::test]
async fn sqlite_store_lists_recent_commerce_orders_with_limit() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let older_order = CommerceOrderRecord::new(
        "order-old",
        "project-1",
        "user-1",
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        4_000,
        "$40.00",
        "$40.00",
        100_000,
        0,
        "fulfilled",
        "workspace_seed",
        100,
    )
    .with_updated_at_ms(120);
    let same_timestamp_order = CommerceOrderRecord::new(
        "order-same-ts",
        "project-2",
        "user-2",
        "coupon_redemption",
        "TEAMREADY",
        "TEAMREADY",
        0,
        0,
        "$0.00",
        "$0.00",
        0,
        25_000,
        "fulfilled",
        "workspace_seed",
        120,
    )
    .with_updated_at_ms(120);
    let newer_order = CommerceOrderRecord::new(
        "order-new",
        "project-3",
        "user-3",
        "custom_recharge",
        "custom-50",
        "Custom Recharge",
        5_000,
        5_000,
        "$50.00",
        "$50.00",
        140_000,
        0,
        "pending_payment",
        "workspace_seed",
        130,
    )
    .with_updated_at_ms(150);

    store.insert_commerce_order(&older_order).await.unwrap();
    store
        .insert_commerce_order(&same_timestamp_order)
        .await
        .unwrap();
    store.insert_commerce_order(&newer_order).await.unwrap();

    let orders = AdminStore::list_recent_commerce_orders(&store, 2)
        .await
        .unwrap();

    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].order_id, "order-new");
    assert_eq!(orders[1].order_id, "order-same-ts");
}
