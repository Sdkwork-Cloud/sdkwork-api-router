use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts},
    response::{Html, IntoResponse},
    routing::{delete, get, patch, post},
};
use sdkwork_api_app_billing::{
    AccountBalanceSnapshot, AccountLedgerHistoryEntry, CommercialBillingAdminKernel,
    list_billing_events, summarize_billing_events, summarize_billing_snapshot,
    synchronize_due_pricing_plan_lifecycle,
};
use sdkwork_api_app_commerce::{
    CommerceError, PortalCommerceCatalog, PortalCommerceCheckoutSession, PortalCommerceOrderRecord,
    PortalCommercePaymentEventRecord, PortalCommercePaymentEventRequest, PortalCommerceQuote,
    PortalCommerceQuoteRequest, PortalProjectMembershipRecord, apply_portal_commerce_payment_event,
    cancel_portal_commerce_order, list_project_commerce_orders, load_portal_commerce_catalog,
    load_portal_commerce_checkout_session, load_project_membership, preview_portal_commerce_quote,
    settle_portal_commerce_order, submit_portal_commerce_order,
};
use sdkwork_api_app_identity::{
    CreatedGatewayApiKey, GatewayRequestContext, PortalApiKeyGroupInput, PortalAuthSession,
    PortalClaims, PortalIdentityError, PortalWorkspaceSummary, change_portal_password,
    create_portal_api_key_group, create_portal_api_key_with_metadata, delete_portal_api_key,
    delete_portal_api_key_group, gateway_auth_subject_from_request_context,
    list_portal_api_key_groups, list_portal_api_keys, load_portal_user_profile,
    load_portal_workspace_summary, login_portal_user, register_portal_user,
    set_portal_api_key_active, set_portal_api_key_group_active, update_portal_api_key_group,
    verify_portal_jwt,
};
use sdkwork_api_app_jobs::{
    find_async_job, list_async_job_assets, list_async_job_attempts, list_async_jobs,
};
use sdkwork_api_app_marketing::{
    CouponValidationDecision, confirm_coupon_redemption, project_legacy_coupon_campaign,
    reserve_coupon_redemption, rollback_coupon_redemption, validate_coupon_stack,
};
use sdkwork_api_app_rate_limit::{
    CouponRateLimitAction, check_coupon_rate_limit, coupon_actor_bucket,
};
use sdkwork_api_app_routing::{
    CreateRoutingProfileInput, RouteSelectionContext, create_routing_profile,
    list_compiled_routing_snapshots, list_routing_profiles, persist_routing_profile,
    select_route_with_store_context, simulate_route_with_store_selection_context,
};
use sdkwork_api_app_usage::summarize_usage_records;
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountHoldRecord, AccountRecord, BillingEventRecord,
    BillingEventSummary, LedgerEntry, PricingPlanRecord, PricingRateRecord, ProjectBillingSummary,
    RequestSettlementRecord,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_identity::{ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserProfile};
use sdkwork_api_domain_jobs::{AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobRecord};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationRecord, CouponRollbackRecord, CouponRollbackType, CouponTemplateRecord,
    MarketingCampaignRecord, MarketingSubjectScope,
};
use sdkwork_api_domain_rate_limit::{RateLimitPolicy, RateLimitWindowSnapshot};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, RoutingDecision, RoutingDecisionLog,
    RoutingDecisionSource, RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_observability::{HttpMetricsRegistry, observe_http_metrics, observe_http_tracing};
use sdkwork_api_storage_core::{
    AdminStore, AtomicCouponConfirmationCommand, AtomicCouponReservationCommand,
    AtomicCouponRollbackCommand, Reloadable,
};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_PORTAL_JWT_SIGNING_SECRET: &str = "local-dev-portal-jwt-secret";

pub struct PortalApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    store: Arc<dyn AdminStore>,
    live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
    commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for PortalApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            store: self.live_store.snapshot(),
            live_commercial_billing: self.live_commercial_billing.clone(),
            commercial_billing: self
                .live_commercial_billing
                .as_ref()
                .map(Reloadable::snapshot),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
        }
    }
}

impl PortalApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
    }

    pub fn with_jwt_secret(pool: SqlitePool, jwt_signing_secret: impl Into<String>) -> Self {
        let store = Arc::new(SqliteAdminStore::new(pool));
        Self::with_store_and_jwt_secret(store, jwt_signing_secret)
    }

    pub fn with_store_and_jwt_secret<S>(
        store: Arc<S>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self
    where
        S: AdminStore + CommercialBillingAdminKernel + 'static,
    {
        let admin_store: Arc<dyn AdminStore> = store.clone();
        let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store;
        Self::with_store_commercial_billing_and_jwt_secret(
            admin_store,
            Some(commercial_billing),
            jwt_signing_secret,
        )
    }

    pub fn with_store_commercial_billing_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_live_store_commercial_billing_and_jwt_secret_handle(
            Reloadable::new(store),
            commercial_billing.map(Reloadable::new),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_commercial_billing_and_jwt_secret_handle(
            live_store,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_commercial_billing_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            live_store,
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_commercial_billing,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
        }
    }
}

#[derive(Clone, Debug)]
struct AuthenticatedPortalClaims(PortalClaims);

impl AuthenticatedPortalClaims {
    fn claims(&self) -> &PortalClaims {
        &self.0
    }
}

