use std::fs;

#[test]
fn postgres_store_implements_account_kernel_pricing_surface() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("postgres storage source");

    assert!(
        source.contains("async fn insert_pricing_plan_record("),
        "expected postgres store pricing plan insert implementation",
    );
    assert!(
        source.contains(
            "async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>>"
        ),
        "expected postgres store pricing plan list implementation",
    );
    assert!(
        source.contains("async fn insert_pricing_rate_record("),
        "expected postgres store pricing rate insert implementation",
    );
    assert!(
        source.contains(
            "async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>>"
        ),
        "expected postgres store pricing rate list implementation",
    );
    assert!(
        source.contains("fn decode_pricing_plan_row(row: PgRow) -> Result<PricingPlanRecord>"),
        "expected postgres pricing plan row decoder",
    );
    assert!(
        source.contains("fn decode_pricing_rate_row(row: PgRow) -> Result<PricingRateRecord>"),
        "expected postgres pricing rate row decoder",
    );
}

#[test]
fn postgres_store_implements_commercial_account_store_surface() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("postgres storage source");

    for snippet in [
        "async fn insert_account_record(&self, record: &AccountRecord) -> Result<AccountRecord>",
        "async fn list_account_records(&self) -> Result<Vec<AccountRecord>>",
        "async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>>",
        "async fn find_account_record_by_owner(",
        "async fn insert_account_benefit_lot(",
        "async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>>",
        "async fn insert_account_hold(&self, record: &AccountHoldRecord) -> Result<AccountHoldRecord>",
        "async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>>",
        "async fn insert_request_settlement_record(",
        "async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>>",
    ] {
        assert!(
            source.contains(snippet),
            "expected postgres commercial account surface to contain {snippet}",
        );
    }
}

#[test]
fn postgres_store_implements_remaining_account_kernel_persistence_surface() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("postgres storage source");

    for snippet in [
        "async fn insert_account_hold_allocation(",
        "async fn list_account_hold_allocations(&self) -> Result<Vec<AccountHoldAllocationRecord>>",
        "async fn insert_account_ledger_entry_record(",
        "async fn list_account_ledger_entry_records(&self) -> Result<Vec<AccountLedgerEntryRecord>>",
        "async fn insert_account_ledger_allocation(",
        "async fn list_account_ledger_allocations(&self) -> Result<Vec<AccountLedgerAllocationRecord>>",
        "async fn insert_request_meter_fact(",
        "async fn list_request_meter_facts(&self) -> Result<Vec<RequestMeterFactRecord>>",
        "async fn insert_request_meter_metric(",
        "async fn list_request_meter_metrics(&self) -> Result<Vec<RequestMeterMetricRecord>>",
        "fn decode_account_ledger_entry_row(row: PgRow) -> Result<AccountLedgerEntryRecord>",
        "fn decode_account_ledger_allocation_row(row: PgRow) -> Result<AccountLedgerAllocationRecord>",
        "fn decode_request_meter_fact_row(row: PgRow) -> Result<RequestMeterFactRecord>",
        "fn decode_request_meter_metric_row(row: PgRow) -> Result<RequestMeterMetricRecord>",
        "fn account_ledger_entry_type_as_str(value: AccountLedgerEntryType) -> &'static str",
        "fn parse_account_ledger_entry_type(value: &str) -> Result<AccountLedgerEntryType>",
        "fn request_status_as_str(value: RequestStatus) -> &'static str",
        "fn parse_request_status(value: &str) -> Result<RequestStatus>",
        "fn usage_capture_status_as_str(value: UsageCaptureStatus) -> &'static str",
        "fn parse_usage_capture_status(value: &str) -> Result<UsageCaptureStatus>",
    ] {
        assert!(
            source.contains(snippet),
            "expected postgres remaining account-kernel surface to contain {snippet}",
        );
    }
}
