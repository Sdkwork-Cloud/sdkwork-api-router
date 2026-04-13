use super::*;

#[utoipa::path(
    get,
    path = "/admin/commerce/orders",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(
        ("limit" = Option<usize>, Query, description = "Maximum number of recent commerce orders to return. Defaults to 24 and is capped at 100.")
    ),
    responses(
        (status = 200, description = "Recent commerce orders ordered by newest activity first.", body = [CommerceOrderRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load recent commerce orders.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_orders_recent() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/catalog-publications",
    tag = "commerce",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Canonical commercial publication projections derived from the current product, offer, and pricing governance truth.", body = [CommercialCatalogPublicationProjection]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical commercial publication projections.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_catalog_publications_list() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/catalog-publications/{publication_id}",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
    responses(
        (status = 200, description = "Canonical commercial publication detail with resolved governed pricing context.", body = CommercialCatalogPublicationDetail),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Publication not found.", body = ErrorResponse),
        (status = 500, description = "Failed to load canonical commercial publication detail.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_catalog_publication_detail() {}

#[utoipa::path(
    post,
    path = "/admin/commerce/catalog-publications/{publication_id}/publish",
    tag = "commerce",
    request_body = PublishCommercialCatalogPublicationRequest,
    security(("bearerAuth" = [])),
    params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
    responses(
        (status = 200, description = "Published the selected canonical commercial publication and recorded lifecycle audit evidence.", body = CommercialCatalogPublicationMutationResult),
        (status = 400, description = "Publication cannot be published from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Publication not found.", body = ErrorResponse),
        (status = 500, description = "Failed to publish canonical commercial publication.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_catalog_publication_publish() {}

#[utoipa::path(
    post,
    path = "/admin/commerce/catalog-publications/{publication_id}/schedule",
    tag = "commerce",
    request_body = ScheduleCommercialCatalogPublicationRequest,
    security(("bearerAuth" = [])),
    params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
    responses(
        (status = 200, description = "Scheduled the selected canonical commercial publication and recorded lifecycle audit evidence.", body = CommercialCatalogPublicationMutationResult),
        (status = 400, description = "Publication cannot be scheduled from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Publication not found.", body = ErrorResponse),
        (status = 500, description = "Failed to schedule canonical commercial publication.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_catalog_publication_schedule() {}

#[utoipa::path(
    post,
    path = "/admin/commerce/catalog-publications/{publication_id}/retire",
    tag = "commerce",
    request_body = RetireCommercialCatalogPublicationRequest,
    security(("bearerAuth" = [])),
    params(("publication_id" = String, Path, description = "Canonical commercial publication identifier.")),
    responses(
        (status = 200, description = "Retired the selected canonical commercial publication and recorded lifecycle audit evidence.", body = CommercialCatalogPublicationMutationResult),
        (status = 400, description = "Publication cannot be retired from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Publication not found.", body = ErrorResponse),
        (status = 500, description = "Failed to retire canonical commercial publication.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_catalog_publication_retire() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/payment-methods",
    tag = "commerce",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Configured payment methods ordered for admin display.", body = [PaymentMethodRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load payment methods.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_payment_methods_list() {}

#[utoipa::path(
    put,
    path = "/admin/commerce/payment-methods/{payment_method_id}",
    tag = "commerce",
    request_body = PaymentMethodRecord,
    security(("bearerAuth" = [])),
    params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
    responses(
        (status = 200, description = "Saved payment method configuration.", body = PaymentMethodRecord),
        (status = 400, description = "Invalid payment method payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to save payment method.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_payment_method_put() {}

#[utoipa::path(
    delete,
    path = "/admin/commerce/payment-methods/{payment_method_id}",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
    responses(
        (status = 204, description = "Payment method deleted."),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Payment method not found.", body = ErrorResponse),
        (status = 500, description = "Failed to delete payment method.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_payment_method_delete() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/payment-methods/{payment_method_id}/credential-bindings",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
    responses(
        (status = 200, description = "Credential bindings under the selected payment method.", body = [PaymentMethodCredentialBindingRecord]),
        (status = 400, description = "Invalid payment method identifier.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load payment method bindings.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_payment_method_bindings_list() {}

#[utoipa::path(
    put,
    path = "/admin/commerce/payment-methods/{payment_method_id}/credential-bindings",
    tag = "commerce",
    request_body = [PaymentMethodCredentialBindingRecord],
    security(("bearerAuth" = [])),
    params(("payment_method_id" = String, Path, description = "Stable payment method identifier.")),
    responses(
        (status = 200, description = "Replaced credential bindings for the selected payment method.", body = [PaymentMethodCredentialBindingRecord]),
        (status = 400, description = "Invalid binding payload.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to replace payment method bindings.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_payment_method_bindings_replace() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/orders/{order_id}/payment-events",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("order_id" = String, Path, description = "Commerce order id.")),
    responses(
        (status = 200, description = "Payment events recorded for the selected commerce order.", body = [CommercePaymentEventRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load commerce payment events.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_order_payment_events() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/orders/{order_id}/payment-attempts",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("order_id" = String, Path, description = "Commerce order id.")),
    responses(
        (status = 200, description = "Payment attempts recorded for the selected commerce order.", body = [CommercePaymentAttemptRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load commerce payment attempts.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_order_payment_attempts() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/orders/{order_id}/refunds",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("order_id" = String, Path, description = "Commerce order id.")),
    responses(
        (status = 200, description = "Refunds recorded for the selected commerce order.", body = [CommerceRefundRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load commerce refunds.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_order_refunds_list() {}

#[utoipa::path(
    post,
    path = "/admin/commerce/orders/{order_id}/refunds",
    tag = "commerce",
    request_body = AdminCommerceRefundCreateRequest,
    security(("bearerAuth" = [])),
    params(("order_id" = String, Path, description = "Commerce order id.")),
    responses(
        (status = 200, description = "Created refund for the selected commerce order.", body = CommerceRefundRecord),
        (status = 400, description = "Invalid refund request.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create commerce refund.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_order_refunds_create() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/orders/{order_id}/audit",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("order_id" = String, Path, description = "Commerce order id.")),
    responses(
        (status = 200, description = "Aggregated payment and coupon evidence chain for the selected commerce order.", body = CommerceOrderAuditRecord),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Commerce order not found.", body = ErrorResponse),
        (status = 500, description = "Failed to load commerce order audit evidence.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_order_audit() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/webhook-inbox",
    tag = "commerce",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Webhook inbox records ordered by newest delivery first.", body = [CommerceWebhookInboxRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load webhook inbox.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_webhook_inbox_list() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/webhook-inbox/{webhook_inbox_id}/delivery-attempts",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("webhook_inbox_id" = String, Path, description = "Webhook inbox id.")),
    responses(
        (status = 200, description = "Delivery attempts recorded for the selected webhook inbox record.", body = [CommerceWebhookDeliveryAttemptRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load webhook delivery attempts.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_webhook_delivery_attempts_list() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/reconciliation-runs",
    tag = "commerce",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Reconciliation runs ordered by newest execution first.", body = [CommerceReconciliationRunRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load reconciliation runs.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_reconciliation_runs_list() {}

#[utoipa::path(
    post,
    path = "/admin/commerce/reconciliation-runs",
    tag = "commerce",
    request_body = AdminCommerceReconciliationRunCreateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Created reconciliation run.", body = CommerceReconciliationRunRecord),
        (status = 400, description = "Invalid reconciliation request.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to create reconciliation run.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_reconciliation_runs_create() {}

#[utoipa::path(
    get,
    path = "/admin/commerce/reconciliation-runs/{reconciliation_run_id}/items",
    tag = "commerce",
    security(("bearerAuth" = [])),
    params(("reconciliation_run_id" = String, Path, description = "Reconciliation run id.")),
    responses(
        (status = 200, description = "Discrepancy items recorded for the selected reconciliation run.", body = [CommerceReconciliationItemRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load reconciliation items.", body = ErrorResponse)
    )
)]
pub(super) async fn commerce_reconciliation_items_list() {}
