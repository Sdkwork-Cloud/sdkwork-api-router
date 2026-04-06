use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationRecord, CouponReservationStatus, CouponRollbackRecord, CouponRollbackStatus,
    CouponRollbackType, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingCampaignStatus, MarketingSubjectScope,
};
use sdkwork_api_storage_core::{
    AdminStore, AtomicCouponConfirmationCommand, AtomicCouponReservationCommand,
    AtomicCouponRollbackCommand,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

const CREATED_AT_MS: u64 = 1_710_000_000_000;
const RESERVED_AT_MS: u64 = 1_710_000_010_000;
const CONFIRMED_AT_MS: u64 = 1_710_000_020_000;
const ROLLED_BACK_AT_MS: u64 = 1_710_000_030_000;
const RESERVATION_TTL_MS: u64 = 300_000;
const RESERVED_BUDGET_MINOR: u64 = 1_200;

struct CouponFixture {
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[tokio::test]
async fn sqlite_store_executes_atomic_coupon_reservation_with_idempotency() {
    let store = build_store().await;
    let fixture = seed_coupon_fixture(&store, "LAUNCH20").await;
    let command = reserve_command(&fixture, "reservation_launch20_project_1");

    let first = AdminStore::reserve_coupon_redemption_atomic(&store, &command)
        .await
        .expect("first atomic reservation should succeed");
    assert!(first.created);
    assert_eq!(first.budget, command.next_budget);
    assert_eq!(first.code, command.next_code);
    assert_eq!(first.reservation, command.reservation);

    let second = AdminStore::reserve_coupon_redemption_atomic(&store, &command)
        .await
        .expect("second atomic reservation should be idempotent");
    assert!(!second.created);
    assert_eq!(second.budget, command.next_budget);
    assert_eq!(second.code, command.next_code);
    assert_eq!(second.reservation, command.reservation);

    let reservations = AdminStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records");
    assert_eq!(reservations.len(), 1);
    assert_eq!(
        reservations[0].reservation_status,
        CouponReservationStatus::Reserved
    );
}

#[tokio::test]
async fn sqlite_store_rejects_stale_atomic_coupon_reservation_snapshots() {
    let store = build_store().await;
    let fixture = seed_coupon_fixture(&store, "STALE20").await;

    let stale_budget = fixture
        .budget
        .clone()
        .with_reserved_budget_minor(300)
        .with_updated_at_ms(CREATED_AT_MS + 5_000);
    AdminStore::insert_campaign_budget_record(&store, &stale_budget)
        .await
        .expect("mutate budget before atomic reservation");

    let failure = AdminStore::reserve_coupon_redemption_atomic(
        &store,
        &reserve_command(&fixture, "reservation_stale20_project_1"),
    )
    .await
    .expect_err("stale atomic reservation command should fail");
    assert!(
        failure.to_string().contains("changed concurrently"),
        "unexpected failure: {failure}"
    );

    let reservations = AdminStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records after stale failure");
    assert!(reservations.is_empty());

    let stored_code = AdminStore::find_coupon_code_record(&store, &fixture.code.coupon_code_id)
        .await
        .expect("code lookup after stale failure")
        .expect("stored code exists");
    assert_eq!(stored_code.status, CouponCodeStatus::Available);

    let stored_budget = select_budget(
        AdminStore::list_campaign_budget_records_for_campaign(
            &store,
            &fixture.campaign.marketing_campaign_id,
        )
        .await
        .expect("budget records after stale failure"),
    );
    assert_eq!(stored_budget, stale_budget);
}

#[tokio::test]
async fn sqlite_store_executes_atomic_coupon_confirmation_and_rollback() {
    let store = build_store().await;
    let fixture = seed_coupon_fixture(&store, "ROLL20").await;
    let reservation_command = reserve_command(&fixture, "reservation_roll20_project_1");
    let reserved = AdminStore::reserve_coupon_redemption_atomic(&store, &reservation_command)
        .await
        .expect("atomic reservation should succeed");
    assert!(reserved.created);

    let confirm_command = confirm_command(&fixture, &reservation_command);
    let confirmed = AdminStore::confirm_coupon_redemption_atomic(&store, &confirm_command)
        .await
        .expect("atomic confirmation should succeed");
    assert!(confirmed.created);
    assert_eq!(confirmed.budget, confirm_command.next_budget);
    assert_eq!(confirmed.code, confirm_command.next_code);
    assert_eq!(confirmed.reservation, confirm_command.next_reservation);
    assert_eq!(confirmed.redemption, confirm_command.redemption);

    let confirmed_again = AdminStore::confirm_coupon_redemption_atomic(&store, &confirm_command)
        .await
        .expect("replayed confirmation should be idempotent");
    assert!(!confirmed_again.created);
    assert_eq!(confirmed_again.redemption, confirm_command.redemption);

    let rollback_command = rollback_command(&confirm_command);
    let rolled_back = AdminStore::rollback_coupon_redemption_atomic(&store, &rollback_command)
        .await
        .expect("atomic rollback should succeed");
    assert!(rolled_back.created);
    assert_eq!(rolled_back.budget, rollback_command.next_budget);
    assert_eq!(rolled_back.code, rollback_command.next_code);
    assert_eq!(rolled_back.redemption, rollback_command.next_redemption);
    assert_eq!(rolled_back.rollback, rollback_command.rollback);

    let rolled_back_again =
        AdminStore::rollback_coupon_redemption_atomic(&store, &rollback_command)
            .await
            .expect("replayed rollback should be idempotent");
    assert!(!rolled_back_again.created);
    assert_eq!(rolled_back_again.rollback, rollback_command.rollback);
}

async fn build_store() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:")
        .await
        .expect("sqlite migrations");
    SqliteAdminStore::new(pool)
}

async fn seed_coupon_fixture(store: &SqliteAdminStore, code: &str) -> CouponFixture {
    let template = CouponTemplateRecord::new(
        format!("template_{code}"),
        code,
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name(format!("{code} coupon"))
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff).with_discount_percent(Some(20)),
    )
    .with_created_at_ms(CREATED_AT_MS)
    .with_updated_at_ms(CREATED_AT_MS);
    AdminStore::insert_coupon_template_record(store, &template)
        .await
        .expect("insert template");

    let campaign = MarketingCampaignRecord::new(
        format!("campaign_{code}"),
        template.coupon_template_id.clone(),
    )
    .with_display_name(format!("{code} campaign"))
    .with_status(MarketingCampaignStatus::Active)
    .with_created_at_ms(CREATED_AT_MS)
    .with_updated_at_ms(CREATED_AT_MS);
    AdminStore::insert_marketing_campaign_record(store, &campaign)
        .await
        .expect("insert campaign");

    let budget = CampaignBudgetRecord::new(
        format!("budget_{code}"),
        campaign.marketing_campaign_id.clone(),
    )
    .with_status(CampaignBudgetStatus::Active)
    .with_total_budget_minor(5_000)
    .with_created_at_ms(CREATED_AT_MS)
    .with_updated_at_ms(CREATED_AT_MS);
    AdminStore::insert_campaign_budget_record(store, &budget)
        .await
        .expect("insert budget");

    let coupon_code = CouponCodeRecord::new(
        format!("coupon_code_{code}"),
        template.coupon_template_id.clone(),
        code,
    )
    .with_status(CouponCodeStatus::Available)
    .with_created_at_ms(CREATED_AT_MS)
    .with_updated_at_ms(CREATED_AT_MS);
    AdminStore::insert_coupon_code_record(store, &coupon_code)
        .await
        .expect("insert coupon code");

    CouponFixture {
        template,
        campaign,
        budget,
        code: coupon_code,
    }
}

