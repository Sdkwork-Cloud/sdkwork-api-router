use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AsyncJobStatus {
    Queued,
    Running,
    AwaitingCallback,
    Succeeded,
    Failed,
    Canceled,
}

impl AsyncJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::AwaitingCallback => "awaiting_callback",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }
}

impl FromStr for AsyncJobStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "awaiting_callback" => Ok(Self::AwaitingCallback),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "canceled" => Ok(Self::Canceled),
            other => Err(format!("unknown async job status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AsyncJobAttemptStatus {
    Pending,
    Running,
    Retrying,
    Succeeded,
    Failed,
    Canceled,
}

impl AsyncJobAttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Retrying => "retrying",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }
}

impl FromStr for AsyncJobAttemptStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "retrying" => Ok(Self::Retrying),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "canceled" => Ok(Self::Canceled),
            other => Err(format!("unknown async job attempt status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AsyncJobCallbackStatus {
    Received,
    Processed,
    Ignored,
    Failed,
}

impl AsyncJobCallbackStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Processed => "processed",
            Self::Ignored => "ignored",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for AsyncJobCallbackStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "received" => Ok(Self::Received),
            "processed" => Ok(Self::Processed),
            "ignored" => Ok(Self::Ignored),
            "failed" => Ok(Self::Failed),
            other => Err(format!("unknown async job callback status: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AsyncJobRecord {
    pub job_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_code: Option<String>,
    pub capability_code: String,
    pub modality: String,
    pub operation_kind: String,
    pub status: AsyncJobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at_ms: Option<u64>,
}

impl AsyncJobRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        capability_code: impl Into<String>,
        modality: impl Into<String>,
        operation_kind: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            tenant_id,
            organization_id,
            user_id,
            account_id: None,
            request_id: None,
            provider_id: None,
            model_code: None,
            capability_code: capability_code.into(),
            modality: modality.into(),
            operation_kind: operation_kind.into(),
            status: AsyncJobStatus::Queued,
            external_job_id: None,
            idempotency_key: None,
            callback_url: None,
            input_summary: None,
            progress_percent: None,
            error_code: None,
            error_message: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
            started_at_ms: None,
            completed_at_ms: None,
        }
    }

    pub fn with_account_id(mut self, account_id: Option<u64>) -> Self {
        self.account_id = account_id;
        self
    }

    pub fn with_request_id(mut self, request_id: Option<u64>) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn with_provider_id(mut self, provider_id: Option<String>) -> Self {
        self.provider_id = provider_id;
        self
    }

    pub fn with_model_code(mut self, model_code: Option<String>) -> Self {
        self.model_code = model_code;
        self
    }

    pub fn with_status(mut self, status: AsyncJobStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_external_job_id(mut self, external_job_id: Option<String>) -> Self {
        self.external_job_id = external_job_id;
        self
    }

    pub fn with_idempotency_key(mut self, idempotency_key: Option<String>) -> Self {
        self.idempotency_key = idempotency_key;
        self
    }

    pub fn with_callback_url(mut self, callback_url: Option<String>) -> Self {
        self.callback_url = callback_url;
        self
    }

    pub fn with_input_summary(mut self, input_summary: Option<String>) -> Self {
        self.input_summary = input_summary;
        self
    }

    pub fn with_progress_percent(mut self, progress_percent: Option<u64>) -> Self {
        self.progress_percent = progress_percent;
        self
    }

    pub fn with_error_code(mut self, error_code: Option<String>) -> Self {
        self.error_code = error_code;
        self
    }

    pub fn with_error_message(mut self, error_message: Option<String>) -> Self {
        self.error_message = error_message;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn with_started_at_ms(mut self, started_at_ms: Option<u64>) -> Self {
        self.started_at_ms = started_at_ms;
        self
    }

    pub fn with_completed_at_ms(mut self, completed_at_ms: Option<u64>) -> Self {
        self.completed_at_ms = completed_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AsyncJobAttemptRecord {
    pub attempt_id: u64,
    pub job_id: String,
    pub attempt_number: u64,
    pub status: AsyncJobAttemptStatus,
    pub runtime_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl AsyncJobAttemptRecord {
    pub fn new(
        attempt_id: u64,
        job_id: impl Into<String>,
        attempt_number: u64,
        runtime_kind: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            attempt_id,
            job_id: job_id.into(),
            attempt_number,
            status: AsyncJobAttemptStatus::Pending,
            runtime_kind: runtime_kind.into(),
            endpoint: None,
            external_job_id: None,
            claimed_at_ms: None,
            finished_at_ms: None,
            error_message: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: AsyncJobAttemptStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_endpoint(mut self, endpoint: Option<String>) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub fn with_external_job_id(mut self, external_job_id: Option<String>) -> Self {
        self.external_job_id = external_job_id;
        self
    }

    pub fn with_claimed_at_ms(mut self, claimed_at_ms: Option<u64>) -> Self {
        self.claimed_at_ms = claimed_at_ms;
        self
    }

    pub fn with_finished_at_ms(mut self, finished_at_ms: Option<u64>) -> Self {
        self.finished_at_ms = finished_at_ms;
        self
    }

    pub fn with_error_message(mut self, error_message: Option<String>) -> Self {
        self.error_message = error_message;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AsyncJobAssetRecord {
    pub asset_id: String,
    pub job_id: String,
    pub asset_kind: String,
    pub storage_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256: Option<String>,
    pub created_at_ms: u64,
}

impl AsyncJobAssetRecord {
    pub fn new(
        asset_id: impl Into<String>,
        job_id: impl Into<String>,
        asset_kind: impl Into<String>,
        storage_key: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            asset_id: asset_id.into(),
            job_id: job_id.into(),
            asset_kind: asset_kind.into(),
            storage_key: storage_key.into(),
            download_url: None,
            mime_type: None,
            size_bytes: None,
            checksum_sha256: None,
            created_at_ms,
        }
    }

    pub fn with_download_url(mut self, download_url: Option<String>) -> Self {
        self.download_url = download_url;
        self
    }

    pub fn with_mime_type(mut self, mime_type: Option<String>) -> Self {
        self.mime_type = mime_type;
        self
    }

    pub fn with_size_bytes(mut self, size_bytes: Option<u64>) -> Self {
        self.size_bytes = size_bytes;
        self
    }

    pub fn with_checksum_sha256(mut self, checksum_sha256: Option<String>) -> Self {
        self.checksum_sha256 = checksum_sha256;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AsyncJobCallbackRecord {
    pub callback_id: u64,
    pub job_id: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedupe_key: Option<String>,
    pub payload_json: String,
    pub status: AsyncJobCallbackStatus,
    pub received_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_at_ms: Option<u64>,
}

impl AsyncJobCallbackRecord {
    pub fn new(
        callback_id: u64,
        job_id: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        received_at_ms: u64,
    ) -> Self {
        Self {
            callback_id,
            job_id: job_id.into(),
            event_type: event_type.into(),
            dedupe_key: None,
            payload_json: payload_json.into(),
            status: AsyncJobCallbackStatus::Received,
            received_at_ms,
            processed_at_ms: None,
        }
    }

    pub fn with_dedupe_key(mut self, dedupe_key: Option<String>) -> Self {
        self.dedupe_key = dedupe_key;
        self
    }

    pub fn with_status(mut self, status: AsyncJobCallbackStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_processed_at_ms(mut self, processed_at_ms: Option<u64>) -> Self {
        self.processed_at_ms = processed_at_ms;
        self
    }
}
