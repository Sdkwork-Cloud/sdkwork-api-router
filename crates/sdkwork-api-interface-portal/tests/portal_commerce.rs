use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_payment::{
    ingest_payment_callback, PaymentCallbackIntakeDisposition, PaymentCallbackIntakeRequest,
    PaymentCallbackNormalizedOutcome, PaymentSubjectScope,
};
use sdkwork_api_domain_payment::PaymentProviderCode;
use sdkwork_api_storage_core::PaymentKernelStore;
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

async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"hunter2!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);
    read_json(register_response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn portal_workspace(app: axum::Router, token: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await
}

async fn settle_portal_order_via_verified_payment(
    pool: &SqlitePool,
    order_id: &str,
    settled_at_ms: u64,
) {
    let store = SqliteAdminStore::new(pool.clone());
    let payment_order = store
        .list_payment_order_records()
        .await
        .unwrap()
        .into_iter()
        .find(|payment_order| payment_order.commerce_order_id == order_id)
        .unwrap();
    let payment_attempt = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope,
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            &format!("evt_settled_{order_id}"),
            &format!("dedupe_evt_settled_{order_id}"),
            settled_at_ms,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some(format!("pi_{order_id}")))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(format!("{{\"id\":\"evt_settled_{order_id}\"}}"))),
    )
    .await
    .unwrap();

    assert_eq!(
        result.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        result.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );
}

#[tokio::test]
async fn portal_commerce_catalog_exposes_plans_packs_and_active_coupons() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_inactive")
    .bind("INACTIVE10")
    .bind("10% inactive discount")
    .bind("internal")
    .bind(40_i64)
    .bind(0_i64)
    .bind("Inactive campaign")
    .bind("2026-05-31")
    .bind(1_710_000_002_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["plans"].as_array().unwrap().len(), 3);
    assert_eq!(json["packs"].as_array().unwrap().len(), 3);
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|coupon| coupon["code"] == "SPRING20"));
    assert!(json["coupons"]
        .as_array()
        .unwrap()
        .iter()
        .all(|coupon| coupon["code"] != "INACTIVE10"));
}

#[tokio::test]
async fn portal_commerce_catalog_exposes_server_managed_recharge_options_and_custom_policy() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["recharge_options"].as_array().unwrap().len(), 4);
    assert_eq!(json["recharge_options"][0]["amount_cents"], 1000);
    assert_eq!(json["recharge_options"][0]["amount_label"], "$10.00");
    assert_eq!(json["recharge_options"][0]["granted_units"], 25000);
    assert_eq!(
        json["recharge_options"][1]["effective_ratio_label"],
        "2,800 units / $1"
    );
    assert_eq!(json["custom_recharge_policy"]["enabled"], true);
    assert_eq!(json["custom_recharge_policy"]["min_amount_cents"], 1000);
    assert_eq!(json["custom_recharge_policy"]["step_amount_cents"], 500);
    assert_eq!(
        json["custom_recharge_policy"]["suggested_amount_cents"],
        5000
    );
    assert_eq!(
        json["custom_recharge_policy"]["rules"][1]["effective_ratio_label"],
        "2,800 units / $1"
    );
}

#[tokio::test]
async fn portal_commerce_catalog_requires_authentication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/catalog")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn portal_commerce_quote_prices_recharge_and_coupon_redemption() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::OK);
    let recharge_json = read_json(recharge_response).await;
    assert_eq!(recharge_json["target_kind"], "recharge_pack");
    assert_eq!(recharge_json["target_name"], "Boost 100k");
    assert_eq!(recharge_json["list_price_label"], "$40.00");
    assert_eq!(recharge_json["payable_price_label"], "$32.00");
    assert_eq!(recharge_json["granted_units"], 100000);
    assert_eq!(recharge_json["projected_remaining_units"], 105000);
    assert_eq!(recharge_json["applied_coupon"]["code"], "SPRING20");

    let coupon_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"coupon_redemption\",\"target_id\":\"WELCOME100\",\"current_remaining_units\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(coupon_response.status(), StatusCode::OK);
    let coupon_json = read_json(coupon_response).await;
    assert_eq!(coupon_json["target_kind"], "coupon_redemption");
    assert_eq!(coupon_json["target_name"], "WELCOME100");
    assert_eq!(coupon_json["payable_price_label"], "$0.00");
    assert_eq!(coupon_json["bonus_units"], 100);
    assert_eq!(coupon_json["projected_remaining_units"], 5100);
}

