use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
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

async fn login_token(app: Router) -> String {
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

#[tokio::test]
async fn admin_marketing_routes_create_and_list_canonical_coupon_records() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_template_id":"template_launch20",
                        "template_key":"launch20",
                        "display_name":"Launch 20",
                        "status":"active",
                        "distribution_kind":"unique_code",
                        "benefit":{"benefit_kind":"percentage_off","discount_percent":20},
                        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::CREATED);

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "marketing_campaign_id":"campaign_launch20",
                        "coupon_template_id":"template_launch20",
                        "display_name":"Launch Campaign",
                        "status":"active",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::CREATED);

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "campaign_budget_id":"budget_launch20",
                        "marketing_campaign_id":"campaign_launch20",
                        "status":"active",
                        "total_budget_minor":500000,
                        "reserved_budget_minor":0,
                        "consumed_budget_minor":0,
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::CREATED);

    let code = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "coupon_code_id":"code_launch20",
                        "coupon_template_id":"template_launch20",
                        "code_value":"LAUNCH20",
                        "status":"available",
                        "created_at_ms":1710000000000,
                        "updated_at_ms":1710000000000
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::CREATED);

    let templates = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(templates.status(), StatusCode::OK);
    let templates_json = read_json(templates).await;
    assert_eq!(templates_json.as_array().unwrap().len(), 1);
    assert_eq!(templates_json[0]["template_key"], "launch20");

    let codes = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(codes.status(), StatusCode::OK);
    let codes_json = read_json(codes).await;
    assert_eq!(codes_json.as_array().unwrap().len(), 1);
    assert_eq!(codes_json[0]["code_value"], "LAUNCH20");

    let coupons = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(coupons.status(), StatusCode::OK);
    let coupons_json = read_json(coupons).await;
    assert!(coupons_json
        .as_array()
        .unwrap()
        .iter()
        .any(|coupon| coupon["code"] == "LAUNCH20"));
}

#[tokio::test]
async fn admin_compatibility_coupon_create_populates_marketing_projection() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"coupon_spring_launch\",\"code\":\"SPRING20\",\"discount_label\":\"20% launch discount\",\"audience\":\"new_signup\",\"remaining\":120,\"active\":true,\"note\":\"Spring launch campaign\",\"expires_on\":\"2026-05-31\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);

    let templates = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/coupon-templates")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(templates.status(), StatusCode::OK);
    let templates_json = read_json(templates).await;
    assert!(templates_json
        .as_array()
        .unwrap()
        .iter()
        .any(|template| template["template_key"] == "spring20"));

    let codes = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/marketing/codes")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(codes.status(), StatusCode::OK);
    let codes_json = read_json(codes).await;
    assert!(codes_json
        .as_array()
        .unwrap()
        .iter()
        .any(|code| code["code_value"] == "SPRING20"));
}

#[tokio::test]
async fn admin_marketing_status_routes_update_canonical_coupon_records() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let template_payload = r#"{
        "coupon_template_id":"template_launch20",
        "template_key":"launch20",
        "display_name":"Launch 20",
        "status":"active",
        "distribution_kind":"unique_code",
        "benefit":{"benefit_kind":"percentage_off","discount_percent":20},
        "restriction":{"subject_scope":"project","stacking_policy":"exclusive"},
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;
    let campaign_payload = r#"{
        "marketing_campaign_id":"campaign_launch20",
        "coupon_template_id":"template_launch20",
        "display_name":"Launch Campaign",
        "status":"active",
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;
    let budget_payload = r#"{
        "campaign_budget_id":"budget_launch20",
        "marketing_campaign_id":"campaign_launch20",
        "status":"active",
        "total_budget_minor":500000,
        "reserved_budget_minor":0,
        "consumed_budget_minor":0,
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;
    let code_payload = r#"{
        "coupon_code_id":"code_launch20",
        "coupon_template_id":"template_launch20",
        "code_value":"LAUNCH20",
        "status":"available",
        "created_at_ms":1710000000000,
        "updated_at_ms":1710000000000
    }"#;

    for (uri, payload) in [
        ("/admin/marketing/coupon-templates", template_payload),
        ("/admin/marketing/campaigns", campaign_payload),
        ("/admin/marketing/budgets", budget_payload),
        ("/admin/marketing/codes", code_payload),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let template = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/coupon-templates/template_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"archived"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(template.status(), StatusCode::OK);
    assert_eq!(read_json(template).await["status"], "archived");

    let campaign = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/campaigns/campaign_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"paused"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(campaign.status(), StatusCode::OK);
    assert_eq!(read_json(campaign).await["status"], "paused");

    let budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/budgets/budget_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"closed"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(budget.status(), StatusCode::OK);
    assert_eq!(read_json(budget).await["status"], "closed");

    let code = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/marketing/codes/code_launch20/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"disabled"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(code.status(), StatusCode::OK);
    assert_eq!(read_json(code).await["status"], "disabled");
}
