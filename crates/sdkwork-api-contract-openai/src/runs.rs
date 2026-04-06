use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateRunRequest {
    pub assistant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl CreateRunRequest {
    pub fn new(assistant_id: impl Into<String>) -> Self {
        Self {
            assistant_id: assistant_id.into(),
            model: None,
            instructions: None,
            metadata: None,
            stream: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateThreadAndRunRequest {
    pub assistant_id: String,
    pub thread: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl CreateThreadAndRunRequest {
    pub fn new(assistant_id: impl Into<String>, thread: Value) -> Self {
        Self {
            assistant_id: assistant_id.into(),
            thread,
            model: None,
            instructions: None,
            metadata: None,
            stream: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateRunRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl UpdateRunRequest {
    pub fn with_metadata(metadata: Value) -> Self {
        Self {
            metadata: Some(metadata),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RunToolOutput {
    pub tool_call_id: String,
    pub output: String,
}

impl RunToolOutput {
    pub fn new(tool_call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            output: output.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubmitToolOutputsRunRequest {
    pub tool_outputs: Vec<RunToolOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl SubmitToolOutputsRunRequest {
    pub fn new(tool_outputs: Vec<RunToolOutput>) -> Self {
        Self {
            tool_outputs,
            stream: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RunObject {
    pub id: String,
    pub object: &'static str,
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    pub status: &'static str,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl RunObject {
    pub fn queued(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        assistant_id: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self::with_status(id, thread_id, assistant_id, model, "queued")
    }

    pub fn in_progress(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        assistant_id: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self::with_status(id, thread_id, assistant_id, model, "in_progress")
    }

    pub fn cancelled(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        assistant_id: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self::with_status(id, thread_id, assistant_id, model, "cancelled")
    }

    pub fn with_metadata(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        assistant_id: impl Into<String>,
        model: impl Into<String>,
        status: &'static str,
        metadata: Value,
    ) -> Self {
        let mut run = Self::with_status(id, thread_id, assistant_id, model, status);
        run.metadata = Some(metadata);
        run
    }

    fn with_status(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        assistant_id: impl Into<String>,
        model: impl Into<String>,
        status: &'static str,
    ) -> Self {
        Self {
            id: id.into(),
            object: "thread.run",
            thread_id: thread_id.into(),
            assistant_id: Some(assistant_id.into()),
            status,
            model: model.into(),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListRunsResponse {
    pub object: &'static str,
    pub data: Vec<RunObject>,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub has_more: bool,
}

impl ListRunsResponse {
    pub fn new(data: Vec<RunObject>) -> Self {
        Self {
            object: "list",
            first_id: data.first().map(|run| run.id.clone()),
            last_id: data.last().map(|run| run.id.clone()),
            has_more: false,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RunStepObject {
    pub id: String,
    pub object: &'static str,
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    pub run_id: String,
    pub r#type: &'static str,
    pub status: &'static str,
    pub step_details: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl RunStepObject {
    pub fn message_creation(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        run_id: impl Into<String>,
        assistant_id: impl Into<String>,
        message_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "thread.run.step",
            thread_id: thread_id.into(),
            assistant_id: Some(assistant_id.into()),
            run_id: run_id.into(),
            r#type: "message_creation",
            status: "completed",
            step_details: serde_json::json!({
                "message_creation": {
                    "message_id": message_id.into()
                }
            }),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListRunStepsResponse {
    pub object: &'static str,
    pub data: Vec<RunStepObject>,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub has_more: bool,
}

impl ListRunStepsResponse {
    pub fn new(data: Vec<RunStepObject>) -> Self {
        Self {
            object: "list",
            first_id: data.first().map(|step| step.id.clone()),
            last_id: data.last().map(|step| step.id.clone()),
            has_more: false,
            data,
        }
    }
}
