use sdkwork_api_app_billing::{
    capture_account_hold, create_account_hold, issue_commerce_order_credits,
    refund_account_settlement, refund_commerce_order_credits, release_account_hold,
    CaptureAccountHoldInput, CreateAccountHoldInput, IssueCommerceOrderCreditsInput,
    RefundAccountSettlementInput, RefundCommerceOrderCreditsInput, ReleaseAccountHoldInput,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldStatus, AccountLedgerEntryType, AccountRecord, AccountType, RequestSettlementStatus,
};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn create_account_hold_reserves_balance_and_replays_existing_request() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_account_with_two_lots(&store).await;

    let created = create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 8101,
            hold_allocation_start_id: 8201,
            request_id: 6001,
            account_id: 7001,
            requested_quantity: 60.0,
            expires_at_ms: 200,
            now_ms: 50,
        },
    )
    .await
    .unwrap();

    let replayed = create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 9101,
            hold_allocation_start_id: 9201,
            request_id: 6001,
            account_id: 7001,
            requested_quantity: 60.0,
            expires_at_ms: 200,
            now_ms: 50,
        },
    )
    .await
    .unwrap();

    assert!(!created.idempotent_replay);
    assert_eq!(created.hold.hold_id, 8101);
    assert_eq!(created.hold.estimated_quantity, 60.0);
    assert_eq!(created.allocations.len(), 2);
    assert_eq!(
        created
            .updated_lots
            .iter()
            .map(|lot| lot.held_quantity)
            .sum::<f64>(),
        60.0
    );

    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.hold.hold_id, 8101);
    assert_eq!(replayed.allocations.len(), 2);
    assert_eq!(
        store
            .list_account_benefit_lots()
            .await
            .unwrap()
            .iter()
            .map(|lot| lot.held_quantity)
            .sum::<f64>(),
        60.0
    );
    let mut ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    ledger_entries.sort_by_key(|entry| (entry.created_at_ms, entry.ledger_entry_id));
    assert_eq!(
        ledger_entries
            .iter()
            .map(|entry| entry.entry_type)
            .collect::<Vec<_>>(),
        vec![AccountLedgerEntryType::HoldCreate]
    );
    assert_eq!(ledger_entries[0].quantity, 60.0);
    assert_eq!(ledger_allocations.len(), 2);
}

#[tokio::test]
async fn create_account_hold_rejects_insufficient_balance() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_single_lot_account(&store, 40.0).await;

    let error = create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 8101,
            hold_allocation_start_id: 8201,
            request_id: 6001,
            account_id: 7001,
            requested_quantity: 55.0,
            expires_at_ms: 200,
            now_ms: 50,
        },
    )
    .await
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "account 7001 has insufficient available balance for request 6001"
    );
}

#[tokio::test]
async fn release_account_hold_restores_held_balance_without_consuming_lots() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_account_with_two_lots(&store).await;

    create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 8101,
            hold_allocation_start_id: 8201,
            request_id: 6001,
            account_id: 7001,
            requested_quantity: 60.0,
            expires_at_ms: 200,
            now_ms: 50,
        },
    )
    .await
    .unwrap();

    let released = release_account_hold(
        &store,
        ReleaseAccountHoldInput {
            request_id: 6001,
            released_at_ms: 75,
        },
    )
    .await
    .unwrap();

    assert_eq!(released.hold.status, AccountHoldStatus::Released);
    assert_eq!(released.hold.released_quantity, 60.0);
    assert_eq!(
        released
            .updated_lots
            .iter()
            .map(|lot| lot.held_quantity)
            .sum::<f64>(),
        0.0
    );
    assert_eq!(
        released
            .updated_lots
            .iter()
            .map(|lot| lot.remaining_quantity)
            .sum::<f64>(),
        90.0
    );
    let mut ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    ledger_entries.sort_by_key(|entry| (entry.created_at_ms, entry.ledger_entry_id));
    assert_eq!(
        ledger_entries
            .iter()
            .map(|entry| entry.entry_type)
            .collect::<Vec<_>>(),
        vec![
            AccountLedgerEntryType::HoldCreate,
            AccountLedgerEntryType::HoldRelease,
        ]
    );
    assert_eq!(ledger_entries[1].quantity, 60.0);
    assert_eq!(ledger_allocations.len(), 4);
}

