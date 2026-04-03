use sdkwork_api_domain_billing::{
    AccountHoldRecord, AccountHoldStatus, AccountLedgerEntryRecord, AccountLedgerEntryType,
    RequestSettlementRecord, RequestSettlementStatus,
};

#[test]
fn account_hold_tracks_estimate_capture_and_release_totals() {
    let hold = AccountHoldRecord::new(8101, 1001, 2002, 7001, 9001, 6001)
        .with_status(AccountHoldStatus::Held)
        .with_estimated_quantity(42.5)
        .with_captured_quantity(0.0)
        .with_released_quantity(0.0)
        .with_expires_at_ms(1_717_171_999)
        .with_created_at_ms(1_717_171_800)
        .with_updated_at_ms(1_717_171_801);

    assert_eq!(hold.hold_id, 8101);
    assert_eq!(hold.request_id, 6001);
    assert_eq!(hold.status, AccountHoldStatus::Held);
    assert_eq!(hold.estimated_quantity, 42.5);
    assert_eq!(hold.captured_quantity, 0.0);
    assert_eq!(hold.released_quantity, 0.0);
}

#[test]
fn ledger_entry_and_request_settlement_keep_request_level_evidence() {
    let ledger_entry = AccountLedgerEntryRecord::new(8201, 1001, 2002, 7001, 9001, AccountLedgerEntryType::HoldCreate)
        .with_request_id(Some(6001))
        .with_hold_id(Some(8101))
        .with_benefit_type(Some("cash_credit".to_owned()))
        .with_quantity(42.5)
        .with_amount(42.5)
        .with_created_at_ms(1_717_171_810);
    let settlement = RequestSettlementRecord::new(8301, 1001, 2002, 6001, 7001, 9001)
        .with_hold_id(Some(8101))
        .with_status(RequestSettlementStatus::Captured)
        .with_estimated_credit_hold(42.5)
        .with_released_credit_amount(2.5)
        .with_captured_credit_amount(40.0)
        .with_provider_cost_amount(18.0)
        .with_retail_charge_amount(40.0)
        .with_shortfall_amount(0.0)
        .with_refunded_amount(0.0)
        .with_settled_at_ms(1_717_171_899)
        .with_created_at_ms(1_717_171_810)
        .with_updated_at_ms(1_717_171_899);

    assert_eq!(ledger_entry.entry_type, AccountLedgerEntryType::HoldCreate);
    assert_eq!(ledger_entry.request_id, Some(6001));
    assert_eq!(ledger_entry.hold_id, Some(8101));
    assert_eq!(ledger_entry.quantity, 42.5);
    assert_eq!(settlement.request_id, 6001);
    assert_eq!(settlement.hold_id, Some(8101));
    assert_eq!(settlement.status, RequestSettlementStatus::Captured);
    assert_eq!(settlement.captured_credit_amount, 40.0);
    assert_eq!(settlement.provider_cost_amount, 18.0);
}
