use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldRecord, AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType, PricingPlanRecord,
    PricingRateRecord, RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
};
use sdkwork_api_domain_marketing::{
    CouponBenefitSpec, CouponCodeRecord, CouponCodeStatus, CouponDistributionKind,
    CouponRedemptionRecord, CouponRedemptionStatus, CouponReservationRecord,
    CouponReservationStatus, CouponRestrictionSpec, CouponRollbackRecord,
    CouponRollbackStatus, CouponRollbackType, CouponTemplateRecord, CouponTemplateStatus,
    MarketingBenefitKind, MarketingCampaignRecord, MarketingCampaignStatus,
    MarketingStackingPolicy, MarketingSubjectScope,
};
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn login_token(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn seed_canonical_billing_fixture(store: &SqliteAdminStore) {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_status(AccountStatus::Active)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    let active_credit_lot =
        AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(120.0)
            .with_remaining_quantity(95.0)
            .with_held_quantity(5.0)
            .with_priority(20)
            .with_created_at_ms(11)
            .with_updated_at_ms(11);
    let expired_promo_lot = AccountBenefitLotRecord::new(
        8002,
        1001,
        2002,
        7001,
        9001,
        AccountBenefitType::PromoCredit,
    )
    .with_source_type(AccountBenefitSourceType::Coupon)
    .with_original_quantity(30.0)
    .with_remaining_quantity(30.0)
    .with_expires_at_ms(Some(1))
    .with_status(AccountBenefitLotStatus::Expired)
    .with_created_at_ms(12)
    .with_updated_at_ms(12);
    let hold = AccountHoldRecord::new(8101, 1001, 2002, 7001, 9001, 6001)
        .with_status(AccountHoldStatus::Captured)
        .with_estimated_quantity(5.0)
        .with_captured_quantity(5.0)
        .with_expires_at_ms(20)
        .with_created_at_ms(13)
        .with_updated_at_ms(13);
    let settlement = RequestSettlementRecord::new(8301, 1001, 2002, 6001, 7001, 9001)
        .with_hold_id(Some(8101))
        .with_status(RequestSettlementStatus::Captured)
        .with_estimated_credit_hold(5.0)
        .with_captured_credit_amount(5.0)
        .with_provider_cost_amount(2.5)
        .with_retail_charge_amount(5.0)
        .with_settled_at_ms(14)
        .with_created_at_ms(14)
        .with_updated_at_ms(14);
    let capture_ledger_entry = AccountLedgerEntryRecord::new(
        8401,
        1001,
        2002,
        7001,
        9001,
        AccountLedgerEntryType::SettlementCapture,
    )
    .with_request_id(Some(6001))
    .with_hold_id(Some(8101))
    .with_quantity(5.0)
    .with_amount(5.0)
    .with_created_at_ms(14);
    let capture_ledger_allocation =
        AccountLedgerAllocationRecord::new(8501, 1001, 2002, 8401, 8001)
            .with_quantity_delta(-5.0)
            .with_created_at_ms(14);
    let refund_ledger_entry =
        AccountLedgerEntryRecord::new(8402, 1001, 2002, 7001, 9001, AccountLedgerEntryType::Refund)
            .with_request_id(Some(6001))
            .with_hold_id(Some(8101))
            .with_quantity(2.0)
            .with_amount(2.0)
            .with_created_at_ms(15);
    let refund_ledger_allocation = AccountLedgerAllocationRecord::new(8502, 1001, 2002, 8402, 8001)
        .with_quantity_delta(2.0)
        .with_created_at_ms(15);
    let pricing_plan = PricingPlanRecord::new(9101, 1001, 2002, "default-retail", 3)
        .with_display_name("Default Retail v3")
        .with_status("active")
        .with_effective_from_ms(10)
        .with_effective_to_ms(Some(100))
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let pricing_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_capability_code(Some("responses".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000.0)
        .with_unit_price(0.25)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail text input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(16)
        .with_updated_at_ms(16);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&active_credit_lot)
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(&expired_promo_lot)
        .await
        .unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();
    store
        .insert_account_ledger_entry_record(&capture_ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&capture_ledger_allocation)
        .await
        .unwrap();
    store
        .insert_account_ledger_entry_record(&refund_ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&refund_ledger_allocation)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&pricing_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&pricing_rate)
        .await
        .unwrap();
}