fn reserve_command(
    fixture: &CouponFixture,
    reservation_id: &str,
) -> AtomicCouponReservationCommand {
    let next_budget = fixture
        .budget
        .clone()
        .with_reserved_budget_minor(RESERVED_BUDGET_MINOR)
        .with_updated_at_ms(RESERVED_AT_MS);
    let next_code = fixture
        .code
        .clone()
        .with_status(CouponCodeStatus::Reserved)
        .with_updated_at_ms(RESERVED_AT_MS);
    let reservation = CouponReservationRecord::new(
        reservation_id,
        fixture.code.coupon_code_id.clone(),
        MarketingSubjectScope::Project,
        "project-1",
        RESERVED_AT_MS + RESERVATION_TTL_MS,
    )
    .with_budget_reserved_minor(RESERVED_BUDGET_MINOR)
    .with_created_at_ms(RESERVED_AT_MS)
    .with_updated_at_ms(RESERVED_AT_MS);

    AtomicCouponReservationCommand {
        template_to_persist: None,
        campaign_to_persist: None,
        expected_budget: fixture.budget.clone(),
        next_budget,
        expected_code: fixture.code.clone(),
        next_code,
        reservation,
    }
}

fn confirm_command(
    fixture: &CouponFixture,
    reservation_command: &AtomicCouponReservationCommand,
) -> AtomicCouponConfirmationCommand {
    let next_budget = reservation_command
        .next_budget
        .clone()
        .with_reserved_budget_minor(0)
        .with_consumed_budget_minor(RESERVED_BUDGET_MINOR)
        .with_updated_at_ms(CONFIRMED_AT_MS);
    let next_code = reservation_command
        .next_code
        .clone()
        .with_status(CouponCodeStatus::Redeemed)
        .with_updated_at_ms(CONFIRMED_AT_MS);
    let next_reservation = reservation_command
        .reservation
        .clone()
        .with_status(CouponReservationStatus::Confirmed)
        .with_updated_at_ms(CONFIRMED_AT_MS);
    let redemption = CouponRedemptionRecord::new(
        "redemption_roll20_project_1",
        reservation_command
            .reservation
            .coupon_reservation_id
            .clone(),
        fixture.code.coupon_code_id.clone(),
        fixture.template.coupon_template_id.clone(),
        CONFIRMED_AT_MS,
    )
    .with_status(CouponRedemptionStatus::Redeemed)
    .with_subsidy_amount_minor(RESERVED_BUDGET_MINOR)
    .with_order_id(Some("order_roll20".to_owned()))
    .with_payment_event_id(Some("payment_roll20".to_owned()))
    .with_updated_at_ms(CONFIRMED_AT_MS);

    AtomicCouponConfirmationCommand {
        expected_budget: reservation_command.next_budget.clone(),
        next_budget,
        expected_code: reservation_command.next_code.clone(),
        next_code,
        expected_reservation: reservation_command.reservation.clone(),
        next_reservation,
        redemption,
    }
}

