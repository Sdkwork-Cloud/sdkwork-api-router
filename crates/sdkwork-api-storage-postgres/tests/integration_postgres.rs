use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldAllocationRecord, AccountHoldRecord, AccountLedgerAllocationRecord,
    AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord, AccountType,
    BillingAccountingMode, BillingEventRecord, PricingPlanRecord, PricingRateRecord, QuotaPolicy,
    RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_catalog::{
    Channel, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestMeterMetricRecord, UsageRecord};
use sdkwork_api_secret_core::encrypt;
use sdkwork_api_storage_core::{AccountKernelStore, AccountKernelTransactionExecutor};
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn postgres_store_persists_catalog_and_credentials_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool.clone());

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

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where tablename = 'ai_model_price'
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();
    assert!(index_names.contains("idx_ai_model_price_model_active"));
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
async fn postgres_store_creates_canonical_account_kernel_tables_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    for table_name in [
        "ai_account",
        "ai_account_benefit_lot",
        "ai_account_hold",
        "ai_account_hold_allocation",
        "ai_account_ledger_entry",
        "ai_account_ledger_allocation",
        "ai_request_meter_fact",
        "ai_request_meter_metric",
        "ai_request_settlement",
        "ai_pricing_plan",
        "ai_pricing_rate",
    ] {
        let row: (String,) = sqlx::query_as(
            "select tablename
             from pg_tables
             where schemaname = 'public' and tablename = $1",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, table_name);
    }

    assert_pg_column(&pool, "ai_account", "tenant_id", "bigint", false, None).await;
    assert_pg_column(
        &pool,
        "ai_account",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(&pool, "ai_account", "user_id", "bigint", false, None).await;
    assert_pg_column(
        &pool,
        "ai_request_meter_fact",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_request_meter_fact",
        "account_id",
        "bigint",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_request_settlement",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_account_hold_allocation",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_account_ledger_allocation",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where schemaname = 'public'
           and tablename in (
             'ai_account',
             'ai_account_benefit_lot',
             'ai_account_hold',
             'ai_account_hold_allocation',
             'ai_account_ledger_allocation',
             'ai_request_meter_fact',
             'ai_request_settlement',
             'ai_pricing_plan'
           )
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();
    for index_name in [
        "idx_ai_account_user_type",
        "idx_ai_account_benefit_lot_account_status_expiry",
        "idx_ai_account_hold_request",
        "idx_ai_account_hold_allocation_hold_lot",
        "idx_ai_account_ledger_allocation_ledger_lot",
        "idx_ai_request_meter_fact_user_created_at",
        "idx_ai_request_meter_fact_api_key_created_at",
        "idx_ai_request_settlement_request",
        "idx_ai_pricing_plan_code_version",
    ] {
        assert!(
            index_names.contains(index_name),
            "missing index {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_store_creates_canonical_identity_kernel_tables_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    for table_name in ["ai_user", "ai_api_key", "ai_identity_binding"] {
        let row: (String,) = sqlx::query_as(
            "select tablename
             from pg_tables
             where schemaname = 'public' and tablename = $1",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, table_name);
    }

    assert_pg_column(
        &pool,
        "ai_user",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_api_key",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_identity_binding",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where schemaname = 'public'
           and tablename in ('ai_user', 'ai_api_key', 'ai_identity_binding')
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();
    for index_name in [
        "idx_ai_user_scope",
        "idx_ai_user_email",
        "idx_ai_api_key_hash",
        "idx_ai_api_key_user_status",
        "idx_ai_identity_binding_lookup",
    ] {
        assert!(
            index_names.contains(index_name),
            "missing index {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_store_round_trips_pricing_plans_and_rates_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let now_ms = 1_717_171_000 + seed;
    let plan_id = 9_100_000 + seed;
    let rate_id = 9_200_000 + seed;

    let plan = PricingPlanRecord::new(plan_id, 1001, 2002, format!("workspace-retail-{seed}"), 1)
        .with_display_name("Workspace Retail")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(now_ms - 10_000)
        .with_effective_to_ms(Some(now_ms + 86_400_000))
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms + 1);
    store.insert_pricing_plan_record(&plan).await.unwrap();

    let rate = PricingRateRecord::new(rate_id, 1001, 2002, plan_id, "token.input")
        .with_capability_code(Some("responses".to_owned()))
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openrouter".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1_000_000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail text input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms + 2);
    store.insert_pricing_rate_record(&rate).await.unwrap();

    let stored_plan = store
        .list_pricing_plan_records()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.pricing_plan_id == plan_id)
        .expect("pricing plan");
    assert_eq!(stored_plan, plan);

    let stored_rate = store
        .list_pricing_rate_records()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.pricing_rate_id == rate_id)
        .expect("pricing rate");
    assert_eq!(stored_rate, rate);
}

#[tokio::test]
async fn postgres_store_round_trips_commercial_account_read_models_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let now_ms = 1_717_181_000 + seed;
    let account_id = 8_100_000 + seed;
    let lot_id = 8_200_000 + seed;
    let hold_id = 8_300_000 + seed;
    let request_id = 8_400_000 + seed;
    let settlement_id = 8_500_000 + seed;

    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms + 1);
    store.insert_account_record(&account).await.unwrap();

    let lot = AccountBenefitLotRecord::new(
        lot_id,
        1001,
        2002,
        account_id,
        9001,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(200.0)
    .with_remaining_quantity(160.0)
    .with_held_quantity(10.0)
    .with_priority(10)
    .with_issued_at_ms(now_ms)
    .with_created_at_ms(now_ms + 2)
    .with_updated_at_ms(now_ms + 3);
    store.insert_account_benefit_lot(&lot).await.unwrap();

    let hold = AccountHoldRecord::new(hold_id, 1001, 2002, account_id, 9001, request_id)
        .with_estimated_quantity(10.0)
        .with_captured_quantity(10.0)
        .with_expires_at_ms(now_ms + 60_000)
        .with_created_at_ms(now_ms + 4)
        .with_updated_at_ms(now_ms + 5);
    store.insert_account_hold(&hold).await.unwrap();

    let settlement =
        RequestSettlementRecord::new(settlement_id, 1001, 2002, request_id, account_id, 9001)
            .with_hold_id(Some(hold_id))
            .with_status(RequestSettlementStatus::Captured)
            .with_estimated_credit_hold(10.0)
            .with_captured_credit_amount(10.0)
            .with_provider_cost_amount(5.0)
            .with_retail_charge_amount(10.0)
            .with_settled_at_ms(now_ms + 6)
            .with_created_at_ms(now_ms + 6)
            .with_updated_at_ms(now_ms + 7);
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();

    let stored_account = store
        .find_account_record(account_id)
        .await
        .unwrap()
        .expect("account");
    assert_eq!(stored_account, account);

    let owner_account = store
        .find_account_record_by_owner(1001, 2002, 9001, AccountType::Primary)
        .await
        .unwrap()
        .expect("owner account");
    assert_eq!(owner_account, account);

    assert!(store
        .list_account_records()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &account));
    assert!(store
        .list_account_benefit_lots()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &lot));
    assert!(store
        .list_account_holds()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &hold));
    assert!(store
        .list_request_settlement_records()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &settlement));
}

