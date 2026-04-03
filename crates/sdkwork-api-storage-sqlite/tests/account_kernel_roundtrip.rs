use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountHoldRecord,
    AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord, AccountType,
    PricingPlanRecord, PricingRateRecord, RequestSettlementRecord,
};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestMeterMetricRecord};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_round_trips_canonical_account_kernel_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    let lot =
        AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(1200.0)
            .with_remaining_quantity(1200.0)
            .with_issued_at_ms(30)
            .with_created_at_ms(30)
            .with_updated_at_ms(30);
    let hold = AccountHoldRecord::new(8101, 1001, 2002, 7001, 9001, 6001)
        .with_estimated_quantity(42.5)
        .with_expires_at_ms(40)
        .with_created_at_ms(35)
        .with_updated_at_ms(35);
    let ledger_entry = AccountLedgerEntryRecord::new(
        8201,
        1001,
        2002,
        7001,
        9001,
        AccountLedgerEntryType::HoldCreate,
    )
    .with_request_id(Some(6001))
    .with_hold_id(Some(8101))
    .with_benefit_type(Some("cash_credit".to_owned()))
    .with_quantity(42.5)
    .with_amount(42.5)
    .with_created_at_ms(36);
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
    .with_protocol_family("openai")
    .with_estimated_credit_hold(24.0)
    .with_created_at_ms(35)
    .with_updated_at_ms(36);
    let metric = RequestMeterMetricRecord::new(7001001, 1001, 2002, 6001, "token.input", 128.0)
        .with_provider_field(Some("prompt_tokens".to_owned()))
        .with_captured_at_ms(37);
    let plan = PricingPlanRecord::new(9101, 1001, 2002, "default-retail", 3)
        .with_display_name("Default Retail v3")
        .with_status("active")
        .with_created_at_ms(38)
        .with_updated_at_ms(38);
    let rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_unit_price(0.0025)
        .with_created_at_ms(39);
    let settlement = RequestSettlementRecord::new(8301, 1001, 2002, 6001, 7001, 9001)
        .with_hold_id(Some(8101))
        .with_estimated_credit_hold(42.5)
        .with_captured_credit_amount(40.0)
        .with_provider_cost_amount(18.0)
        .with_retail_charge_amount(40.0)
        .with_settled_at_ms(41)
        .with_created_at_ms(41)
        .with_updated_at_ms(41);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&lot).await.unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_account_ledger_entry_record(&ledger_entry)
        .await
        .unwrap();
    store.insert_request_meter_fact(&fact).await.unwrap();
    store.insert_request_meter_metric(&metric).await.unwrap();
    store.insert_pricing_plan_record(&plan).await.unwrap();
    store.insert_pricing_rate_record(&rate).await.unwrap();
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();

    assert_eq!(
        store.find_account_record(7001).await.unwrap(),
        Some(account)
    );
    assert_eq!(store.list_account_records().await.unwrap().len(), 1);
    assert_eq!(store.list_account_benefit_lots().await.unwrap().len(), 1);
    assert_eq!(store.list_account_holds().await.unwrap().len(), 1);
    assert_eq!(
        store
            .list_account_ledger_entry_records()
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(store.list_request_meter_facts().await.unwrap().len(), 1);
    assert_eq!(store.list_request_meter_metrics().await.unwrap().len(), 1);
    assert_eq!(store.list_pricing_plan_records().await.unwrap().len(), 1);
    assert_eq!(store.list_pricing_rate_records().await.unwrap().len(), 1);
    assert_eq!(
        store.list_request_settlement_records().await.unwrap().len(),
        1
    );
}
