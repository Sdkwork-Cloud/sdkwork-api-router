use std::collections::{HashMap, VecDeque};

use super::*;

pub fn gemini_request_to_chat_completion(
    model: &str,
    payload: &Value,
) -> Result<CreateChatCompletionRequest> {
    let object = payload
        .as_object()
        .context("gemini request body must be a JSON object")?;

    let mut extra = passthrough_fields(
        object,
        &["contents", "tools", "generationConfig", "systemInstruction"],
    );

    if let Some(generation_config) = object.get("generationConfig").and_then(Value::as_object) {
        if let Some(max_output_tokens) = generation_config.get("maxOutputTokens").cloned() {
            extra.insert("max_tokens".to_owned(), max_output_tokens);
        }
        if let Some(candidate_count) = generation_config.get("candidateCount").cloned() {
            extra.insert("n".to_owned(), candidate_count);
        }
        if let Some(stop_sequences) = generation_config.get("stopSequences").cloned() {
            extra.insert("stop".to_owned(), stop_sequences);
        }
        for key in [
            "temperature",
            "topP",
            "topK",
            "presencePenalty",
            "frequencyPenalty",
        ] {
            if let Some(value) = generation_config.get(key).cloned() {
                extra.insert(generation_key_to_openai(key).to_owned(), value);
            }
        }
        for (key, value) in generation_config {
            if !matches!(
                key.as_str(),
                "maxOutputTokens"
                    | "candidateCount"
                    | "stopSequences"
                    | "temperature"
                    | "topP"
                    | "topK"
                    | "presencePenalty"
                    | "frequencyPenalty"
            ) {
                extra.insert(key.clone(), value.clone());
            }
        }
    }

    if let Some(tools) = object.get("tools") {
        let openai_tools = gemini_tools_to_openai(tools);
        if let Some(array) = openai_tools.as_array() {
            if !array.is_empty() {
                extra.insert("tools".to_owned(), openai_tools);
            }
        }
    }

    let mut messages = Vec::new();
    if let Some(system_instruction) = object.get("systemInstruction") {
        let text = gemini_system_instruction_text(system_instruction);
        if !text.is_empty() {
            messages.push(ChatMessageInput {
                role: "system".to_owned(),
                content: Value::String(text),
                extra: Map::new(),
            });
        }
    }

    if let Some(contents) = object.get("contents").and_then(Value::as_array) {
        messages.extend(gemini_contents_to_openai(contents)?);
    }

    Ok(CreateChatCompletionRequest {
        model: model.to_owned(),
        messages,
        stream: None,
        extra,
    })
}

pub fn gemini_count_tokens_request(
    model: &str,
    payload: &Value,
) -> CountResponseInputTokensRequest {
    CountResponseInputTokensRequest::new(model, payload.clone())
}