#[tokio::test]
async fn portal_commerce_quote_and_order_support_custom_recharge_from_server_policy() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let quote_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/quote")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"custom_recharge\",\"target_id\":\"custom\",\"custom_amount_cents\":5000,\"current_remaining_units\":260}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(quote_response.status(), StatusCode::OK);
    let quote_json = read_json(quote_response).await;
    assert_eq!(quote_json["target_kind"], "custom_recharge");
    assert_eq!(quote_json["target_id"], "custom-5000");
    assert_eq!(quote_json["target_name"], "Custom recharge");
    assert_eq!(quote_json["amount_cents"], 5000);
    assert_eq!(quote_json["list_price_label"], "$50.00");
    assert_eq!(quote_json["payable_price_label"], "$50.00");
    assert_eq!(quote_json["granted_units"], 140000);
    assert_eq!(quote_json["projected_remaining_units"], 140260);
    assert_eq!(quote_json["pricing_rule_label"], "Tiered custom recharge");
    assert_eq!(quote_json["effective_ratio_label"], "2,800 units / $1");

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"custom_recharge\",\"target_id\":\"custom\",\"custom_amount_cents\":5000}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["target_kind"], "custom_recharge");
    assert_eq!(create_json["target_id"], "custom-5000");
    assert_eq!(create_json["target_name"], "Custom recharge");
    assert_eq!(create_json["payable_price_label"], "$50.00");
    assert_eq!(create_json["granted_units"], 140000);
    assert_eq!(create_json["status"], "pending_payment");

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/settle"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::CONFLICT);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        format!("order {order_id} requires verified payment confirmation before fulfillment")
    );

    settle_portal_order_via_verified_payment(&pool, &order_id, 1_710_000_120).await;

    let billing_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 140260);
}

#[tokio::test]
async fn portal_commerce_orders_queue_paid_checkout_and_fulfill_coupon_redemption() {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::CREATED);
    let recharge_json = read_json(recharge_response).await;
    assert_eq!(recharge_json["project_id"], project_id);
    assert_eq!(recharge_json["user_id"], user_id);
    assert_eq!(recharge_json["target_kind"], "recharge_pack");
    assert_eq!(recharge_json["target_name"], "Boost 100k");
    assert_eq!(recharge_json["payable_price_label"], "$32.00");
    assert_eq!(recharge_json["status"], "pending_payment");

    let billing_after_recharge = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_after_recharge.status(), StatusCode::OK);
    let billing_after_recharge_json = read_json(billing_after_recharge).await;
    assert_eq!(billing_after_recharge_json["remaining_units"], 260);

    let coupon_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"coupon_redemption\",\"target_id\":\"WELCOME100\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(coupon_response.status(), StatusCode::CREATED);
    let coupon_json = read_json(coupon_response).await;
    assert_eq!(coupon_json["target_kind"], "coupon_redemption");
    assert_eq!(coupon_json["bonus_units"], 100);
    assert_eq!(coupon_json["status"], "fulfilled");

    let billing_after_coupon = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_after_coupon.status(), StatusCode::OK);
    let billing_after_coupon_json = read_json(billing_after_coupon).await;
    assert_eq!(billing_after_coupon_json["remaining_units"], 360);

    let history_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(history_response.status(), StatusCode::OK);
    let history_json = read_json(history_response).await;
    assert_eq!(history_json.as_array().unwrap().len(), 2);
    assert_eq!(history_json[0]["status"], "fulfilled");
    assert_eq!(history_json[0]["project_id"], project_id);
    assert_eq!(history_json[1]["status"], "pending_payment");
    assert_eq!(history_json[1]["project_id"], project_id);
}

