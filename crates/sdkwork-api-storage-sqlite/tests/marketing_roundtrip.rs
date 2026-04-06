use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationRecord, CouponReservationStatus, CouponRollbackRecord, CouponRollbackStatus,
    CouponRollbackType, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingCampaignStatus, MarketingOutboxEventRecord,
    MarketingOutboxEventStatus, MarketingStackingPolicy, MarketingSubjectScope,
};
use sdkwork_api_storage_core::MarketingStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_round_trips_marketing_kernel_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        "tpl_launch_1",
        "launch-boost",
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name("Launch Boost")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_benefit(
        sdkwork_api_domain_marketing::CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
            .with_discount_percent(Some(20))
            .with_max_discount_minor(Some(5_000))
            .with_currency_code(Some("USD".to_owned())),
    )
    .with_restriction(
        sdkwork_api_domain_marketing::CouponRestrictionSpec::new(MarketingSubjectScope::Project)
            .with_min_order_amount_minor(Some(10_000))
            .with_exclusive_group(Some("launch".to_owned()))
            .with_stacking_policy(MarketingStackingPolicy::Exclusive)
            .with_eligible_target_kinds(vec![
                "workspace_recharge".to_owned(),
                "subscription".to_owned(),
            ]),
    )
    .with_created_at_ms(100)
    .with_updated_at_ms(110);

    let campaign =
        MarketingCampaignRecord::new("cmp_launch_1", template.coupon_template_id.clone())
            .with_display_name("Launch Week Campaign")
            .with_status(MarketingCampaignStatus::Active)
            .with_start_at_ms(Some(1_000))
            .with_end_at_ms(Some(9_000))
            .with_created_at_ms(120)
            .with_updated_at_ms(130);

    let budget =
        CampaignBudgetRecord::new("budget_launch_1", campaign.marketing_campaign_id.clone())
            .with_status(CampaignBudgetStatus::Active)
            .with_total_budget_minor(100_000)
            .with_reserved_budget_minor(20_000)
            .with_consumed_budget_minor(5_000)
            .with_created_at_ms(140)
            .with_updated_at_ms(150);

    let redeemable_code = CouponCodeRecord::new(
        "code_launch_live",
        template.coupon_template_id.clone(),
        "LAUNCH20",
    )
    .with_status(CouponCodeStatus::Available)
    .with_expires_at_ms(Some(6_000))
    .with_created_at_ms(160)
    .with_updated_at_ms(170);
    let disabled_code = CouponCodeRecord::new(
        "code_launch_disabled",
        template.coupon_template_id.clone(),
        "LAUNCHX",
    )
    .with_status(CouponCodeStatus::Disabled)
    .with_created_at_ms(161)
    .with_updated_at_ms(171);

    let active_reservation = CouponReservationRecord::new(
        "reservation_launch_live",
        redeemable_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-1",
        4_000,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(2_000)
    .with_created_at_ms(180)
    .with_updated_at_ms(181);
    let released_reservation = CouponReservationRecord::new(
        "reservation_launch_released",
        disabled_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-2",
        4_000,
    )
    .with_status(CouponReservationStatus::Released)
    .with_budget_reserved_minor(1_000)
    .with_created_at_ms(182)
    .with_updated_at_ms(183);

    let redemption = CouponRedemptionRecord::new(
        "redemption_launch_1",
        active_reservation.coupon_reservation_id.clone(),
        redeemable_code.coupon_code_id.clone(),
        template.coupon_template_id.clone(),
        190,
    )
    .with_status(CouponRedemptionStatus::Redeemed)
    .with_subsidy_amount_minor(2_000)
    .with_order_id(Some("order-1".to_owned()))
    .with_payment_event_id(Some("payment-1".to_owned()))
    .with_updated_at_ms(191);

    let rollback = CouponRollbackRecord::new(
        "rollback_launch_1",
        redemption.coupon_redemption_id.clone(),
        CouponRollbackType::Refund,
        200,
    )
    .with_status(CouponRollbackStatus::Completed)
    .with_restored_budget_minor(2_000)
    .with_restored_inventory_count(1)
    .with_updated_at_ms(201);

    let outbox = MarketingOutboxEventRecord::new(
        "outbox_launch_1",
        "coupon_redemption",
        redemption.coupon_redemption_id.clone(),
        "coupon.redemption.redeemed",
        "{\"redemption_id\":\"redemption_launch_1\"}",
        210,
    )
    .with_status(MarketingOutboxEventStatus::Pending)
    .with_updated_at_ms(211);

    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();
    store.insert_campaign_budget_record(&budget).await.unwrap();
    store
        .insert_coupon_code_record(&redeemable_code)
        .await
        .unwrap();
    store
        .insert_coupon_code_record(&disabled_code)
        .await
        .unwrap();
    store
        .insert_coupon_reservation_record(&active_reservation)
        .await
        .unwrap();
    store
        .insert_coupon_reservation_record(&released_reservation)
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(&redemption)
        .await
        .unwrap();
    store
        .insert_coupon_rollback_record(&rollback)
        .await
        .unwrap();
    store
        .insert_marketing_outbox_event_record(&outbox)
        .await
        .unwrap();

    assert_eq!(
        store
            .find_coupon_template_record(&template.coupon_template_id)
            .await
            .unwrap(),
        Some(template.clone())
    );
    assert_eq!(
        store
            .find_coupon_template_record_by_template_key(&template.template_key)
            .await
            .unwrap(),
        Some(template.clone())
    );
    assert_eq!(
        store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .unwrap(),
        vec![campaign.clone()]
    );
    assert_eq!(
        store
            .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
            .await
            .unwrap(),
        vec![budget.clone()]
    );
    assert_eq!(
        store
            .find_coupon_code_record(&redeemable_code.coupon_code_id)
            .await
            .unwrap(),
        Some(redeemable_code.clone())
    );
    assert_eq!(
        store
            .find_coupon_code_record_by_value(&redeemable_code.code_value)
            .await
            .unwrap(),
        Some(redeemable_code.clone())
    );
    assert_eq!(
        store
            .list_redeemable_coupon_code_records_at(5_000)
            .await
            .unwrap(),
        vec![redeemable_code.clone()]
    );
    assert_eq!(
        store
            .find_coupon_reservation_record(&active_reservation.coupon_reservation_id)
            .await
            .unwrap(),
        Some(active_reservation.clone())
    );
    assert_eq!(
        store
            .list_active_coupon_reservation_records_at(3_000)
            .await
            .unwrap(),
        vec![active_reservation.clone()]
    );
    assert_eq!(
        store
            .find_coupon_redemption_record(&redemption.coupon_redemption_id)
            .await
            .unwrap(),
        Some(redemption.clone())
    );
    assert_eq!(
        store.list_coupon_rollback_records().await.unwrap(),
        vec![rollback]
    );
    assert_eq!(
        store.list_marketing_outbox_event_records().await.unwrap(),
        vec![outbox]
    );
}
