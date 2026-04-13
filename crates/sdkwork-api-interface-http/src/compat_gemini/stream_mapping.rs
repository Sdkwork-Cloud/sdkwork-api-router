use std::collections::BTreeMap;

use super::response_mapping::openai_finish_reason_to_gemini;
use super::*;

pub fn gemini_stream_from_openai(response: ProviderStreamOutput) -> ProviderStreamOutput {
    transform_openai_sse_stream(response, GeminiStreamState::default(), |state, event| {
        state.map_event(event)
    })
}

#[derive(Default)]
struct GeminiStreamState {
    model: String,
    pending_tool_calls: BTreeMap<usize, ToolCallBuffer>,
}

impl GeminiStreamState {
    fn map_event(&mut self, event: OpenAiSseEvent) -> Vec<String> {
        match event {
            OpenAiSseEvent::Json(value) => self.map_json_event(value),
            OpenAiSseEvent::Done => Vec::new(),
        }
    }

    fn map_json_event(&mut self, value: Value) -> Vec<String> {
        if self.model.is_empty() {
            self.model = value
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
        }

        let Some(choice) = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return Vec::new();
        };

        let mut frames = Vec::new();
        let mut finish_reason = choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .map(openai_finish_reason_to_gemini);

        if let Some(tool_calls) = choice
            .get("delta")
            .and_then(|delta| delta.get("tool_calls"))
            .and_then(Value::as_array)
        {
            for tool_call in tool_calls {
                let index = tool_call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                let entry = self.pending_tool_calls.entry(index).or_default();
                if entry.id.is_empty() {
                    entry.id = tool_call
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                }
                if let Some(function) = tool_call.get("function") {
                    if entry.name.is_empty() {
                        entry.name = function
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned();
                    }
                    if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                        entry.arguments.push_str(arguments);
                    }
                }
            }
        }

        if let Some(delta_text) = choice
            .get("delta")
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
        {
            if !delta_text.is_empty() {
                let mut candidate = json!({
                    "content": {
                        "role": "model",
                        "parts": [
                            { "text": delta_text }
                        ]
                    }
                });
                if let Some(reason) = finish_reason.take() {
                    candidate["finishReason"] = Value::String(reason.to_owned());
                }
                frames.push(sse_data_frame(&json!({
                    "candidates": [candidate]
                })));
            }
        }

        if let Some(reason) = finish_reason {
            if !self.pending_tool_calls.is_empty() {
                let parts = self
                    .pending_tool_calls
                    .values()
                    .map(|tool_call| {
                        let args = serde_json::from_str::<Value>(&tool_call.arguments)
                            .unwrap_or_else(|_| Value::String(tool_call.arguments.clone()));
                        json!({
                            "functionCall": {
                                "name": tool_call.name,
                                "args": args
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                frames.push(sse_data_frame(&json!({
                    "candidates": [{
                        "content": {
                            "role": "model",
                            "parts": parts
                        },
                        "finishReason": reason
                    }]
                })));
                self.pending_tool_calls.clear();
            } else if frames.is_empty() {
                frames.push(sse_data_frame(&json!({
                    "candidates": [{
                        "content": {
                            "role": "model",
                            "parts": [
                                { "text": "" }
                            ]
                        },
                        "finishReason": reason
                    }]
                })));
            }
        }

        frames
    }
}

#[derive(Default)]
struct ToolCallBuffer {
    id: String,
    name: String,
    arguments: String,
}