fn gemini_contents_to_openai(contents: &[Value]) -> Result<Vec<ChatMessageInput>> {
    let mut messages = Vec::new();
    let mut call_counters: HashMap<String, usize> = HashMap::new();
    let mut outstanding_calls: HashMap<String, VecDeque<String>> = HashMap::new();

    for content in contents {
        let object = content
            .as_object()
            .context("gemini content entries must be JSON objects")?;
        let role = object.get("role").and_then(Value::as_str).unwrap_or("user");
        let parts = object
            .get("parts")
            .and_then(Value::as_array)
            .context("gemini content parts must be an array")?;

        match role {
            "model" => {
                let mut text_segments = Vec::new();
                let mut tool_calls = Vec::new();

                for part in parts {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        text_segments.push(text.to_owned());
                        continue;
                    }

                    let Some(function_call) = part.get("functionCall") else {
                        continue;
                    };
                    let name = function_call
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let call_id = next_tool_call_id(&name, &mut call_counters);
                    outstanding_calls
                        .entry(name.clone())
                        .or_default()
                        .push_back(call_id.clone());
                    let args = function_call
                        .get("args")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    tool_calls.push(json!({
                        "id": call_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": serde_json::to_string(&args).unwrap_or_else(|_| "{}".to_owned())
                        }
                    }));
                }

                if !text_segments.is_empty() || !tool_calls.is_empty() {
                    let mut extra = Map::new();
                    if !tool_calls.is_empty() {
                        extra.insert("tool_calls".to_owned(), Value::Array(tool_calls));
                    }
                    messages.push(ChatMessageInput {
                        role: "assistant".to_owned(),
                        content: Value::String(text_segments.join("\n")),
                        extra,
                    });
                }
            }
            _ => {
                let mut text_segments = Vec::new();
                let mut tool_messages = Vec::new();

                for part in parts {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        text_segments.push(text.to_owned());
                        continue;
                    }

                    let Some(function_response) = part.get("functionResponse") else {
                        continue;
                    };
                    let name = function_response
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned();
                    let tool_call_id = outstanding_calls
                        .get_mut(&name)
                        .and_then(|calls| calls.pop_front())
                        .unwrap_or_else(|| next_tool_call_id(&name, &mut call_counters));
                    let mut extra = Map::new();
                    extra.insert("tool_call_id".to_owned(), Value::String(tool_call_id));
                    extra.insert("name".to_owned(), Value::String(name));
                    tool_messages.push(ChatMessageInput {
                        role: "tool".to_owned(),
                        content: Value::String(
                            function_response
                                .get("response")
                                .map(extract_text_from_value)
                                .unwrap_or_default(),
                        ),
                        extra,
                    });
                }

                if !text_segments.is_empty() {
                    messages.push(ChatMessageInput {
                        role: "user".to_owned(),
                        content: Value::String(text_segments.join("\n")),
                        extra: Map::new(),
                    });
                }
                messages.extend(tool_messages);
            }
        }
    }

    Ok(messages)
}

fn gemini_system_instruction_text(system_instruction: &Value) -> String {
    if let Some(parts) = system_instruction.get("parts").and_then(Value::as_array) {
        parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        extract_text_from_value(system_instruction)
    }
}

fn gemini_tools_to_openai(tools: &Value) -> Value {
    let Some(tools) = tools.as_array() else {
        return Value::Array(Vec::new());
    };

    let mut openai_tools = Vec::new();
    for tool in tools {
        let Some(declarations) = tool.get("functionDeclarations").and_then(Value::as_array) else {
            continue;
        };
        for declaration in declarations {
            openai_tools.push(json!({
                "type": "function",
                "function": {
                    "name": declaration.get("name").cloned().unwrap_or_else(|| Value::String(String::new())),
                    "description": declaration.get("description").cloned().unwrap_or(Value::Null),
                    "parameters": declaration
                        .get("parameters")
                        .cloned()
                        .unwrap_or_else(|| json!({"type":"object","properties":{}}))
                }
            }));
        }
    }

    Value::Array(openai_tools)
}

fn generation_key_to_openai(key: &str) -> &str {
    match key {
        "topP" => "top_p",
        "presencePenalty" => "presence_penalty",
        "frequencyPenalty" => "frequency_penalty",
        _ => key,
    }
}

fn next_tool_call_id(name: &str, counters: &mut HashMap<String, usize>) -> String {
    let counter = counters.entry(name.to_owned()).or_insert(0);
    *counter += 1;
    format!("gemini_tool_call_{}_{}", sanitize_identifier(name), counter)
}

fn sanitize_identifier(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if sanitized.is_empty() {
        "tool".to_owned()
    } else {
        sanitized
    }
}

fn extract_text_from_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str().map(ToOwned::to_owned).or_else(|| {
                    item.get("text")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn passthrough_fields(source: &Map<String, Value>, known_fields: &[&str]) -> Map<String, Value> {
    source
        .iter()
        .filter(|(key, _)| !known_fields.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