#[tokio::test]
async fn postgres_store_round_trips_remaining_account_kernel_records_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let request_id = 6_001_000 + seed;
    let account = AccountRecord::new(7_001_000 + seed, 1001, 2002, 9001, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    let lot = AccountBenefitLotRecord::new(
        8_001_000 + seed,
        1001,
        2002,
        account.account_id,
        9001,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(1200.0)
    .with_remaining_quantity(1200.0)
    .with_issued_at_ms(30)
    .with_created_at_ms(30)
    .with_updated_at_ms(30);
    let hold = AccountHoldRecord::new(
        8_101_000 + seed,
        1001,
        2002,
        account.account_id,
        9001,
        request_id,
    )
    .with_estimated_quantity(42.5)
    .with_expires_at_ms(40)
    .with_created_at_ms(35)
    .with_updated_at_ms(35);
    let ledger_entry = AccountLedgerEntryRecord::new(
        8_201_000 + seed,
        1001,
        2002,
        account.account_id,
        9001,
        AccountLedgerEntryType::HoldCreate,
    )
    .with_request_id(Some(request_id))
    .with_hold_id(Some(hold.hold_id))
    .with_benefit_type(Some("cash_credit".to_owned()))
    .with_quantity(42.5)
    .with_amount(42.5)
    .with_created_at_ms(36);
    let hold_allocation =
        AccountHoldAllocationRecord::new(8_401_000 + seed, 1001, 2002, hold.hold_id, lot.lot_id)
            .with_allocated_quantity(42.5)
            .with_captured_quantity(40.0)
            .with_released_quantity(2.5)
            .with_created_at_ms(36)
            .with_updated_at_ms(41);
    let ledger_allocation = AccountLedgerAllocationRecord::new(
        8_501_000 + seed,
        1001,
        2002,
        ledger_entry.ledger_entry_id,
        lot.lot_id,
    )
    .with_quantity_delta(-40.0)
    .with_created_at_ms(41);
    let fact = RequestMeterFactRecord::new(
        request_id,
        1001,
        2002,
        9001,
        account.account_id,
        "api_key",
        "responses",
        "openai",
        "gpt-4.1",
        "provider-openai-official",
    )
    .with_api_key_id(Some(778899))
    .with_api_key_hash(Some("key_hash_live".to_owned()))
    .with_protocol_family("openai")
    .with_estimated_credit_hold(24.0)
    .with_created_at_ms(35)
    .with_updated_at_ms(36);
    let metric = RequestMeterMetricRecord::new(
        7_001_001 + seed,
        1001,
        2002,
        request_id,
        "token.input",
        128.0,
    )
    .with_provider_field(Some("prompt_tokens".to_owned()))
    .with_captured_at_ms(37);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&lot).await.unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_account_ledger_entry_record(&ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_hold_allocation(&hold_allocation)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&ledger_allocation)
        .await
        .unwrap();
    store.insert_request_meter_fact(&fact).await.unwrap();
    store.insert_request_meter_metric(&metric).await.unwrap();

    assert!(store
        .list_account_hold_allocations()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &hold_allocation));
    assert!(store
        .list_account_ledger_entry_records()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &ledger_entry));
    assert!(store
        .list_account_ledger_allocations()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &ledger_allocation));
    assert!(store
        .list_request_meter_facts()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &fact));
    assert!(store
        .list_request_meter_metrics()
        .await
        .unwrap()
        .iter()
        .any(|entry| entry == &metric));
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
async fn postgres_store_replaces_provider_health_snapshot_for_same_provider_runtime_and_instance() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let original = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1_000,
    )
    .with_running(true)
    .with_healthy(false)
    .with_message("first failure");
    let replacement = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        2_000,
    )
    .with_running(true)
    .with_healthy(true)
    .with_message("recovered");

    store
        .insert_provider_health_snapshot(&original)
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(&replacement)
        .await
        .unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert_eq!(snapshots, vec![replacement]);
}

