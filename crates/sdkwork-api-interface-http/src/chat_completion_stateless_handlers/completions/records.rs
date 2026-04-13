use super::*;

pub(super) async fn chat_completions_list_handler(
    request_context: StatelessGatewayRequest,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletionsList).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            list_chat_completions(request_context.tenant_id(), request_context.project_id())
                .expect("chat completions"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion list"),
    }
}

pub(super) async fn chat_completion_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsRetrieve(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    Json(
        get_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion"),
    )
    .into_response()
}

pub(super) async fn chat_completion_update_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsUpdate(&completion_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    Json(
        update_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
            request.metadata.unwrap_or(serde_json::json!({})),
        )
        .expect("chat completion update"),
    )
    .into_response()
}

pub(super) async fn chat_completion_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsDelete(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    Json(
        delete_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion delete"),
    )
    .into_response()
}
