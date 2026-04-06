use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{gateway_auth_subject_from_request_context, GatewayRequestContext};
use sdkwork_api_domain_billing::{
    AccountCommerceReconciliationStateRecord, AccountRecord, AccountStatus, AccountType,
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

#[tokio::test]
async fn portal_commerce_orders_list_reflects_refunded_recharge_flow() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let refunded_event_id = format!("evt_refunded_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);
    let settled_json = read_json(settled_response).await;
    assert_eq!(settled_json["status"], "fulfilled");

    let refunded_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refunded_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(refunded_response.status(), StatusCode::OK);
    let refunded_json = read_json(refunded_response).await;
    assert_eq!(refunded_json["order_id"], order_id);
    assert_eq!(refunded_json["status"], "refunded");

    let response = app
        .clone()
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

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["order_id"], order_id);
    assert_eq!(json[0]["status"], "refunded");

    let checkout_response = app
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

    assert_eq!(checkout_response.status(), StatusCode::OK);
    let checkout_json = read_json(checkout_response).await;
    assert_eq!(checkout_json["order_status"], "refunded");
    assert_eq!(checkout_json["session_status"], "refunded");
    assert_eq!(checkout_json["methods"].as_array().unwrap().len(), 0);

    let events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| {
        event.event_type == "settled"
            && event.provider == "stripe"
            && event.provider_event_id.as_deref() == Some(settled_event_id.as_str())
            && event.processing_status.as_str() == "processed"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
    assert!(events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider == "stripe"
            && event.provider_event_id.as_deref() == Some(refunded_event_id.as_str())
            && event.processing_status.as_str() == "processed"
            && event.order_status_after.as_deref() == Some("refunded")
    }));
}

#[tokio::test]
async fn portal_refund_payment_event_is_idempotent_for_owned_paid_recharge_order() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let refund_event_id = format!("evt_refund_idempotent_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

    let first_refund_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refund_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(first_refund_response.status(), StatusCode::OK);
    let first_refund_json = read_json(first_refund_response).await;
    assert_eq!(first_refund_json["status"], "refunded");

    let replay_response = apply_portal_payment_event(
        app,
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refund_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(replay_response.status(), StatusCode::OK);
    let replay_json = read_json(replay_response).await;
    assert_eq!(replay_json["order_id"], order_id);
    assert_eq!(replay_json["status"], "refunded");

    let events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    let refunded_events = events
        .iter()
        .filter(|event| event.event_type == "refunded")
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 2);
    assert_eq!(refunded_events.len(), 1);
    assert_eq!(
        refunded_events[0].dedupe_key,
        format!("stripe:{refund_event_id}")
    );
    assert_eq!(refunded_events[0].processing_status.as_str(), "processed");
    assert_eq!(
        refunded_events[0].order_status_after.as_deref(),
        Some("refunded")
    );
}

