use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct UsageRecord {
    pub project_id: String,
    pub model: String,
    pub provider: String,
    #[serde(default)]
    pub units: u64,
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_amount: Option<f64>,
}

impl UsageRecord {
    pub fn new(
        project_id: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            model: model.into(),
            provider: provider.into(),
            units: 0,
            amount: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            created_at_ms: 0,
            api_key_hash: None,
            channel_id: None,
            latency_ms: None,
            reference_amount: None,
        }
    }

    pub fn with_metering(mut self, units: u64, amount: f64, created_at_ms: u64) -> Self {
        self.units = units;
        self.amount = amount;
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_token_usage(
        mut self,
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
    ) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self.total_tokens = total_tokens;
        self
    }

    pub fn with_request_facts(
        mut self,
        api_key_hash: Option<&str>,
        channel_id: Option<&str>,
        latency_ms: Option<u64>,
        reference_amount: Option<f64>,
    ) -> Self {
        self.api_key_hash = api_key_hash.map(ToOwned::to_owned);
        self.channel_id = channel_id.map(ToOwned::to_owned);
        self.latency_ms = latency_ms;
        self.reference_amount = reference_amount;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageProjectSummary {
    pub project_id: String,
    pub request_count: u64,
}

impl UsageProjectSummary {
    pub fn new(project_id: impl Into<String>, request_count: u64) -> Self {
        Self {
            project_id: project_id.into(),
            request_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageProviderSummary {
    pub provider: String,
    pub request_count: u64,
    pub project_count: u64,
}

impl UsageProviderSummary {
    pub fn new(provider: impl Into<String>, request_count: u64, project_count: u64) -> Self {
        Self {
            provider: provider.into(),
            request_count,
            project_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageModelSummary {
    pub model: String,
    pub request_count: u64,
    pub provider_count: u64,
}

impl UsageModelSummary {
    pub fn new(model: impl Into<String>, request_count: u64, provider_count: u64) -> Self {
        Self {
            model: model.into(),
            request_count,
            provider_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsageSummary {
    pub total_requests: u64,
    pub project_count: u64,
    pub model_count: u64,
    pub provider_count: u64,
    pub projects: Vec<UsageProjectSummary>,
    pub providers: Vec<UsageProviderSummary>,
    pub models: Vec<UsageModelSummary>,
}

impl UsageSummary {
    pub fn empty() -> Self {
        Self {
            total_requests: 0,
            project_count: 0,
            model_count: 0,
            provider_count: 0,
            projects: Vec::new(),
            providers: Vec::new(),
            models: Vec::new(),
        }
    }
}

pub type RequestId = u64;
pub type RequestMetricId = u64;
pub type AccountId = u64;
pub type PricingPlanId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageCaptureStatus {
    Pending,
    Estimated,
    Captured,
    Reconciled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMeterFactRecord {
    pub request_id: RequestId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub account_id: AccountId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    pub auth_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jwt_subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_request_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_request_ref: Option<String>,
    pub protocol_family: String,
    pub capability_code: String,
    pub channel_code: String,
    pub model_code: String,
    pub provider_code: String,
    pub request_status: RequestStatus,
    pub usage_capture_status: UsageCaptureStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_pricing_plan_id: Option<PricingPlanId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retail_pricing_plan_id: Option<PricingPlanId>,
    pub estimated_credit_hold: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_credit_charge: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_provider_cost: Option<f64>,
    pub started_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl RequestMeterFactRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        request_id: RequestId,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        account_id: AccountId,
        auth_type: impl Into<String>,
        capability_code: impl Into<String>,
        channel_code: impl Into<String>,
        model_code: impl Into<String>,
        provider_code: impl Into<String>,
    ) -> Self {
        Self {
            request_id,
            tenant_id,
            organization_id,
            user_id,
            account_id,
            api_key_id: None,
            api_key_hash: None,
            auth_type: auth_type.into(),
            jwt_subject: None,
            platform: None,
            owner: None,
            request_trace_id: None,
            gateway_request_ref: None,
            upstream_request_ref: None,
            protocol_family: String::new(),
            capability_code: capability_code.into(),
            channel_code: channel_code.into(),
            model_code: model_code.into(),
            provider_code: provider_code.into(),
            request_status: RequestStatus::Pending,
            usage_capture_status: UsageCaptureStatus::Pending,
            cost_pricing_plan_id: None,
            retail_pricing_plan_id: None,
            estimated_credit_hold: 0.0,
            actual_credit_charge: None,
            actual_provider_cost: None,
            started_at_ms: 0,
            finished_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_api_key_id(mut self, api_key_id: Option<u64>) -> Self {
        self.api_key_id = api_key_id;
        self
    }

    pub fn with_api_key_hash(mut self, api_key_hash: Option<String>) -> Self {
        self.api_key_hash = api_key_hash;
        self
    }

    pub fn with_jwt_subject(mut self, jwt_subject: Option<String>) -> Self {
        self.jwt_subject = jwt_subject;
        self
    }

    pub fn with_platform(mut self, platform: Option<String>) -> Self {
        self.platform = platform;
        self
    }

    pub fn with_owner(mut self, owner: Option<String>) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_request_trace_id(mut self, request_trace_id: Option<String>) -> Self {
        self.request_trace_id = request_trace_id;
        self
    }

    pub fn with_gateway_request_ref(mut self, gateway_request_ref: Option<String>) -> Self {
        self.gateway_request_ref = gateway_request_ref;
        self
    }

    pub fn with_upstream_request_ref(mut self, upstream_request_ref: Option<String>) -> Self {
        self.upstream_request_ref = upstream_request_ref;
        self
    }

    pub fn with_protocol_family(mut self, protocol_family: impl Into<String>) -> Self {
        self.protocol_family = protocol_family.into();
        self
    }

    pub fn with_request_status(mut self, request_status: RequestStatus) -> Self {
        self.request_status = request_status;
        self
    }

    pub fn with_usage_capture_status(
        mut self,
        usage_capture_status: UsageCaptureStatus,
    ) -> Self {
        self.usage_capture_status = usage_capture_status;
        self
    }

    pub fn with_cost_pricing_plan_id(
        mut self,
        cost_pricing_plan_id: Option<PricingPlanId>,
    ) -> Self {
        self.cost_pricing_plan_id = cost_pricing_plan_id;
        self
    }

    pub fn with_retail_pricing_plan_id(
        mut self,
        retail_pricing_plan_id: Option<PricingPlanId>,
    ) -> Self {
        self.retail_pricing_plan_id = retail_pricing_plan_id;
        self
    }

    pub fn with_estimated_credit_hold(mut self, estimated_credit_hold: f64) -> Self {
        self.estimated_credit_hold = estimated_credit_hold;
        self
    }

    pub fn with_actual_credit_charge(mut self, actual_credit_charge: Option<f64>) -> Self {
        self.actual_credit_charge = actual_credit_charge;
        self
    }

    pub fn with_actual_provider_cost(mut self, actual_provider_cost: Option<f64>) -> Self {
        self.actual_provider_cost = actual_provider_cost;
        self
    }

    pub fn with_started_at_ms(mut self, started_at_ms: u64) -> Self {
        self.started_at_ms = started_at_ms;
        self
    }

    pub fn with_finished_at_ms(mut self, finished_at_ms: Option<u64>) -> Self {
        self.finished_at_ms = finished_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMeterMetricRecord {
    pub request_metric_id: RequestMetricId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub request_id: RequestId,
    pub metric_code: String,
    pub quantity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_field: Option<String>,
    pub source_kind: String,
    pub capture_stage: String,
    pub is_billable: bool,
    pub captured_at_ms: u64,
}

impl RequestMeterMetricRecord {
    pub fn new(
        request_metric_id: RequestMetricId,
        tenant_id: u64,
        organization_id: u64,
        request_id: RequestId,
        metric_code: impl Into<String>,
        quantity: f64,
    ) -> Self {
        Self {
            request_metric_id,
            tenant_id,
            organization_id,
            request_id,
            metric_code: metric_code.into(),
            quantity,
            provider_field: None,
            source_kind: "provider".to_owned(),
            capture_stage: "final".to_owned(),
            is_billable: true,
            captured_at_ms: 0,
        }
    }

    pub fn with_provider_field(mut self, provider_field: Option<String>) -> Self {
        self.provider_field = provider_field;
        self
    }

    pub fn with_source_kind(mut self, source_kind: impl Into<String>) -> Self {
        self.source_kind = source_kind.into();
        self
    }

    pub fn with_capture_stage(mut self, capture_stage: impl Into<String>) -> Self {
        self.capture_stage = capture_stage.into();
        self
    }

    pub fn with_is_billable(mut self, is_billable: bool) -> Self {
        self.is_billable = is_billable;
        self
    }

    pub fn with_captured_at_ms(mut self, captured_at_ms: u64) -> Self {
        self.captured_at_ms = captured_at_ms;
        self
    }
}