async fn seed_commerce_audit_fixture(store: &SqliteAdminStore) {
    let template = CouponTemplateRecord::new(
        "template-launch20",
        "launch20",
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name("Spring launch 20%")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::SharedCode)
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
            .with_discount_percent(Some(20)),
    )
    .with_restriction(
        CouponRestrictionSpec::new(MarketingSubjectScope::Project)
            .with_stacking_policy(MarketingStackingPolicy::Exclusive)
            .with_eligible_target_kinds(vec!["recharge_pack".to_owned()]),
    )
    .with_created_at_ms(80)
    .with_updated_at_ms(80);
    let campaign = MarketingCampaignRecord::new("campaign-launch20", "template-launch20")
        .with_display_name("Spring launch")
        .with_status(MarketingCampaignStatus::Active)
        .with_start_at_ms(Some(80))
        .with_created_at_ms(80)
        .with_updated_at_ms(80);
    let code = CouponCodeRecord::new("code-launch20", "template-launch20", "SPRING20")
        .with_status(CouponCodeStatus::Redeemed)
        .with_created_at_ms(90)
        .with_updated_at_ms(210);
    let reservation = CouponReservationRecord::new(
        "reservation-order-refunded",
        "code-launch20",
        MarketingSubjectScope::Project,
        "project-a",
        600,
    )
    .with_status(CouponReservationStatus::Confirmed)
    .with_budget_reserved_minor(800)
    .with_created_at_ms(95)
    .with_updated_at_ms(160);
    let redemption = CouponRedemptionRecord::new(
        "redemption-order-refunded",
        "reservation-order-refunded",
        "code-launch20",
        "template-launch20",
        160,
    )
    .with_status(CouponRedemptionStatus::PartiallyRolledBack)
    .with_subsidy_amount_minor(800)
    .with_order_id(Some("order-refunded".to_owned()))
    .with_payment_event_id(Some("payevt-order-refunded-settled".to_owned()))
    .with_updated_at_ms(310);
    let rollback = CouponRollbackRecord::new(
        "rollback-order-refunded",
        "redemption-order-refunded",
        CouponRollbackType::Refund,
        320,
    )
    .with_status(CouponRollbackStatus::Completed)
    .with_restored_budget_minor(800)
    .with_restored_inventory_count(1)
    .with_updated_at_ms(330);
    let refunded_order = CommerceOrderRecord::new(
        "order-refunded",
        "project-a",
        "user-a",
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        3_200,
        "$40.00",
        "$32.00",
        100_000,
        0,
        "refunded",
        "live",
        100,
    )
    .with_applied_coupon_code_option(Some("SPRING20".to_owned()))
    .with_coupon_reservation_id_option(Some("reservation-order-refunded".to_owned()))
    .with_coupon_redemption_id_option(Some("redemption-order-refunded".to_owned()))
    .with_marketing_campaign_id_option(Some("campaign-launch20".to_owned()))
    .with_subsidy_amount_minor(800)
    .with_updated_at_ms(300);
    let pending_order = CommerceOrderRecord::new(
        "order-pending",
        "project-b",
        "user-b",
        "subscription_plan",
        "growth",
        "Growth",
        7_900,
        7_900,
        "$79.00",
        "$79.00",
        100_000,
        0,
        "pending_payment",
        "live",
        190,
    )
    .with_updated_at_ms(200);
    let older_order = CommerceOrderRecord::new(
        "order-old",
        "project-c",
        "user-c",
        "coupon_redemption",
        "TEAMREADY",
        "TEAMREADY",
        0,
        0,
        "$0.00",
        "$0.00",
        0,
        25_000,
        "fulfilled",
        "live",
        50,
    )
    .with_updated_at_ms(60);

    let settled_event = CommercePaymentEventRecord::new(
        "payevt-order-refunded-settled",
        "order-refunded",
        "project-a",
        "user-a",
        "stripe",
        "stripe:evt_stripe_1",
        "settled",
        "{\"event_type\":\"settled\"}",
        150,
    )
    .with_provider_event_id(Some("evt_stripe_1".to_owned()))
    .with_processing_status(CommercePaymentEventProcessingStatus::Processed)
    .with_processed_at_ms(Some(160))
    .with_order_status_after(Some("fulfilled".to_owned()));
    let failed_event = CommercePaymentEventRecord::new(
        "payevt-order-refunded-failed",
        "order-refunded",
        "project-a",
        "user-a",
        "stripe",
        "stripe:evt_stripe_fail",
        "failed",
        "{\"event_type\":\"failed\"}",
        120,
    )
    .with_provider_event_id(Some("evt_stripe_fail".to_owned()))
    .with_processing_status(CommercePaymentEventProcessingStatus::Rejected)
    .with_processing_message(Some("provider signature mismatch".to_owned()));

    AdminStore::insert_coupon_template_record(store, &template)
        .await
        .unwrap();
    AdminStore::insert_marketing_campaign_record(store, &campaign)
        .await
        .unwrap();
    AdminStore::insert_coupon_code_record(store, &code)
        .await
        .unwrap();
    AdminStore::insert_coupon_reservation_record(store, &reservation)
        .await
        .unwrap();
    AdminStore::insert_coupon_redemption_record(store, &redemption)
        .await
        .unwrap();
    AdminStore::insert_coupon_rollback_record(store, &rollback)
        .await
        .unwrap();
    store.insert_commerce_order(&refunded_order).await.unwrap();
    store.insert_commerce_order(&pending_order).await.unwrap();
    store.insert_commerce_order(&older_order).await.unwrap();
    store
        .upsert_commerce_payment_event(&settled_event)
        .await
        .unwrap();
    store
        .upsert_commerce_payment_event(&failed_event)
        .await
        .unwrap();
}