#[tokio::test]
async fn capture_account_hold_consumes_balance_releases_remainder_and_is_idempotent() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_account_with_two_lots(&store).await;

    create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 8101,
            hold_allocation_start_id: 8201,
            request_id: 6001,
            account_id: 7001,
            requested_quantity: 60.0,
            expires_at_ms: 200,
            now_ms: 50,
        },
    )
    .await
    .unwrap();

    let captured = capture_account_hold(
        &store,
        CaptureAccountHoldInput {
            request_settlement_id: 8301,
            request_id: 6001,
            captured_quantity: 45.0,
            provider_cost_amount: 18.0,
            retail_charge_amount: 45.0,
            settled_at_ms: 90,
        },
    )
    .await
    .unwrap();

    let replayed = capture_account_hold(
        &store,
        CaptureAccountHoldInput {
            request_settlement_id: 9301,
            request_id: 6001,
            captured_quantity: 45.0,
            provider_cost_amount: 18.0,
            retail_charge_amount: 45.0,
            settled_at_ms: 95,
        },
    )
    .await
    .unwrap();

    assert!(!captured.idempotent_replay);
    assert_eq!(captured.hold.captured_quantity, 45.0);
    assert_eq!(captured.hold.released_quantity, 15.0);
    assert_eq!(captured.hold.status, AccountHoldStatus::PartiallyReleased);
    assert_eq!(
        captured.settlement.status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_eq!(captured.settlement.request_settlement_id, 8301);
    assert_eq!(
        captured
            .updated_lots
            .iter()
            .map(|lot| lot.held_quantity)
            .sum::<f64>(),
        0.0
    );
    assert_eq!(
        captured
            .updated_lots
            .iter()
            .map(|lot| lot.remaining_quantity)
            .sum::<f64>(),
        45.0
    );

    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.settlement.request_settlement_id, 8301);
    assert_eq!(
        store.list_request_settlement_records().await.unwrap().len(),
        1
    );
    let mut ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    ledger_entries.sort_by_key(|entry| (entry.created_at_ms, entry.ledger_entry_id));
    assert_eq!(
        ledger_entries
            .iter()
            .map(|entry| entry.entry_type)
            .collect::<Vec<_>>(),
        vec![
            AccountLedgerEntryType::HoldCreate,
            AccountLedgerEntryType::SettlementCapture,
            AccountLedgerEntryType::HoldRelease,
        ]
    );
    assert_eq!(ledger_entries[1].quantity, 45.0);
    assert_eq!(ledger_entries[2].quantity, 15.0);
    assert_eq!(ledger_allocations.len(), 5);
}

#[tokio::test]
async fn refund_account_settlement_restores_balance_records_history_and_is_idempotent() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_account_with_two_lots(&store).await;

    create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 8101,
            hold_allocation_start_id: 8201,
            request_id: 6001,
            account_id: 7001,
            requested_quantity: 60.0,
            expires_at_ms: 200,
            now_ms: 50,
        },
    )
    .await
    .unwrap();

    capture_account_hold(
        &store,
        CaptureAccountHoldInput {
            request_settlement_id: 8301,
            request_id: 6001,
            captured_quantity: 60.0,
            provider_cost_amount: 18.0,
            retail_charge_amount: 60.0,
            settled_at_ms: 90,
        },
    )
    .await
    .unwrap();

    let refunded = refund_account_settlement(
        &store,
        RefundAccountSettlementInput {
            request_settlement_id: 8301,
            refund_ledger_entry_id: 8401,
            refund_ledger_allocation_start_id: 8501,
            refunded_amount: 20.0,
            refunded_at_ms: 120,
        },
    )
    .await
    .unwrap();

    let replayed = refund_account_settlement(
        &store,
        RefundAccountSettlementInput {
            request_settlement_id: 8301,
            refund_ledger_entry_id: 8401,
            refund_ledger_allocation_start_id: 8501,
            refunded_amount: 20.0,
            refunded_at_ms: 125,
        },
    )
    .await
    .unwrap();

    assert!(!refunded.idempotent_replay);
    assert_eq!(
        refunded.settlement.status,
        RequestSettlementStatus::Refunded
    );
    assert_eq!(refunded.settlement.refunded_amount, 20.0);
    assert_eq!(refunded.updated_lots.len(), 1);
    assert_eq!(
        refunded.ledger_entry.entry_type,
        AccountLedgerEntryType::Refund
    );
    assert_eq!(refunded.ledger_entry.quantity, 20.0);
    assert_eq!(
        refunded
            .ledger_allocations
            .iter()
            .map(|allocation| allocation.quantity_delta)
            .sum::<f64>(),
        20.0
    );

    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.settlement.request_settlement_id, 8301);
    assert_eq!(replayed.ledger_entry.ledger_entry_id, 8401);

    let mut ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    ledger_entries.sort_by_key(|entry| (entry.created_at_ms, entry.ledger_entry_id));
    assert_eq!(
        ledger_entries
            .iter()
            .map(|entry| entry.entry_type)
            .collect::<Vec<_>>(),
        vec![
            AccountLedgerEntryType::HoldCreate,
            AccountLedgerEntryType::SettlementCapture,
            AccountLedgerEntryType::Refund,
        ]
    );
    assert_eq!(ledger_allocations.len(), 5);
    assert_eq!(
        store
            .list_account_benefit_lots()
            .await
            .unwrap()
            .iter()
            .map(|lot| lot.remaining_quantity)
            .sum::<f64>(),
        50.0
    );
}

