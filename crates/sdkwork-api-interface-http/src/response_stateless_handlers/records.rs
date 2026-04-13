use super::*;

pub(super) async fn response_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesRetrieve(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    Json(
        get_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response retrieve"),
    )
    .into_response()
}

pub(super) async fn response_input_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputItemsList(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    Json(
        list_response_input_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response input items"),
    )
    .into_response()
}

pub(super) async fn response_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesDelete(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    Json(
        delete_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response delete"),
    )
    .into_response()
}

pub(super) async fn response_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCancel(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    Json(
        cancel_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response cancel"),
    )
    .into_response()
}