impl FromRequestParts<PortalApiState> for AuthenticatedPortalClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &PortalApiState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Ok(header_value) = header_value.to_str() else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Some(token) = header_value
            .strip_prefix("Bearer ")
            .or_else(|| header_value.strip_prefix("bearer "))
        else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        verify_portal_jwt(token, &state.jwt_signing_secret)
            .map(Self)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    environment: String,
    label: String,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    api_key_group_id: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyGroupRequest {
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyGroupRequest {
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct SaveRoutingPreferencesRequest {
    preset_id: String,
    strategy: RoutingStrategy,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
    #[serde(default)]
    preferred_region: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PortalRoutingPreviewRequest {
    capability: String,
    model: String,
    #[serde(default)]
    requested_region: Option<String>,
    #[serde(default)]
    selection_seed: Option<u64>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct CreatePortalRoutingProfileRequest {
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_true")]
    active: bool,
    #[serde(default)]
    strategy: Option<RoutingStrategy>,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
    #[serde(default)]
    preferred_region: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

#[derive(Debug, Serialize)]
struct PortalDashboardSummary {
    workspace: PortalWorkspaceSummary,
    usage_summary: UsageSummary,
    billing_summary: ProjectBillingSummary,
    recent_requests: Vec<UsageRecord>,
    api_key_count: usize,
}
#[derive(Debug, Serialize)]
struct PortalBillingAccountResponse {
    account: AccountRecord,
    #[serde(flatten)]
    balance: AccountBalanceSnapshot,
}

#[derive(Debug, Serialize)]
struct PortalOrderCenterEntry {
    order: PortalCommerceOrderRecord,
    payment_events: Vec<PortalCommercePaymentEventRecord>,
    latest_payment_event: Option<PortalCommercePaymentEventRecord>,
    checkout_session: PortalCommerceCheckoutSession,
}

#[derive(Debug, Serialize)]
struct PortalCommerceReconciliationSummary {
    account_id: u64,
    last_reconciled_order_id: String,
    last_reconciled_order_updated_at_ms: u64,
    last_reconciled_order_created_at_ms: u64,
    last_reconciled_at_ms: u64,
    backlog_order_count: usize,
    checkpoint_lag_ms: u64,
    healthy: bool,
}

#[derive(Debug, Serialize)]
struct PortalCommerceOrderCenterResponse {
    project_id: String,
    membership: Option<PortalProjectMembershipRecord>,
    reconciliation: Option<PortalCommerceReconciliationSummary>,
    orders: Vec<PortalOrderCenterEntry>,
}

#[derive(Debug, Serialize)]
struct PortalBillingAccountHistoryResponse {
    account: AccountRecord,
    balance: AccountBalanceSnapshot,
    benefit_lots: Vec<AccountBenefitLotRecord>,
    holds: Vec<AccountHoldRecord>,
    request_settlements: Vec<RequestSettlementRecord>,
    ledger: Vec<AccountLedgerHistoryEntry>,
}

#[derive(Debug, Serialize)]
struct PortalGatewayRateLimitSnapshot {
    project_id: String,
    policy_count: usize,
    active_policy_count: usize,
    window_count: usize,
    exceeded_window_count: usize,
    headline: String,
    detail: String,
    generated_at_ms: u64,
    policies: Vec<RateLimitPolicy>,
    windows: Vec<RateLimitWindowSnapshot>,
}

#[derive(Debug, Serialize)]
struct PortalRoutingProviderOption {
    provider_id: String,
    display_name: String,
    channel_id: String,
    #[serde(default)]
    preferred: bool,
    #[serde(default)]
    default_provider: bool,
}

#[derive(Debug, Serialize)]
struct PortalRoutingSummary {
    project_id: String,
    preferences: ProjectRoutingPreferences,
    latest_model_hint: String,
    preview: RoutingDecision,
    provider_options: Vec<PortalRoutingProviderOption>,
}

#[derive(Debug, Deserialize, Default)]
struct PortalMarketingRedemptionsQuery {
    #[serde(default)]
    status: Option<CouponRedemptionStatus>,
}

#[derive(Debug, Deserialize, Default)]
struct PortalMarketingCodesQuery {
    #[serde(default)]
    status: Option<CouponCodeStatus>,
}

#[derive(Debug, Deserialize)]
struct PortalCouponValidationRequest {
    coupon_code: String,
    subject_scope: MarketingSubjectScope,
    target_kind: String,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
}

#[derive(Debug, Serialize)]
struct PortalCouponValidationDecisionResponse {
    eligible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rejection_reason: Option<String>,
    reservable_budget_minor: u64,
}

#[derive(Debug, Serialize)]
struct PortalCouponValidationResponse {
    decision: PortalCouponValidationDecisionResponse,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize)]
struct PortalCouponReservationRequest {
    coupon_code: String,
    subject_scope: MarketingSubjectScope,
    target_kind: String,
    reserve_amount_minor: u64,
    ttl_ms: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct PortalCouponReservationResponse {
    reservation: CouponReservationRecord,
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize)]
struct PortalCouponRedemptionConfirmRequest {
    coupon_reservation_id: String,
    subsidy_amount_minor: u64,
    #[serde(default)]
    order_id: Option<String>,
    #[serde(default)]
    payment_event_id: Option<String>,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct PortalCouponRedemptionConfirmResponse {
    reservation: CouponReservationRecord,
    redemption: CouponRedemptionRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Deserialize)]
struct PortalCouponRedemptionRollbackRequest {
    coupon_redemption_id: String,
    rollback_type: CouponRollbackType,
    restored_budget_minor: u64,
    restored_inventory_count: u64,
    #[serde(default)]
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct PortalCouponRedemptionRollbackResponse {
    redemption: CouponRedemptionRecord,
    rollback: CouponRollbackRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
}

#[derive(Debug, Serialize, Default)]
struct PortalMarketingRedemptionSummary {
    total_count: usize,
    redeemed_count: usize,
    partially_rolled_back_count: usize,
    rolled_back_count: usize,
    failed_count: usize,
}

#[derive(Debug, Serialize, Default)]
struct PortalMarketingCodeSummary {
    total_count: usize,
    available_count: usize,
    reserved_count: usize,
    redeemed_count: usize,
    disabled_count: usize,
    expired_count: usize,
}

#[derive(Debug, Serialize)]
struct PortalMarketingRedemptionsResponse {
    summary: PortalMarketingRedemptionSummary,
    items: Vec<CouponRedemptionRecord>,
}

#[derive(Debug, Serialize)]
struct PortalMarketingCodeItem {
    code: CouponCodeRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    latest_reservation: Option<CouponReservationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    latest_redemption: Option<CouponRedemptionRecord>,
}

#[derive(Debug, Serialize)]
struct PortalMarketingCodesResponse {
    summary: PortalMarketingCodeSummary,
    items: Vec<PortalMarketingCodeItem>,
}

#[derive(Debug, Serialize)]
struct PortalMarketingRewardHistoryItem {
    redemption: CouponRedemptionRecord,
    code: CouponCodeRecord,
    #[serde(default)]
    rollbacks: Vec<CouponRollbackRecord>,
}

const PORTAL_OPENAPI_DOCUMENT: &str = r##"{
  "openapi": "3.1.0",
  "info": {
    "title": "SDKWORK Portal API",
    "version": "0.1.0",
    "description": "OpenAPI 3.1 schema published from the current portal router surface."
  },
  "servers": [
    {
      "url": "/"
    }
  ],
  "tags": [
    {
      "name": "system",
      "description": "Portal health and system-facing routes."
    },
    {
      "name": "auth",
      "description": "Portal authentication and workspace access routes."
    },
    {
      "name": "marketing",
      "description": "Portal coupon validation, reservation, redemption, and reward history routes."
    },
    {
      "name": "billing",
      "description": "Portal billing account, ledger, and pricing visibility routes."
    },
    {
      "name": "jobs",
      "description": "Portal async job tracking routes."
    }
  ],
  "paths": {
    "/portal/health": {
      "get": {
        "tags": ["system"],
        "responses": {
          "200": {
            "description": "Portal health check response.",
            "content": {
              "text/plain": {
                "schema": {
                  "type": "string"
                }
              }
            }
          }
        }
      }
    },
    "/portal/auth/login": {
      "post": {
        "tags": ["auth"],
        "security": [],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/LoginRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Portal login session.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalAuthSession"
                }
              }
            }
          },
          "401": {
            "description": "Authentication failed.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/workspace": {
      "get": {
        "tags": ["auth"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Current portal workspace summary.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalWorkspaceSummary"
                }
              }
            }
          },
          "401": {
            "description": "Portal authentication is required.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-validations": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponValidationRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon validation result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponValidationResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-reservations": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponReservationRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon reservation result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponReservationResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-redemptions/confirm": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponRedemptionConfirmRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon redemption confirmation result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponRedemptionConfirmResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/coupon-redemptions/rollback": {
      "post": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/PortalCouponRedemptionRollbackRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Coupon redemption rollback result.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCouponRedemptionRollbackResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/my-coupons": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Coupons visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalMarketingCodesResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/reward-history": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Reward history for the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "type": "object"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/redemptions": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Coupon redemptions visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalMarketingRedemptionsResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/marketing/codes": {
      "get": {
        "tags": ["marketing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Coupon codes visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalMarketingCodesResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/commerce/order-center": {
      "get": {
        "tags": ["commerce"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Aggregated order center view for the current workspace, including payment events and checkout status.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalCommerceOrderCenterResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Commercial billing account and live balance snapshot for the current workspace.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalBillingAccountResponse"
                }
              }
            }
          },
          "401": {
            "description": "Portal authentication is required.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          },
          "501": {
            "description": "Commercial billing kernel is not configured.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account-history": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Aggregated account history view for the current commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/PortalBillingAccountHistoryResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/balance": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Current commercial billing balance snapshot.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/AccountBalanceSnapshot"
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/benefit-lots": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Benefit lots attached to the current commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AccountBenefitLotRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/holds": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Current outstanding holds for the commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AccountHoldRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/request-settlements": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Request settlement history for the commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/RequestSettlementRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/account/ledger": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Canonical ledger history for the commercial billing account.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AccountLedgerHistoryEntry"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/pricing-plans": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Pricing plans visible to the current commercial billing account scope.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/PricingPlanRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/billing/pricing-rates": {
      "get": {
        "tags": ["billing"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Pricing rates visible to the current commercial billing account scope.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/PricingRateRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/async-jobs": {
      "get": {
        "tags": ["jobs"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "responses": {
          "200": {
            "description": "Async jobs visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AsyncJobRecord"
                  }
                }
              }
            }
          }
        }
      }
    },
    "/portal/async-jobs/{job_id}/attempts": {
      "get": {
        "tags": ["jobs"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "job_id",
            "in": "path",
            "required": true,
            "description": "Async job identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Attempts recorded for the selected async job.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AsyncJobAttemptRecord"
                  }
                }
              }
            }
          },
          "404": {
            "description": "Async job not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    },
    "/portal/async-jobs/{job_id}/assets": {
      "get": {
        "tags": ["jobs"],
        "security": [
          {
            "bearerAuth": []
          }
        ],
        "parameters": [
          {
            "name": "job_id",
            "in": "path",
            "required": true,
            "description": "Async job identifier.",
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Assets recorded for the selected async job.",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/AsyncJobAssetRecord"
                  }
                }
              }
            }
          },
          "404": {
            "description": "Async job not found or is not visible to the current portal subject.",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ErrorResponse"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "securitySchemes": {
      "bearerAuth": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    },
    "schemas": {
      "ErrorResponse": {
        "type": "object"
      },
      "LoginRequest": {
        "type": "object"
      },
      "PortalAuthSession": {
        "type": "object"
      },
      "PortalWorkspaceSummary": {
        "type": "object"
      },
      "PortalCouponValidationRequest": {
        "type": "object",
        "required": ["coupon_code", "subject_scope", "target_kind", "order_amount_minor", "reserve_amount_minor"],
        "properties": {
          "coupon_code": { "type": "string" },
          "subject_scope": { "type": "string" },
          "target_kind": { "type": "string" },
          "order_amount_minor": { "type": "integer", "format": "uint64", "minimum": 0 },
          "reserve_amount_minor": { "type": "integer", "format": "uint64", "minimum": 0 }
        }
      },
      "PortalCouponValidationResponse": {
        "type": "object"
      },
      "PortalCouponReservationRequest": {
        "type": "object",
        "required": ["coupon_code", "subject_scope", "target_kind", "reserve_amount_minor", "ttl_ms"],
        "properties": {
          "coupon_code": { "type": "string" },
          "subject_scope": { "type": "string" },
          "target_kind": { "type": "string" },
          "reserve_amount_minor": { "type": "integer", "format": "uint64", "minimum": 0 },
          "ttl_ms": { "type": "integer", "format": "uint64", "minimum": 0 },
          "idempotency_key": { "type": ["string", "null"] }
        }
      },
      "PortalCouponReservationResponse": {
        "type": "object"
      },
      "PortalCouponRedemptionConfirmRequest": {
        "type": "object"
      },
      "PortalCouponRedemptionConfirmResponse": {
        "type": "object"
      },
      "PortalCouponRedemptionRollbackRequest": {
        "type": "object"
      },
      "PortalCouponRedemptionRollbackResponse": {
        "type": "object"
      },
      "PortalMarketingCodesResponse": {
        "type": "object"
      },
      "PortalMarketingRedemptionsResponse": {
        "type": "object"
      },
      "PortalCommerceOrderCenterResponse": {
        "type": "object"
      },
      "PortalBillingAccountHistoryResponse": {
        "type": "object"
      },
      "PortalBillingAccountResponse": {
        "type": "object"
      },
      "AccountBalanceSnapshot": {
        "type": "object"
      },
      "AccountBenefitLotRecord": {
        "type": "object"
      },
      "AccountHoldRecord": {
        "type": "object"
      },
      "RequestSettlementRecord": {
        "type": "object"
      },
      "AccountLedgerHistoryEntry": {
        "type": "object"
      },
      "PricingPlanRecord": {
        "type": "object"
      },
      "PricingRateRecord": {
        "type": "object"
      },
      "AsyncJobRecord": {
        "type": "object"
      },
      "AsyncJobAttemptRecord": {
        "type": "object"
      },
      "AsyncJobAssetRecord": {
        "type": "object"
      }
    }
  }
}"##;

async fn portal_openapi_handler() -> impl axum::response::IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        PORTAL_OPENAPI_DOCUMENT,
    )
}

async fn portal_docs_index_handler() -> Html<String> {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SDKWORK Portal API</title>
    <style>
      :root {
        color-scheme: light;
        font-family: "Segoe UI", "PingFang SC", sans-serif;
      }
      body {
        margin: 0;
        background: linear-gradient(180deg, #f5f7fb 0%, #eef2f8 100%);
        color: #132238;
      }
      main {
        max-width: 960px;
        margin: 0 auto;
        padding: 48px 24px 64px;
      }
      .card {
        background: rgba(255, 255, 255, 0.92);
        border: 1px solid rgba(19, 34, 56, 0.08);
        border-radius: 20px;
        box-shadow: 0 20px 60px rgba(19, 34, 56, 0.08);
        padding: 32px;
      }
      code {
        background: rgba(19, 34, 56, 0.08);
        border-radius: 8px;
        padding: 2px 8px;
      }
      a {
        color: #0f6ab4;
        text-decoration: none;
      }
      a:hover {
        text-decoration: underline;
      }
    </style>
  </head>
  <body>
    <main>
      <section class="card">
        <p>OpenAPI 3.1</p>
        <h1>SDKWORK Portal API</h1>
        <p>The live contract for the current portal surface is published at <code>/portal/openapi.json</code>.</p>
        <p><a href="/portal/openapi.json">Open the raw schema</a></p>
      </section>
    </main>
  </body>
</html>"#
            .to_string(),
    )
}

fn portal_docs_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/portal/openapi.json", get(portal_openapi_handler))
        .route("/portal/docs", get(portal_docs_index_handler))
}

