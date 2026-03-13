use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    pub model: String,
    pub input: Value,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub output: Vec<ResponseOutputItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseOutputItem {
    pub r#type: &'static str,
}

impl ResponseObject {
    pub fn empty(id: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "response",
            model: model.into(),
            output: Vec::new(),
        }
    }
}
