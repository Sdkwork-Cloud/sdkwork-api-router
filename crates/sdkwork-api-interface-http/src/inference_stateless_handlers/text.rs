use super::*;

pub(super) async fn completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Completions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("completion"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream completion"),
    }
}

pub(super) async fn embeddings_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Embeddings(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_embedding(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("embedding"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream embedding"),
    }
}

pub(super) async fn moderations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Moderations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }

    Json(
        create_moderation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("moderation"),
    )
    .into_response()
}