pub fn try_portal_router() -> anyhow::Result<Router> {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    let http_exposure = http_exposure_config()?;
    Ok(Router::new()
        .merge(portal_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(|| async { "register" }))
        .route("/portal/auth/login", post(|| async { "login" }))
        .route("/portal/auth/me", get(|| async { "me" }))
        .route(
            "/portal/auth/change-password",
            post(|| async { "change-password" }),
        )
        .route("/portal/dashboard", get(|| async { "dashboard" }))
        .route("/portal/workspace", get(|| async { "workspace" }))
        .route(
            "/portal/marketing/coupon-validations",
            post(|| async { "marketing-coupon-validations" }),
        )
        .route(
            "/portal/marketing/coupon-reservations",
            post(|| async { "marketing-coupon-reservations" }),
        )
        .route(
            "/portal/marketing/coupon-redemptions/confirm",
            post(|| async { "marketing-coupon-redemptions-confirm" }),
        )
        .route(
            "/portal/marketing/coupon-redemptions/rollback",
            post(|| async { "marketing-coupon-redemptions-rollback" }),
        )
        .route(
            "/portal/marketing/my-coupons",
            get(|| async { "marketing-my-coupons" }),
        )
        .route(
            "/portal/marketing/reward-history",
            get(|| async { "marketing-reward-history" }),
        )
        .route(
            "/portal/marketing/redemptions",
            get(|| async { "marketing-redemptions" }),
        )
        .route(
            "/portal/marketing/codes",
            get(|| async { "marketing-codes" }),
        )
        .route(
            "/portal/commerce/catalog",
            get(|| async { "commerce-catalog" }),
        )
        .route(
            "/portal/commerce/quote",
            post(|| async { "commerce-quote" }),
        )
        .route(
            "/portal/commerce/orders",
            get(|| async { "commerce-orders" }).post(|| async { "commerce-orders" }),
        )
        .route(
            "/portal/commerce/order-center",
            get(|| async { "commerce-order-center" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/settle",
            post(|| async { "commerce-order-settle" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/cancel",
            post(|| async { "commerce-order-cancel" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-events",
            post(|| async { "commerce-order-payment-events" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/checkout-session",
            get(|| async { "commerce-order-checkout-session" }),
        )
        .route(
            "/portal/commerce/membership",
            get(|| async { "commerce-membership" }),
        )
        .route("/portal/api-keys", get(|| async { "api-keys" }))
        .route("/portal/api-key-groups", get(|| async { "api-key-groups" }))
        .route(
            "/portal/api-key-groups/{group_id}",
            patch(|| async { "api-key-groups" }).delete(|| async { "api-key-groups" }),
        )
        .route(
            "/portal/api-key-groups/{group_id}/status",
            post(|| async { "api-key-groups-status" }),
        )
        .route("/portal/usage/records", get(|| async { "usage-records" }))
        .route("/portal/usage/summary", get(|| async { "usage-summary" }))
        .route(
            "/portal/billing/account",
            get(|| async { "billing-account" }),
        )
        .route(
            "/portal/billing/account-history",
            get(|| async { "billing-account-history" }),
        )
        .route(
            "/portal/billing/account/balance",
            get(|| async { "billing-account-balance" }),
        )
        .route(
            "/portal/billing/account/benefit-lots",
            get(|| async { "billing-account-benefit-lots" }),
        )
        .route(
            "/portal/billing/account/holds",
            get(|| async { "billing-account-holds" }),
        )
        .route(
            "/portal/billing/account/request-settlements",
            get(|| async { "billing-account-request-settlements" }),
        )
        .route(
            "/portal/billing/account/ledger",
            get(|| async { "billing-account-ledger" }),
        )
        .route(
            "/portal/billing/pricing-plans",
            get(|| async { "billing-pricing-plans" }),
        )
        .route(
            "/portal/billing/pricing-rates",
            get(|| async { "billing-pricing-rates" }),
        )
        .route(
            "/portal/billing/summary",
            get(|| async { "billing-summary" }),
        )
        .route("/portal/billing/ledger", get(|| async { "billing-ledger" }))
        .route("/portal/billing/events", get(|| async { "billing-events" }))
        .route(
            "/portal/billing/events/summary",
            get(|| async { "billing-events-summary" }),
        )
        .route(
            "/portal/routing/summary",
            get(|| async { "routing-summary" }),
        )
        .route(
            "/portal/routing/profiles",
            get(|| async { "routing-profiles" }).post(|| async { "routing-profiles" }),
        )
        .route(
            "/portal/routing/preferences",
            get(|| async { "routing-preferences" }).post(|| async { "routing-preferences" }),
        )
        .route(
            "/portal/routing/snapshots",
            get(|| async { "routing-snapshots" }),
        )
        .route(
            "/portal/routing/preview",
            post(|| async { "routing-preview" }),
        )
        .route(
            "/portal/routing/decision-logs",
            get(|| async { "routing-decision-logs" }),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        )))
}

pub fn portal_router() -> Router {
    try_portal_router().expect("http exposure config should load from process env")
}

pub fn portal_router_with_pool(pool: SqlitePool) -> Router {
    portal_router_with_pool_and_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_pool_and_jwt_secret(
    pool: SqlitePool,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    portal_router_with_store_and_jwt_secret(
        Arc::new(SqliteAdminStore::new(pool)),
        jwt_signing_secret,
    )
}

pub fn portal_router_with_store<S>(store: Arc<S>) -> Router
where
    S: AdminStore + CommercialBillingAdminKernel + 'static,
{
    portal_router_with_store_and_jwt_secret(store, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_store_and_jwt_secret<S>(
    store: Arc<S>,
    jwt_signing_secret: impl Into<String>,
) -> Router
where
    S: AdminStore + CommercialBillingAdminKernel + 'static,
{
    portal_router_with_state(PortalApiState::with_store_and_jwt_secret(
        store,
        jwt_signing_secret,
    ))
}

pub fn try_portal_router_with_state(state: PortalApiState) -> anyhow::Result<Router> {
    Ok(portal_router_with_state_and_http_exposure(
        state,
        http_exposure_config()?,
    ))
}

pub fn portal_router_with_state(state: PortalApiState) -> Router {
    try_portal_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn portal_router_with_state_and_http_exposure(
    state: PortalApiState,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    Router::new()
        .merge(portal_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(register_handler))
        .route("/portal/auth/login", post(login_handler))
        .route("/portal/auth/me", get(me_handler))
        .route(
            "/portal/auth/change-password",
            post(change_password_handler),
        )
        .route("/portal/dashboard", get(dashboard_handler))
        .route("/portal/workspace", get(workspace_handler))
        .route(
            "/portal/marketing/coupon-validations",
            post(validate_marketing_coupon_handler),
        )
        .route(
            "/portal/marketing/coupon-reservations",
            post(reserve_marketing_coupon_handler),
        )
        .route(
            "/portal/marketing/coupon-redemptions/confirm",
            post(confirm_marketing_coupon_redemption_handler),
        )
        .route(
            "/portal/marketing/coupon-redemptions/rollback",
            post(rollback_marketing_coupon_redemption_handler),
        )
        .route("/portal/marketing/my-coupons", get(list_my_coupons_handler))
        .route(
            "/portal/marketing/reward-history",
            get(list_marketing_reward_history_handler),
        )
        .route(
            "/portal/marketing/redemptions",
            get(list_marketing_redemptions_handler),
        )
        .route("/portal/marketing/codes", get(list_marketing_codes_handler))
        .route("/portal/async-jobs", get(list_async_jobs_handler))
        .route(
            "/portal/async-jobs/{job_id}/attempts",
            get(list_async_job_attempts_handler),
        )
        .route(
            "/portal/async-jobs/{job_id}/assets",
            get(list_async_job_assets_handler),
        )
        .route(
            "/portal/gateway/rate-limit-snapshot",
            get(gateway_rate_limit_snapshot_handler),
        )
        .route("/portal/commerce/catalog", get(commerce_catalog_handler))
        .route("/portal/commerce/quote", post(commerce_quote_handler))
        .route(
            "/portal/commerce/orders",
            get(list_commerce_orders_handler).post(create_commerce_order_handler),
        )
        .route(
            "/portal/commerce/order-center",
            get(commerce_order_center_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/settle",
            post(settle_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/cancel",
            post(cancel_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-events",
            post(apply_commerce_payment_event_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/checkout-session",
            get(get_commerce_checkout_session_handler),
        )
        .route(
            "/portal/commerce/membership",
            get(get_project_membership_handler),
        )
        .route(
            "/portal/api-keys",
            get(list_api_keys_handler).post(create_api_key_handler),
        )
        .route(
            "/portal/api-keys/{hashed_key}/status",
            post(update_api_key_status_handler),
        )
        .route(
            "/portal/api-keys/{hashed_key}",
            delete(delete_api_key_handler),
        )
        .route(
            "/portal/api-key-groups",
            get(list_api_key_groups_handler).post(create_api_key_group_handler),
        )
        .route(
            "/portal/api-key-groups/{group_id}",
            patch(update_api_key_group_handler).delete(delete_api_key_group_handler),
        )
        .route(
            "/portal/api-key-groups/{group_id}/status",
            post(update_api_key_group_status_handler),
        )
        .route("/portal/usage/records", get(list_usage_records_handler))
        .route("/portal/usage/summary", get(usage_summary_handler))
        .route("/portal/billing/account", get(billing_account_handler))
        .route(
            "/portal/billing/account-history",
            get(billing_account_history_handler),
        )
        .route(
            "/portal/billing/account/balance",
            get(billing_account_balance_handler),
        )
        .route(
            "/portal/billing/account/benefit-lots",
            get(list_billing_account_benefit_lots_handler),
        )
        .route(
            "/portal/billing/account/holds",
            get(list_billing_account_holds_handler),
        )
        .route(
            "/portal/billing/account/request-settlements",
            get(list_billing_request_settlements_handler),
        )
        .route(
            "/portal/billing/account/ledger",
            get(list_billing_account_ledger_handler),
        )
        .route(
            "/portal/billing/pricing-plans",
            get(list_billing_pricing_plans_handler),
        )
        .route(
            "/portal/billing/pricing-rates",
            get(list_billing_pricing_rates_handler),
        )
        .route("/portal/billing/summary", get(billing_summary_handler))
        .route("/portal/billing/ledger", get(list_billing_ledger_handler))
        .route("/portal/billing/events", get(list_billing_events_handler))
        .route(
            "/portal/billing/events/summary",
            get(billing_events_summary_handler),
        )
        .route("/portal/routing/summary", get(routing_summary_handler))
        .route(
            "/portal/routing/profiles",
            get(list_routing_profiles_handler).post(create_routing_profile_handler),
        )
        .route(
            "/portal/routing/preferences",
            get(get_routing_preferences_handler).post(save_routing_preferences_handler),
        )
        .route(
            "/portal/routing/snapshots",
            get(list_routing_snapshots_handler),
        )
        .route("/portal/routing/preview", post(preview_routing_handler))
        .route(
            "/portal/routing/decision-logs",
            get(list_routing_decision_logs_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

fn http_exposure_config() -> anyhow::Result<HttpExposureConfig> {
    HttpExposureConfig::from_env()
}

fn metrics_route<S>(
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: &HttpExposureConfig,
) -> axum::routing::MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    let expected_token: Arc<str> = Arc::from(http_exposure.metrics_bearer_token.clone());
    get(move |headers: HeaderMap| {
        let metrics = metrics.clone();
        let expected_token = expected_token.clone();
        async move {
            if !metrics_request_authorized(&headers, expected_token.as_ref()) {
                return (
                    StatusCode::UNAUTHORIZED,
                    [(header::WWW_AUTHENTICATE, "Bearer")],
                    "metrics bearer token required",
                )
                    .into_response();
            }

            (
                [(
                    header::CONTENT_TYPE,
                    "text/plain; version=0.0.4; charset=utf-8",
                )],
                metrics.render_prometheus(),
            )
                .into_response()
        }
    })
}

fn metrics_request_authorized(headers: &HeaderMap, expected_token: &str) -> bool {
    if expected_token.is_empty() {
        return false;
    }

    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, token)) = value.trim().split_once(' ') else {
        return false;
    };
    scheme.eq_ignore_ascii_case("Bearer") && token.trim() == expected_token
}

fn browser_cors_layer(http_exposure: &HttpExposureConfig) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    if http_exposure.browser_allowed_origins.is_empty() {
        return layer;
    }

    let origins = http_exposure
        .browser_allowed_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin)
                .expect("browser allowed origins must be valid HTTP header values")
        })
        .collect::<Vec<_>>();
    layer.allow_origin(origins)
}

async fn register_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<PortalAuthSession>), (StatusCode, Json<ErrorResponse>)> {
    register_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &request.display_name,
        &state.jwt_signing_secret,
    )
    .await
    .map(|session| (StatusCode::CREATED, Json(session)))
    .map_err(portal_error_response)
}

async fn login_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<PortalAuthSession>, (StatusCode, Json<ErrorResponse>)> {
    login_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn me_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalUserProfile>, StatusCode> {
    load_portal_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn change_password_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_portal_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn workspace_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalWorkspaceSummary>, StatusCode> {
    load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
}

async fn validate_marketing_coupon_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCouponValidationRequest>,
) -> Result<Json<PortalCouponValidationResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let Some(subject_id) = subjects.subject_id_for_scope(request.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let target_kind = request.target_kind.trim();
    if target_kind.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Validate,
        request.subject_scope,
        &subject_id,
        &request.coupon_code,
    )
    .await?;

    let now_ms = current_time_millis();
    let Some(context) =
        load_marketing_coupon_context_by_value(state.store.as_ref(), &request.coupon_code, now_ms)
            .await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        request.order_amount_minor,
        request.reserve_amount_minor,
    );
    let decision = if decision.eligible
        && !portal_marketing_target_kind_allowed(&context.template, target_kind)
    {
        CouponValidationDecision::rejected("target_kind_not_eligible")
    } else {
        decision
    };

    Ok(Json(PortalCouponValidationResponse {
        decision: coupon_validation_decision_response(decision),
        template: context.template,
        campaign: context.campaign,
        budget: context.budget,
        code: context.code,
    }))
}

async fn reserve_marketing_coupon_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponReservationRequest>,
) -> Result<(StatusCode, Json<PortalCouponReservationResponse>), StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let target_kind = request.target_kind.trim();
    let Some(subject_id) = subjects.subject_id_for_scope(request.subject_scope) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    if target_kind.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    let coupon_reservation_id = idempotency_key
        .as_deref()
        .map(|key| {
            derive_coupon_reservation_id(request.subject_scope, &subject_id, target_kind, key)
        })
        .unwrap_or_else(|| {
            format!(
                "coupon_reservation_{}_{}",
                normalize_coupon_code(&request.coupon_code).to_ascii_lowercase(),
                now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_reservation) = state
            .store
            .find_coupon_reservation_record(&coupon_reservation_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            let Some(existing_code) = state
                .store
                .find_coupon_code_record(&existing_reservation.coupon_code_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };
            let existing_ttl_ms = existing_reservation
                .expires_at_ms
                .saturating_sub(existing_reservation.created_at_ms);
            if existing_reservation.subject_scope != request.subject_scope
                || existing_reservation.subject_id != subject_id
                || normalize_coupon_code(&existing_code.code_value)
                    != normalize_coupon_code(&request.coupon_code)
                || existing_reservation.budget_reserved_minor != request.reserve_amount_minor
                || existing_ttl_ms != request.ttl_ms
            {
                return Err(StatusCode::CONFLICT);
            }

            let context = load_marketing_coupon_context_from_code_record(
                state.store.as_ref(),
                existing_code,
                now_ms,
            )
            .await?
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok((
                StatusCode::OK,
                Json(PortalCouponReservationResponse {
                    reservation: existing_reservation,
                    template: context.template,
                    campaign: context.campaign,
                    budget: context.budget,
                    code: context.code,
                }),
            ));
        }
    }
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Reserve,
        request.subject_scope,
        &subject_id,
        &request.coupon_code,
    )
    .await?;

    let Some(context) =
        load_marketing_coupon_context_by_value(state.store.as_ref(), &request.coupon_code, now_ms)
            .await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    if !portal_marketing_target_kind_allowed(&context.template, target_kind) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let decision = validate_coupon_stack(
        &context.template,
        &context.campaign,
        &context.budget,
        &context.code,
        now_ms,
        request.reserve_amount_minor,
        request.reserve_amount_minor,
    );
    if !decision.eligible {
        return Err(StatusCode::CONFLICT);
    }

    let (reserved_code, reservation) = reserve_coupon_redemption(
        &context.code,
        coupon_reservation_id,
        request.subject_scope,
        subject_id,
        request.reserve_amount_minor,
        now_ms,
        request.ttl_ms,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    let atomic_result = state
        .store
        .reserve_coupon_redemption_atomic(&AtomicCouponReservationCommand {
            template_to_persist: context
                .compatibility_source
                .then_some(context.template.clone()),
            campaign_to_persist: context
                .compatibility_source
                .then_some(context.campaign.clone()),
            expected_budget: context.budget.clone(),
            next_budget: reserve_campaign_budget(
                &context.budget,
                request.reserve_amount_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_reservation(
                &context.template,
                &context.code,
                &reserved_code,
                now_ms,
            ),
            reservation,
        })
        .await
        .map_err(marketing_atomic_status)?;

    Ok((
        StatusCode::CREATED,
        Json(PortalCouponReservationResponse {
            reservation: atomic_result.reservation,
            template: context.template,
            campaign: context.campaign,
            budget: atomic_result.budget,
            code: atomic_result.code,
        }),
    ))
}

async fn confirm_marketing_coupon_redemption_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponRedemptionConfirmRequest>,
) -> Result<Json<PortalCouponRedemptionConfirmResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());

    let reservation = portal_marketing_reservation_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &request.coupon_reservation_id,
    )
    .await?;
    if request.subsidy_amount_minor > reservation.budget_reserved_minor {
        return Err(StatusCode::BAD_REQUEST);
    }

    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    let coupon_redemption_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_redemption_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_redemption_{}_{}",
                reservation.coupon_reservation_id, now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_redemption) = state
            .store
            .find_coupon_redemption_record(&coupon_redemption_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            if existing_redemption.coupon_reservation_id != reservation.coupon_reservation_id
                || existing_redemption.subsidy_amount_minor != request.subsidy_amount_minor
                || existing_redemption.order_id != request.order_id
                || existing_redemption.payment_event_id != request.payment_event_id
            {
                return Err(StatusCode::CONFLICT);
            }

            let current_reservation = portal_marketing_reservation_owned_by_subject(
                state.store.as_ref(),
                &subjects,
                &existing_redemption.coupon_reservation_id,
            )
            .await?;
            let context = load_marketing_coupon_context_for_code_id(
                state.store.as_ref(),
                &existing_redemption.coupon_code_id,
                now_ms,
            )
            .await?;

            return Ok(Json(PortalCouponRedemptionConfirmResponse {
                reservation: current_reservation,
                redemption: existing_redemption,
                budget: context.budget,
                code: context.code,
            }));
        }
    }

    let Some(code) = state
        .store
        .find_coupon_code_record(&reservation.coupon_code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Confirm,
        reservation.subject_scope,
        &reservation.subject_id,
        &code.code_value,
    )
    .await?;
    let Some(context) =
        load_marketing_coupon_context_from_code_record(state.store.as_ref(), code, now_ms).await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        coupon_redemption_id,
        context.code.coupon_code_id.clone(),
        context.template.coupon_template_id.clone(),
        request.subsidy_amount_minor,
        request.order_id.clone(),
        request.payment_event_id.clone(),
        now_ms,
    )
    .map_err(|_| StatusCode::CONFLICT)?;
    let atomic_result = state
        .store
        .confirm_coupon_redemption_atomic(&AtomicCouponConfirmationCommand {
            expected_budget: context.budget.clone(),
            next_budget: confirm_campaign_budget(
                &context.budget,
                request.subsidy_amount_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_confirmation(&context.template, &context.code, now_ms),
            expected_reservation: reservation.clone(),
            next_reservation: confirmed_reservation,
            redemption,
        })
        .await
        .map_err(marketing_atomic_status)?;

    Ok(Json(PortalCouponRedemptionConfirmResponse {
        reservation: atomic_result.reservation,
        redemption: atomic_result.redemption,
        budget: atomic_result.budget,
        code: atomic_result.code,
    }))
}

async fn rollback_marketing_coupon_redemption_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCouponRedemptionRollbackRequest>,
) -> Result<Json<PortalCouponRedemptionRollbackResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());

    let redemption = portal_marketing_redemption_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &request.coupon_redemption_id,
    )
    .await?;
    if request.restored_budget_minor > redemption.subsidy_amount_minor {
        return Err(StatusCode::BAD_REQUEST);
    }

    let reservation = portal_marketing_reservation_owned_by_subject(
        state.store.as_ref(),
        &subjects,
        &redemption.coupon_reservation_id,
    )
    .await?;
    let idempotency_key =
        resolve_portal_idempotency_key(&headers, request.idempotency_key.as_deref())?;
    let now_ms = current_time_millis();
    let coupon_rollback_id = idempotency_key
        .as_deref()
        .map(|key| derive_coupon_rollback_id(&reservation, key))
        .unwrap_or_else(|| {
            format!(
                "coupon_rollback_{}_{}",
                redemption.coupon_redemption_id, now_ms
            )
        });
    if idempotency_key.is_some() {
        if let Some(existing_rollback) =
            find_coupon_rollback_record(state.store.as_ref(), &coupon_rollback_id).await?
        {
            if existing_rollback.coupon_redemption_id != redemption.coupon_redemption_id
                || existing_rollback.rollback_type != request.rollback_type
                || existing_rollback.restored_budget_minor != request.restored_budget_minor
                || existing_rollback.restored_inventory_count != request.restored_inventory_count
            {
                return Err(StatusCode::CONFLICT);
            }

            let current_redemption = portal_marketing_redemption_owned_by_subject(
                state.store.as_ref(),
                &subjects,
                &existing_rollback.coupon_redemption_id,
            )
            .await?;
            let context = load_marketing_coupon_context_for_code_id(
                state.store.as_ref(),
                &current_redemption.coupon_code_id,
                now_ms,
            )
            .await?;

            return Ok(Json(PortalCouponRedemptionRollbackResponse {
                redemption: current_redemption,
                rollback: existing_rollback,
                budget: context.budget,
                code: context.code,
            }));
        }
    }

    let Some(code) = state
        .store
        .find_coupon_code_record(&redemption.coupon_code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    enforce_portal_coupon_rate_limit(
        state.store.as_ref(),
        &workspace.project.id,
        CouponRateLimitAction::Rollback,
        reservation.subject_scope,
        &reservation.subject_id,
        &code.code_value,
    )
    .await?;
    let Some(context) =
        load_marketing_coupon_context_from_code_record(state.store.as_ref(), code, now_ms).await?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        coupon_rollback_id,
        request.rollback_type,
        request.restored_budget_minor,
        request.restored_inventory_count,
        now_ms,
    )
    .map_err(|_| StatusCode::CONFLICT)?;
    let atomic_result = state
        .store
        .rollback_coupon_redemption_atomic(&AtomicCouponRollbackCommand {
            expected_budget: context.budget.clone(),
            next_budget: rollback_campaign_budget(
                &context.budget,
                request.restored_budget_minor,
                now_ms,
            ),
            expected_code: context.code.clone(),
            next_code: code_after_rollback(&context.template, &context.code, now_ms),
            expected_redemption: redemption.clone(),
            next_redemption: rolled_back_redemption,
            rollback,
        })
        .await
        .map_err(marketing_atomic_status)?;

    Ok(Json(PortalCouponRedemptionRollbackResponse {
        redemption: atomic_result.redemption,
        rollback: atomic_result.rollback,
        budget: atomic_result.budget,
        code: atomic_result.code,
    }))
}

