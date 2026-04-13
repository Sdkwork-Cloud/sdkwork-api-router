use super::*;
use crate::marketing::CouponCodeMutationResult;

#[utoipa::path(
    get,
    path = "/admin/marketing/codes",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical coupon codes.", body = [CouponCodeRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical coupon codes.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_codes_list() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/codes",
    tag = "marketing",
    request_body = CouponCodeRecord,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created canonical coupon code.", body = CouponCodeRecord),
        (status = 400, description = "Coupon code create request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Referenced coupon template not found.", body = ErrorResponse),
        (status = 409, description = "Coupon code id or code value already exists.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical coupon code.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_codes_create() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/codes/{coupon_code_id}/status",
    tag = "marketing",
    params(("coupon_code_id" = String, Path, description = "Coupon code id")),
    request_body = UpdateCouponCodeStatusRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated canonical coupon code status.", body = CouponCodeRecord),
        (status = 400, description = "Coupon code status update request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical coupon code status.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_codes_status_update() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/codes/{coupon_code_id}/disable",
    tag = "marketing",
    params(("coupon_code_id" = String, Path, description = "Coupon code id")),
    request_body = DisableCouponCodeRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Disabled the selected canonical coupon code with semantic lifecycle evidence.", body = CouponCodeMutationResult),
        (status = 400, description = "Coupon code cannot be disabled from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
        (status = 500, description = "Failed to disable canonical coupon code.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_codes_disable() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/codes/{coupon_code_id}/restore",
    tag = "marketing",
    params(("coupon_code_id" = String, Path, description = "Coupon code id")),
    request_body = RestoreCouponCodeRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Restored the selected canonical coupon code with semantic lifecycle evidence.", body = CouponCodeMutationResult),
        (status = 400, description = "Coupon code cannot be restored from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon code not found.", body = ErrorResponse),
        (status = 500, description = "Failed to restore canonical coupon code.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_codes_restore() {}

#[utoipa::path(
    get,
    path = "/admin/marketing/codes/{coupon_code_id}/lifecycle-audits",
    tag = "marketing",
    params(("coupon_code_id" = String, Path, description = "Coupon code id")),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Lifecycle audit trail for the selected canonical coupon code.", body = [CouponCodeLifecycleAuditRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load coupon code lifecycle audit trail.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_code_lifecycle_audits_list() {}
