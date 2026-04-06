use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldAllocationRecord, AccountHoldRecord, AccountHoldStatus, AccountRecord, AccountType,
    RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_storage_core::{AccountKernelStore, AccountKernelTransactionExecutor};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn account_kernel_transaction_creates_hold_and_allocations_atomically() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_account_with_credit_lot(&store).await;

    let created_hold = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let account = tx.find_account_record(7001).await?.unwrap();
                let lot = tx.find_account_benefit_lot(8001).await?.unwrap();
                let held_quantity = 40.0;

                tx.upsert_account_benefit_lot(
                    &lot.clone()
                        .with_held_quantity(lot.held_quantity + held_quantity)
                        .with_updated_at_ms(35),
                )
                .await?;

                let hold = AccountHoldRecord::new(
                    8101,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    6001,
                )
                .with_estimated_quantity(held_quantity)
                .with_expires_at_ms(120)
                .with_created_at_ms(35)
                .with_updated_at_ms(35);
                tx.upsert_account_hold(&hold).await?;

                let allocation = AccountHoldAllocationRecord::new(8201, 1001, 2002, 8101, 8001)
                    .with_allocated_quantity(held_quantity)
                    .with_created_at_ms(35)
                    .with_updated_at_ms(35);
                tx.upsert_account_hold_allocation(&allocation).await?;

                Ok(hold)
            })
        })
        .await
        .unwrap();

    let lots = store.list_account_benefit_lots().await.unwrap();
    let holds = store.list_account_holds().await.unwrap();
    let allocations = store.list_account_hold_allocations().await.unwrap();

    assert_eq!(created_hold.hold_id, 8101);
    assert_eq!(holds.len(), 1);
    assert_eq!(allocations.len(), 1);
    assert_eq!(lots[0].remaining_quantity, 100.0);
    assert_eq!(lots[0].held_quantity, 40.0);
    assert_eq!(allocations[0].allocated_quantity, 40.0);
}

#[tokio::test]
async fn account_kernel_transaction_releases_existing_hold_and_restores_available_balance() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_held_account(&store).await;

    let released_hold = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx.find_account_hold_by_request_id(6001).await?.unwrap();
                let allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;

                for allocation in &allocations {
                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .unwrap();
                    tx.upsert_account_benefit_lot(
                        &lot.clone()
                            .with_held_quantity(lot.held_quantity - allocation.allocated_quantity)
                            .with_updated_at_ms(60),
                    )
                    .await?;

                    tx.upsert_account_hold_allocation(
                        &allocation
                            .clone()
                            .with_released_quantity(allocation.allocated_quantity)
                            .with_updated_at_ms(60),
                    )
                    .await?;
                }

                let released_hold = hold
                    .with_status(AccountHoldStatus::Released)
                    .with_released_quantity(40.0)
                    .with_updated_at_ms(60);
                tx.upsert_account_hold(&released_hold).await?;

                Ok(released_hold)
            })
        })
        .await
        .unwrap();

    let lot = store.list_account_benefit_lots().await.unwrap().remove(0);
    let hold = store.list_account_holds().await.unwrap().remove(0);
    let allocation = store
        .list_account_hold_allocations()
        .await
        .unwrap()
        .remove(0);

    assert_eq!(released_hold.status, AccountHoldStatus::Released);
    assert_eq!(hold.released_quantity, 40.0);
    assert_eq!(allocation.released_quantity, 40.0);
    assert_eq!(lot.held_quantity, 0.0);
    assert_eq!(lot.remaining_quantity, 100.0);
}

#[tokio::test]
async fn account_kernel_transaction_makes_request_settlement_idempotent_by_request_id() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_held_account(&store).await;

    let first = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                if let Some(existing) = tx.find_request_settlement_by_request_id(6001).await? {
                    return Ok((false, existing));
                }

                let settlement = RequestSettlementRecord::new(8301, 1001, 2002, 6001, 7001, 9001)
                    .with_hold_id(Some(8101))
                    .with_status(RequestSettlementStatus::Captured)
                    .with_estimated_credit_hold(40.0)
                    .with_captured_credit_amount(40.0)
                    .with_retail_charge_amount(40.0)
                    .with_settled_at_ms(75)
                    .with_created_at_ms(75)
                    .with_updated_at_ms(75);
                let inserted = tx.upsert_request_settlement_record(&settlement).await?;

                Ok((true, inserted))
            })
        })
        .await
        .unwrap();

    let second = store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                if let Some(existing) = tx.find_request_settlement_by_request_id(6001).await? {
                    return Ok((false, existing));
                }

                let settlement = RequestSettlementRecord::new(8302, 1001, 2002, 6001, 7001, 9001)
                    .with_hold_id(Some(8101))
                    .with_status(RequestSettlementStatus::Captured)
                    .with_estimated_credit_hold(40.0)
                    .with_captured_credit_amount(40.0)
                    .with_retail_charge_amount(40.0)
                    .with_settled_at_ms(80)
                    .with_created_at_ms(80)
                    .with_updated_at_ms(80);
                let inserted = tx.upsert_request_settlement_record(&settlement).await?;

                Ok((true, inserted))
            })
        })
        .await
        .unwrap();

    let settlements = store.list_request_settlement_records().await.unwrap();

    assert!(first.0);
    assert_eq!(first.1.request_settlement_id, 8301);
    assert!(!second.0);
    assert_eq!(second.1.request_settlement_id, 8301);
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].request_settlement_id, 8301);
}

async fn seed_account_with_credit_lot(store: &SqliteAdminStore) {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let lot =
        AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(100.0)
            .with_remaining_quantity(100.0)
            .with_created_at_ms(20)
            .with_updated_at_ms(20);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&lot).await.unwrap();
}

async fn seed_held_account(store: &SqliteAdminStore) {
    seed_account_with_credit_lot(store).await;

    let held_lot =
        AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(100.0)
            .with_remaining_quantity(100.0)
            .with_held_quantity(40.0)
            .with_created_at_ms(20)
            .with_updated_at_ms(35);
    let hold = AccountHoldRecord::new(8101, 1001, 2002, 7001, 9001, 6001)
        .with_estimated_quantity(40.0)
        .with_expires_at_ms(120)
        .with_created_at_ms(35)
        .with_updated_at_ms(35);
    let allocation = AccountHoldAllocationRecord::new(8201, 1001, 2002, 8101, 8001)
        .with_allocated_quantity(40.0)
        .with_created_at_ms(35)
        .with_updated_at_ms(35);

    store.insert_account_benefit_lot(&held_lot).await.unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_account_hold_allocation(&allocation)
        .await
        .unwrap();
}
