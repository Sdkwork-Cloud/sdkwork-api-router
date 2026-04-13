use super::*;

pub(super) async fn containers_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateContainerRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Containers(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container");
        }
    }

    Json(
        sdkwork_api_app_gateway::create_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("container"),
    )
    .into_response()
}

pub(super) async fn containers_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ContainersList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream containers list");
        }
    }

    Json(
        sdkwork_api_app_gateway::list_containers(
            request_context.tenant_id(),
            request_context.project_id(),
        )
        .expect("containers list"),
    )
    .into_response()
}

pub(super) async fn container_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersRetrieve(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container retrieve");
        }
    }

    Json(
        sdkwork_api_app_gateway::get_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container retrieve"),
    )
    .into_response()
}

pub(super) async fn container_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersDelete(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container delete");
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container delete"),
    )
    .into_response()
}
