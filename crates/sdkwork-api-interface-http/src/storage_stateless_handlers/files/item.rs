use super::*;

pub(super) async fn file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesRetrieve(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    Json(
        get_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &file_id,
        )
        .expect("file retrieve"),
    )
    .into_response()
}

pub(super) async fn file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesDelete(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    Json(
        delete_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &file_id,
        )
        .expect("file delete"),
    )
    .into_response()
}