#[tokio::test]
async fn portal_commerce_pending_recharge_requires_verified_payment_before_fulfillment_and_can_be_canceled(
) {
    let pool = memory_pool().await;
    sqlx::query(
        "INSERT INTO ai_coupon_campaigns (id, code, discount_label, audience, remaining, active, note, expires_on, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("coupon_spring_launch")
    .bind("SPRING20")
    .bind("20% launch discount")
    .bind("new_signup")
    .bind(120_i64)
    .bind(1_i64)
    .bind("Spring launch campaign")
    .bind("2026-05-31")
    .bind(1_710_000_001_i64)
    .execute(&pool)
    .await
    .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let recharge_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\",\"coupon_code\":\"SPRING20\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(recharge_response.status(), StatusCode::CREATED);
    let recharge_json = read_json(recharge_response).await;
    let settled_order_id = recharge_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(recharge_json["status"], "pending_payment");

    let checkout_session_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{settled_order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(checkout_session_response.status(), StatusCode::OK);
    let checkout_session_json = read_json(checkout_session_response).await;
    assert_eq!(checkout_session_json["order_id"], settled_order_id);
    assert_eq!(checkout_session_json["order_status"], "pending_payment");
    assert_eq!(checkout_session_json["session_status"], "open");
    assert_eq!(checkout_session_json["provider"], "payment_orchestrator");
    assert_eq!(checkout_session_json["mode"], "checkout_bridge");
    assert!(checkout_session_json["reference"]
        .as_str()
        .unwrap()
        .starts_with("PAY-"));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["action"] == "provider_handoff"));
    assert!(!checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["action"] == "settle_order"));

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{settled_order_id}/settle"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::CONFLICT);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        format!(
            "order {settled_order_id} requires verified payment confirmation before fulfillment"
        )
    );

    let settled_checkout_session_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{settled_order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settled_checkout_session_response.status(), StatusCode::OK);
    let settled_checkout_session_json = read_json(settled_checkout_session_response).await;
    assert_eq!(settled_checkout_session_json["session_status"], "open");
    assert!(settled_checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["action"] == "provider_handoff"));

    let billing_after_settle = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_after_settle.status(), StatusCode::OK);
    let billing_after_settle_json = read_json(billing_after_settle).await;
    assert_eq!(billing_after_settle_json["remaining_units"], 260);

    let cancel_create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-500k\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_create_response.status(), StatusCode::CREATED);
    let cancel_create_json = read_json(cancel_create_response).await;
    let canceled_order_id = cancel_create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(cancel_create_json["status"], "pending_payment");

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{canceled_order_id}/cancel"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["order_id"], canceled_order_id);
    assert_eq!(cancel_json["status"], "canceled");

    let billing_after_cancel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_after_cancel.status(), StatusCode::OK);
    let billing_after_cancel_json = read_json(billing_after_cancel).await;
    assert_eq!(billing_after_cancel_json["remaining_units"], 260);
}

#[tokio::test]
async fn portal_commerce_subscription_checkout_requires_verified_payment_before_membership_activation(
) {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let subscription_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(subscription_response.status(), StatusCode::CREATED);
    let subscription_json = read_json(subscription_response).await;
    assert_eq!(subscription_json["project_id"], project_id);
    assert_eq!(subscription_json["user_id"], user_id);
    assert_eq!(subscription_json["target_kind"], "subscription_plan");
    assert_eq!(subscription_json["target_name"], "Growth");
    assert_eq!(subscription_json["payable_price_label"], "$79.00");
    assert_eq!(subscription_json["status"], "pending_payment");

    let billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 260);

    let membership_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/membership")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(membership_response.status(), StatusCode::OK);
    let membership_json = read_json(membership_response).await;
    assert!(membership_json.is_null());

    let order_id = subscription_json["order_id"].as_str().unwrap();
    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/portal/commerce/orders/{order_id}/settle"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::CONFLICT);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        format!("order {order_id} requires verified payment confirmation before fulfillment")
    );

    settle_portal_order_via_verified_payment(&pool, order_id, 1_710_000_240).await;

    let settled_billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settled_billing_response.status(), StatusCode::OK);
    let settled_billing_json = read_json(settled_billing_response).await;
    assert_eq!(settled_billing_json["remaining_units"], 99760);

    let settled_membership_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/membership")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settled_membership_response.status(), StatusCode::OK);
    let membership_json = read_json(settled_membership_response).await;
    assert_eq!(membership_json["project_id"], project_id);
    assert_eq!(membership_json["user_id"], user_id);
    assert_eq!(membership_json["plan_id"], "growth");
    assert_eq!(membership_json["plan_name"], "Growth");
    assert_eq!(membership_json["included_units"], 100000);
    assert_eq!(membership_json["cadence"], "/month");
    assert_eq!(membership_json["status"], "active");
}

