use super::*;

pub fn anthropic_request_to_chat_completion(
    payload: &Value,
) -> Result<CreateChatCompletionRequest> {
    let object = payload
        .as_object()
        .context("anthropic request body must be a JSON object")?;
    let model = required_string(object, "model")?.to_owned();
    let stream = object.get("stream").and_then(Value::as_bool);

    let mut extra = passthrough_fields(
        object,
        &[
            "model",
            "messages",
            "system",
            "stream",
            "tools",
            "tool_choice",
            "max_tokens",
            "stop_sequences",
        ],
    );

    if let Some(max_tokens) = object.get("max_tokens").cloned() {
        extra.insert("max_tokens".to_owned(), max_tokens);
    }
    if let Some(stop_sequences) = object.get("stop_sequences").cloned() {
        extra.insert("stop".to_owned(), stop_sequences);
    }
    if let Some(tool_choice) = object.get("tool_choice") {
        extra.insert(
            "tool_choice".to_owned(),
            anthropic_tool_choice_to_openai(tool_choice),
        );
    }
    if let Some(tools) = object.get("tools") {
        let openai_tools = anthropic_tools_to_openai(tools);
        if let Some(array) = openai_tools.as_array() {
            if !array.is_empty() {
                extra.insert("tools".to_owned(), openai_tools);
            }
        }
    }

    let mut messages = Vec::new();
    if let Some(system) = object.get("system") {
        let system_text = anthropic_system_text(system);
        if !system_text.is_empty() {
            messages.push(ChatMessageInput {
                role: "system".to_owned(),
                content: Value::String(system_text),
                extra: Map::new(),
            });
        }
    }
    if let Some(input_messages) = object.get("messages").and_then(Value::as_array) {
        messages.extend(anthropic_messages_to_openai(input_messages)?);
    }

    Ok(CreateChatCompletionRequest {
        model,
        messages,
        stream,
        extra,
    })
}

pub fn anthropic_count_tokens_request(payload: &Value) -> Result<CountResponseInputTokensRequest> {
    let object = payload
        .as_object()
        .context("anthropic count_tokens body must be a JSON object")?;
    Ok(CountResponseInputTokensRequest::new(
        required_string(object, "model")?,
        payload.clone(),
    ))
}

fn anthropic_messages_to_openai(messages: &[Value]) -> Result<Vec<ChatMessageInput>> {
    let mut result = Vec::new();

    for message in messages {
        let object = message
            .as_object()
            .context("anthropic message entries must be JSON objects")?;
        let role = required_string(object, "role")?;
        let content = object.get("content").cloned().unwrap_or(Value::Null);

        match role {
            "user" => translate_anthropic_user_message(&content, &mut result)?,
            "assistant" => translate_anthropic_assistant_message(&content, &mut result)?,
            "system" => result.push(ChatMessageInput {
                role: "system".to_owned(),
                content: Value::String(extract_text_from_value(&content)),
                extra: Map::new(),
            }),
            _ => {
                result.push(ChatMessageInput {
                    role: role.to_owned(),
                    content,
                    extra: Map::new(),
                });
            }
        }
    }

    Ok(result)
}

fn translate_anthropic_user_message(
    content: &Value,
    out: &mut Vec<ChatMessageInput>,
) -> Result<()> {
    match content {
        Value::String(text) => {
            out.push(ChatMessageInput {
                role: "user".to_owned(),
                content: Value::String(text.clone()),
                extra: Map::new(),
            });
        }
        Value::Array(blocks) => {
            let mut content_parts = Vec::new();
            let mut tool_messages = Vec::new();

            for block in blocks {
                let block_type = block
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            content_parts.push(Value::String(text.to_owned()));
                        }
                    }
                    "image" => {
                        if let Some(image_part) = anthropic_image_block_to_openai(block) {
                            content_parts.push(image_part);
                        }
                    }
                    "tool_result" => {
                        let tool_call_id = block
                            .get("tool_use_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned();
                        let mut extra = Map::new();
                        extra.insert("tool_call_id".to_owned(), Value::String(tool_call_id));
                        if let Some(is_error) = block.get("is_error").cloned() {
                            extra.insert("anthropic_is_error".to_owned(), is_error);
                        }
                        tool_messages.push(ChatMessageInput {
                            role: "tool".to_owned(),
                            content: Value::String(extract_text_from_value(
                                block.get("content").unwrap_or(&Value::Null),
                            )),
                            extra,
                        });
                    }
                    _ => {}
                }
            }

            if !content_parts.is_empty() {
                out.push(ChatMessageInput {
                    role: "user".to_owned(),
                    content: collapse_message_content(content_parts),
                    extra: Map::new(),
                });
            }
            out.extend(tool_messages);
        }
        _ => {}
    }

    Ok(())
}