async fn list_my_coupons_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalMarketingCodeItem>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    load_marketing_code_items(state.store.as_ref(), &subjects)
        .await
        .map(Json)
}

async fn list_marketing_reward_history_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalMarketingRewardHistoryItem>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    load_marketing_reward_history_items(state.store.as_ref(), &subjects)
        .await
        .map(Json)
}

async fn list_marketing_redemptions_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingRedemptionsQuery>,
) -> Result<Json<PortalMarketingRedemptionsResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let items =
        load_marketing_redemptions_for_subject(state.store.as_ref(), &subjects, query.status)
            .await?;
    let summary = summarize_marketing_redemptions(&items);
    Ok(Json(PortalMarketingRedemptionsResponse { summary, items }))
}

async fn list_marketing_codes_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingCodesQuery>,
) -> Result<Json<PortalMarketingCodesResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subjects = PortalMarketingSubjectSet::new(&workspace, claims.claims());
    let mut items = load_marketing_code_items(state.store.as_ref(), &subjects).await?;
    if let Some(status) = query.status {
        items.retain(|item| item.code.status == status);
    }
    let summary = summarize_marketing_code_items(&items);
    Ok(Json(PortalMarketingCodesResponse { summary, items }))
}

#[derive(Debug, Clone)]
struct MarketingCouponContext {
    template: CouponTemplateRecord,
    campaign: MarketingCampaignRecord,
    budget: CampaignBudgetRecord,
    code: CouponCodeRecord,
    compatibility_source: bool,
}

