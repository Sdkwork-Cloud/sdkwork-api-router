use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRoutingPolicyRequest {
    pub(crate) policy_id: String,
    pub(crate) capability: String,
    pub(crate) model_pattern: String,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) priority: i32,
    #[serde(default)]
    pub(crate) strategy: Option<RoutingStrategy>,
    #[serde(default)]
    pub(crate) ordered_provider_ids: Vec<String>,
    #[serde(default)]
    pub(crate) default_provider_id: Option<String>,
    #[serde(default)]
    pub(crate) max_cost: Option<f64>,
    #[serde(default)]
    pub(crate) max_latency_ms: Option<u64>,
    #[serde(default)]
    pub(crate) require_healthy: bool,
    #[serde(default = "default_true")]
    pub(crate) execution_failover_enabled: bool,
    #[serde(default)]
    pub(crate) upstream_retry_max_attempts: Option<u32>,
    #[serde(default)]
    pub(crate) upstream_retry_base_delay_ms: Option<u64>,
    #[serde(default)]
    pub(crate) upstream_retry_max_delay_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateRoutingProfileRequest {
    pub(crate) profile_id: String,
    pub(crate) tenant_id: String,
    pub(crate) project_id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default = "default_true")]
    pub(crate) active: bool,
    #[serde(default)]
    pub(crate) strategy: Option<RoutingStrategy>,
    #[serde(default)]
    pub(crate) ordered_provider_ids: Vec<String>,
    #[serde(default)]
    pub(crate) default_provider_id: Option<String>,
    #[serde(default)]
    pub(crate) max_cost: Option<f64>,
    #[serde(default)]
    pub(crate) max_latency_ms: Option<u64>,
    #[serde(default)]
    pub(crate) require_healthy: bool,
    #[serde(default)]
    pub(crate) preferred_region: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoutingSimulationRequest {
    pub(crate) capability: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) tenant_id: Option<String>,
    #[serde(default)]
    pub(crate) project_id: Option<String>,
    #[serde(default)]
    pub(crate) api_key_group_id: Option<String>,
    #[serde(default)]
    pub(crate) requested_region: Option<String>,
    #[serde(default)]
    pub(crate) selection_seed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RoutingSimulationResponse {
    pub(crate) selected_provider_id: String,
    pub(crate) candidate_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) matched_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) applied_routing_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) compiled_routing_snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selection_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selection_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fallback_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_region: Option<String>,
    #[serde(default)]
    pub(crate) slo_applied: bool,
    #[serde(default)]
    pub(crate) slo_degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selected_candidate: Option<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) rejected_candidates: Vec<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) assessments: Vec<RoutingCandidateAssessment>,
}
