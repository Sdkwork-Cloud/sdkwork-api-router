use super::*;

#[utoipa::path(
    get,
    path = "/admin/billing/ledger",
    tag = "billing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible billing ledger entries.", body = [LedgerEntry]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load billing ledger.")
    )
)]
pub(super) async fn billing_ledger_list() {}

#[utoipa::path(
    get,
    path = "/admin/billing/events",
    tag = "billing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible billing events.", body = [BillingEventRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load billing events.")
    )
)]
pub(super) async fn billing_events_list() {}

#[utoipa::path(
    get,
    path = "/admin/billing/events/summary",
    tag = "billing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Billing events summary.", body = BillingEventSummary),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load billing event summary.")
    )
)]
pub(super) async fn billing_events_summary() {}

#[utoipa::path(
    get,
    path = "/admin/billing/summary",
    tag = "billing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Billing summary.", body = BillingSummary),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load billing summary.")
    )
)]
pub(super) async fn billing_summary() {}

#[utoipa::path(
    post,
    path = "/admin/billing/pricing-lifecycle/synchronize",
    tag = "billing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Synchronized due planned commercial pricing lifecycle state.", body = PricingLifecycleSynchronizationReport),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to synchronize commercial pricing lifecycle.", body = ErrorResponse)
    )
)]
pub(super) async fn billing_pricing_lifecycle_synchronize() {}

#[utoipa::path(
    get,
    path = "/admin/billing/accounts/{account_id}/ledger",
    tag = "billing",
    security(("bearerAuth" = [])),
    params(("account_id" = u64, Path, description = "Canonical commercial account identifier.")),
    responses(
        (status = 200, description = "Canonical account ledger history.", body = [AccountLedgerHistoryEntry]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Account not found.", body = ErrorResponse),
        (status = 501, description = "Commercial billing kernel is not configured.", body = ErrorResponse),
        (status = 500, description = "Failed to load canonical account ledger history.", body = ErrorResponse)
    )
)]
pub(super) async fn billing_account_ledger() {}