fn translate_anthropic_assistant_message(
    content: &Value,
    out: &mut Vec<ChatMessageInput>,
) -> Result<()> {
    match content {
        Value::String(text) => {
            out.push(ChatMessageInput {
                role: "assistant".to_owned(),
                content: Value::String(text.clone()),
                extra: Map::new(),
            });
        }
        Value::Array(blocks) => {
            let mut content_parts = Vec::new();
            let mut tool_calls = Vec::new();

            for block in blocks {
                let block_type = block
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                match block_type {
                    "text" => {
                        if let Some(text) = block.get("text").and_then(Value::as_str) {
                            content_parts.push(Value::String(text.to_owned()));
                        }
                    }
                    "tool_use" => {
                        let name = block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        let id = block.get("id").and_then(Value::as_str).unwrap_or_default();
                        let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                        tool_calls.push(json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_owned())
                            }
                        }));
                    }
                    _ => {}
                }
            }

            if !content_parts.is_empty() || !tool_calls.is_empty() {
                let mut extra = Map::new();
                if !tool_calls.is_empty() {
                    extra.insert("tool_calls".to_owned(), Value::Array(tool_calls));
                }
                out.push(ChatMessageInput {
                    role: "assistant".to_owned(),
                    content: if content_parts.is_empty() {
                        Value::String(String::new())
                    } else {
                        collapse_message_content(content_parts)
                    },
                    extra,
                });
            }
        }
        _ => {}
    }

    Ok(())
}

fn anthropic_tool_choice_to_openai(tool_choice: &Value) -> Value {
    match tool_choice {
        Value::Object(object) => match object.get("type").and_then(Value::as_str) {
            Some("any") => Value::String("required".to_owned()),
            Some("auto") => Value::String("auto".to_owned()),
            Some("tool") => {
                let name = object
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                json!({
                    "type": "function",
                    "function": {
                        "name": name
                    }
                })
            }
            _ => tool_choice.clone(),
        },
        _ => tool_choice.clone(),
    }
}

fn anthropic_tools_to_openai(tools: &Value) -> Value {
    let Some(tools) = tools.as_array() else {
        return Value::Array(Vec::new());
    };

    Value::Array(
        tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.get("name").cloned().unwrap_or_else(|| Value::String(String::new())),
                        "description": tool.get("description").cloned().unwrap_or(Value::Null),
                        "parameters": tool
                            .get("input_schema")
                            .cloned()
                            .unwrap_or_else(|| json!({"type":"object","properties":{}}))
                    }
                })
            })
            .collect(),
    )
}

fn anthropic_system_text(system: &Value) -> String {
    match system {
        Value::String(text) => text.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn anthropic_image_block_to_openai(block: &Value) -> Option<Value> {
    let source = block.get("source")?;
    let media_type = source.get("media_type").and_then(Value::as_str)?;
    let data = source.get("data").and_then(Value::as_str)?;
    Some(json!({
        "type": "image_url",
        "image_url": {
            "url": format!("data:{media_type};base64,{data}")
        }
    }))
}

fn collapse_message_content(parts: Vec<Value>) -> Value {
    if parts.iter().all(Value::is_string) {
        let text = parts
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
        Value::String(text)
    } else {
        Value::Array(
            parts
                .into_iter()
                .map(|part| match part {
                    Value::String(text) => json!({
                        "type": "text",
                        "text": text
                    }),
                    other => other,
                })
                .collect(),
        )
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

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing or invalid string field `{key}`"))
}
