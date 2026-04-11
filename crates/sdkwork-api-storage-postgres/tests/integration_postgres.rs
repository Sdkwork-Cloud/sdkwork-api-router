use sdkwork_api_domain_billing::{BillingAccountingMode, BillingEventRecord, QuotaPolicy};
use sdkwork_api_domain_catalog::{
    Channel, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_payment::{
    FinanceDirection, FinanceEntryCode, FinanceJournalEntryRecord, FinanceJournalLineRecord,
    PaymentAttemptRecord, PaymentAttemptStatus, PaymentCallbackEventRecord,
    PaymentCallbackProcessingStatus, PaymentChannelPolicyRecord, PaymentGatewayAccountRecord,
    PaymentOrderRecord, PaymentOrderStatus, PaymentProviderCode, PaymentSessionKind,
    PaymentSessionRecord, PaymentSessionStatus, PaymentTransactionKind, PaymentTransactionRecord,
    ReconciliationMatchStatus, ReconciliationMatchSummaryRecord, RefundOrderRecord,
    RefundOrderStatus,
};
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_secret_core::encrypt;
use sdkwork_api_storage_core::PaymentKernelStore;
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};
use sqlx::PgPool;

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
async fn postgres_store_creates_canonical_payment_kernel_tables_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    for table_name in [
        "ai_payment_gateway_account",
        "ai_payment_channel_policy",
        "ai_payment_order",
        "ai_payment_attempt",
        "ai_payment_session",
        "ai_payment_transaction",
        "ai_payment_callback_event",
        "ai_refund_order",
        "ai_refund_attempt",
        "ai_refund_order_processing_steps",
        "ai_finance_journal_entry",
        "ai_finance_journal_line",
        "ai_payment_reconciliation_batch",
        "ai_payment_reconciliation_line",
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
}

