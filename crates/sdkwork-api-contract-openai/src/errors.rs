use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OpenAiErrorEnvelope {
    pub message: String,
    pub r#type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OpenAiErrorResponse {
    pub error: OpenAiErrorEnvelope,
}

impl OpenAiErrorResponse {
    pub fn new(message: impl Into<String>, error_type: impl Into<String>) -> Self {
        Self {
            error: OpenAiErrorEnvelope {
                message: message.into(),
                r#type: error_type.into(),
                param: None,
                code: None,
            },
        }
    }
}