#[derive(Debug, Clone)]
struct PortalMarketingSubjectSet {
    user_id: String,
    project_id: String,
    workspace_id: String,
}

impl PortalMarketingSubjectSet {
    fn new(workspace: &PortalWorkspaceSummary, claims: &PortalClaims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            project_id: workspace.project.id.clone(),
            workspace_id: format!("{}:{}", workspace.tenant.id, workspace.project.id),
        }
    }

    fn subject_id_for_scope(&self, scope: MarketingSubjectScope) -> Option<String> {
        match scope {
            MarketingSubjectScope::User => Some(self.user_id.clone()),
            MarketingSubjectScope::Project => Some(self.project_id.clone()),
            MarketingSubjectScope::Workspace => Some(self.workspace_id.clone()),
            MarketingSubjectScope::Account => None,
        }
    }

    fn matches(&self, scope: MarketingSubjectScope, subject_id: &str) -> bool {
        match scope {
            MarketingSubjectScope::User => self.user_id == subject_id,
            MarketingSubjectScope::Project => self.project_id == subject_id,
            MarketingSubjectScope::Workspace => self.workspace_id == subject_id,
            MarketingSubjectScope::Account => false,
        }
    }
}

fn coupon_validation_decision_response(
    decision: CouponValidationDecision,
) -> PortalCouponValidationDecisionResponse {
    PortalCouponValidationDecisionResponse {
        eligible: decision.eligible,
        rejection_reason: decision.rejection_reason,
        reservable_budget_minor: decision.reservable_budget_minor,
    }
}

fn portal_marketing_target_kind_allowed(
    template: &CouponTemplateRecord,
    target_kind: &str,
) -> bool {
    template.restriction.eligible_target_kinds.is_empty()
        || template
            .restriction
            .eligible_target_kinds
            .iter()
            .any(|eligible| eligible == target_kind)
}

async fn load_marketing_coupon_context_by_value(
    store: &dyn AdminStore,
    code: &str,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>, StatusCode> {
    let normalized = normalize_coupon_code(code);
    if let Some(code_record) = store
        .find_coupon_code_record_by_value(&normalized)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        if let Some(context) =
            load_marketing_coupon_context_from_code_record(store, code_record, now_ms).await?
        {
            return Ok(Some(context));
        }
    }

    Ok(store
        .list_active_coupons()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .find(|coupon| normalize_coupon_code(&coupon.code) == normalized)
        .map(|coupon| {
            let (template, campaign, budget, code) = project_legacy_coupon_campaign(&coupon);
            MarketingCouponContext {
                template,
                campaign,
                budget,
                code,
                compatibility_source: true,
            }
        }))
}

async fn load_marketing_coupon_context_from_code_record(
    store: &dyn AdminStore,
    code: CouponCodeRecord,
    now_ms: u64,
) -> Result<Option<MarketingCouponContext>, StatusCode> {
    let Some(template) = store
        .find_coupon_template_record(&code.coupon_template_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(None);
    };

    let Some(campaign) = select_effective_marketing_campaign(
        store
            .list_marketing_campaign_records_for_template(&template.coupon_template_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        now_ms,
    ) else {
        return Ok(None);
    };

    let Some(budget) = select_campaign_budget_record(
        store
            .list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ) else {
        return Ok(None);
    };

    Ok(Some(MarketingCouponContext {
        template,
        campaign,
        budget,
        code,
        compatibility_source: false,
    }))
}

async fn load_marketing_coupon_context_for_code_id(
    store: &dyn AdminStore,
    coupon_code_id: &str,
    now_ms: u64,
) -> Result<MarketingCouponContext, StatusCode> {
    let Some(code) = store
        .find_coupon_code_record(coupon_code_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    load_marketing_coupon_context_from_code_record(store, code, now_ms)
        .await?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn find_coupon_rollback_record(
    store: &dyn AdminStore,
    rollback_id: &str,
) -> Result<Option<CouponRollbackRecord>, StatusCode> {
    Ok(store
        .list_coupon_rollback_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .find(|rollback| rollback.coupon_rollback_id == rollback_id))
}

async fn portal_marketing_reservation_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    reservation_id: &str,
) -> Result<CouponReservationRecord, StatusCode> {
    let Some(reservation) = store
        .find_coupon_reservation_record(reservation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !subjects.matches(reservation.subject_scope, &reservation.subject_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(reservation)
}

async fn portal_marketing_redemption_owned_by_subject(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    redemption_id: &str,
) -> Result<CouponRedemptionRecord, StatusCode> {
    let Some(redemption) = store
        .find_coupon_redemption_record(redemption_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let reservation = portal_marketing_reservation_owned_by_subject(
        store,
        subjects,
        &redemption.coupon_reservation_id,
    )
    .await?;
    if reservation.coupon_reservation_id != redemption.coupon_reservation_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(redemption)
}

async fn load_marketing_redemptions_for_subject(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
    status: Option<CouponRedemptionStatus>,
) -> Result<Vec<CouponRedemptionRecord>, StatusCode> {
    let reservations = store
        .list_coupon_reservation_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reservation_ids = reservations
        .into_iter()
        .filter(|reservation| subjects.matches(reservation.subject_scope, &reservation.subject_id))
        .map(|reservation| reservation.coupon_reservation_id)
        .collect::<HashSet<_>>();

    let mut redemptions = store
        .list_coupon_redemption_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|redemption| reservation_ids.contains(&redemption.coupon_reservation_id))
        .filter(|redemption| status.is_none_or(|expected| redemption.redemption_status == expected))
        .collect::<Vec<_>>();

    redemptions.sort_by(|left, right| {
        right
            .redeemed_at_ms
            .cmp(&left.redeemed_at_ms)
            .then_with(|| right.coupon_redemption_id.cmp(&left.coupon_redemption_id))
    });
    Ok(redemptions)
}

async fn load_marketing_code_items(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
) -> Result<Vec<PortalMarketingCodeItem>, StatusCode> {
    let reservations = store
        .list_coupon_reservation_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|reservation| subjects.matches(reservation.subject_scope, &reservation.subject_id))
        .collect::<Vec<_>>();

    let reservation_ids = reservations
        .iter()
        .map(|reservation| reservation.coupon_reservation_id.clone())
        .collect::<HashSet<_>>();
    let redemptions = store
        .list_coupon_redemption_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|redemption| reservation_ids.contains(&redemption.coupon_reservation_id))
        .collect::<Vec<_>>();
    let codes = store
        .list_coupon_code_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut latest_reservations = HashMap::new();
    for reservation in reservations {
        latest_reservations
            .entry(reservation.coupon_code_id.clone())
            .and_modify(|current: &mut CouponReservationRecord| {
                if reservation.updated_at_ms > current.updated_at_ms
                    || (reservation.updated_at_ms == current.updated_at_ms
                        && reservation.coupon_reservation_id > current.coupon_reservation_id)
                {
                    *current = reservation.clone();
                }
            })
            .or_insert(reservation);
    }

    let mut latest_redemptions = HashMap::new();
    for redemption in redemptions {
        latest_redemptions
            .entry(redemption.coupon_code_id.clone())
            .and_modify(|current: &mut CouponRedemptionRecord| {
                if redemption.updated_at_ms > current.updated_at_ms
                    || (redemption.updated_at_ms == current.updated_at_ms
                        && redemption.coupon_redemption_id > current.coupon_redemption_id)
                {
                    *current = redemption.clone();
                }
            })
            .or_insert(redemption);
    }

    let mut related_code_ids = latest_reservations.keys().cloned().collect::<HashSet<_>>();
    related_code_ids.extend(latest_redemptions.keys().cloned());

    let mut items = codes
        .into_iter()
        .filter(|code| {
            related_code_ids.contains(&code.coupon_code_id)
                || code
                    .claimed_subject_id
                    .as_deref()
                    .zip(code.claimed_subject_scope)
                    .is_some_and(|(subject_id, scope)| subjects.matches(scope, subject_id))
        })
        .map(|code| PortalMarketingCodeItem {
            latest_reservation: latest_reservations.get(&code.coupon_code_id).cloned(),
            latest_redemption: latest_redemptions.get(&code.coupon_code_id).cloned(),
            code,
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .code
            .updated_at_ms
            .cmp(&left.code.updated_at_ms)
            .then_with(|| right.code.coupon_code_id.cmp(&left.code.coupon_code_id))
    });
    Ok(items)
}

async fn load_marketing_reward_history_items(
    store: &dyn AdminStore,
    subjects: &PortalMarketingSubjectSet,
) -> Result<Vec<PortalMarketingRewardHistoryItem>, StatusCode> {
    let redemptions = load_marketing_redemptions_for_subject(store, subjects, None).await?;
    let codes = store
        .list_coupon_code_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|code| (code.coupon_code_id.clone(), code))
        .collect::<HashMap<_, _>>();
    let mut rollbacks_by_redemption = HashMap::new();
    for rollback in store
        .list_coupon_rollback_records()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        rollbacks_by_redemption
            .entry(rollback.coupon_redemption_id.clone())
            .or_insert_with(Vec::new)
            .push(rollback);
    }

    let mut items = redemptions
        .into_iter()
        .filter_map(|redemption| {
            codes.get(&redemption.coupon_code_id).cloned().map(|code| {
                let mut rollbacks = rollbacks_by_redemption
                    .remove(&redemption.coupon_redemption_id)
                    .unwrap_or_default();
                rollbacks.sort_by(|left, right| {
                    right
                        .created_at_ms
                        .cmp(&left.created_at_ms)
                        .then_with(|| right.coupon_rollback_id.cmp(&left.coupon_rollback_id))
                });
                PortalMarketingRewardHistoryItem {
                    redemption,
                    code,
                    rollbacks,
                }
            })
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .redemption
            .redeemed_at_ms
            .cmp(&left.redemption.redeemed_at_ms)
            .then_with(|| {
                right
                    .redemption
                    .coupon_redemption_id
                    .cmp(&left.redemption.coupon_redemption_id)
            })
    });
    Ok(items)
}

