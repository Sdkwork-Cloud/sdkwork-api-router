use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_payment::ensure_portal_payment_subject_scope;
use sdkwork_api_domain_billing::{
    AccountCommerceReconciliationStateRecord, AccountRecord, AccountStatus, AccountType,
};
use sdkwork_api_domain_commerce::{CommercePaymentAttemptRecord, PaymentMethodRecord};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponReservationRecord, CouponReservationStatus,
    CouponRestrictionSpec, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingCampaignStatus, MarketingSubjectScope,
};
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod account_reconciliation;
mod catalog_quote;
mod order_checkout;
mod order_views;
mod payment_events;
mod subscription_checkout;
mod support;

use support::*;