#[tokio::test]
async fn issue_commerce_order_credits_creates_lot_and_replays_existing_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_empty_account(&store).await;

    let issued = issue_commerce_order_credits(
        &store,
        IssueCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_issue_1",
            project_id: "project_alpha",
            target_kind: "recharge_pack",
            granted_quantity: 100_000.0,
            payable_amount: 40.0,
            issued_at_ms: 50,
        },
    )
    .await
    .unwrap();

    let replayed = issue_commerce_order_credits(
        &store,
        IssueCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_issue_1",
            project_id: "project_alpha",
            target_kind: "recharge_pack",
            granted_quantity: 100_000.0,
            payable_amount: 40.0,
            issued_at_ms: 55,
        },
    )
    .await
    .unwrap();

    assert!(!issued.idempotent_replay);
    assert_eq!(issued.lot.benefit_type, AccountBenefitType::CashCredit);
    assert_eq!(issued.lot.source_type, AccountBenefitSourceType::Order);
    assert_eq!(issued.lot.original_quantity, 100_000.0);
    assert_eq!(issued.lot.remaining_quantity, 100_000.0);
    assert_eq!(issued.lot.status, AccountBenefitLotStatus::Active);
    assert_eq!(
        issued.ledger_entry.entry_type,
        AccountLedgerEntryType::GrantIssue
    );
    assert_eq!(issued.ledger_entry.quantity, 100_000.0);
    assert_eq!(issued.ledger_entry.amount, 40.0);
    assert_eq!(issued.ledger_allocations.len(), 1);
    assert_eq!(issued.ledger_allocations[0].quantity_delta, 100_000.0);

    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.lot.lot_id, issued.lot.lot_id);
    assert_eq!(
        replayed.ledger_entry.ledger_entry_id,
        issued.ledger_entry.ledger_entry_id
    );
    assert_eq!(store.list_account_benefit_lots().await.unwrap().len(), 1);
    assert_eq!(
        store
            .list_account_ledger_entry_records()
            .await
            .unwrap()
            .iter()
            .map(|entry| entry.entry_type)
            .collect::<Vec<_>>(),
        vec![AccountLedgerEntryType::GrantIssue]
    );
}