fn summarize_marketing_redemptions(
    items: &[CouponRedemptionRecord],
) -> PortalMarketingRedemptionSummary {
    let mut summary = PortalMarketingRedemptionSummary {
        total_count: items.len(),
        ..PortalMarketingRedemptionSummary::default()
    };
    for item in items {
        match item.redemption_status {
            CouponRedemptionStatus::Redeemed => summary.redeemed_count += 1,
            CouponRedemptionStatus::PartiallyRolledBack => summary.partially_rolled_back_count += 1,
            CouponRedemptionStatus::RolledBack => summary.rolled_back_count += 1,
            CouponRedemptionStatus::Failed => summary.failed_count += 1,
            CouponRedemptionStatus::Pending => {}
        }
    }
    summary
}

fn summarize_marketing_code_items(items: &[PortalMarketingCodeItem]) -> PortalMarketingCodeSummary {
    let mut summary = PortalMarketingCodeSummary {
        total_count: items.len(),
        ..PortalMarketingCodeSummary::default()
    };
    for item in items {
        match item.code.status {
            CouponCodeStatus::Available => summary.available_count += 1,
            CouponCodeStatus::Reserved => summary.reserved_count += 1,
            CouponCodeStatus::Redeemed => summary.redeemed_count += 1,
            CouponCodeStatus::Disabled => summary.disabled_count += 1,
            CouponCodeStatus::Expired => summary.expired_count += 1,
        }
    }
    summary
}

fn select_effective_marketing_campaign(
    campaigns: Vec<MarketingCampaignRecord>,
    now_ms: u64,
) -> Option<MarketingCampaignRecord> {
    campaigns
        .into_iter()
        .filter(|campaign| campaign.is_effective_at(now_ms))
        .max_by(|left, right| {
            left.updated_at_ms
                .cmp(&right.updated_at_ms)
                .then_with(|| left.marketing_campaign_id.cmp(&right.marketing_campaign_id))
        })
}

fn select_campaign_budget_record(
    budgets: Vec<CampaignBudgetRecord>,
) -> Option<CampaignBudgetRecord> {
    budgets.into_iter().max_by(|left, right| {
        left.updated_at_ms
            .cmp(&right.updated_at_ms)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    })
}

fn normalize_coupon_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn normalize_portal_idempotency_key(value: Option<&str>) -> Result<Option<String>, StatusCode> {
    let Some(value) = value.map(str::trim) else {
        return Ok(None);
    };
    if value.is_empty() || value.len() > 128 || value.chars().any(|ch| ch.is_control()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Some(value.to_owned()))
}

fn normalize_portal_idempotency_header_value(
    value: Option<&HeaderValue>,
) -> Result<Option<String>, StatusCode> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    normalize_portal_idempotency_key(Some(value))
}

fn resolve_portal_idempotency_key(
    headers: &HeaderMap,
    body_value: Option<&str>,
) -> Result<Option<String>, StatusCode> {
    let body_value = normalize_portal_idempotency_key(body_value)?;
    let header_value = normalize_portal_idempotency_header_value(headers.get("idempotency-key"))?;
    match (body_value, header_value) {
        (Some(body_value), Some(header_value)) if body_value != header_value => {
            Err(StatusCode::BAD_REQUEST)
        }
        (Some(body_value), Some(_)) | (Some(body_value), None) => Ok(Some(body_value)),
        (None, Some(header_value)) => Ok(Some(header_value)),
        (None, None) => Ok(None),
    }
}

fn marketing_subject_scope_token(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

async fn enforce_portal_coupon_rate_limit(
    store: &dyn AdminStore,
    project_id: &str,
    action: CouponRateLimitAction,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    coupon_code: &str,
) -> Result<(), StatusCode> {
    let actor_bucket =
        coupon_actor_bucket(marketing_subject_scope_token(subject_scope), subject_id);
    let evaluation = check_coupon_rate_limit(
        store,
        project_id,
        action,
        Some(actor_bucket.as_str()),
        Some(coupon_code),
        1,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if evaluation.allowed {
        Ok(())
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

fn marketing_idempotency_fingerprint(
    operation: &str,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    idempotency_key: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(operation.as_bytes());
    hasher.update([0x1f]);
    hasher.update(marketing_subject_scope_token(subject_scope).as_bytes());
    hasher.update([0x1f]);
    hasher.update(subject_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(idempotency_key.as_bytes());

    let digest = hasher.finalize();
    let mut fingerprint = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(&mut fingerprint, "{byte:02x}");
    }
    fingerprint
}

fn derive_coupon_reservation_id(
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    target_kind: &str,
    idempotency_key: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update("reserve".as_bytes());
    hasher.update([0x1f]);
    hasher.update(marketing_subject_scope_token(subject_scope).as_bytes());
    hasher.update([0x1f]);
    hasher.update(subject_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(target_kind.as_bytes());
    hasher.update([0x1f]);
    hasher.update(idempotency_key.as_bytes());

    let digest = hasher.finalize();
    let mut fingerprint = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        let _ = write!(&mut fingerprint, "{byte:02x}");
    }

    format!("coupon_reservation_{fingerprint}",)
}

fn derive_coupon_redemption_id(
    reservation: &CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_redemption_{}",
        marketing_idempotency_fingerprint(
            "confirm",
            reservation.subject_scope,
            &reservation.subject_id,
            idempotency_key,
        )
    )
}

fn derive_coupon_rollback_id(
    reservation: &CouponReservationRecord,
    idempotency_key: &str,
) -> String {
    format!(
        "coupon_rollback_{}",
        marketing_idempotency_fingerprint(
            "rollback",
            reservation.subject_scope,
            &reservation.subject_id,
            idempotency_key,
        )
    )
}

fn coupon_code_is_exclusive(template: &CouponTemplateRecord) -> bool {
    !matches!(
        template.distribution_kind,
        CouponDistributionKind::SharedCode
    )
}

fn code_after_reservation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    reserved_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        reserved_code.clone()
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_confirmation(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        original_code
            .clone()
            .with_status(CouponCodeStatus::Redeemed)
            .with_updated_at_ms(now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn code_after_rollback(
    template: &CouponTemplateRecord,
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    if coupon_code_is_exclusive(template) {
        restore_coupon_code_availability(original_code, now_ms)
    } else {
        original_code.clone().with_updated_at_ms(now_ms)
    }
}

fn restore_coupon_code_availability(
    original_code: &CouponCodeRecord,
    now_ms: u64,
) -> CouponCodeRecord {
    let next_status = if original_code
        .expires_at_ms
        .is_some_and(|value| now_ms > value)
    {
        CouponCodeStatus::Expired
    } else {
        CouponCodeStatus::Available
    };
    original_code
        .clone()
        .with_status(next_status)
        .with_updated_at_ms(now_ms)
}

fn reserve_campaign_budget(
    budget: &CampaignBudgetRecord,
    reserved_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_add(reserved_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            budget.consumed_budget_minor,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn confirm_campaign_budget(
    budget: &CampaignBudgetRecord,
    consumed_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_reserved = budget
        .reserved_budget_minor
        .saturating_sub(consumed_amount_minor);
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_add(consumed_amount_minor);
    budget
        .clone()
        .with_reserved_budget_minor(next_reserved)
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            next_reserved,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn rollback_campaign_budget(
    budget: &CampaignBudgetRecord,
    restored_amount_minor: u64,
    now_ms: u64,
) -> CampaignBudgetRecord {
    let next_consumed = budget
        .consumed_budget_minor
        .saturating_sub(restored_amount_minor);
    budget
        .clone()
        .with_consumed_budget_minor(next_consumed)
        .with_status(campaign_budget_status_after_mutation(
            budget.total_budget_minor,
            budget.reserved_budget_minor,
            next_consumed,
            budget.status,
        ))
        .with_updated_at_ms(now_ms)
}

fn campaign_budget_status_after_mutation(
    total_budget_minor: u64,
    reserved_budget_minor: u64,
    consumed_budget_minor: u64,
    prior_status: CampaignBudgetStatus,
) -> CampaignBudgetStatus {
    if matches!(
        prior_status,
        CampaignBudgetStatus::Closed | CampaignBudgetStatus::Draft
    ) {
        return prior_status;
    }

    let available_budget_minor = total_budget_minor
        .saturating_sub(reserved_budget_minor)
        .saturating_sub(consumed_budget_minor);
    if available_budget_minor == 0 {
        CampaignBudgetStatus::Exhausted
    } else {
        CampaignBudgetStatus::Active
    }
}

async fn commerce_catalog_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceCatalog>, StatusCode> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_portal_commerce_catalog(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn commerce_quote_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<Json<PortalCommerceQuote>, (StatusCode, Json<ErrorResponse>)> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    preview_portal_commerce_quote(state.store.as_ref(), &request)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

async fn list_commerce_orders_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalCommerceOrderRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    list_project_commerce_orders(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

async fn commerce_order_center_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderCenterResponse>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    let orders = list_project_commerce_orders(state.store.as_ref(), &workspace.project.id)
        .await
        .map_err(portal_commerce_error_response)?;
    let membership = load_project_membership(state.store.as_ref(), &workspace.project.id)
        .await
        .map_err(portal_commerce_error_response)?;

    let mut order_center_entries = Vec::with_capacity(orders.len());
    for order in orders {
        let mut payment_events = state
            .store
            .list_commerce_payment_events_for_order(&order.order_id)
            .await
            .map_err(CommerceError::from)
            .map_err(portal_commerce_error_response)?;
        payment_events.sort_by(|left, right| {
            right
                .received_at_ms
                .cmp(&left.received_at_ms)
                .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
        });
        let latest_payment_event = payment_events.first().cloned();
        let checkout_session = load_portal_commerce_checkout_session(
            state.store.as_ref(),
            &claims.claims().sub,
            &workspace.project.id,
            &order.order_id,
        )
        .await
        .map_err(portal_commerce_error_response)?;
        order_center_entries.push(PortalOrderCenterEntry {
            order,
            payment_events,
            latest_payment_event,
            checkout_session,
        });
    }
    let reconciliation =
        load_portal_commerce_reconciliation_summary(&state, &workspace, &order_center_entries)
            .await?;

    Ok(Json(PortalCommerceOrderCenterResponse {
        project_id: workspace.project.id,
        membership,
        reconciliation,
        orders: order_center_entries,
    }))
}

async fn create_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<(StatusCode, Json<PortalCommerceOrderRecord>), (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    submit_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &request,
    )
    .await
    .map(|order| (StatusCode::CREATED, Json(order)))
    .map_err(portal_commerce_error_response)
}

async fn settle_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    settle_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn cancel_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    cancel_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn apply_commerce_payment_event_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommercePaymentEventRequest>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    apply_portal_commerce_payment_event(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        &request,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn get_commerce_checkout_session_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceCheckoutSession>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_portal_commerce_checkout_session(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn get_project_membership_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Option<PortalProjectMembershipRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_project_membership(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

async fn list_api_keys_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

async fn list_api_key_groups_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<ApiKeyGroupRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_key_groups(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

async fn create_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyGroupRequest>,
) -> Result<(StatusCode, Json<ApiKeyGroupRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key_group(
        state.store.as_ref(),
        &claims.claims().sub,
        PortalApiKeyGroupInput {
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    .map(|group| (StatusCode::CREATED, Json(group)))
    .map_err(portal_error_response)
}

async fn update_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyGroupRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_portal_api_key_group(
        state.store.as_ref(),
        &claims.claims().sub,
        &group_id,
        PortalApiKeyGroupInput {
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(group) => Ok(Json(group)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
    }
}

async fn update_api_key_group_status_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyStatusRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_portal_api_key_group_active(
        state.store.as_ref(),
        &claims.claims().sub,
        &group_id,
        request.active,
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(group) => Ok(Json(group)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
    }
}

async fn delete_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deleted =
        delete_portal_api_key_group(state.store.as_ref(), &claims.claims().sub, &group_id)
            .await
            .map_err(portal_error_response)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        )))
    }
}

async fn create_api_key_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key_with_metadata(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.environment,
        &request.label,
        request.expires_at_ms,
        request.api_key.as_deref(),
        request.notes.as_deref(),
        request.api_key_group_id.as_deref(),
    )
    .await
    .map(|created| (StatusCode::CREATED, Json(created)))
    .map_err(portal_error_response)
}

async fn update_api_key_status_handler(
    claims: AuthenticatedPortalClaims,
    Path(hashed_key): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyStatusRequest>,
) -> Result<Json<GatewayApiKeyRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_portal_api_key_active(
        state.store.as_ref(),
        &claims.claims().sub,
        &hashed_key,
        request.active,
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(record) => Ok(Json(record)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key not found".to_owned(),
        ))),
    }
}

async fn delete_api_key_handler(
    claims: AuthenticatedPortalClaims,
    Path(hashed_key): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deleted = delete_portal_api_key(state.store.as_ref(), &claims.claims().sub, &hashed_key)
        .await
        .map_err(portal_error_response)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(portal_error_response(PortalIdentityError::NotFound(
            "api key not found".to_owned(),
        )))
    }
}

async fn dashboard_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalDashboardSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records =
        load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    let usage_summary = summarize_usage_records(&usage_records);
    let billing_summary =
        load_project_billing_summary(state.store.as_ref(), &workspace.project.id).await?;
    let api_key_count = list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    let recent_requests = usage_records.iter().take(10).cloned().collect();

    Ok(Json(PortalDashboardSummary {
        workspace,
        usage_summary,
        billing_summary,
        recent_requests,
        api_key_count,
    }))
}

async fn gateway_rate_limit_snapshot_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalGatewayRateLimitSnapshot>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let policies = state
        .store
        .list_rate_limit_policies_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let windows = state
        .store
        .list_rate_limit_window_snapshots_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_policy_count = policies.iter().filter(|policy| policy.enabled).count();
    let window_count = windows.len();
    let exceeded_window_count = windows.iter().filter(|window| window.exceeded).count();
    let headline = if policies.is_empty() {
        "No rate-limit policies configured yet".to_owned()
    } else if exceeded_window_count > 0 {
        "Rate-limit pressure is visible on the current project".to_owned()
    } else if active_policy_count > 0 {
        "Rate-limit posture is within configured headroom".to_owned()
    } else {
        "Rate-limit policies exist but are currently disabled".to_owned()
    };
    let detail = if policies.is_empty() {
        "The workspace has no visible project-scoped rate-limit policy yet, so the gateway still relies on the default protection surface.".to_owned()
    } else if exceeded_window_count > 0 {
        format!(
            "{} window(s) are currently over limit across {} policy record(s), so the portal is surfacing the live pressure state instead of waiting for a later audit.",
            exceeded_window_count,
            policies.len()
        )
    } else {
        format!(
            "{} active policy record(s) and {} live window snapshot(s) are currently within the configured limit posture for project {}.",
            active_policy_count, window_count, workspace.project.id
        )
    };

    Ok(Json(PortalGatewayRateLimitSnapshot {
        project_id: workspace.project.id,
        policy_count: policies.len(),
        active_policy_count,
        window_count,
        exceeded_window_count,
        headline,
        detail,
        generated_at_ms: current_time_millis(),
        policies,
        windows,
    }))
}

async fn load_portal_async_job_or_404(
    state: &PortalApiState,
    claims: &AuthenticatedPortalClaims,
    job_id: &str,
) -> Result<AsyncJobRecord, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subject =
        gateway_auth_subject_from_request_context(&portal_workspace_request_context(&workspace));
    let job = find_async_job(state.store.as_ref(), job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if job.tenant_id != subject.tenant_id
        || job.organization_id != subject.organization_id
        || job.user_id != subject.user_id
    {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(job)
}

async fn list_async_jobs_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AsyncJobRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let subject =
        gateway_auth_subject_from_request_context(&portal_workspace_request_context(&workspace));
    let mut jobs = list_async_jobs(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|job| {
            job.tenant_id == subject.tenant_id
                && job.organization_id == subject.organization_id
                && job.user_id == subject.user_id
        })
        .collect::<Vec<_>>();
    jobs.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| left.job_id.cmp(&right.job_id))
    });
    Ok(Json(jobs))
}

