use std::collections::BTreeMap;

use super::response_mapping::openai_finish_reason_to_anthropic;
use super::*;

pub fn anthropic_stream_from_openai(response: ProviderStreamOutput) -> ProviderStreamOutput {
    transform_openai_sse_stream(response, AnthropicStreamState::default(), |state, event| {
        state.map_event(event)
    })
}

#[derive(Default)]
struct AnthropicStreamState {
    started: bool,
    finished: bool,
    text_block_open: bool,
    message_id: String,
    model: String,
    pending_tool_calls: BTreeMap<usize, ToolCallBuffer>,
}

impl AnthropicStreamState {
    fn map_event(&mut self, event: OpenAiSseEvent) -> Vec<String> {
        match event {
            OpenAiSseEvent::Json(value) => self.map_json_event(value),
            OpenAiSseEvent::Done => self.finish(Value::Null),
        }
    }

    fn map_json_event(&mut self, value: Value) -> Vec<String> {
        if self.message_id.is_empty() {
            self.message_id = value
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("msg_stream")
                .to_owned();
        }
        if self.model.is_empty() {
            self.model = value
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
        }

        let mut frames = Vec::new();
        if !self.started {
            self.started = true;
            frames.push(sse_named_event_frame(
                "message_start",
                &json!({
                    "type": "message_start",
                    "message": {
                        "id": self.message_id,
                        "type": "message",
                        "role": "assistant",
                        "model": self.model,
                        "content": [],
                        "stop_reason": Value::Null,
                        "stop_sequence": Value::Null,
                        "usage": {
                            "input_tokens": 0,
                            "output_tokens": 0
                        }
                    }
                }),
            ));
        }

        let Some(choice) = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return frames;
        };

        if let Some(delta_text) = choice
            .get("delta")
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
        {
            if !delta_text.is_empty() {
                if !self.text_block_open {
                    self.text_block_open = true;
                    frames.push(sse_named_event_frame(
                        "content_block_start",
                        &json!({
                            "type": "content_block_start",
                            "index": 0,
                            "content_block": {
                                "type": "text",
                                "text": ""
                            }
                        }),
                    ));
                }
                frames.push(sse_named_event_frame(
                    "content_block_delta",
                    &json!({
                        "type": "content_block_delta",
                        "index": 0,
                        "delta": {
                            "type": "text_delta",
                            "text": delta_text
                        }
                    }),
                ));
            }
        }

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

        if let Some(finish_reason) = choice.get("finish_reason").and_then(Value::as_str) {
            frames.extend(self.finish(openai_finish_reason_to_anthropic(finish_reason)));
        }

        frames
    }

    fn finish(&mut self, stop_reason: Value) -> Vec<String> {
        if self.finished || !self.started {
            return Vec::new();
        }
        self.finished = true;

        let mut frames = Vec::new();
        if self.text_block_open {
            frames.push(sse_named_event_frame(
                "content_block_stop",
                &json!({
                    "type": "content_block_stop",
                    "index": 0
                }),
            ));
            self.text_block_open = false;
        }

        if !self.pending_tool_calls.is_empty() {
            let base_index = if frames.is_empty() { 0 } else { 1 };
            for (offset, (_, tool_call)) in self.pending_tool_calls.iter().enumerate() {
                let input = serde_json::from_str::<Value>(&tool_call.arguments)
                    .unwrap_or_else(|_| Value::String(tool_call.arguments.clone()));
                let index = base_index + offset;
                frames.push(sse_named_event_frame(
                    "content_block_start",
                    &json!({
                        "type": "content_block_start",
                        "index": index,
                        "content_block": {
                            "type": "tool_use",
                            "id": tool_call.id,
                            "name": tool_call.name,
                            "input": input
                        }
                    }),
                ));
                frames.push(sse_named_event_frame(
                    "content_block_stop",
                    &json!({
                        "type": "content_block_stop",
                        "index": index
                    }),
                ));
            }
            self.pending_tool_calls.clear();
        }

        frames.push(sse_named_event_frame(
            "message_delta",
            &json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": stop_reason,
                    "stop_sequence": Value::Null
                },
                "usage": {
                    "output_tokens": 0
                }
            }),
        ));
        frames.push(sse_named_event_frame(
            "message_stop",
            &json!({
                "type": "message_stop"
            }),
        ));

        frames
    }
}

#[derive(Default)]
struct ToolCallBuffer {
    id: String,
    name: String,
    arguments: String,
}
