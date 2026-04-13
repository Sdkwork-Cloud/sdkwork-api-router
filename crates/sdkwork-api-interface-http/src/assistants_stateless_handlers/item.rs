use super::*;

pub(super) async fn assistant_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsRetrieve(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    Json(
        get_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
        )
        .expect("assistant retrieve"),
    )
    .into_response()
}

pub(super) async fn assistant_update_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsUpdate(&assistant_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    Json(
        update_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
            request.name.as_deref().unwrap_or("assistant"),
        )
        .expect("assistant update"),
    )
    .into_response()
}

pub(super) async fn assistant_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsDelete(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    Json(
        delete_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
        )
        .expect("assistant delete"),
    )
    .into_response()
}
