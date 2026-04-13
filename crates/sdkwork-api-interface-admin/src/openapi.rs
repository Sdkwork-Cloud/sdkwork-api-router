use super::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Admin API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current admin router implementation."
    ),
    modifiers(&AdminApiDocModifier),
    tags(
        (name = "system", description = "Admin health and system-facing routes."),
        (name = "auth", description = "Admin authentication and session management routes."),
        (name = "catalog", description = "Provider and model catalog administration routes."),
        (name = "marketing", description = "Coupon template, campaign, budget, and redemption administration routes."),
        (name = "tenants", description = "Tenant and project administration routes."),
        (name = "users", description = "Operator and portal user administration routes."),
        (name = "gateway", description = "Gateway API key and API key group administration routes."),
        (name = "billing", description = "Billing summary, event, and ledger administration routes."),
        (name = "commerce", description = "Recent order and payment callback audit routes.")
    )
)]
struct AdminApiDoc;

struct AdminApiDocModifier;

impl Modify for AdminApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/")]);
        openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
    }
}

mod auth_paths;
mod billing_paths;
mod catalog_paths;
mod commerce_paths;
mod gateway_paths;
mod marketing_budget_paths;
mod marketing_campaign_paths;
mod marketing_code_paths;
mod marketing_runtime_paths;
mod marketing_template_paths;
mod system_paths;
mod tenant_paths;
mod user_paths;

fn admin_openapi() -> utoipa::openapi::OpenApi {
    OpenApiRouter::<()>::with_openapi(AdminApiDoc::openapi())
        .routes(routes!(system_paths::health))
        .routes(routes!(auth_paths::auth_login))
        .routes(routes!(auth_paths::auth_change_password))
        .routes(routes!(tenant_paths::tenants_list))
        .routes(routes!(tenant_paths::tenants_create))
        .routes(routes!(catalog_paths::tenant_provider_readiness_list))
        .routes(routes!(tenant_paths::projects_list))
        .routes(routes!(tenant_paths::projects_create))
        .routes(routes!(catalog_paths::providers_list))
        .routes(routes!(catalog_paths::providers_create))
        .routes(routes!(user_paths::operator_users_list))
        .routes(routes!(user_paths::operator_users_upsert))
        .routes(routes!(user_paths::operator_user_status_update))
        .routes(routes!(user_paths::portal_users_list))
        .routes(routes!(user_paths::portal_users_upsert))
        .routes(routes!(user_paths::portal_user_status_update))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_list
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_create
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_status_update
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_clone
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_compare
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_submit_for_approval
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_approve
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_reject
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_publish
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_schedule
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_templates_retire
        ))
        .routes(routes!(
            marketing_template_paths::marketing_coupon_template_lifecycle_audits_list
        ))
        .routes(routes!(marketing_campaign_paths::marketing_campaigns_list))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_create
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_status_update
        ))
        .routes(routes!(marketing_campaign_paths::marketing_campaigns_clone))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_compare
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_submit_for_approval
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_approve
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_reject
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_publish
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_schedule
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaigns_retire
        ))
        .routes(routes!(
            marketing_campaign_paths::marketing_campaign_lifecycle_audits_list
        ))
        .routes(routes!(marketing_budget_paths::marketing_budgets_list))
        .routes(routes!(marketing_budget_paths::marketing_budgets_create))
        .routes(routes!(
            marketing_budget_paths::marketing_budgets_status_update
        ))
        .routes(routes!(marketing_budget_paths::marketing_budgets_activate))
        .routes(routes!(marketing_budget_paths::marketing_budgets_close))
        .routes(routes!(
            marketing_budget_paths::marketing_budget_lifecycle_audits_list
        ))
        .routes(routes!(marketing_code_paths::marketing_codes_list))
        .routes(routes!(marketing_code_paths::marketing_codes_create))
        .routes(routes!(marketing_code_paths::marketing_codes_status_update))
        .routes(routes!(marketing_code_paths::marketing_codes_disable))
        .routes(routes!(marketing_code_paths::marketing_codes_restore))
        .routes(routes!(
            marketing_code_paths::marketing_code_lifecycle_audits_list
        ))
        .routes(routes!(
            marketing_runtime_paths::marketing_reservations_list
        ))
        .routes(routes!(marketing_runtime_paths::marketing_redemptions_list))
        .routes(routes!(marketing_runtime_paths::marketing_rollbacks_list))
        .routes(routes!(gateway_paths::api_keys_list))
        .routes(routes!(gateway_paths::api_keys_create))
        .routes(routes!(gateway_paths::api_key_update))
        .routes(routes!(gateway_paths::api_key_groups_list))
        .routes(routes!(gateway_paths::api_key_groups_create))
        .routes(routes!(gateway_paths::api_key_group_update))
        .routes(routes!(billing_paths::billing_ledger_list))
        .routes(routes!(billing_paths::billing_events_list))
        .routes(routes!(billing_paths::billing_events_summary))
        .routes(routes!(billing_paths::billing_summary))
        .routes(routes!(
            billing_paths::billing_pricing_lifecycle_synchronize
        ))
        .routes(routes!(billing_paths::billing_account_ledger))
        .routes(routes!(commerce_paths::commerce_orders_recent))
        .routes(routes!(commerce_paths::commerce_catalog_publications_list))
        .routes(routes!(commerce_paths::commerce_catalog_publication_detail))
        .routes(routes!(
            commerce_paths::commerce_catalog_publication_publish
        ))
        .routes(routes!(
            commerce_paths::commerce_catalog_publication_schedule
        ))
        .routes(routes!(commerce_paths::commerce_catalog_publication_retire))
        .routes(routes!(commerce_paths::commerce_payment_methods_list))
        .routes(routes!(commerce_paths::commerce_payment_method_put))
        .routes(routes!(commerce_paths::commerce_payment_method_delete))
        .routes(routes!(
            commerce_paths::commerce_payment_method_bindings_list
        ))
        .routes(routes!(
            commerce_paths::commerce_payment_method_bindings_replace
        ))
        .routes(routes!(commerce_paths::commerce_order_payment_events))
        .routes(routes!(commerce_paths::commerce_order_payment_attempts))
        .routes(routes!(commerce_paths::commerce_order_refunds_list))
        .routes(routes!(commerce_paths::commerce_order_refunds_create))
        .routes(routes!(commerce_paths::commerce_order_audit))
        .routes(routes!(commerce_paths::commerce_webhook_inbox_list))
        .routes(routes!(
            commerce_paths::commerce_webhook_delivery_attempts_list
        ))
        .routes(routes!(commerce_paths::commerce_reconciliation_runs_list))
        .routes(routes!(commerce_paths::commerce_reconciliation_runs_create))
        .routes(routes!(commerce_paths::commerce_reconciliation_items_list))
        .into_openapi()
}