#[tokio::test]
async fn refund_commerce_order_credits_disables_issued_lot_and_replays_existing_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_empty_account(&store).await;

    issue_commerce_order_credits(
        &store,
        IssueCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_refund_1",
            project_id: "project_alpha",
            target_kind: "recharge_pack",
            granted_quantity: 100_000.0,
            payable_amount: 40.0,
            issued_at_ms: 50,
        },
    )
    .await
    .unwrap();

    let refunded = refund_commerce_order_credits(
        &store,
        RefundCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_refund_1",
            refunded_quantity: 100_000.0,
            refunded_amount: 40.0,
            refunded_at_ms: 60,
        },
    )
    .await
    .unwrap();

    let replayed = refund_commerce_order_credits(
        &store,
        RefundCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_refund_1",
            refunded_quantity: 100_000.0,
            refunded_amount: 40.0,
            refunded_at_ms: 70,
        },
    )
    .await
    .unwrap();

    assert!(!refunded.idempotent_replay);
    assert_eq!(refunded.lot.status, AccountBenefitLotStatus::Disabled);
    assert_eq!(refunded.lot.original_quantity, 0.0);
    assert_eq!(refunded.lot.remaining_quantity, 0.0);
    assert_eq!(
        refunded.ledger_entry.entry_type,
        AccountLedgerEntryType::Refund
    );
    assert_eq!(refunded.ledger_entry.quantity, 100_000.0);
    assert_eq!(refunded.ledger_entry.amount, 40.0);
    assert_eq!(refunded.ledger_allocations.len(), 1);
    assert_eq!(refunded.ledger_allocations[0].quantity_delta, -100_000.0);

    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.lot.lot_id, refunded.lot.lot_id);
    assert_eq!(
        replayed.ledger_entry.ledger_entry_id,
        refunded.ledger_entry.ledger_entry_id
    );
    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(
        ledger_entries
            .iter()
            .map(|entry| entry.entry_type)
            .collect::<Vec<_>>(),
        vec![
            AccountLedgerEntryType::Refund,
            AccountLedgerEntryType::GrantIssue
        ]
    );
}

#[tokio::test]
async fn issue_commerce_order_credits_replays_after_refund_tombstones_the_lot() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_empty_account(&store).await;

    let issued = issue_commerce_order_credits(
        &store,
        IssueCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_refund_replay_1",
            project_id: "project_alpha",
            target_kind: "recharge_pack",
            granted_quantity: 100_000.0,
            payable_amount: 40.0,
            issued_at_ms: 50,
        },
    )
    .await
    .unwrap();

    refund_commerce_order_credits(
        &store,
        RefundCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_refund_replay_1",
            refunded_quantity: 100_000.0,
            refunded_amount: 40.0,
            refunded_at_ms: 60,
        },
    )
    .await
    .unwrap();

    let replayed_issue = issue_commerce_order_credits(
        &store,
        IssueCommerceOrderCreditsInput {
            account_id: 7001,
            order_id: "commerce_order_refund_replay_1",
            project_id: "project_alpha",
            target_kind: "recharge_pack",
            granted_quantity: 100_000.0,
            payable_amount: 40.0,
            issued_at_ms: 70,
        },
    )
    .await
    .unwrap();

    assert!(replayed_issue.idempotent_replay);
    assert_eq!(
        replayed_issue.ledger_entry.ledger_entry_id,
        issued.ledger_entry.ledger_entry_id
    );
    assert_eq!(replayed_issue.lot.status, AccountBenefitLotStatus::Disabled);
    assert_eq!(replayed_issue.lot.original_quantity, 0.0);
    assert_eq!(replayed_issue.lot.remaining_quantity, 0.0);
}

async fn seed_account_with_two_lots(store: &SqliteAdminStore) {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let request_allowance = AccountBenefitLotRecord::new(
        8001,
        1001,
        2002,
        7001,
        9001,
        AccountBenefitType::RequestAllowance,
    )
    .with_source_type(AccountBenefitSourceType::Grant)
    .with_original_quantity(30.0)
    .with_remaining_quantity(30.0)
    .with_expires_at_ms(Some(150))
    .with_created_at_ms(20)
    .with_updated_at_ms(20);
    let cash_credit =
        AccountBenefitLotRecord::new(8002, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(60.0)
            .with_remaining_quantity(60.0)
            .with_created_at_ms(21)
            .with_updated_at_ms(21);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&request_allowance)
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(&cash_credit)
        .await
        .unwrap();
}

async fn seed_single_lot_account(store: &SqliteAdminStore, available_quantity: f64) {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let cash_credit =
        AccountBenefitLotRecord::new(8002, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(available_quantity)
            .with_remaining_quantity(available_quantity)
            .with_created_at_ms(21)
            .with_updated_at_ms(21);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&cash_credit)
        .await
        .unwrap();
}

async fn seed_empty_account(store: &SqliteAdminStore) {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);

    store.insert_account_record(&account).await.unwrap();
}