async fn assert_pg_column(
    pool: &PgPool,
    table_name: &str,
    column_name: &str,
    data_type: &str,
    nullable: bool,
    default_contains: Option<&str>,
) {
    let row: (String, String, Option<String>) = sqlx::query_as(
        "select data_type, is_nullable, column_default
         from information_schema.columns
         where table_schema = 'public'
           and table_name = $1
           and column_name = $2",
    )
    .bind(table_name)
    .bind(column_name)
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(row.0, data_type);
    assert_eq!(row.1 == "YES", nullable);
    match default_contains {
        Some(expected) => assert!(
            row.2
                .as_deref()
                .is_some_and(|value| value.contains(expected)),
            "expected default for {table_name}.{column_name} to contain {expected:?}, got {:?}",
            row.2
        ),
        None => {}
    }
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

#[tokio::test]
async fn postgres_store_persists_billing_events_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let event = BillingEventRecord::new(
        "evt-postgres-1",
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
    assert!(events.iter().any(|entry| entry == &event));
}

#[tokio::test]
async fn postgres_store_finds_latest_project_routing_log_and_usage_record_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

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
async fn postgres_store_finds_any_model_without_full_scan_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

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
        .insert_model(&ModelCatalogEntry::new(
            "z-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "a-model",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let model = store.find_any_model().await.unwrap().unwrap();
    assert_eq!(model.external_name, "a-model");
}

#[tokio::test]
async fn postgres_store_lists_providers_for_model_without_full_scan_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

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
        .insert_model(&ModelCatalogEntry::new(
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
async fn postgres_store_lists_provider_bindings_for_model_without_drop_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let provider = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .with_channel_binding(ProviderChannelBinding::new(
        "provider-openrouter-main",
        "openai",
    ));

    store.insert_provider(&provider).await.unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openrouter-main",
        ))
        .await
        .unwrap();

    let providers = store.list_providers_for_model("gpt-4.1").await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].channel_bindings.len(), 2);
    assert_eq!(providers[0].channel_bindings[1].channel_id, "openai");
}

