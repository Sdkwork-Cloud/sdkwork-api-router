use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponReservationRecord, CouponReservationStatus,
    CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord,
    MarketingCampaignStatus, MarketingSubjectScope,
};
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_storage_core::MarketingStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

use sdkwork_api_app_jobs::recover_expired_coupon_reservations;

#[tokio::test]
async fn expires_stale_coupon_reservations_and_releases_resources_idempotently() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let metrics = HttpMetricsRegistry::new("marketing-recovery-expired-reservations");
    let now_ms = 10_000;

    let template =
        CouponTemplateRecord::new("tpl_recovery_1", "growth-recovery", MarketingBenefitKind::GrantUnits)
            .with_display_name("Growth Recovery")
            .with_status(CouponTemplateStatus::Active)
            .with_distribution_kind(CouponDistributionKind::UniqueCode)
            .with_created_at_ms(100)
            .with_updated_at_ms(100);
    let campaign =
        MarketingCampaignRecord::new("cmp_recovery_1", template.coupon_template_id.clone())
            .with_display_name("Recovery Campaign")
            .with_status(MarketingCampaignStatus::Active)
            .with_start_at_ms(Some(0))
            .with_end_at_ms(Some(now_ms + 20_000))
            .with_created_at_ms(120)
            .with_updated_at_ms(120);
    let budget = CampaignBudgetRecord::new(
        "budget_recovery_1",
        campaign.marketing_campaign_id.clone(),
    )
    .with_status(CampaignBudgetStatus::Active)
    .with_total_budget_minor(50_000)
    .with_reserved_budget_minor(5_000)
    .with_created_at_ms(130)
    .with_updated_at_ms(130);

    let expired_code = CouponCodeRecord::new(
        "code_recovery_expired",
        template.coupon_template_id.clone(),
        "RECOVER-EXPIRED",
    )
    .with_status(CouponCodeStatus::Reserved)
    .with_expires_at_ms(Some(now_ms + 30_000))
    .with_created_at_ms(140)
    .with_updated_at_ms(140);
    let active_code = CouponCodeRecord::new(
        "code_recovery_active",
        template.coupon_template_id.clone(),
        "RECOVER-ACTIVE",
    )
    .with_status(CouponCodeStatus::Reserved)
    .with_expires_at_ms(Some(now_ms + 30_000))
    .with_created_at_ms(150)
    .with_updated_at_ms(150);

    let expired_reservation = CouponReservationRecord::new(
        "reservation_recovery_expired",
        expired_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-expired",
        now_ms - 1,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(3_000)
    .with_created_at_ms(160)
    .with_updated_at_ms(160);
    let active_reservation = CouponReservationRecord::new(
        "reservation_recovery_active",
        active_code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-active",
        now_ms + 5_000,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_budget_reserved_minor(2_000)
    .with_created_at_ms(170)
    .with_updated_at_ms(170);

    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();
    store.insert_campaign_budget_record(&budget).await.unwrap();
    store.insert_coupon_code_record(&expired_code).await.unwrap();
    store.insert_coupon_code_record(&active_code).await.unwrap();
    store
        .insert_coupon_reservation_record(&expired_reservation)
        .await
        .unwrap();
    store
        .insert_coupon_reservation_record(&active_reservation)
        .await
        .unwrap();

    let report = recover_expired_coupon_reservations(&store, Some(&metrics), now_ms)
        .await
        .unwrap();

    assert_eq!(report.scanned_reservations, 2);
    assert_eq!(report.expired_reservations, 1);
    assert_eq!(report.released_codes, 1);
    assert_eq!(report.released_budget_minor, 3_000);
    assert_eq!(report.outbox_events_created, 1);

    let recovered_reservation = store
        .find_coupon_reservation_record(&expired_reservation.coupon_reservation_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        recovered_reservation.reservation_status,
        CouponReservationStatus::Expired
    );
    assert_eq!(recovered_reservation.updated_at_ms, now_ms);

    let untouched_reservation = store
        .find_coupon_reservation_record(&active_reservation.coupon_reservation_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        untouched_reservation.reservation_status,
        CouponReservationStatus::Reserved
    );

    let recovered_code = store
        .find_coupon_code_record(&expired_code.coupon_code_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered_code.status, CouponCodeStatus::Available);
    assert_eq!(recovered_code.updated_at_ms, now_ms);

    let untouched_code = store
        .find_coupon_code_record(&active_code.coupon_code_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(untouched_code.status, CouponCodeStatus::Reserved);

    let recovered_budget = store
        .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(recovered_budget.reserved_budget_minor, 2_000);
    assert_eq!(recovered_budget.updated_at_ms, now_ms);

    let outbox_records = store.list_marketing_outbox_event_records().await.unwrap();
    assert_eq!(outbox_records.len(), 1);
    assert_eq!(outbox_records[0].aggregate_type, "coupon_reservation");
    assert_eq!(
        outbox_records[0].aggregate_id,
        expired_reservation.coupon_reservation_id
    );
    assert_eq!(outbox_records[0].event_type, "coupon.reservation.expired");
    assert!(
        outbox_records[0]
            .payload_json
            .contains(&expired_reservation.coupon_reservation_id)
    );

    let telemetry = metrics.render_prometheus();
    assert!(telemetry.contains(
        "sdkwork_marketing_recovery_attempts_total{service=\"marketing-recovery-expired-reservations\",outcome=\"success\"} 1"
    ));
    assert!(telemetry.contains(
        "sdkwork_marketing_expired_reservations_total{service=\"marketing-recovery-expired-reservations\"} 1"
    ));
    assert!(telemetry.contains(
        "sdkwork_marketing_released_codes_total{service=\"marketing-recovery-expired-reservations\"} 1"
    ));
    assert!(telemetry.contains(
        "sdkwork_marketing_released_budget_minor_total{service=\"marketing-recovery-expired-reservations\"} 3000"
    ));
    assert!(telemetry.contains(
        "sdkwork_marketing_recovery_outbox_events_total{service=\"marketing-recovery-expired-reservations\"} 1"
    ));
    assert!(telemetry.contains(
        "sdkwork_marketing_recovery_last_success_at_ms{service=\"marketing-recovery-expired-reservations\"} 10000"
    ));

    let rerun = recover_expired_coupon_reservations(&store, Some(&metrics), now_ms + 1_000)
        .await
        .unwrap();
    assert_eq!(rerun.scanned_reservations, 2);
    assert_eq!(rerun.expired_reservations, 0);
    assert_eq!(rerun.released_codes, 0);
    assert_eq!(rerun.released_budget_minor, 0);
    assert_eq!(rerun.outbox_events_created, 0);
    assert_eq!(store.list_marketing_outbox_event_records().await.unwrap().len(), 1);
}