fn rollback_command(
    confirm_command: &AtomicCouponConfirmationCommand,
) -> AtomicCouponRollbackCommand {
    let next_budget = confirm_command
        .next_budget
        .clone()
        .with_consumed_budget_minor(0)
        .with_updated_at_ms(ROLLED_BACK_AT_MS);
    let next_code = confirm_command
        .next_code
        .clone()
        .with_status(CouponCodeStatus::Available)
        .with_updated_at_ms(ROLLED_BACK_AT_MS);
    let next_redemption = confirm_command
        .redemption
        .clone()
        .with_status(CouponRedemptionStatus::RolledBack)
        .with_updated_at_ms(ROLLED_BACK_AT_MS);
    let rollback = CouponRollbackRecord::new(
        "rollback_roll20_project_1_refund",
        confirm_command.redemption.coupon_redemption_id.clone(),
        CouponRollbackType::Refund,
        ROLLED_BACK_AT_MS,
    )
    .with_status(CouponRollbackStatus::Completed)
    .with_restored_budget_minor(RESERVED_BUDGET_MINOR)
    .with_restored_inventory_count(1)
    .with_updated_at_ms(ROLLED_BACK_AT_MS);

    AtomicCouponRollbackCommand {
        expected_budget: confirm_command.next_budget.clone(),
        next_budget,
        expected_code: confirm_command.next_code.clone(),
        next_code,
        expected_redemption: confirm_command.redemption.clone(),
        next_redemption,
        rollback,
    }
}

fn select_budget(mut budgets: Vec<CampaignBudgetRecord>) -> CampaignBudgetRecord {
    budgets
        .drain(..)
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
        })
        .expect("budget record should exist")
}
