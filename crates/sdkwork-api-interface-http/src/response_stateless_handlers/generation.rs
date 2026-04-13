use super::*;

pub(super) async fn responses_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ResponsesStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        return local_response_stream_response("resp_1", &request.model);
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::Responses(&request)).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("response"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream response"),
    }
}

pub(super) async fn response_input_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    Json(
        count_response_input_tokens(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response input tokens"),
    )
    .into_response()
}

pub(super) async fn response_compact_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCompact(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    Json(
        compact_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response compact"),
    )
    .into_response()
}
