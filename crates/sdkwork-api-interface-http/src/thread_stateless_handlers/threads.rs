use super::*;

pub(super) async fn threads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Threads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread");
        }
    }

    Json(create_thread(request_context.tenant_id(), request_context.project_id()).expect("thread"))
        .into_response()
}

pub(super) async fn thread_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsRetrieve(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread retrieve");
        }
    }

    Json(
        get_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread retrieve"),
    )
    .into_response()
}

pub(super) async fn thread_update_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsUpdate(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread update");
        }
    }

    Json(
        update_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread update"),
    )
    .into_response()
}

pub(super) async fn thread_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsDelete(&thread_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread delete");
        }
    }

    Json(
        delete_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread delete"),
    )
    .into_response()
}