#[tokio::test]
async fn portal_commerce_payment_events_can_fail_checkout_and_block_invalid_recovery() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["status"], "pending_payment");

    let failed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"failed\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(failed_response.status(), StatusCode::OK);
    let failed_json = read_json(failed_response).await;
    assert_eq!(failed_json["order_id"], order_id);
    assert_eq!(failed_json["status"], "failed");

    let checkout_session_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(checkout_session_response.status(), StatusCode::OK);
    let checkout_session_json = read_json(checkout_session_response).await;
    assert_eq!(checkout_session_json["order_status"], "failed");
    assert_eq!(checkout_session_json["session_status"], "failed");
    assert_eq!(
        checkout_session_json["methods"].as_array().unwrap().len(),
        0
    );

    let billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 260);

    let invalid_recovery_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"settled\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid_recovery_response.status(), StatusCode::CONFLICT);
    let invalid_recovery_json = read_json(invalid_recovery_response).await;
    assert_eq!(
        invalid_recovery_json["error"]["message"],
        format!("order {order_id} cannot be settled from status failed")
    );
}

#[tokio::test]
async fn portal_commerce_verified_payment_callback_activates_membership_and_quota() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let user_id = workspace["user"]["id"].as_str().unwrap().to_owned();

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(&project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(&pool)
    .await
    .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_json = read_json(create_response).await;
    let order_id = create_json["order_id"].as_str().unwrap().to_owned();
    assert_eq!(create_json["status"], "pending_payment");

    let settle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"event_type\":\"settled\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(settle_response.status(), StatusCode::CONFLICT);
    let settle_json = read_json(settle_response).await;
    assert_eq!(
        settle_json["error"]["message"],
        format!("order {order_id} requires verified payment confirmation before fulfillment")
    );

    settle_portal_order_via_verified_payment(&pool, &order_id, 1_710_000_360).await;

    let billing_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(billing_response.status(), StatusCode::OK);
    let billing_json = read_json(billing_response).await;
    assert_eq!(billing_json["remaining_units"], 99760);

    let membership_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/membership")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(membership_response.status(), StatusCode::OK);
    let membership_json = read_json(membership_response).await;
    assert_eq!(membership_json["project_id"], project_id);
    assert_eq!(membership_json["user_id"], user_id);
    assert_eq!(membership_json["plan_id"], "growth");
    assert_eq!(membership_json["status"], "active");
}

#[tokio::test]
async fn portal_commerce_reuses_existing_pending_payable_order_for_repeat_create() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first_response.status(), StatusCode::CREATED);
    let first_json = read_json(first_response).await;
    let first_order_id = first_json["order_id"].as_str().unwrap().to_owned();

    let second_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"target_kind\":\"subscription_plan\",\"target_id\":\"growth\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second_response.status(), StatusCode::CREATED);
    let second_json = read_json(second_response).await;
    assert_eq!(second_json["order_id"], first_order_id);

    let order_center_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/commerce/order-center")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(order_center_response.status(), StatusCode::OK);
    let order_center_json = read_json(order_center_response).await;
    assert_eq!(order_center_json.as_array().unwrap().len(), 1);
    assert_eq!(order_center_json[0]["order"]["order_id"], first_order_id);

    let commerce_order_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ai_commerce_orders WHERE project_id = ?")
            .bind(&project_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(commerce_order_count, 1);

    let payment_order_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ai_payment_order WHERE project_id = ?")
            .bind(&project_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(payment_order_count, 1);
}