#[tokio::test]
async fn admin_billing_accounts_expose_canonical_balance_summaries_and_lot_details() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_canonical_billing_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let accounts = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(accounts.status(), StatusCode::OK);
    let accounts_json = read_json(accounts).await;
    assert_eq!(accounts_json.as_array().unwrap().len(), 1);
    assert_eq!(accounts_json[0]["account"]["account_id"], 7001);
    assert_eq!(accounts_json[0]["available_balance"], 90.0);
    assert_eq!(accounts_json[0]["held_balance"], 5.0);
    assert_eq!(accounts_json[0]["consumed_balance"], 25.0);
    assert_eq!(accounts_json[0]["grant_balance"], 150.0);
    assert_eq!(accounts_json[0]["active_lot_count"], 1);

    let balance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/7001/balance")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(balance.status(), StatusCode::OK);
    let balance_json = read_json(balance).await;
    assert_eq!(balance_json["account_id"], 7001);
    assert_eq!(balance_json["available_balance"], 90.0);
    assert_eq!(balance_json["held_balance"], 5.0);
    assert_eq!(balance_json["consumed_balance"], 25.0);
    assert_eq!(balance_json["grant_balance"], 150.0);
    assert_eq!(balance_json["active_lot_count"], 1);
    assert_eq!(balance_json["lots"].as_array().unwrap().len(), 1);
    assert_eq!(balance_json["lots"][0]["lot_id"], 8001);

    let lots = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/7001/benefit-lots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(lots.status(), StatusCode::OK);
    let lots_json = read_json(lots).await;
    assert_eq!(lots_json.as_array().unwrap().len(), 2);
    assert!(lots_json
        .as_array()
        .unwrap()
        .iter()
        .any(|lot| lot["lot_id"] == 8001 && lot["status"] == "active"));
    assert!(lots_json
        .as_array()
        .unwrap()
        .iter()
        .any(|lot| lot["lot_id"] == 8002 && lot["status"] == "expired"));
}

