use anyhow::{Context, Result};
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_contract_openai::responses::CountResponseInputTokensRequest;
use sdkwork_api_provider_core::ProviderStreamOutput;
use serde_json::{Map, Value, json};

use crate::compat_streaming::{OpenAiSseEvent, sse_data_frame, transform_openai_sse_stream};

mod error_responses;
mod request_mapping;
mod response_mapping;
mod stream_mapping;

pub use error_responses::{
    gemini_bad_gateway_response, gemini_error_response, gemini_invalid_request_response,
};
pub use request_mapping::{gemini_count_tokens_request, gemini_request_to_chat_completion};
pub use response_mapping::{openai_chat_response_to_gemini, openai_count_tokens_to_gemini};
pub use stream_mapping::gemini_stream_from_openai;
