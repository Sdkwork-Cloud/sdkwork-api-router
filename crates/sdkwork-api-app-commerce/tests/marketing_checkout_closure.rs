use sdkwork_api_app_commerce::{
    apply_portal_commerce_payment_event, load_portal_commerce_checkout_session,
    preview_portal_commerce_quote, submit_portal_commerce_order,
    PortalCommercePaymentEventRequest, PortalCommerceQuoteRequest,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRedemptionStatus, CouponReservationStatus,
    CouponRollbackStatus, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingCampaignStatus,
};
use sdkwork_api_storage_core::MarketingStore;
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn preview_quote_uses_marketing_coupon_discount() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "LAUNCH20", 20).await;

    let quote = preview_portal_commerce_quote(
        &store,
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("launch20".to_owned()),
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("marketing coupon quote");

    assert_eq!(quote.payable_price_cents, 3_200);
    assert_eq!(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.code.as_str()),
        Some("LAUNCH20")
    );
    assert_eq!(
        quote
            .applied_coupon
            .as_ref()
            .map(|coupon| coupon.source.as_str()),
        Some("marketing")
    );
}

#[tokio::test]
async fn preview_quote_rejects_coupon_when_target_kind_is_not_eligible() {
    let store = build_store().await;
    seed_percent_off_coupon_for_targets(&store, "PLANONLY20", 20, &["subscription_plan"]).await;

    let error = preview_portal_commerce_quote(
        &store,
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("planonly20".to_owned()),
            current_remaining_units: Some(5_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect_err("coupon should reject ineligible target kind");

    assert!(
        error
            .to_string()
            .contains("coupon PLANONLY20 is not eligible: target_kind_not_eligible"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn discounted_order_reserves_confirms_and_rolls_back_marketing_coupon() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "LAUNCH20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-1",
        "project-1",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("LAUNCH20".to_owned()),
            current_remaining_units: Some(3_000),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    assert_eq!(order.status, "pending_payment");

    let reservations = MarketingStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records");
    assert_eq!(reservations.len(), 1);
    assert_eq!(
        reservations[0].reservation_status,
        CouponReservationStatus::Reserved
    );

    let reserved_code = MarketingStore::find_coupon_code_record_by_value(&store, "LAUNCH20")
        .await
        .expect("coupon code lookup")
        .expect("coupon code exists");
    assert_eq!(reserved_code.status, CouponCodeStatus::Reserved);

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-1",
        "project-1",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_launch20_paid".to_owned()),
            checkout_method_id: None,
        },
    )
    .await
    .expect("settled order");

    assert_eq!(settled.status, "fulfilled");

    let redemptions = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("redemption records");
    assert_eq!(redemptions.len(), 1);
    assert_eq!(
        redemptions[0].redemption_status,
        CouponRedemptionStatus::Redeemed
    );
    assert_eq!(redemptions[0].subsidy_amount_minor, 800);

    let refunded = apply_portal_commerce_payment_event(
        &store,
        "user-1",
        "project-1",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_launch20_refund".to_owned()),
            checkout_method_id: None,
        },
    )
    .await
    .expect("refunded order");

    assert_eq!(refunded.status, "refunded");

    let rollbacks = MarketingStore::list_coupon_rollback_records(&store)
        .await
        .expect("rollback records");
    assert_eq!(rollbacks.len(), 1);
    assert_eq!(
        rollbacks[0].rollback_status,
        CouponRollbackStatus::Completed
    );

    let rolled_back_redemption = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("redemptions after refund");
    assert_eq!(
        rolled_back_redemption[0].redemption_status,
        CouponRedemptionStatus::RolledBack
    );

    let recycled_code = MarketingStore::find_coupon_code_record_by_value(&store, "LAUNCH20")
        .await
        .expect("coupon code lookup after refund")
        .expect("coupon code exists after refund");
    assert_eq!(recycled_code.status, CouponCodeStatus::Available);
}

#[tokio::test]
async fn failed_payment_releases_marketing_coupon_reservation() {
    let store = build_store().await;
    seed_percent_off_coupon(&store, "FAILSAFE20", 20).await;

    let order = submit_portal_commerce_order(
        &store,
        "user-2",
        "project-2",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("FAILSAFE20".to_owned()),
            current_remaining_units: Some(0),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let failed = apply_portal_commerce_payment_event(
        &store,
        "user-2",
        "project-2",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "failed".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_failsafe20_failed".to_owned()),
            checkout_method_id: None,
        },
    )
    .await
    .expect("failed order");

    assert_eq!(failed.status, "failed");

    let reservations = MarketingStore::list_coupon_reservation_records(&store)
        .await
        .expect("reservation records");
    assert_eq!(reservations.len(), 1);
    assert_eq!(
        reservations[0].reservation_status,
        CouponReservationStatus::Released
    );

    let released_code = MarketingStore::find_coupon_code_record_by_value(&store, "FAILSAFE20")
        .await
        .expect("released coupon code lookup")
        .expect("coupon code exists after failure");
    assert_eq!(released_code.status, CouponCodeStatus::Available);
}

#[tokio::test]
async fn paid_checkout_session_exposes_structured_payment_rails_and_normalizes_provider_aliases() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-3",
        "project-3",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(1_250),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let session = load_portal_commerce_checkout_session(
        &store,
        "user-3",
        "project-3",
        &order.order_id,
    )
    .await
    .expect("checkout session");

    assert_eq!(session.provider, "manual_lab");
    assert_eq!(session.mode, "operator_settlement");
    assert!(session.methods.iter().any(|method| {
        method.id == "manual_settlement"
            && method.provider == "manual_lab"
            && method.channel == "operator_settlement"
            && method.session_kind == "operator_action"
            && method.session_reference.starts_with("MANUAL-")
            && method.qr_code_payload.is_none()
            && method.webhook_verification == "manual"
            && method.supports_refund
            && !method.recommended
            && !method.supports_webhook
    }));
    assert!(session.methods.iter().any(|method| {
        method.id == "stripe_checkout"
            && method.provider == "stripe"
            && method.channel == "hosted_checkout"
            && method.session_kind == "hosted_checkout"
            && method.session_reference.starts_with("STRIPE-")
            && method.qr_code_payload.is_none()
            && method.webhook_verification == "stripe_signature"
            && method.supports_refund
            && method.supports_partial_refund
            && method.recommended
            && method.supports_webhook
    }));
    assert!(session.methods.iter().any(|method| {
        method.id == "alipay_qr"
            && method.provider == "alipay"
            && method.channel == "scan_qr"
            && method.session_kind == "qr_code"
            && method.session_reference.starts_with("ALIPAY-")
            && method
                .qr_code_payload
                .as_deref()
                .is_some_and(|payload| payload.contains("sdkworkpay://alipay_qr/"))
            && method.webhook_verification == "alipay_rsa_sha256"
            && method.supports_refund
            && !method.supports_partial_refund
            && !method.recommended
            && method.supports_webhook
    }));
    assert!(session.methods.iter().any(|method| {
        method.id == "wechat_pay_qr"
            && method.provider == "wechat_pay"
            && method.channel == "scan_qr"
            && method.session_kind == "qr_code"
            && method.session_reference.starts_with("WECHAT-")
            && method
                .qr_code_payload
                .as_deref()
                .is_some_and(|payload| payload.contains("sdkworkpay://wechat_pay_qr/"))
            && method.webhook_verification == "wechatpay_rsa_sha256"
            && method.supports_refund
            && !method.supports_partial_refund
            && !method.recommended
            && method.supports_webhook
    }));

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-3",
        "project-3",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("wechat".to_owned()),
            provider_event_id: Some("evt_alias_paid".to_owned()),
            checkout_method_id: Some("wechat_pay_qr".to_owned()),
        },
    )
    .await
    .expect("settled order");

    assert_eq!(settled.status, "fulfilled");

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].provider, "wechat_pay");
    assert_eq!(events[0].dedupe_key, "wechat_pay:evt_alias_paid");
    assert!(
        events[0].payload_json.contains("\"checkout_method_id\":\"wechat_pay_qr\""),
        "payload should preserve the originating checkout method: {}",
        events[0].payload_json
    );
}

