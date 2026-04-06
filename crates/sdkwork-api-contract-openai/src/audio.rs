use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTranscriptionRequest {
    pub model: String,
    pub file_id: String,
}

impl CreateTranscriptionRequest {
    pub fn new(model: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TranscriptionObject {
    pub text: String,
}

impl TranscriptionObject {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTranslationRequest {
    pub model: String,
    pub file_id: String,
}

impl CreateTranslationRequest {
    pub fn new(model: impl Into<String>, file_id: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            file_id: file_id.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TranslationObject {
    pub text: String,
}

impl TranslationObject {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateVoiceConsentRequest {
    pub voice: String,
    pub name: String,
    pub consent_text: String,
}

impl CreateVoiceConsentRequest {
    pub fn new(
        voice: impl Into<String>,
        name: impl Into<String>,
        consent_text: impl Into<String>,
    ) -> Self {
        Self {
            voice: voice.into(),
            name: name.into(),
            consent_text: consent_text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VoiceObject {
    pub id: String,
    pub object: &'static str,
    pub name: String,
}

impl VoiceObject {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "voice",
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListVoicesResponse {
    pub object: &'static str,
    pub data: Vec<VoiceObject>,
}

impl ListVoicesResponse {
    pub fn new(data: Vec<VoiceObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VoiceConsentObject {
    pub id: String,
    pub object: &'static str,
    pub status: &'static str,
    pub voice: String,
    pub name: String,
}

impl VoiceConsentObject {
    pub fn approved(
        id: impl Into<String>,
        voice: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "voice_consent",
            status: "approved",
            voice: voice.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSpeechRequest {
    pub model: String,
    pub voice: String,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_format: Option<String>,
}

impl CreateSpeechRequest {
    pub fn new(
        model: impl Into<String>,
        voice: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            model: model.into(),
            voice: voice.into(),
            input: input.into(),
            instructions: None,
            response_format: None,
            speed: None,
            stream_format: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SpeechResponse {
    pub format: String,
    pub audio_base64: String,
}

impl SpeechResponse {
    pub fn new(format: impl Into<String>, audio_base64: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            audio_base64: audio_base64.into(),
        }
    }
}
