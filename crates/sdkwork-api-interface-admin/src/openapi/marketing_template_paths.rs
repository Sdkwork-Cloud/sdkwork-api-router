use super::*;
use crate::marketing::{CouponTemplateComparisonResult, CouponTemplateMutationResult};

#[utoipa::path(
    get,
    path = "/admin/marketing/coupon-templates",
    tag = "marketing",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Visible canonical coupon templates.", body = [CouponTemplateRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load canonical coupon templates.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_list() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates",
    tag = "marketing",
    request_body = CouponTemplateRecord,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Created canonical coupon template.", body = CouponTemplateRecord),
        (status = 400, description = "Coupon template create request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 409, description = "Coupon template id or template key already exists.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_create() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/status",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = UpdateCouponTemplateStatusRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Updated canonical coupon template status.", body = CouponTemplateRecord),
        (status = 400, description = "Coupon template status update request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to persist canonical coupon template status.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_status_update() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/clone",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Source coupon template id")),
    request_body = CloneCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Cloned the selected canonical coupon template into a governed draft revision.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template clone request is invalid.", body = ErrorResponse),
        (status = 409, description = "Target coupon template id or template key already exists.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to clone canonical coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_clone() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/compare",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Source coupon template id")),
    request_body = CompareCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Compared two coupon template revisions.", body = CouponTemplateComparisonResult),
        (status = 400, description = "Coupon template compare request is invalid.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to compare canonical coupon templates.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_compare() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/submit-for-approval",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = SubmitCouponTemplateForApprovalRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Submitted the selected coupon template revision for approval.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template cannot enter approval from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to submit coupon template for approval.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_submit_for_approval() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/approve",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = ApproveCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Approved the selected coupon template revision.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template cannot be approved from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to approve coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_approve() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/reject",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = RejectCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Rejected the selected coupon template revision.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template cannot be rejected from the current governance state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to reject coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_reject() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/publish",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = PublishCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Published the selected canonical coupon template with semantic lifecycle evidence.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template cannot be published from the current coupon lifecycle state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to publish canonical coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_publish() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/schedule",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = ScheduleCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Scheduled the selected canonical coupon template with semantic lifecycle evidence.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template cannot be scheduled from the current coupon lifecycle state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to schedule canonical coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_schedule() {}

#[utoipa::path(
    post,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/retire",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    request_body = RetireCouponTemplateRequest,
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Retired the selected canonical coupon template with semantic lifecycle evidence.", body = CouponTemplateMutationResult),
        (status = 400, description = "Coupon template cannot be retired from the current coupon lifecycle state.", body = ErrorResponse),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 404, description = "Canonical coupon template not found.", body = ErrorResponse),
        (status = 500, description = "Failed to retire canonical coupon template.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_templates_retire() {}

#[utoipa::path(
    get,
    path = "/admin/marketing/coupon-templates/{coupon_template_id}/lifecycle-audits",
    tag = "marketing",
    params(("coupon_template_id" = String, Path, description = "Coupon template id")),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Lifecycle audit trail for the selected canonical coupon template.", body = [CouponTemplateLifecycleAuditRecord]),
        (status = 401, description = "Missing or invalid admin bearer token."),
        (status = 500, description = "Failed to load coupon template lifecycle audit trail.", body = ErrorResponse)
    )
)]
pub(super) async fn marketing_coupon_template_lifecycle_audits_list() {}