#[tokio::test]
async fn admin_billing_investigation_routes_list_holds_settlements_and_pricing() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_canonical_billing_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let holds = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/account-holds")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(holds.status(), StatusCode::OK);
    let holds_json = read_json(holds).await;
    assert_eq!(holds_json.as_array().unwrap().len(), 1);
    assert_eq!(holds_json[0]["hold_id"], 8101);
    assert_eq!(holds_json[0]["status"], "captured");

    let settlements = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/request-settlements")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settlements.status(), StatusCode::OK);
    let settlements_json = read_json(settlements).await;
    assert_eq!(settlements_json.as_array().unwrap().len(), 1);
    assert_eq!(settlements_json[0]["request_settlement_id"], 8301);
    assert_eq!(settlements_json[0]["status"], "captured");

    let plans = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(plans.status(), StatusCode::OK);
    let plans_json = read_json(plans).await;
    assert_eq!(plans_json.as_array().unwrap().len(), 1);
    assert_eq!(plans_json[0]["pricing_plan_id"], 9101);
    assert_eq!(plans_json[0]["status"], "active");
    assert_eq!(plans_json[0]["effective_from_ms"], 10);
    assert_eq!(plans_json[0]["effective_to_ms"], 100);

    let rates = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rates.status(), StatusCode::OK);
    let rates_json = read_json(rates).await;
    assert_eq!(rates_json.as_array().unwrap().len(), 1);
    assert_eq!(rates_json[0]["pricing_rate_id"], 9201);
    assert_eq!(rates_json[0]["metric_code"], "token.input");
    assert_eq!(rates_json[0]["charge_unit"], "input_token");
    assert_eq!(rates_json[0]["pricing_method"], "per_unit");
    assert_eq!(rates_json[0]["display_price_unit"], "USD / 1M input tokens");
    assert_eq!(rates_json[0]["status"], "active");
}

