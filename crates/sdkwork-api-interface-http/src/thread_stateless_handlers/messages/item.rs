use super::*;

pub(super) async fn thread_message_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesRetrieve(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message retrieve");
        }
    }

    Json(
        get_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message retrieve"),
    )
    .into_response()
}

pub(super) async fn thread_message_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesUpdate(&thread_id, &message_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message update");
        }
    }

    Json(
        update_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message update"),
    )
    .into_response()
}

pub(super) async fn thread_message_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesDelete(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message delete");
        }
    }

    Json(
        delete_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message delete"),
    )
    .into_response()
}
