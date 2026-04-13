use super::*;

pub(super) async fn thread_messages_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessages(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message");
        }
    }

    let text = request.content.as_str().unwrap_or("hello");
    Json(
        create_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &request.role,
            text,
        )
        .expect("thread message create"),
    )
    .into_response()
}

pub(super) async fn thread_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread messages list");
        }
    }

    Json(
        list_thread_messages(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread messages list"),
    )
    .into_response()
}
