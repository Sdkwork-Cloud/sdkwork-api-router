use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateExtensionInstallationRequest {
    pub(crate) installation_id: String,
    pub(crate) extension_id: String,
    pub(crate) runtime: ExtensionRuntime,
    pub(crate) enabled: bool,
    pub(crate) entrypoint: Option<String>,
    #[serde(default)]
    pub(crate) config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateExtensionInstanceRequest {
    pub(crate) instance_id: String,
    pub(crate) installation_id: String,
    pub(crate) extension_id: String,
    pub(crate) enabled: bool,
    pub(crate) base_url: Option<String>,
    pub(crate) credential_ref: Option<String>,
    #[serde(default)]
    pub(crate) config: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExtensionRuntimeReloadScope {
    All,
    Extension,
    Instance,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ExtensionRuntimeReloadRequest {
    #[serde(default)]
    pub(crate) extension_id: Option<String>,
    #[serde(default)]
    pub(crate) instance_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ExtensionRuntimeRolloutCreateRequest {
    #[serde(default)]
    pub(crate) extension_id: Option<String>,
    #[serde(default)]
    pub(crate) instance_id: Option<String>,
    #[serde(default)]
    pub(crate) timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionRuntimeReloadResponse {
    pub(crate) scope: ExtensionRuntimeReloadScope,
    pub(crate) requested_extension_id: Option<String>,
    pub(crate) requested_instance_id: Option<String>,
    pub(crate) resolved_extension_id: Option<String>,
    pub(crate) discovered_package_count: usize,
    pub(crate) loadable_package_count: usize,
    pub(crate) active_runtime_count: usize,
    pub(crate) reloaded_at_ms: u64,
    pub(crate) runtime_statuses: Vec<sdkwork_api_app_extension::ExtensionRuntimeStatusRecord>,
}

pub(crate) struct ResolvedExtensionRuntimeReloadRequest {
    pub(crate) scope: ExtensionRuntimeReloadScope,
    pub(crate) requested_extension_id: Option<String>,
    pub(crate) requested_instance_id: Option<String>,
    pub(crate) resolved_extension_id: Option<String>,
    pub(crate) gateway_scope: ConfiguredExtensionHostReloadScope,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionRuntimeRolloutParticipantResponse {
    pub(crate) node_id: String,
    pub(crate) service_kind: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    pub(crate) updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExtensionRuntimeRolloutResponse {
    pub(crate) rollout_id: String,
    pub(crate) status: String,
    pub(crate) scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_extension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_extension_id: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_at_ms: u64,
    pub(crate) deadline_at_ms: u64,
    pub(crate) participant_count: usize,
    pub(crate) participants: Vec<ExtensionRuntimeRolloutParticipantResponse>,
}

impl From<ExtensionRuntimeRolloutDetails> for ExtensionRuntimeRolloutResponse {
    fn from(value: ExtensionRuntimeRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            scope: value.scope,
            requested_extension_id: value.requested_extension_id,
            requested_instance_id: value.requested_instance_id,
            resolved_extension_id: value.resolved_extension_id,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| ExtensionRuntimeRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct StandaloneConfigRolloutCreateRequest {
    #[serde(default)]
    pub(crate) service_kind: Option<String>,
    #[serde(default)]
    pub(crate) timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct StandaloneConfigRolloutParticipantResponse {
    pub(crate) node_id: String,
    pub(crate) service_kind: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    pub(crate) updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct StandaloneConfigRolloutResponse {
    pub(crate) rollout_id: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) requested_service_kind: Option<String>,
    pub(crate) created_by: String,
    pub(crate) created_at_ms: u64,
    pub(crate) deadline_at_ms: u64,
    pub(crate) participant_count: usize,
    pub(crate) participants: Vec<StandaloneConfigRolloutParticipantResponse>,
}

impl From<StandaloneConfigRolloutDetails> for StandaloneConfigRolloutResponse {
    fn from(value: StandaloneConfigRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            requested_service_kind: value.requested_service_kind,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| StandaloneConfigRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}