#[tokio::test]
async fn portal_refund_payment_event_restores_quota_and_blocks_unsafe_recovery() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let refunded_event_id = format!("evt_refunded_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

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
    assert_eq!(billing_after_settle_json["remaining_units"], 100260);

    let refunded_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refunded_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(refunded_response.status(), StatusCode::OK);
    let refunded_json = read_json(refunded_response).await;
    assert_eq!(refunded_json["status"], "refunded");

    let billing_after_refund = app
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

    assert_eq!(billing_after_refund.status(), StatusCode::OK);
    let billing_after_refund_json = read_json(billing_after_refund).await;
    assert_eq!(billing_after_refund_json["remaining_units"], 260);

    let unsafe_order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let unsafe_settled_event_id = format!("evt_settled_{unsafe_order_id}");
    let unsafe_refund_event_id = format!("evt_refunded_{unsafe_order_id}");

    let unsafe_settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &unsafe_order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{unsafe_settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(unsafe_settled_response.status(), StatusCode::OK);

    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(&project_id)
    .bind(100_000_i64)
    .bind(175.0_f64)
    .execute(&pool)
    .await
    .unwrap();

    let unsafe_refund_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &unsafe_order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{unsafe_refund_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(unsafe_refund_response.status(), StatusCode::CONFLICT);
    let unsafe_refund_json = read_json(unsafe_refund_response).await;
    assert_eq!(
        unsafe_refund_json["error"]["message"],
        format!(
            "order {unsafe_order_id} cannot be refunded because recharge headroom has already been consumed"
        )
    );

    let unsafe_checkout_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/portal/commerce/orders/{unsafe_order_id}/checkout-session"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unsafe_checkout_response.status(), StatusCode::OK);
    let unsafe_checkout_json = read_json(unsafe_checkout_response).await;
    assert_eq!(unsafe_checkout_json["order_status"], "fulfilled");
    assert_eq!(unsafe_checkout_json["session_status"], "settled");

    let unsafe_events = store
        .list_commerce_payment_events_for_order(&unsafe_order_id)
        .await
        .unwrap();
    assert_eq!(unsafe_events.len(), 2);
    assert!(unsafe_events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider_event_id.as_deref() == Some(unsafe_refund_event_id.as_str())
            && event.processing_status.as_str() == "rejected"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
}

#[tokio::test]
async fn portal_commerce_order_center_aggregates_order_payment_and_checkout_views() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let refunded_event_id = format!("evt_refunded_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

    let refunded_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"stripe\",\"provider_event_id\":\"{refunded_event_id}\"}}"
        ),
    )
    .await;
    assert_eq!(refunded_response.status(), StatusCode::OK);

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["project_id"], project_id);
    assert!(json["membership"].is_null());
    assert_eq!(json["orders"].as_array().unwrap().len(), 1);
    assert_eq!(json["orders"][0]["order"]["order_id"], order_id);
    assert_eq!(json["orders"][0]["order"]["status"], "refunded");
    assert_eq!(
        json["orders"][0]["payment_events"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        json["orders"][0]["latest_payment_event"]["event_type"],
        "refunded"
    );
    assert_eq!(json["orders"][0]["checkout_session"]["order_status"], "refunded");
    assert_eq!(
        json["orders"][0]["checkout_session"]["session_status"],
        "refunded"
    );
}

#[tokio::test]
async fn portal_commerce_order_center_reports_reconciliation_backlog_for_workspace_account() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;
    let account = seed_portal_workspace_commercial_account(&store, &workspace).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    store
        .insert_account_commerce_reconciliation_state(
            &AccountCommerceReconciliationStateRecord::new(
                account.tenant_id,
                account.organization_id,
                account.account_id,
                &project_id,
                "checkpoint-order",
            )
            .with_last_order_updated_at_ms(1)
            .with_last_order_created_at_ms(1)
            .with_updated_at_ms(1),
        )
        .await
        .unwrap();

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["orders"].as_array().unwrap().len(), 1);
    assert_eq!(json["orders"][0]["order"]["order_id"], order_id);
    assert_eq!(json["reconciliation"]["account_id"], account.account_id);
    assert_eq!(
        json["reconciliation"]["last_reconciled_order_id"],
        "checkpoint-order"
    );
    assert_eq!(
        json["reconciliation"]["last_reconciled_order_updated_at_ms"],
        1
    );
    assert_eq!(
        json["reconciliation"]["last_reconciled_order_created_at_ms"],
        1
    );
    assert_eq!(json["reconciliation"]["last_reconciled_at_ms"], 1);
    assert_eq!(json["reconciliation"]["backlog_order_count"], 1);
    assert_eq!(json["reconciliation"]["healthy"], false);
    assert!(json["reconciliation"]["checkpoint_lag_ms"].as_u64().unwrap() > 0);
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
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
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

fn workspace_request_context(workspace: &Value) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace["tenant"]["id"].as_str().unwrap().to_owned(),
        project_id: workspace["project"]["id"].as_str().unwrap().to_owned(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
    }
}

async fn seed_portal_workspace_commercial_account(
    store: &SqliteAdminStore,
    workspace: &Value,
) -> AccountRecord {
    let subject = gateway_auth_subject_from_request_context(&workspace_request_context(workspace));
    let account = AccountRecord::new(
        7001,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        AccountType::Primary,
    )
    .with_status(AccountStatus::Active)
    .with_currency_code("USD")
    .with_credit_unit_code("credit")
    .with_created_at_ms(10)
    .with_updated_at_ms(10);

    store.insert_account_record(&account).await.unwrap();
    account
}