#[tokio::test]
async fn payment_events_reject_unsupported_provider_values() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-4",
        "project-4",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(0),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-4",
        "project-4",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("paypal".to_owned()),
            provider_event_id: Some("evt_unsupported".to_owned()),
            checkout_method_id: None,
        },
    )
    .await
    .expect_err("unsupported providers should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported commerce payment provider"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn webhook_backed_checkout_methods_require_provider_event_id() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-5",
        "project-5",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(25),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-5",
        "project-5",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: None,
            checkout_method_id: Some("stripe_checkout".to_owned()),
        },
    )
    .await
    .expect_err("webhook-backed methods should require provider_event_id");

    assert!(
        error
            .to_string()
            .contains("provider_event_id is required for webhook-backed checkout methods"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn refund_events_reject_provider_mismatch_against_processed_settlement_provider() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-6",
        "project-6",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(50),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-6",
        "project-6",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_provider_match_paid".to_owned()),
            checkout_method_id: Some("stripe_checkout".to_owned()),
        },
    )
    .await
    .expect("settled order");
    assert_eq!(settled.status, "fulfilled");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-6",
        "project-6",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("alipay".to_owned()),
            provider_event_id: Some("evt_provider_mismatch_refund".to_owned()),
            checkout_method_id: Some("alipay_qr".to_owned()),
        },
    )
    .await
    .expect_err("refund provider mismatch should be rejected");

    assert!(
        error
            .to_string()
            .contains("refund provider alipay does not match settled provider stripe"),
        "unexpected error: {error}"
    );

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider == "alipay"
            && event.provider_event_id.as_deref() == Some("evt_provider_mismatch_refund")
            && event.processing_status.as_str() == "rejected"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
}