#[tokio::test]
async fn admin_billing_account_ledger_route_exposes_account_history_with_allocations() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_canonical_billing_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let ledger = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/7001/ledger")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ledger.status(), StatusCode::OK);
    let ledger_json = read_json(ledger).await;
    assert_eq!(ledger_json.as_array().unwrap().len(), 2);
    assert_eq!(ledger_json[0]["entry"]["ledger_entry_id"], 8402);
    assert_eq!(ledger_json[0]["entry"]["entry_type"], "refund");
    assert_eq!(ledger_json[0]["entry"]["account_id"], 7001);
    assert_eq!(ledger_json[0]["allocations"][0]["lot_id"], 8001);
    assert_eq!(ledger_json[0]["allocations"][0]["quantity_delta"], 2.0);
    assert_eq!(ledger_json[1]["entry"]["ledger_entry_id"], 8401);
    assert_eq!(ledger_json[1]["entry"]["entry_type"], "settlement_capture");
    assert_eq!(ledger_json[1]["allocations"][0]["quantity_delta"], -5.0);
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_create_canonical_plans_and_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let existing_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    store
        .insert_pricing_plan_record(&existing_plan)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_plan = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"plan_code":"media-studio","plan_version":2,"display_name":"Media Studio","currency_code":"USD","credit_unit_code":"credit","status":"draft","effective_from_ms":1717171730000,"effective_to_ms":1719773730000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_plan.status(), StatusCode::CREATED);
    let created_plan_json = read_json(create_plan).await;
    assert_eq!(created_plan_json["plan_code"], "media-studio");
    assert_eq!(created_plan_json["plan_version"], 2);
    assert_eq!(created_plan_json["display_name"], "Media Studio");
    assert_eq!(created_plan_json["effective_from_ms"], 1717171730000u64);
    assert_eq!(created_plan_json["effective_to_ms"], 1719773730000u64);
    assert!(created_plan_json["pricing_plan_id"].as_u64().unwrap() > 0);

    let create_rate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"pricing_plan_id":9101,"metric_code":"image.output","capability_code":"image_generation","model_code":"gpt-image-1","provider_code":"provider-openai-official","charge_unit":"image","pricing_method":"per_unit","quantity_step":1.0,"unit_price":0.08,"display_price_unit":"USD / image","minimum_billable_quantity":1.0,"minimum_charge":0.08,"rounding_increment":1.0,"rounding_mode":"ceil","included_quantity":0.0,"priority":200,"notes":"Image generation retail pricing","status":"active"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_rate.status(), StatusCode::CREATED);
    let created_rate_json = read_json(create_rate).await;
    assert_eq!(created_rate_json["pricing_plan_id"], 9101);
    assert_eq!(created_rate_json["metric_code"], "image.output");
    assert_eq!(created_rate_json["capability_code"], "image_generation");
    assert_eq!(created_rate_json["charge_unit"], "image");
    assert_eq!(created_rate_json["pricing_method"], "per_unit");
    assert_eq!(created_rate_json["display_price_unit"], "USD / image");
    assert_eq!(created_rate_json["rounding_mode"], "ceil");
    assert_eq!(created_rate_json["status"], "active");
    assert!(created_rate_json["pricing_rate_id"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_update_canonical_plans_and_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let existing_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1717171730000)
        .with_effective_to_ms(Some(1718035730000))
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let existing_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_capability_code(Some("responses".to_owned()))
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail text input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(16)
        .with_updated_at_ms(16);

    store
        .insert_pricing_plan_record(&existing_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&existing_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let update_plan = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/admin/billing/pricing-plans/9101")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"plan_code":"retail-pro","plan_version":2,"display_name":"Retail Pro Updated","currency_code":"USD","credit_unit_code":"credit","status":"draft","effective_from_ms":1718035730000,"effective_to_ms":1720627730000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_plan.status(), StatusCode::OK);
    let updated_plan_json = read_json(update_plan).await;
    assert_eq!(updated_plan_json["pricing_plan_id"], 9101);
    assert_eq!(updated_plan_json["plan_version"], 2);
    assert_eq!(updated_plan_json["display_name"], "Retail Pro Updated");
    assert_eq!(updated_plan_json["status"], "draft");
    assert_eq!(updated_plan_json["effective_from_ms"], 1718035730000u64);
    assert_eq!(updated_plan_json["effective_to_ms"], 1720627730000u64);

    let update_rate = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/admin/billing/pricing-rates/9201")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":1001,"organization_id":2002,"pricing_plan_id":9101,"metric_code":"image.output","capability_code":"images","model_code":"gpt-image-1","provider_code":"provider-openai-official","charge_unit":"image","pricing_method":"flat","quantity_step":1.0,"unit_price":0.08,"display_price_unit":"USD / image","minimum_billable_quantity":1.0,"minimum_charge":0.08,"rounding_increment":1.0,"rounding_mode":"ceil","included_quantity":0.0,"priority":200,"notes":"Updated image pricing","status":"draft"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_rate.status(), StatusCode::OK);
    let updated_rate_json = read_json(update_rate).await;
    assert_eq!(updated_rate_json["pricing_rate_id"], 9201);
    assert_eq!(updated_rate_json["metric_code"], "image.output");
    assert_eq!(updated_rate_json["capability_code"], "images");
    assert_eq!(updated_rate_json["charge_unit"], "image");
    assert_eq!(updated_rate_json["pricing_method"], "flat");
    assert_eq!(updated_rate_json["display_price_unit"], "USD / image");
    assert_eq!(updated_rate_json["minimum_charge"], 0.08);
    assert_eq!(updated_rate_json["priority"], 200);
    assert_eq!(updated_rate_json["status"], "draft");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_clone_canonical_plan_versions_with_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let existing_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let existing_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_capability_code(Some("responses".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail text input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(16)
        .with_updated_at_ms(16);

    store
        .insert_pricing_plan_record(&existing_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&existing_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let clone_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9101/clone")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(clone_plan.status(), StatusCode::CREATED);
    let cloned_plan_json = read_json(clone_plan).await;
    let cloned_plan_id = cloned_plan_json["pricing_plan_id"].as_u64().unwrap();
    assert!(cloned_plan_id > 9101);
    assert_eq!(cloned_plan_json["plan_code"], "retail-pro");
    assert_eq!(cloned_plan_json["plan_version"], 2);
    assert_eq!(cloned_plan_json["status"], "draft");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    assert_eq!(stored_plans.len(), 2);

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    assert_eq!(stored_rates.len(), 2);
    let cloned_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_plan_id == cloned_plan_id)
        .unwrap();
    assert_eq!(cloned_rate.metric_code, "token.input");
    assert_eq!(cloned_rate.charge_unit, "input_token");
    assert_eq!(cloned_rate.pricing_method, "per_unit");
    assert_eq!(cloned_rate.display_price_unit, "USD / 1M input tokens");
    assert_eq!(cloned_rate.status, "draft");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_publish_cloned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let draft_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let draft_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store.insert_pricing_plan_record(&draft_plan).await.unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store.insert_pricing_rate_record(&draft_rate).await.unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let publish_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9102/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(publish_plan.status(), StatusCode::OK);
    let published_plan_json = read_json(publish_plan).await;
    assert_eq!(published_plan_json["pricing_plan_id"], 9102);
    assert_eq!(published_plan_json["status"], "active");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let published_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(published_plan.status, "active");
    assert_eq!(archived_plan.status, "archived");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let published_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(published_rate.status, "active");
    assert_eq!(archived_rate.status, "archived");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_reject_publish_for_future_effective_plan_versions()
{
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let future_draft_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let future_draft_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_draft_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_draft_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let publish_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9102/publish")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(publish_plan.status(), StatusCode::BAD_REQUEST);
    let error_json = read_json(publish_plan).await;
    assert_eq!(
        error_json["error"]["message"],
        "pricing plan 9102 cannot be published before effective_from_ms"
    );

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let stored_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let stored_active_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(stored_future_plan.status, "draft");
    assert_eq!(stored_active_plan.status, "active");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let stored_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let stored_active_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(stored_future_rate.status, "draft");
    assert_eq!(stored_active_rate.status, "active");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_schedule_future_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1_717_171_700_000)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let future_draft_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro Summer")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("draft")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let future_draft_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("draft")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_draft_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_draft_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let schedule_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9102/schedule")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(schedule_plan.status(), StatusCode::OK);
    let scheduled_plan_json = read_json(schedule_plan).await;
    assert_eq!(scheduled_plan_json["pricing_plan_id"], 9102);
    assert_eq!(scheduled_plan_json["status"], "planned");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let scheduled_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_active_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(scheduled_plan.status, "planned");
    assert_eq!(still_active_plan.status, "active");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let scheduled_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_active_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(scheduled_rate.status, "planned");
    assert_eq!(still_active_rate.status, "active");
}

#[tokio::test]
async fn admin_billing_pricing_reads_auto_activate_due_planned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let due_planned_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(2)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let future_planned_plan = PricingPlanRecord::new(9103, 1001, 2002, "retail-pro", 3)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);
    let due_planned_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let future_planned_rate = PricingRateRecord::new(9203, 1001, 2002, 9103, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(3.1)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&due_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&due_planned_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_planned_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let plans = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-plans")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(plans.status(), StatusCode::OK);
    let plans_json = read_json(plans).await;
    assert_eq!(plans_json.as_array().unwrap().len(), 3);
    assert_eq!(plans_json[0]["status"], "archived");
    assert_eq!(plans_json[1]["status"], "active");
    assert_eq!(plans_json[2]["status"], "planned");

    let rates = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/pricing-rates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(rates.status(), StatusCode::OK);
    let rates_json = read_json(rates).await;
    assert_eq!(rates_json.as_array().unwrap().len(), 3);
    assert_eq!(rates_json[0]["status"], "archived");
    assert_eq!(rates_json[1]["status"], "active");
    assert_eq!(rates_json[2]["status"], "planned");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    let activated_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9103)
        .unwrap();
    assert_eq!(archived_plan.status, "archived");
    assert_eq!(activated_plan.status, "active");
    assert_eq!(still_future_plan.status, "planned");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    let activated_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9203)
        .unwrap();
    assert_eq!(archived_rate.status, "archived");
    assert_eq!(activated_rate.status, "active");
    assert_eq!(still_future_rate.status, "planned");
}

