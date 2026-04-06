use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::sync::{Mutex, OnceLock};
use tower::ServiceExt;

fn http_exposure_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[tokio::test]
async fn openapi_routes_expose_admin_api_inventory_with_schema_components() {
    let _lock = http_exposure_env_lock().lock().unwrap();
    let app = sdkwork_api_interface_admin::admin_router();

    let openapi = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(openapi.status(), StatusCode::OK);
    let bytes = to_bytes(openapi.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["openapi"], "3.1.0");
    assert_eq!(json["info"]["title"], "SDKWORK Admin API");
    assert!(json["paths"]["/admin/health"]["get"].is_object());
    assert!(json["paths"]["/admin/auth/login"]["post"].is_object());
    assert!(json["paths"]["/admin/tenants"]["get"].is_object());
    assert!(json["paths"]["/admin/tenants"]["post"].is_object());
    assert!(json["paths"]["/admin/projects"]["get"].is_object());
    assert!(json["paths"]["/admin/projects"]["post"].is_object());
    assert!(json["paths"]["/admin/users/operators"]["get"].is_object());
    assert!(json["paths"]["/admin/users/operators"]["post"].is_object());
    assert!(json["paths"]["/admin/users/portal"]["get"].is_object());
    assert!(json["paths"]["/admin/users/portal"]["post"].is_object());
    assert!(json["paths"]["/admin/marketing/coupon-templates"]["get"].is_object());
    assert!(json["paths"]["/admin/marketing/coupon-templates"]["post"].is_object());
    assert!(
        json["paths"]["/admin/marketing/coupon-templates/{coupon_template_id}/status"]["post"]
            .is_object()
    );
    assert!(json["paths"]["/admin/marketing/campaigns"]["get"].is_object());
    assert!(json["paths"]["/admin/marketing/campaigns"]["post"].is_object());
    assert!(
        json["paths"]["/admin/marketing/campaigns/{marketing_campaign_id}/status"]["post"]
            .is_object()
    );
    assert!(json["paths"]["/admin/marketing/budgets"]["get"].is_object());
    assert!(json["paths"]["/admin/marketing/budgets"]["post"].is_object());
    assert!(
        json["paths"]["/admin/marketing/budgets/{campaign_budget_id}/status"]["post"].is_object()
    );
    assert!(json["paths"]["/admin/marketing/codes"]["get"].is_object());
    assert!(json["paths"]["/admin/marketing/codes"]["post"].is_object());
    assert!(json["paths"]["/admin/marketing/codes/{coupon_code_id}/status"]["post"].is_object());
    assert!(json["paths"]["/admin/marketing/reservations"]["get"].is_object());
    assert!(json["paths"]["/admin/marketing/redemptions"]["get"].is_object());
    assert!(json["paths"]["/admin/marketing/rollbacks"]["get"].is_object());
    assert!(json["paths"]["/admin/api-keys"]["get"].is_object());
    assert!(json["paths"]["/admin/api-keys"]["post"].is_object());
    assert!(json["paths"]["/admin/api-key-groups"]["get"].is_object());
    assert!(json["paths"]["/admin/api-key-groups"]["post"].is_object());
    assert!(json["paths"]["/admin/billing/ledger"]["get"].is_object());
    assert!(json["paths"]["/admin/billing/events"]["get"].is_object());
    assert!(json["paths"]["/admin/billing/events/summary"]["get"].is_object());
    assert!(json["paths"]["/admin/billing/summary"]["get"].is_object());
    assert!(json["paths"]["/admin/commerce/orders/{order_id}/audit"]["get"].is_object());
    assert!(json["components"]["schemas"].is_object());
    assert!(json["components"]["schemas"]["LoginRequest"].is_object());
    assert!(json["components"]["schemas"]["LoginResponse"].is_object());
    assert!(json["components"]["schemas"]["Claims"].is_object());
    assert!(json["components"]["schemas"]["AdminUserProfile"].is_object());
    assert!(json["components"]["schemas"]["ErrorResponse"].is_object());
    assert!(json["components"]["schemas"]["Tenant"].is_object());
    assert!(json["components"]["schemas"]["CreateTenantRequest"].is_object());
    assert!(json["components"]["schemas"]["Project"].is_object());
    assert!(json["components"]["schemas"]["CreateProjectRequest"].is_object());
    assert!(json["components"]["schemas"]["UpsertOperatorUserRequest"].is_object());
    assert!(json["components"]["schemas"]["PortalUserProfile"].is_object());
    assert!(json["components"]["schemas"]["UpsertPortalUserRequest"].is_object());
    assert!(json["components"]["schemas"]["CouponTemplateRecord"].is_object());
    assert!(json["components"]["schemas"]["UpdateCouponTemplateStatusRequest"].is_object());
    assert!(json["components"]["schemas"]["MarketingCampaignRecord"].is_object());
    assert!(json["components"]["schemas"]["UpdateMarketingCampaignStatusRequest"].is_object());
    assert!(json["components"]["schemas"]["CampaignBudgetRecord"].is_object());
    assert!(json["components"]["schemas"]["UpdateCampaignBudgetStatusRequest"].is_object());
    assert!(json["components"]["schemas"]["CouponCodeRecord"].is_object());
    assert!(json["components"]["schemas"]["UpdateCouponCodeStatusRequest"].is_object());
    assert!(json["components"]["schemas"]["CouponReservationRecord"].is_object());
    assert!(json["components"]["schemas"]["CouponRedemptionRecord"].is_object());
    assert!(json["components"]["schemas"]["CouponRollbackRecord"].is_object());
    assert!(json["components"]["schemas"]["GatewayApiKeyRecord"].is_object());
    assert!(json["components"]["schemas"]["CreatedGatewayApiKey"].is_object());
    assert!(json["components"]["schemas"]["CreateApiKeyRequest"].is_object());
    assert!(json["components"]["schemas"]["ApiKeyGroupRecord"].is_object());
    assert!(json["components"]["schemas"]["CreateApiKeyGroupRequest"].is_object());
    assert!(json["components"]["schemas"]["LedgerEntry"].is_object());
    assert!(json["components"]["schemas"]["BillingEventRecord"].is_object());
    assert!(json["components"]["schemas"]["BillingEventSummary"].is_object());
    assert!(json["components"]["schemas"]["BillingSummary"].is_object());
    assert!(json["components"]["schemas"]["CommerceOrderAuditRecord"].is_object());
    assert_eq!(
        json["paths"]["/admin/auth/login"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/LoginRequest"
    );
    assert_eq!(
        json["paths"]["/admin/auth/login"]["post"]["responses"]["200"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/LoginResponse"
    );
    assert_eq!(
        json["paths"]["/admin/tenants"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/CreateTenantRequest"
    );
    assert_eq!(
        json["paths"]["/admin/tenants"]["get"]["responses"]["200"]["content"]["application/json"]["schema"]
            ["items"]["$ref"],
        "#/components/schemas/Tenant"
    );
    assert_eq!(
        json["paths"]["/admin/projects"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/CreateProjectRequest"
    );
    assert_eq!(
        json["paths"]["/admin/users/operators"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/UpsertOperatorUserRequest"
    );
    assert_eq!(
        json["paths"]["/admin/users/portal"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/UpsertPortalUserRequest"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/coupon-templates"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/CouponTemplateRecord"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/campaigns"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/MarketingCampaignRecord"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/campaigns/{marketing_campaign_id}/status"]["post"]["requestBody"]
            ["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/UpdateMarketingCampaignStatusRequest"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/budgets"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/CampaignBudgetRecord"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/budgets/{campaign_budget_id}/status"]["post"]["requestBody"]
            ["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/UpdateCampaignBudgetStatusRequest"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/codes"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/CouponCodeRecord"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/codes/{coupon_code_id}/status"]["post"]["requestBody"]["content"]
            ["application/json"]["schema"]["$ref"],
        "#/components/schemas/UpdateCouponCodeStatusRequest"
    );
    assert_eq!(
        json["paths"]["/admin/marketing/coupon-templates/{coupon_template_id}/status"]["post"]["requestBody"]
            ["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/UpdateCouponTemplateStatusRequest"
    );
    assert_eq!(
        json["paths"]["/admin/api-keys"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/CreateApiKeyRequest"
    );
    assert_eq!(
        json["paths"]["/admin/api-key-groups"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/CreateApiKeyGroupRequest"
    );
    assert_eq!(
        json["paths"]["/admin/tenants"]["get"]["security"][0]["bearerAuth"],
        serde_json::json!([])
    );
    assert!(
        json["paths"]["/admin/auth/login"]["post"]["security"].is_null()
            || json["paths"]["/admin/auth/login"]["post"]["security"]
                .as_array()
                .is_some_and(Vec::is_empty)
    );

    let docs = app
        .oneshot(
            Request::builder()
                .uri("/admin/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(docs.status(), StatusCode::OK);
    let bytes = to_bytes(docs.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("SDKWORK Admin API"));
    assert!(html.contains("/admin/openapi.json"));
}

#[test]
fn try_admin_router_returns_error_for_invalid_http_exposure_env() {
    let _lock = http_exposure_env_lock().lock().unwrap();
    let key = "SDKWORK_BROWSER_ALLOWED_ORIGINS";
    let previous = std::env::var(key).ok();
    std::env::set_var(key, ";;;");

    let result = sdkwork_api_interface_admin::try_admin_router();

    match previous {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }

    let error = result.expect_err("invalid env should return an error");
    assert!(
        error
            .to_string()
            .contains("invalid list value for SDKWORK_BROWSER_ALLOWED_ORIGINS")
    );
}