#[tokio::test]
async fn postgres_payment_kernel_tables_keep_scope_and_money_columns_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    assert_pg_column(
        &pool,
        "ai_payment_order",
        "tenant_id",
        "bigint",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_payment_order",
        "organization_id",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(&pool, "ai_payment_order", "user_id", "bigint", false, None).await;
    assert_pg_column(
        &pool,
        "ai_payment_order",
        "payable_minor",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_payment_order",
        "captured_amount_minor",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_refund_order",
        "refund_reason_code",
        "text",
        false,
        Some("''"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_refund_order",
        "requested_by_type",
        "text",
        false,
        Some("''"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_refund_order",
        "requested_by_id",
        "text",
        false,
        Some("''"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_refund_order",
        "requested_amount_minor",
        "bigint",
        false,
        Some("0"),
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_finance_journal_line",
        "amount_minor",
        "bigint",
        false,
        Some("0"),
    )
    .await;
}

#[tokio::test]
async fn postgres_payment_kernel_tables_create_operational_indexes_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where schemaname = 'public'
           and tablename in (
             'ai_payment_gateway_account',
             'ai_payment_order',
             'ai_payment_attempt',
             'ai_payment_session',
             'ai_payment_transaction',
             'ai_payment_callback_event',
             'ai_refund_order',
             'ai_finance_journal_entry',
             'ai_payment_reconciliation_line'
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
        "idx_ai_payment_gateway_account_provider_status_priority",
        "idx_ai_payment_order_user_created_at",
        "idx_ai_payment_order_provider_status_updated_at",
        "idx_ai_payment_attempt_order_attempt",
        "idx_ai_payment_session_attempt",
        "idx_ai_payment_transaction_order_occurred_at",
        "idx_ai_payment_callback_event_processing_received_at",
        "idx_ai_refund_order_payment_created_at",
        "idx_ai_finance_journal_entry_source",
        "idx_ai_payment_reconciliation_line_batch_status",
    ] {
        assert!(
            index_names.contains(index_name),
            "missing index {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_payment_store_round_trips_order_attempt_session_and_transaction_when_url_is_provided(
) {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let order = PaymentOrderRecord::new(
        "payment-order-pg-1",
        "commerce-order-pg-1",
        1,
        0,
        7,
        "project-1",
        "recharge",
        "workspace",
        "project-1",
        "USD",
        12_000,
    )
    .with_payable_minor(10_500)
    .with_provider_code(PaymentProviderCode::Stripe)
    .with_method_code("checkout")
    .with_payment_status(PaymentOrderStatus::AwaitingCustomer)
    .with_created_at_ms(1_700_000_000)
    .with_updated_at_ms(1_700_000_001);
    store.insert_payment_order_record(&order).await.unwrap();

    let attempt = PaymentAttemptRecord::new(
        "payment-attempt-pg-1",
        1,
        0,
        "payment-order-pg-1",
        1,
        "gateway-account-pg-1",
        PaymentProviderCode::Stripe,
        "checkout",
        "portal_web",
        "idem-pg-1",
    )
    .with_attempt_status(PaymentAttemptStatus::HandoffReady)
    .with_request_payload_hash("hash-pg-1")
    .with_created_at_ms(1_700_000_010)
    .with_updated_at_ms(1_700_000_011);
    store.insert_payment_attempt_record(&attempt).await.unwrap();

    let session = PaymentSessionRecord::new(
        "payment-session-pg-1",
        1,
        0,
        "payment-attempt-pg-1",
        PaymentSessionKind::HostedCheckout,
        PaymentSessionStatus::Open,
    )
    .with_redirect_url(Some("https://checkout.example/session".to_owned()))
    .with_expires_at_ms(1_700_000_600)
    .with_created_at_ms(1_700_000_020)
    .with_updated_at_ms(1_700_000_021);
    store.insert_payment_session_record(&session).await.unwrap();

    let transaction = PaymentTransactionRecord::new(
        "payment-tx-pg-1",
        1,
        0,
        "payment-order-pg-1",
        PaymentTransactionKind::Sale,
        PaymentProviderCode::Stripe,
        "provider-tx-pg-1",
        "USD",
        10_500,
        1_700_000_040,
    )
    .with_payment_attempt_id(Some("payment-attempt-pg-1".to_owned()))
    .with_provider_status("succeeded")
    .with_created_at_ms(1_700_000_041);
    store
        .insert_payment_transaction_record(&transaction)
        .await
        .unwrap();

    assert_eq!(
        store
            .find_payment_order_record("payment-order-pg-1")
            .await
            .unwrap()
            .unwrap(),
        order
    );
    assert_eq!(
        store
            .list_payment_attempt_records_for_order("payment-order-pg-1")
            .await
            .unwrap(),
        vec![attempt]
    );
    assert_eq!(
        store
            .list_payment_session_records_for_attempt("payment-attempt-pg-1")
            .await
            .unwrap(),
        vec![session]
    );
    assert_eq!(
        store
            .list_payment_transaction_records_for_order("payment-order-pg-1")
            .await
            .unwrap(),
        vec![transaction]
    );
}

#[tokio::test]
async fn postgres_payment_store_round_trips_callback_refund_finance_and_reconciliation_when_url_is_provided(
) {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let order = PaymentOrderRecord::new(
        "payment-order-pg-2",
        "commerce-order-pg-2",
        2,
        0,
        9,
        "project-2",
        "recharge",
        "workspace",
        "project-2",
        "USD",
        5_000,
    )
    .with_provider_code(PaymentProviderCode::Stripe)
    .with_payment_status(PaymentOrderStatus::Captured)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_001);
    store.insert_payment_order_record(&order).await.unwrap();

    let callback = PaymentCallbackEventRecord::new(
        "callback-pg-1",
        2,
        0,
        PaymentProviderCode::Stripe,
        "gateway-account-pg-2",
        "checkout.session.completed",
        "evt_pg_123",
        "dedupe_evt_pg_123",
        1_710_000_010,
    )
    .with_payment_order_id(Some("payment-order-pg-2".to_owned()))
    .with_processing_status(PaymentCallbackProcessingStatus::Processed)
    .with_payload_json(Some("{\"id\":\"evt_pg_123\"}".to_owned()))
    .with_processed_at_ms(Some(1_710_000_011));
    store
        .insert_payment_callback_event_record(&callback)
        .await
        .unwrap();

    let refund = RefundOrderRecord::new(
        "refund-order-pg-1",
        2,
        0,
        "payment-order-pg-2",
        "commerce-order-pg-2",
        "buyer_request",
        "portal_user",
        "user-9",
        "USD",
        2_000,
    )
    .with_refunded_amount_minor(1_000)
    .with_refund_status(RefundOrderStatus::Processing)
    .with_created_at_ms(1_710_000_020)
    .with_updated_at_ms(1_710_000_021);
    store.insert_refund_order_record(&refund).await.unwrap();

    let journal = FinanceJournalEntryRecord::new(
        "journal-pg-1",
        2,
        0,
        "payment_order",
        "payment-order-pg-2",
        FinanceEntryCode::CustomerPrepaidLiabilityIncrease,
        "USD",
        1_710_000_030,
    )
    .with_entry_status("posted")
    .with_created_at_ms(1_710_000_031);
    store
        .insert_finance_journal_entry_record(&journal)
        .await
        .unwrap();

    let journal_line = FinanceJournalLineRecord::new(
        "journal-line-pg-1",
        2,
        0,
        "journal-pg-1",
        1,
        "gateway_clearing_asset",
        FinanceDirection::Debit,
        5_000,
    )
    .with_party_type(Some("payment_order".to_owned()))
    .with_party_id(Some("payment-order-pg-2".to_owned()));
    store
        .insert_finance_journal_line_record(&journal_line)
        .await
        .unwrap();

    let reconciliation = ReconciliationMatchSummaryRecord::new(
        "recon-line-pg-1",
        2,
        0,
        "recon-batch-pg-1",
        "provider-tx-pg-2",
        ReconciliationMatchStatus::MismatchAmount,
        5_000,
    )
    .with_local_amount_minor(Some(4_900))
    .with_payment_order_id(Some("payment-order-pg-2".to_owned()))
    .with_reason_code(Some("provider_fee_pending".to_owned()))
    .with_created_at_ms(1_710_000_040)
    .with_updated_at_ms(1_710_000_041);
    store
        .insert_reconciliation_match_summary_record(&reconciliation)
        .await
        .unwrap();

    assert_eq!(
        store.list_payment_callback_event_records().await.unwrap(),
        vec![callback]
    );
    assert_eq!(
        store
            .list_refund_order_records_for_payment_order("payment-order-pg-2")
            .await
            .unwrap(),
        vec![refund]
    );
    assert_eq!(
        store.list_finance_journal_entry_records().await.unwrap(),
        vec![journal]
    );
    assert_eq!(
        store
            .list_finance_journal_line_records("journal-pg-1")
            .await
            .unwrap(),
        vec![journal_line]
    );
    assert_eq!(
        store
            .list_reconciliation_match_summary_records("recon-batch-pg-1")
            .await
            .unwrap(),
        vec![reconciliation.clone()]
    );
    assert_eq!(
        store
            .find_reconciliation_match_summary_record("recon-line-pg-1")
            .await
            .unwrap(),
        Some(reconciliation)
    );
    assert_eq!(
        store
            .find_reconciliation_match_summary_record("recon-line-pg-missing")
            .await
            .unwrap(),
        None
    );

    let gateway_account =
        PaymentGatewayAccountRecord::new("gateway-account-pg-1", 2, 0, PaymentProviderCode::Stripe)
            .with_environment("production")
            .with_merchant_id("merchant-pg-1")
            .with_app_id("app-pg-1")
            .with_status("active")
            .with_priority(100)
            .with_created_at_ms(1_710_000_050)
            .with_updated_at_ms(1_710_000_051);
    store
        .insert_payment_gateway_account_record(&gateway_account)
        .await
        .unwrap();

    let channel_policy = PaymentChannelPolicyRecord::new(
        "channel-policy-pg-1",
        2,
        0,
        PaymentProviderCode::Stripe,
        "hosted_checkout",
    )
    .with_scene_code("recharge_pack")
    .with_currency_code("USD")
    .with_client_kind("portal_web")
    .with_status("active")
    .with_priority(80)
    .with_created_at_ms(1_710_000_052)
    .with_updated_at_ms(1_710_000_053);
    store
        .insert_payment_channel_policy_record(&channel_policy)
        .await
        .unwrap();

    assert_eq!(
        store.list_payment_gateway_account_records().await.unwrap(),
        vec![gateway_account]
    );
    assert_eq!(
        store.list_payment_channel_policy_records().await.unwrap(),
        vec![channel_policy]
    );
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