#[tokio::test]
async fn postgres_account_kernel_transaction_round_trips_hold_and_settlement_when_url_is_provided()
{
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool.clone());
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let account_id = 8_000_000 + seed;
    let lot_id = 8_100_000 + seed;
    let hold_id = 8_200_000 + seed;
    let hold_allocation_id = 8_300_000 + seed;
    let settlement_id = 8_400_000 + seed;
    let request_id = 8_500_000 + seed;

    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    sqlx::query(
        "INSERT INTO ai_account (
            account_id, tenant_id, organization_id, user_id, account_type,
            currency_code, credit_unit_code, status, allow_overdraft, overdraft_limit,
            created_at_ms, updated_at_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(i64::try_from(account.account_id).unwrap())
    .bind(i64::try_from(account.tenant_id).unwrap())
    .bind(i64::try_from(account.organization_id).unwrap())
    .bind(i64::try_from(account.user_id).unwrap())
    .bind("primary")
    .bind(&account.currency_code)
    .bind(&account.credit_unit_code)
    .bind("active")
    .bind(account.allow_overdraft)
    .bind(account.overdraft_limit)
    .bind(i64::try_from(account.created_at_ms).unwrap())
    .bind(i64::try_from(account.updated_at_ms).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let lot = AccountBenefitLotRecord::new(
        lot_id,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(100.0)
    .with_remaining_quantity(100.0)
    .with_created_at_ms(20)
    .with_updated_at_ms(20);
    sqlx::query(
        "INSERT INTO ai_account_benefit_lot (
            lot_id, tenant_id, organization_id, account_id, user_id, benefit_type,
            source_type, source_id, scope_json, original_quantity, remaining_quantity,
            held_quantity, priority, acquired_unit_cost, issued_at_ms, expires_at_ms, status,
            created_at_ms, updated_at_ms
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL, $8, $9, $10, $11, NULL, $12, NULL, $13, $14, $15)",
    )
    .bind(i64::try_from(lot.lot_id).unwrap())
    .bind(i64::try_from(lot.tenant_id).unwrap())
    .bind(i64::try_from(lot.organization_id).unwrap())
    .bind(i64::try_from(lot.account_id).unwrap())
    .bind(i64::try_from(lot.user_id).unwrap())
    .bind("cash_credit")
    .bind("recharge")
    .bind(lot.original_quantity)
    .bind(lot.remaining_quantity)
    .bind(lot.held_quantity)
    .bind(lot.priority)
    .bind(i64::try_from(lot.issued_at_ms).unwrap())
    .bind("active")
    .bind(i64::try_from(lot.created_at_ms).unwrap())
    .bind(i64::try_from(lot.updated_at_ms).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let seeded_account = tx.find_account_record(account_id).await?.unwrap();
                let seeded_lot = tx.find_account_benefit_lot(lot_id).await?.unwrap();

                tx.upsert_account_benefit_lot(
                    &seeded_lot
                        .clone()
                        .with_held_quantity(40.0)
                        .with_updated_at_ms(35),
                )
                .await?;

                let hold = AccountHoldRecord::new(
                    hold_id,
                    seeded_account.tenant_id,
                    seeded_account.organization_id,
                    seeded_account.account_id,
                    seeded_account.user_id,
                    request_id,
                )
                .with_estimated_quantity(40.0)
                .with_expires_at_ms(120)
                .with_created_at_ms(35)
                .with_updated_at_ms(35);
                tx.upsert_account_hold(&hold).await?;

                let allocation = AccountHoldAllocationRecord::new(
                    hold_allocation_id,
                    seeded_account.tenant_id,
                    seeded_account.organization_id,
                    hold_id,
                    lot_id,
                )
                .with_allocated_quantity(40.0)
                .with_created_at_ms(35)
                .with_updated_at_ms(35);
                tx.upsert_account_hold_allocation(&allocation).await?;

                let settlement = RequestSettlementRecord::new(
                    settlement_id,
                    seeded_account.tenant_id,
                    seeded_account.organization_id,
                    request_id,
                    seeded_account.account_id,
                    seeded_account.user_id,
                )
                .with_hold_id(Some(hold_id))
                .with_status(RequestSettlementStatus::Captured)
                .with_estimated_credit_hold(40.0)
                .with_captured_credit_amount(40.0)
                .with_retail_charge_amount(40.0)
                .with_settled_at_ms(36)
                .with_created_at_ms(36)
                .with_updated_at_ms(36);
                tx.upsert_request_settlement_record(&settlement).await?;

                Ok(())
            })
        })
        .await
        .unwrap();

    let verified = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx
                    .find_account_hold_by_request_id(request_id)
                    .await?
                    .unwrap();
                let allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;
                let lot = tx.find_account_benefit_lot(lot_id).await?.unwrap();
                let settlement = tx
                    .find_request_settlement_by_request_id(request_id)
                    .await?
                    .unwrap();
                Ok((hold, allocations, lot, settlement))
            })
        })
        .await
        .unwrap();

    assert_eq!(verified.0.hold_id, hold_id);
    assert_eq!(verified.1.len(), 1);
    assert_eq!(verified.1[0].allocated_quantity, 40.0);
    assert_eq!(verified.2.held_quantity, 40.0);
    assert_eq!(verified.3.request_settlement_id, settlement_id);

    let existing = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                Ok(tx
                    .find_request_settlement_by_request_id(request_id)
                    .await?
                    .unwrap())
            })
        })
        .await
        .unwrap();
    assert_eq!(existing.request_settlement_id, settlement_id);
}