async fn admin_openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(admin_openapi())
}

async fn admin_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Admin API</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }

      body {
        margin: 0;
        background: #f5f7fb;
        color: #101828;
      }

      .shell {
        display: grid;
        min-height: 100vh;
        grid-template-rows: auto 1fr;
      }

      .hero {
        padding: 20px 24px 16px;
        border-bottom: 1px solid rgba(15, 23, 42, 0.08);
        background: rgba(255, 255, 255, 0.96);
      }

      .eyebrow {
        margin: 0 0 8px;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: #475467;
      }

      h1 {
        margin: 0 0 8px;
        font-size: 28px;
        line-height: 1.1;
      }

      p {
        margin: 0;
        font-size: 14px;
        line-height: 1.6;
        color: #475467;
      }

      code {
        padding: 2px 6px;
        border-radius: 999px;
        background: rgba(15, 23, 42, 0.06);
        font-size: 12px;
      }

      iframe {
        width: 100%;
        height: 100%;
        border: 0;
        background: white;
      }

      @media (prefers-color-scheme: dark) {
        body {
          background: #09090b;
          color: #fafafa;
        }

        .hero {
          background: rgba(24, 24, 27, 0.96);
          border-bottom-color: rgba(255, 255, 255, 0.08);
        }

        .eyebrow,
        p {
          color: #a1a1aa;
        }

        code {
          background: rgba(255, 255, 255, 0.08);
        }
      }
    </style>
  </head>
  <body>
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">OpenAPI 3.1</p>
        <h1>SDKWORK Admin API</h1>
        <p>Interactive documentation is backed by the live schema endpoint <code>/admin/openapi.json</code>.</p>
      </section>
      <iframe src="/admin/docs/ui/" title="SDKWORK Admin API"></iframe>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

pub(crate) fn admin_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_index_handler))
        .merge(
            SwaggerUi::new("/admin/docs/ui/").config(SwaggerUiConfig::new([
                SwaggerUiUrl::with_primary("SDKWORK Admin API", "/admin/openapi.json", true),
            ])),
        )
}