async fn create_portal_recharge_order(app: axum::Router, token: &str, body_json: &str) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/commerce/orders")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body_json.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await["order_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn apply_portal_payment_event(
    app: axum::Router,
    token: &str,
    order_id: &str,
    body_json: &str,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(&format!(
                "/portal/commerce/orders/{order_id}/payment-events"
            ))
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(body_json.to_owned()))
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn seed_portal_recharge_capacity_fixture(pool: &SqlitePool, project_id: &str) {
    sqlx::query(
        "INSERT INTO ai_billing_ledger_entries (project_id, units, amount) VALUES (?, ?, ?)",
    )
    .bind(project_id)
    .bind(240_i64)
    .bind(0.42_f64)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO ai_billing_quota_policies (policy_id, project_id, max_units, enabled)
         VALUES (?, ?, ?, ?)",
    )
    .bind("quota-portal")
    .bind(project_id)
    .bind(500_i64)
    .bind(1_i64)
    .execute(pool)
    .await
    .unwrap();
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

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

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
async fn portal_commerce_pending_recharge_can_be_settled_or_canceled() {
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
    assert_eq!(checkout_session_json["provider"], "manual_lab");
    assert_eq!(checkout_session_json["mode"], "operator_settlement");
    assert!(checkout_session_json["reference"]
        .as_str()
        .unwrap()
        .starts_with("PAY-"));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["action"] == "settle_order"
                && method["provider"] == "manual_lab"
                && method["channel"] == "operator_settlement"
                && method["session_kind"] == "operator_action"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("MANUAL-")
                && method["qr_code_payload"].is_null()
                && method["webhook_verification"] == "manual"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == false
                && method["supports_webhook"] == false
        }));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["action"] == "provider_handoff"
                && method["provider"] == "stripe"
                && method["channel"] == "hosted_checkout"
                && method["session_kind"] == "hosted_checkout"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("STRIPE-")
                && method["qr_code_payload"].is_null()
                && method["webhook_verification"] == "stripe_signature"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == true
                && method["recommended"] == true
                && method["supports_webhook"] == true
        }));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["provider"] == "alipay"
                && method["channel"] == "scan_qr"
                && method["session_kind"] == "qr_code"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("ALIPAY-")
                && method["qr_code_payload"]
                    .as_str()
                    .unwrap()
                    .contains("sdkworkpay://alipay_qr/")
                && method["webhook_verification"] == "alipay_rsa_sha256"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == false
                && method["supports_webhook"] == true
        }));
    assert!(checkout_session_json["methods"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| {
            method["provider"] == "wechat_pay"
                && method["channel"] == "scan_qr"
                && method["session_kind"] == "qr_code"
                && method["session_reference"]
                    .as_str()
                    .unwrap()
                    .starts_with("WECHAT-")
                && method["qr_code_payload"]
                    .as_str()
                    .unwrap()
                    .contains("sdkworkpay://wechat_pay_qr/")
                && method["webhook_verification"] == "wechatpay_rsa_sha256"
                && method["supports_refund"] == true
                && method["supports_partial_refund"] == false
                && method["supports_webhook"] == true
        }));

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

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["order_id"], settled_order_id);
    assert_eq!(settle_json["status"], "fulfilled");

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
    assert_eq!(settled_checkout_session_json["session_status"], "settled");
    assert_eq!(
        settled_checkout_session_json["methods"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

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
    assert_eq!(billing_after_settle_json["remaining_units"], 100260);

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
    assert_eq!(billing_after_cancel_json["remaining_units"], 100260);
}

#[tokio::test]
async fn portal_commerce_subscription_checkout_requires_settlement_before_membership_activation() {
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

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

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
async fn portal_commerce_payment_events_reject_unsupported_provider_names() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

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

    let invalid_provider_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/portal/commerce/orders/{order_id}/payment-events"
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"event_type\":\"settled\",\"provider\":\"paypal\",\"provider_event_id\":\"evt_paypal\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid_provider_response.status(), StatusCode::BAD_REQUEST);
    let invalid_provider_json = read_json(invalid_provider_response).await;
    assert_eq!(
        invalid_provider_json["error"]["message"],
        "unsupported commerce payment provider: paypal"
    );
}

#[tokio::test]
async fn portal_refund_payment_event_rejects_provider_mismatch_against_settlement_provider() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;
    let settled_event_id = format!("evt_settled_{order_id}");
    let mismatched_refund_event_id = format!("evt_refund_alipay_{order_id}");

    let settled_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"settled\",\"provider\":\"stripe\",\"provider_event_id\":\"{settled_event_id}\",\"checkout_method_id\":\"stripe_checkout\"}}"
        ),
    )
    .await;
    assert_eq!(settled_response.status(), StatusCode::OK);

    let mismatched_refund_response = apply_portal_payment_event(
        app.clone(),
        &token,
        &order_id,
        &format!(
            "{{\"event_type\":\"refunded\",\"provider\":\"alipay\",\"provider_event_id\":\"{mismatched_refund_event_id}\",\"checkout_method_id\":\"alipay_qr\"}}"
        ),
    )
    .await;
    assert_eq!(mismatched_refund_response.status(), StatusCode::CONFLICT);
    let mismatched_refund_json = read_json(mismatched_refund_response).await;
    assert_eq!(
        mismatched_refund_json["error"]["message"],
        format!(
            "refund provider alipay does not match settled provider stripe for order {order_id}"
        )
    );

    let events = store
        .list_commerce_payment_events_for_order(&order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider == "alipay"
            && event.provider_event_id.as_deref() == Some(mismatched_refund_event_id.as_str())
            && event.processing_status.as_str() == "rejected"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
}

#[tokio::test]
async fn portal_provider_backed_payment_events_require_provider_event_id_without_checkout_method_hint() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool.clone());
    let token = portal_token(app.clone()).await;
    let workspace = portal_workspace(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();

    seed_portal_recharge_capacity_fixture(&pool, &project_id).await;

    let order_id = create_portal_recharge_order(
        app.clone(),
        &token,
        "{\"target_kind\":\"recharge_pack\",\"target_id\":\"pack-100k\"}",
    )
    .await;

    let response = apply_portal_payment_event(
        app,
        &token,
        &order_id,
        "{\"event_type\":\"settled\",\"provider\":\"stripe\"}",
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "provider_event_id is required for provider-backed payment events"
    );
}

#[tokio::test]
async fn portal_commerce_payment_settlement_event_activates_membership_and_quota() {
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

    assert_eq!(settle_response.status(), StatusCode::OK);
    let settle_json = read_json(settle_response).await;
    assert_eq!(settle_json["status"], "fulfilled");

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
