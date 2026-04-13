use super::*;

pub(super) async fn anthropic_messages_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(response));
            }
            Ok(None) => return local_anthropic_stream_response(&request.model),
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }
    }

    match relay_stateless_json_request_with_options(
        &request_context,
        ProviderRequest::ChatCompletions(&request),
        &options,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_chat_response_to_anthropic(&response)).into_response(),
        Ok(None) => Json(openai_chat_response_to_anthropic(
            &serde_json::to_value(
                create_chat_completion(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                )
                .expect("chat completion"),
            )
            .expect("chat completion value"),
        ))
        .into_response(),
        Err(_) => anthropic_bad_gateway_response("failed to relay upstream anthropic message"),
    }
}

pub(super) async fn anthropic_count_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => Json(openai_count_tokens_to_anthropic(
            &serde_json::to_value(
                count_response_input_tokens(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                )
                .expect("response input tokens"),
            )
            .expect("response input token value"),
        ))
        .into_response(),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}