#[tokio::test]
async fn admin_billing_pricing_lifecycle_sync_route_activates_due_planned_plan_versions() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 1)
        .with_display_name("Retail Pro")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_effective_from_ms(1)
        .with_created_at_ms(15)
        .with_updated_at_ms(15);
    let due_planned_plan = PricingPlanRecord::new(9102, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(2)
        .with_created_at_ms(16)
        .with_updated_at_ms(16);
    let future_planned_plan = PricingPlanRecord::new(9103, 1001, 2002, "retail-pro", 3)
        .with_display_name("Retail Pro Future")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("planned")
        .with_effective_from_ms(4_102_444_800_000)
        .with_created_at_ms(17)
        .with_updated_at_ms(17);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.5)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(18)
        .with_updated_at_ms(18);
    let due_planned_rate = PricingRateRecord::new(9202, 1001, 2002, 9102, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let future_planned_rate = PricingRateRecord::new(9203, 1001, 2002, 9103, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(3.1)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("planned")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&due_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_plan_record(&future_planned_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&due_planned_rate)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&future_planned_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-lifecycle/synchronize")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let response_json = read_json(response).await;
    assert_eq!(response_json["changed"], true);
    assert_eq!(response_json["due_group_count"], 1);
    assert_eq!(response_json["activated_plan_count"], 1);
    assert_eq!(response_json["archived_plan_count"], 1);
    assert_eq!(response_json["activated_rate_count"], 1);
    assert_eq!(response_json["archived_rate_count"], 1);
    assert_eq!(response_json["skipped_plan_count"], 0);
    assert!(
        response_json["synchronized_at_ms"]
            .as_u64()
            .unwrap_or_default()
            > 0
    );

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    let archived_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    let activated_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9102)
        .unwrap();
    let still_future_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9103)
        .unwrap();
    assert_eq!(archived_plan.status, "archived");
    assert_eq!(activated_plan.status, "active");
    assert_eq!(still_future_plan.status, "planned");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    let archived_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    let activated_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9202)
        .unwrap();
    let still_future_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9203)
        .unwrap();
    assert_eq!(archived_rate.status, "archived");
    assert_eq!(activated_rate.status, "active");
    assert_eq!(still_future_rate.status, "planned");
}