#[tokio::test]
async fn provider_backed_payment_events_require_provider_event_id_without_checkout_method_hint() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-7",
        "project-7",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(70),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-7",
        "project-7",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: None,
            checkout_method_id: None,
        },
    )
    .await
    .expect_err("provider-backed events should require provider_event_id");

    assert!(
        error
            .to_string()
            .contains("provider_event_id is required for provider-backed payment events"),
        "unexpected error: {error}"
    );
}

async fn build_store() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:")
        .await
        .expect("sqlite migrations");
    SqliteAdminStore::new(pool)
}

async fn seed_percent_off_coupon(store: &SqliteAdminStore, code: &str, discount_percent: u8) {
    seed_percent_off_coupon_for_targets(store, code, discount_percent, &[]).await;
}

async fn seed_percent_off_coupon_for_targets(
    store: &SqliteAdminStore,
    code: &str,
    discount_percent: u8,
    eligible_target_kinds: &[&str],
) {
    let template = CouponTemplateRecord::new(
        format!("template_{code}"),
        code,
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name(format!("{code} launch coupon"))
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_restriction(
        sdkwork_api_domain_marketing::CouponRestrictionSpec::new(
            sdkwork_api_domain_marketing::MarketingSubjectScope::Project,
        )
        .with_eligible_target_kinds(
            eligible_target_kinds
                .iter()
                .map(|kind| (*kind).to_owned())
                .collect(),
        ),
    )
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
            .with_discount_percent(Some(discount_percent)),
    )
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_coupon_template_record(store, &template)
        .await
        .expect("insert coupon template");

    let campaign = MarketingCampaignRecord::new(
        format!("campaign_{code}"),
        template.coupon_template_id.clone(),
    )
    .with_display_name(format!("{code} campaign"))
    .with_status(MarketingCampaignStatus::Active)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_marketing_campaign_record(store, &campaign)
        .await
        .expect("insert marketing campaign");

    let budget = CampaignBudgetRecord::new(
        format!("budget_{code}"),
        campaign.marketing_campaign_id.clone(),
    )
    .with_status(CampaignBudgetStatus::Active)
    .with_total_budget_minor(10_000)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_campaign_budget_record(store, &budget)
        .await
        .expect("insert budget");

    let coupon_code = CouponCodeRecord::new(
        format!("coupon_code_{code}"),
        template.coupon_template_id.clone(),
        code,
    )
    .with_status(CouponCodeStatus::Available)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_coupon_code_record(store, &coupon_code)
        .await
        .expect("insert coupon code");
}