async fn list_async_job_attempts_handler(
    claims: AuthenticatedPortalClaims,
    Path(job_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AsyncJobAttemptRecord>>, StatusCode> {
    let _job = load_portal_async_job_or_404(&state, &claims, &job_id).await?;
    let mut attempts = list_async_job_attempts(state.store.as_ref(), &job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    attempts.sort_by_key(|attempt| attempt.attempt_id);
    Ok(Json(attempts))
}

async fn list_async_job_assets_handler(
    claims: AuthenticatedPortalClaims,
    Path(job_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AsyncJobAssetRecord>>, StatusCode> {
    let _job = load_portal_async_job_or_404(&state, &claims, &job_id).await?;
    let mut assets = list_async_job_assets(state.store.as_ref(), &job_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    assets.sort_by(|left, right| left.asset_id.cmp(&right.asset_id));
    Ok(Json(assets))
}
async fn list_usage_records_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_usage_records(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn usage_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records =
        load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    Ok(Json(summarize_usage_records(&usage_records)))
}

async fn billing_account_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalBillingAccountResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (account, balance) = load_portal_billing_account_context(&state, &claims).await?;
    Ok(Json(PortalBillingAccountResponse { account, balance }))
}

async fn billing_account_balance_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<AccountBalanceSnapshot>, (StatusCode, Json<ErrorResponse>)> {
    let (_, balance) = load_portal_billing_account_context(&state, &claims).await?;
    Ok(Json(balance))
}

async fn billing_account_history_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalBillingAccountHistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (account, balance) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut benefit_lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();
    benefit_lots.sort_by_key(|lot| lot.lot_id);

    let mut holds = commercial_billing
        .list_account_holds()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|hold| hold.account_id == account.account_id)
        .collect::<Vec<_>>();
    holds.sort_by_key(|hold| hold.hold_id);

    let mut request_settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|settlement| settlement.account_id == account.account_id)
        .collect::<Vec<_>>();
    request_settlements.sort_by_key(|settlement| settlement.request_settlement_id);

    let ledger = commercial_billing
        .list_account_ledger_history(account.account_id)
        .await
        .map_err(commercial_billing_error_response)?;

    Ok(Json(PortalBillingAccountHistoryResponse {
        account,
        balance,
        benefit_lots,
        holds,
        request_settlements,
        ledger,
    }))
}

async fn list_billing_account_benefit_lots_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AccountBenefitLotRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut lots = commercial_billing
        .list_account_benefit_lots()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();
    lots.sort_by_key(|lot| lot.lot_id);
    Ok(Json(lots))
}

async fn list_billing_account_holds_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AccountHoldRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut holds = commercial_billing
        .list_account_holds()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|hold| hold.account_id == account.account_id)
        .collect::<Vec<_>>();
    holds.sort_by_key(|hold| hold.hold_id);
    Ok(Json(holds))
}

async fn list_billing_request_settlements_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RequestSettlementRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let mut settlements = commercial_billing
        .list_request_settlement_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|settlement| settlement.account_id == account.account_id)
        .collect::<Vec<_>>();
    settlements.sort_by_key(|settlement| settlement.request_settlement_id);
    Ok(Json(settlements))
}

async fn list_billing_account_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<AccountLedgerHistoryEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    let history = commercial_billing
        .list_account_ledger_history(account.account_id)
        .await
        .map_err(commercial_billing_error_response)?;
    Ok(Json(history))
}

async fn list_billing_pricing_plans_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PricingPlanRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), current_time_millis())
        .await
        .map_err(commercial_billing_error_response)?;
    let mut plans = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|plan| {
            plan.tenant_id == account.tenant_id && plan.organization_id == account.organization_id
        })
        .collect::<Vec<_>>();
    plans.sort_by_key(|plan| plan.pricing_plan_id);
    Ok(Json(plans))
}

async fn list_billing_pricing_rates_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PricingRateRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let (account, _) = load_portal_billing_account_context(&state, &claims).await?;
    let commercial_billing = commercial_billing_kernel(&state)?.clone();
    synchronize_due_pricing_plan_lifecycle(commercial_billing.as_ref(), current_time_millis())
        .await
        .map_err(commercial_billing_error_response)?;
    let scoped_plan_ids = commercial_billing
        .list_pricing_plan_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|plan| {
            plan.tenant_id == account.tenant_id && plan.organization_id == account.organization_id
        })
        .map(|plan| plan.pricing_plan_id)
        .collect::<HashSet<_>>();
    let mut rates = commercial_billing
        .list_pricing_rate_records()
        .await
        .map_err(commercial_billing_error_response)?
        .into_iter()
        .filter(|rate| scoped_plan_ids.contains(&rate.pricing_plan_id))
        .collect::<Vec<_>>();
    rates.sort_by_key(|rate| rate.pricing_rate_id);
    Ok(Json(rates))
}
async fn billing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectBillingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_summary(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn list_billing_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let ledger = state
        .store
        .list_ledger_entries_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ledger))
}

async fn list_billing_events_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<BillingEventRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_events(
        state.store.as_ref(),
        &workspace.tenant.id,
        &workspace.project.id,
    )
    .await
    .map(Json)
}

async fn billing_events_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<BillingEventSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let events = load_project_billing_events(
        state.store.as_ref(),
        &workspace.tenant.id,
        &workspace.project.id,
    )
    .await?;
    Ok(Json(summarize_billing_events(&events)))
}

