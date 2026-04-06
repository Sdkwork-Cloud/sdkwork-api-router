use anyhow::anyhow;
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponReservationRecord, CouponReservationStatus, CouponTemplateRecord,
    CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord, MarketingCampaignStatus,
    MarketingOutboxEventRecord, MarketingSubjectScope,
};
use sdkwork_api_storage_core::{MarketingKernelTransactionExecutor, MarketingStore};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_marketing_transaction_rolls_back_all_writes_on_error() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template =
        CouponTemplateRecord::new("tpl_tx_1", "tx-recovery", MarketingBenefitKind::GrantUnits)
            .with_display_name("Tx Recovery")
            .with_status(CouponTemplateStatus::Active)
            .with_distribution_kind(CouponDistributionKind::UniqueCode)
            .with_created_at_ms(100)
            .with_updated_at_ms(100);
    let campaign = MarketingCampaignRecord::new("cmp_tx_1", template.coupon_template_id.clone())
        .with_display_name("Tx Campaign")
        .with_status(MarketingCampaignStatus::Active)
        .with_start_at_ms(Some(0))
        .with_end_at_ms(Some(50_000))
        .with_created_at_ms(110)
        .with_updated_at_ms(110);
    let budget = CampaignBudgetRecord::new("budget_tx_1", campaign.marketing_campaign_id.clone())
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(10_000)
        .with_reserved_budget_minor(5_000)
        .with_created_at_ms(120)
        .with_updated_at_ms(120);
    let code = CouponCodeRecord::new(
        "code_tx_1",
        template.coupon_template_id.clone(),
        "TX-ROLLBACK",
    )
    .with_status(CouponCodeStatus::Reserved)
    .with_created_at_ms(130)
    .with_updated_at_ms(130);
    let reservation = CouponReservationRecord::new(
        "reservation_tx_1",
        code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-1",
        9_000,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(3_000)
    .with_created_at_ms(140)
    .with_updated_at_ms(140);

    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();
    store.insert_campaign_budget_record(&budget).await.unwrap();
    store.insert_coupon_code_record(&code).await.unwrap();
    store
        .insert_coupon_reservation_record(&reservation)
        .await
        .unwrap();

    let reservation_id = reservation.coupon_reservation_id.clone();
    let code_id = code.coupon_code_id.clone();
    let campaign_id = campaign.marketing_campaign_id.clone();

    let failure = store
        .with_marketing_kernel_transaction(|tx| {
            Box::pin(async move {
                let mut updated_reservation = tx
                    .find_coupon_reservation_record(&reservation_id)
                    .await?
                    .unwrap();
                updated_reservation.reservation_status = CouponReservationStatus::Expired;
                updated_reservation.updated_at_ms = 10_000;
                tx.upsert_coupon_reservation_record(&updated_reservation)
                    .await?;

                let mut updated_code = tx.find_coupon_code_record(&code_id).await?.unwrap();
                updated_code.status = CouponCodeStatus::Available;
                updated_code.updated_at_ms = 10_000;
                tx.upsert_coupon_code_record(&updated_code).await?;

                let mut updated_budget = tx
                    .list_campaign_budget_records_for_campaign(&campaign_id)
                    .await?
                    .into_iter()
                    .next()
                    .unwrap();
                updated_budget.reserved_budget_minor = 2_000;
                updated_budget.updated_at_ms = 10_000;
                tx.upsert_campaign_budget_record(&updated_budget).await?;

                let outbox = MarketingOutboxEventRecord::new(
                    "outbox_tx_rollback",
                    "coupon_reservation",
                    reservation_id.clone(),
                    "coupon.reservation.expired",
                    "{\"coupon_reservation_id\":\"reservation_tx_1\"}",
                    10_000,
                );
                tx.upsert_marketing_outbox_event_record(&outbox).await?;

                Err::<Option<()>, _>(anyhow!("force rollback"))
            })
        })
        .await
        .unwrap_err();
    assert_eq!(failure.to_string(), "force rollback");

    let persisted_reservation = store
        .find_coupon_reservation_record(&reservation.coupon_reservation_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        persisted_reservation.reservation_status,
        CouponReservationStatus::Reserved
    );
    assert_eq!(
        persisted_reservation.updated_at_ms,
        reservation.updated_at_ms
    );

    let persisted_code = store
        .find_coupon_code_record(&code.coupon_code_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted_code.status, CouponCodeStatus::Reserved);
    assert_eq!(persisted_code.updated_at_ms, code.updated_at_ms);

    let persisted_budget = store
        .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(persisted_budget.reserved_budget_minor, 5_000);
    assert_eq!(persisted_budget.updated_at_ms, budget.updated_at_ms);

    assert!(store
        .list_marketing_outbox_event_records()
        .await
        .unwrap()
        .is_empty());
}
