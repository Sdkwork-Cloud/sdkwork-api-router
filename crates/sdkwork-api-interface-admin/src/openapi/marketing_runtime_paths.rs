use super::*;

#[utoipa::path(
    get,
    path = "/admin/marketing/reservations",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical coupon reservations.", body = [CouponReservationRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical coupon reservations.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_reservations_list() {}

#[utoipa::path(
    get,
    path = "/admin/marketing/redemptions",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical coupon redemptions.", body = [CouponRedemptionRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical coupon redemptions.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_redemptions_list() {}

#[utoipa::path(
    get,
    path = "/admin/marketing/rollbacks",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical coupon rollback records.", body = [CouponRollbackRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical coupon rollback records.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_rollbacks_list() {}
