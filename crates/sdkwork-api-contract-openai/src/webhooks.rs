use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
}

impl CreateWebhookRequest {
    pub fn new<I, S>(url: impl Into<String>, events: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            url: url.into(),
            events: events.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateWebhookRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl UpdateWebhookRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WebhookObject {
    pub id: String,
    pub object: &'static str,
    pub url: String,
    pub status: &'static str,
}

impl WebhookObject {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "webhook_endpoint",
            url: url.into(),
            status: "enabled",
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListWebhooksResponse {
    pub object: &'static str,
    pub data: Vec<WebhookObject>,
}

impl ListWebhooksResponse {
    pub fn new(data: Vec<WebhookObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteWebhookResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteWebhookResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "webhook_endpoint.deleted",
            deleted: true,
        }
    }
}