async fn get_routing_preferences_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectRoutingPreferences>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_routing_preferences_or_default(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn list_routing_profiles_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RoutingProfileRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    list_routing_profiles(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|profiles| {
            profiles
                .into_iter()
                .filter(|profile| {
                    profile.tenant_id == workspace.tenant.id
                        && profile.project_id == workspace.project.id
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
}

async fn list_routing_snapshots_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<CompiledRoutingSnapshotRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    list_compiled_routing_snapshots(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|snapshots| {
            snapshots
                .into_iter()
                .filter(|snapshot| {
                    snapshot.tenant_id.as_deref() == Some(workspace.tenant.id.as_str())
                        && snapshot.project_id.as_deref() == Some(workspace.project.id.as_str())
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
}

async fn create_routing_profile_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreatePortalRoutingProfileRequest>,
) -> Result<(StatusCode, Json<RoutingProfileRecord>), StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let normalized_name = normalize_portal_routing_profile_name(&request.name)?;
    let normalized_slug =
        normalize_portal_routing_profile_slug(&normalized_name, request.slug.as_deref())?;
    let profile_id = format!(
        "routing-profile-{}-{}",
        normalized_slug,
        current_time_millis()
    );

    let profile = create_routing_profile(CreateRoutingProfileInput {
        profile_id: &profile_id,
        tenant_id: &workspace.tenant.id,
        project_id: &workspace.project.id,
        name: &normalized_name,
        slug: &normalized_slug,
        description: request.description.as_deref(),
        active: request.active,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
        preferred_region: request.preferred_region.as_deref(),
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let profile = persist_routing_profile(state.store.as_ref(), &profile)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(profile)))
}

async fn save_routing_preferences_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<SaveRoutingPreferencesRequest>,
) -> Result<Json<ProjectRoutingPreferences>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let preferences = ProjectRoutingPreferences::new(workspace.project.id.clone())
        .with_preset_id(request.preset_id)
        .with_strategy(request.strategy)
        .with_ordered_provider_ids(request.ordered_provider_ids)
        .with_default_provider_id_option(request.default_provider_id)
        .with_max_cost_option(request.max_cost)
        .with_max_latency_ms_option(request.max_latency_ms)
        .with_require_healthy(request.require_healthy)
        .with_preferred_region_option(request.preferred_region)
        .with_updated_at_ms(current_time_millis());

    state
        .store
        .insert_project_routing_preferences(&preferences)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn preview_routing_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalRoutingPreviewRequest>,
) -> Result<Json<RoutingDecision>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    select_route_with_store_context(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        portal_route_selection_context(
            &workspace,
            RoutingDecisionSource::PortalSimulation,
            request.requested_region.as_deref(),
            request.selection_seed,
        ),
    )
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_routing_decision_logs_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RoutingDecisionLog>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_routing_decision_logs(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn routing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalRoutingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let preferences =
        load_project_routing_preferences_or_default(state.store.as_ref(), &workspace.project.id)
            .await?;
    let (latest_capability_hint, latest_model_hint) =
        load_latest_route_hint(state.store.as_ref(), &workspace.project.id).await?;
    let preview = simulate_route_with_store_selection_context(
        state.store.as_ref(),
        &latest_capability_hint,
        &latest_model_hint,
        portal_route_selection_context(
            &workspace,
            RoutingDecisionSource::PortalSimulation,
            None,
            None,
        ),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let provider_options =
        load_routing_provider_options(state.store.as_ref(), &latest_model_hint, &preferences)
            .await?;

    Ok(Json(PortalRoutingSummary {
        project_id: workspace.project.id,
        preferences,
        latest_model_hint,
        preview,
        provider_options,
    }))
}

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    let body = ErrorResponse {
        error: ErrorBody {
            message: message.into(),
        },
    };
    (status, Json(body))
}
fn portal_error_response(error: PortalIdentityError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        PortalIdentityError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        PortalIdentityError::DuplicateEmail => StatusCode::CONFLICT,
        PortalIdentityError::Protected(_) => StatusCode::CONFLICT,
        PortalIdentityError::InvalidCredentials | PortalIdentityError::InactiveUser => {
            StatusCode::UNAUTHORIZED
        }
        PortalIdentityError::NotFound(_) => StatusCode::NOT_FOUND,
        PortalIdentityError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

fn portal_commerce_error_response(error: CommerceError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        CommerceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CommerceError::NotFound(_) => StatusCode::NOT_FOUND,
        CommerceError::Conflict(_) => StatusCode::CONFLICT,
        CommerceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

fn marketing_atomic_status(error: anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("changed concurrently")
        || message.contains("already exists with different state")
        || message.contains(" is missing")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn commercial_billing_kernel(
    state: &PortalApiState,
) -> Result<&Arc<dyn CommercialBillingAdminKernel>, (StatusCode, Json<ErrorResponse>)> {
    state.commercial_billing.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "commercial billing portal views are unavailable for the current storage runtime",
        )
    })
}

fn commercial_billing_error_response(error: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = if message.starts_with("account ") && message.ends_with(" does not exist") {
        StatusCode::NOT_FOUND
    } else if message.contains("does not implement canonical account kernel method") {
        StatusCode::NOT_IMPLEMENTED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    error_response(status, message)
}

fn portal_workspace_request_context(workspace: &PortalWorkspaceSummary) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace.tenant.id.clone(),
        project_id: workspace.project.id.clone(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
    }
}

async fn load_portal_billing_account_context(
    state: &PortalApiState,
    claims: &AuthenticatedPortalClaims,
) -> Result<(AccountRecord, AccountBalanceSnapshot), (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| error_response(status, "portal workspace is unavailable"))?;
    let commercial_billing = commercial_billing_kernel(state)?.clone();
    let account = commercial_billing
        .resolve_payable_account_for_gateway_request_context(&portal_workspace_request_context(
            &workspace,
        ))
        .await
        .map_err(commercial_billing_error_response)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "workspace commercial account is not provisioned",
            )
        })?;
    let balance = commercial_billing
        .summarize_account_balance(account.account_id, current_time_millis())
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((account, balance))
}

async fn load_portal_commerce_reconciliation_summary(
    state: &PortalApiState,
    workspace: &PortalWorkspaceSummary,
    order_center_entries: &[PortalOrderCenterEntry],
) -> Result<Option<PortalCommerceReconciliationSummary>, (StatusCode, Json<ErrorResponse>)> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(None);
    };
    let account = commercial_billing
        .resolve_payable_account_for_gateway_request_context(&portal_workspace_request_context(
            workspace,
        ))
        .await
        .map_err(commercial_billing_error_response)?;
    let Some(account) = account else {
        return Ok(None);
    };
    let checkpoint = commercial_billing
        .find_account_commerce_reconciliation_state(account.account_id, &workspace.project.id)
        .await
        .map_err(commercial_billing_error_response)?;
    let last_reconciled_order_id = checkpoint
        .as_ref()
        .map(|record| record.last_order_id.clone())
        .unwrap_or_default();
    let last_reconciled_order_updated_at_ms = checkpoint
        .as_ref()
        .map(|record| record.last_order_updated_at_ms)
        .unwrap_or_default();
    let last_reconciled_order_created_at_ms = checkpoint
        .as_ref()
        .map(|record| record.last_order_created_at_ms)
        .unwrap_or_default();
    let last_reconciled_at_ms = checkpoint
        .as_ref()
        .map(|record| record.updated_at_ms)
        .unwrap_or_default();
    let latest_order_updated_at_ms = order_center_entries
        .iter()
        .map(|entry| entry.order.updated_at_ms)
        .max()
        .unwrap_or_default();
    let backlog_order_count = order_center_entries
        .iter()
        .filter(|entry| entry.order.updated_at_ms > last_reconciled_order_updated_at_ms)
        .count();

    Ok(Some(PortalCommerceReconciliationSummary {
        account_id: account.account_id,
        last_reconciled_order_id,
        last_reconciled_order_updated_at_ms,
        last_reconciled_order_created_at_ms,
        last_reconciled_at_ms,
        backlog_order_count,
        checkpoint_lag_ms: latest_order_updated_at_ms
            .saturating_sub(last_reconciled_order_updated_at_ms),
        healthy: backlog_order_count == 0,
    }))
}

async fn load_workspace_for_user(
    store: &dyn AdminStore,
    user_id: &str,
) -> Result<PortalWorkspaceSummary, StatusCode> {
    load_portal_workspace_summary(store, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn load_project_usage_records(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<Vec<UsageRecord>, StatusCode> {
    let usage_records = store
        .list_usage_records_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(usage_records)
}

async fn load_project_billing_summary(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectBillingSummary, StatusCode> {
    let ledger = store
        .list_ledger_entries_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let policies = store
        .list_quota_policies_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let billing = summarize_billing_snapshot(&ledger, &policies);

    Ok(billing
        .projects
        .into_iter()
        .next()
        .unwrap_or_else(|| ProjectBillingSummary::new(project_id.to_owned())))
}

async fn load_project_billing_events(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
) -> Result<Vec<BillingEventRecord>, StatusCode> {
    let events = list_billing_events(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(events
        .into_iter()
        .filter(|event| event.tenant_id == tenant_id && event.project_id == project_id)
        .collect())
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn portal_route_selection_context<'a>(
    workspace: &'a PortalWorkspaceSummary,
    decision_source: RoutingDecisionSource,
    requested_region: Option<&'a str>,
    selection_seed: Option<u64>,
) -> RouteSelectionContext<'a> {
    RouteSelectionContext::new(decision_source)
        .with_tenant_id_option(Some(workspace.tenant.id.as_str()))
        .with_project_id_option(Some(workspace.project.id.as_str()))
        .with_requested_region_option(requested_region)
        .with_selection_seed_option(selection_seed)
}

async fn load_project_routing_preferences_or_default(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectRoutingPreferences, StatusCode> {
    store
        .find_project_routing_preferences(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Ok)
        .unwrap_or_else(|| {
            Ok(ProjectRoutingPreferences::new(project_id.to_owned())
                .with_preset_id("platform_default"))
        })
}

async fn load_project_routing_decision_logs(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<Vec<RoutingDecisionLog>, StatusCode> {
    let logs = store
        .list_routing_decision_logs_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(logs)
}

async fn load_latest_route_hint(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<(String, String), StatusCode> {
    if let Some(log) = store
        .find_latest_routing_decision_log_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok((log.capability.clone(), log.route_key.clone()));
    }

    if let Some(record) = store
        .find_latest_usage_record_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(("chat_completion".to_owned(), record.model.clone()));
    }

    if let Some(model) = store
        .find_any_model()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(("chat_completion".to_owned(), model.external_name.clone()));
    }

    Ok(("chat_completion".to_owned(), "gpt-4.1".to_owned()))
}

async fn load_routing_provider_options(
    store: &dyn AdminStore,
    model: &str,
    preferences: &ProjectRoutingPreferences,
) -> Result<Vec<PortalRoutingProviderOption>, StatusCode> {
    let mut providers = store
        .list_providers_for_model(model)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .collect::<Vec<_>>();

    if providers.is_empty() {
        providers = store
            .list_providers()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let preference_ranks = provider_preference_ranks(preferences);
    let preferred_provider_ids = preferences
        .ordered_provider_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    sort_routing_provider_options(&mut providers, &preference_ranks);

    Ok(providers
        .into_iter()
        .map(|provider| PortalRoutingProviderOption {
            preferred: preferred_provider_ids.contains(&provider.id),
            default_provider: preferences.default_provider_id.as_deref() == Some(&provider.id),
            provider_id: provider.id,
            display_name: provider.display_name,
            channel_id: provider.channel_id,
        })
        .collect())
}

fn sort_routing_provider_options(
    providers: &mut [ProxyProvider],
    preference_ranks: &HashMap<String, usize>,
) {
    providers.sort_by(|left, right| {
        provider_preference_rank(preference_ranks, &left.id)
            .cmp(&provider_preference_rank(preference_ranks, &right.id))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn provider_preference_ranks(preferences: &ProjectRoutingPreferences) -> HashMap<String, usize> {
    preferences
        .ordered_provider_ids
        .iter()
        .enumerate()
        .map(|(index, provider_id)| (provider_id.clone(), index))
        .collect()
}

fn provider_preference_rank(preference_ranks: &HashMap<String, usize>, provider_id: &str) -> usize {
    preference_ranks
        .get(provider_id)
        .copied()
        .unwrap_or(usize::MAX)
}

fn normalize_portal_routing_profile_name(name: &str) -> Result<String, StatusCode> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(normalized.to_owned())
}

fn normalize_portal_routing_profile_slug(
    name: &str,
    slug: Option<&str>,
) -> Result<String, StatusCode> {
    let source = normalize_portal_routing_profile_optional_value(slug).unwrap_or(name.to_owned());
    let mut normalized = String::new();
    let mut previous_was_dash = false;

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_dash = false;
        } else if !normalized.is_empty() && !previous_was_dash {
            normalized.push('-');
            previous_was_dash = true;
        }
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(normalized)
}

fn normalize_portal_routing_profile_optional_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
