use super::*;

pub(super) async fn anthropic_messages_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 100).await {
        Ok(Some(response)) => return response,
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate quota",
            )
                .into_response();
        }
    }

    if request.stream.unwrap_or(false) {
        match relay_chat_completion_stream_from_store_with_options(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                if record_gateway_usage_for_project(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "chat_completion",
                    &request.model,
                    100,
                    0.10,
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return upstream_passthrough_response(anthropic_stream_from_openai(response));
            }
            Ok(None) => {}
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        if record_gateway_usage_for_project(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "chat_completion",
            &request.model,
            100,
            0.10,
        )
        .await
        .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to record usage",
            )
                .into_response();
        }

        return local_anthropic_stream_response(&request.model);
    }

    match relay_chat_completion_from_store_with_options(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
        &options,
    )
    .await
    {
        Ok(Some(response)) => {
            let token_usage = extract_token_usage_metrics(&response);
            if record_gateway_usage_for_project_with_route_key_and_tokens(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                &request.model,
                100,
                0.10,
                token_usage,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(openai_chat_response_to_anthropic(&response)).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return anthropic_bad_gateway_response("failed to relay upstream anthropic message");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        100,
        0.10,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(openai_chat_response_to_anthropic(
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
    .into_response()
}

pub(super) async fn anthropic_count_tokens_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_count_response_input_tokens_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
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
