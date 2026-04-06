use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct CreateThreadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<Value>,
}

impl CreateThreadRequest {
    pub fn with_metadata(metadata: Value) -> Self {
        Self {
            messages: None,
            metadata: Some(metadata),
            tool_resources: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateThreadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<Value>,
}

impl UpdateThreadRequest {
    pub fn with_metadata(metadata: Value) -> Self {
        Self {
            metadata: Some(metadata),
            tool_resources: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ThreadObject {
    pub id: String,
    pub object: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_resources: Option<Value>,
}

impl ThreadObject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "thread",
            metadata: None,
            tool_resources: None,
        }
    }

    pub fn with_metadata(id: impl Into<String>, metadata: Value) -> Self {
        let mut thread = Self::new(id);
        thread.metadata = Some(metadata);
        thread
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteThreadResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteThreadResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "thread.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateThreadMessageRequest {
    pub role: String,
    pub content: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl CreateThreadMessageRequest {
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: Value::String(text.into()),
            attachments: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateThreadMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl UpdateThreadMessageRequest {
    pub fn with_metadata(metadata: Value) -> Self {
        Self {
            metadata: Some(metadata),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ThreadTextObject {
    pub value: String,
    pub annotations: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ThreadMessageContentObject {
    pub r#type: &'static str,
    pub text: ThreadTextObject,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ThreadMessageObject {
    pub id: String,
    pub object: &'static str,
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub role: String,
    pub status: &'static str,
    pub content: Vec<ThreadMessageContentObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl ThreadMessageObject {
    pub fn text(
        id: impl Into<String>,
        thread_id: impl Into<String>,
        role: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "thread.message",
            thread_id: thread_id.into(),
            assistant_id: None,
            run_id: None,
            role: role.into(),
            status: "completed",
            content: vec![ThreadMessageContentObject {
                r#type: "text",
                text: ThreadTextObject {
                    value: text.into(),
                    annotations: vec![],
                },
            }],
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListThreadMessagesResponse {
    pub object: &'static str,
    pub data: Vec<ThreadMessageObject>,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
    pub has_more: bool,
}

impl ListThreadMessagesResponse {
    pub fn new(data: Vec<ThreadMessageObject>) -> Self {
        Self {
            object: "list",
            first_id: data.first().map(|message| message.id.clone()),
            last_id: data.last().map(|message| message.id.clone()),
            has_more: false,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteThreadMessageResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteThreadMessageResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "thread.message.deleted",
            deleted: true,
        }
    }
}
