use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct CreateConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Value>>,
}

impl CreateConversationRequest {
    pub fn with_metadata(metadata: Value) -> Self {
        Self {
            metadata: Some(metadata),
            items: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Value>>,
}

impl UpdateConversationRequest {
    pub fn with_metadata(metadata: Value) -> Self {
        Self {
            metadata: Some(metadata),
            items: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateConversationItemsRequest {
    pub items: Vec<Value>,
}

impl CreateConversationItemsRequest {
    pub fn new(items: Vec<Value>) -> Self {
        Self { items }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConversationObject {
    pub id: String,
    pub object: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl ConversationObject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "conversation",
            metadata: None,
        }
    }

    pub fn with_metadata(id: impl Into<String>, metadata: Value) -> Self {
        let mut conversation = Self::new(id);
        conversation.metadata = Some(metadata);
        conversation
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListConversationsResponse {
    pub object: &'static str,
    pub data: Vec<ConversationObject>,
}

impl ListConversationsResponse {
    pub fn new(data: Vec<ConversationObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteConversationResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteConversationResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "conversation.deleted",
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConversationItemObject {
    pub id: String,
    pub object: &'static str,
    pub r#type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Value>,
}

impl ConversationItemObject {
    pub fn message(
        id: impl Into<String>,
        role: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "conversation.item",
            r#type: "message",
            role: Some(role.into()),
            content: Some(Value::Array(vec![serde_json::json!({
                "type":"output_text",
                "text":text.into()
            })])),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListConversationItemsResponse {
    pub object: &'static str,
    pub data: Vec<ConversationItemObject>,
}

impl ListConversationItemsResponse {
    pub fn new(data: Vec<ConversationItemObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteConversationItemResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteConversationItemResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "conversation.item.deleted",
            deleted: true,
        }
    }
}