#[tokio::test]
async fn admin_billing_pricing_management_routes_retire_plan_versions_and_rates() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());

    let active_plan = PricingPlanRecord::new(9101, 1001, 2002, "retail-pro", 2)
        .with_display_name("Retail Pro v2")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(19)
        .with_updated_at_ms(19);
    let active_rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_quantity_step(1000000.0)
        .with_unit_price(2.8)
        .with_display_price_unit("USD / 1M input tokens")
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_status("active")
        .with_created_at_ms(20)
        .with_updated_at_ms(20);

    store
        .insert_pricing_plan_record(&active_plan)
        .await
        .unwrap();
    store
        .insert_pricing_rate_record(&active_rate)
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let retire_plan = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/pricing-plans/9101/retire")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retire_plan.status(), StatusCode::OK);
    let retired_plan_json = read_json(retire_plan).await;
    assert_eq!(retired_plan_json["pricing_plan_id"], 9101);
    assert_eq!(retired_plan_json["status"], "archived");

    let stored_plans = store.list_pricing_plan_records().await.unwrap();
    assert_eq!(stored_plans.len(), 1);
    let retired_plan = stored_plans
        .iter()
        .find(|plan| plan.pricing_plan_id == 9101)
        .unwrap();
    assert_eq!(retired_plan.status, "archived");

    let stored_rates = store.list_pricing_rate_records().await.unwrap();
    assert_eq!(stored_rates.len(), 1);
    let retired_rate = stored_rates
        .iter()
        .find(|rate| rate.pricing_rate_id == 9201)
        .unwrap();
    assert_eq!(retired_rate.status, "archived");
}

