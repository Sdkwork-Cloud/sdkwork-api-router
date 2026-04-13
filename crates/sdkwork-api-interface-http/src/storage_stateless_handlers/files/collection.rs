use super::*;

pub(super) async fn files_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_file_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(&request_context, ProviderRequest::Files(&request))
                .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream file");
                }
            }

            Json(
                create_file(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("file"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

pub(super) async fn files_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    Json(list_files(request_context.tenant_id(), request_context.project_id()).expect("files list"))
        .into_response()
}