#[tokio::test]
async fn admin_billing_balance_returns_not_found_for_unknown_account() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/accounts/9999/balance")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "account 9999 does not exist");
}

#[tokio::test]
async fn admin_commerce_routes_expose_recent_orders_and_payment_events() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_commerce_audit_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let recent_orders = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders?limit=2")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recent_orders.status(), StatusCode::OK);
    let recent_orders_json = read_json(recent_orders).await;
    assert_eq!(recent_orders_json.as_array().unwrap().len(), 2);
    assert_eq!(recent_orders_json[0]["order_id"], "order-refunded");
    assert_eq!(recent_orders_json[1]["order_id"], "order-pending");

    let payment_events = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders/order-refunded/payment-events")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(payment_events.status(), StatusCode::OK);
    let payment_events_json = read_json(payment_events).await;
    assert_eq!(payment_events_json.as_array().unwrap().len(), 2);
    assert_eq!(payment_events_json[0]["payment_event_id"], "payevt-order-refunded-settled");
    assert_eq!(payment_events_json[0]["provider"], "stripe");
    assert_eq!(payment_events_json[0]["provider_event_id"], "evt_stripe_1");
    assert_eq!(payment_events_json[0]["processing_status"], "processed");
    assert_eq!(payment_events_json[1]["payment_event_id"], "payevt-order-refunded-failed");
    assert_eq!(payment_events_json[1]["processing_status"], "rejected");
}

#[tokio::test]
async fn admin_commerce_order_audit_route_returns_coupon_and_payment_evidence_chain() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_commerce_audit_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let audit = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders/order-refunded/audit")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(audit.status(), StatusCode::OK);
    let audit_json = read_json(audit).await;
    assert_eq!(audit_json["order"]["order_id"], "order-refunded");
    assert_eq!(audit_json["order"]["applied_coupon_code"], "SPRING20");
    assert_eq!(audit_json["payment_events"].as_array().unwrap().len(), 2);
    assert_eq!(
        audit_json["coupon_reservation"]["coupon_reservation_id"],
        "reservation-order-refunded"
    );
    assert_eq!(
        audit_json["coupon_redemption"]["coupon_redemption_id"],
        "redemption-order-refunded"
    );
    assert_eq!(audit_json["coupon_redemption"]["order_id"], "order-refunded");
    assert_eq!(audit_json["coupon_rollbacks"].as_array().unwrap().len(), 1);
    assert_eq!(
        audit_json["coupon_rollbacks"][0]["coupon_rollback_id"],
        "rollback-order-refunded"
    );
    assert_eq!(audit_json["coupon_code"]["code_value"], "SPRING20");
    assert_eq!(
        audit_json["coupon_template"]["display_name"],
        "Spring launch 20%"
    );
    assert_eq!(
        audit_json["marketing_campaign"]["display_name"],
        "Spring launch"
    );
}

#[tokio::test]
async fn admin_commerce_order_audit_route_returns_not_found_for_unknown_order() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    seed_commerce_audit_fixture(&store).await;

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let audit = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/commerce/orders/order-missing/audit")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(audit.status(), StatusCode::NOT_FOUND);
    let audit_json = read_json(audit).await;
    assert_eq!(
        audit_json["error"]["message"],
        "commerce order order-missing not found"
    );
}
